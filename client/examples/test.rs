use anyhow::Ok;
use cgmath::{Matrix4, SquareMatrix};
use hashbrown::HashMap;
use rayon::prelude::*;
use std::{
    net::TcpStream,
    ops::Range,
    process::exit,
    sync::{Arc, Mutex, RwLock},
};
use steamengine_client::camera::Camera;
use steamengine_client::render::DrawQueueBuffer;
use steamengine_client::resources::model::RenderPassAttachModels;
use steamengine_client::resources::{Identifier, ResourceLoader, model::ModelResourceLoader};
use steamengine_communication::{Package, ReadPackageImpl, ToPackage, errors::PCSError};
use steamengine_wgpu_core::render::{
    Renderer, RendererBuilder,
    bind_group::BindGroupEntryBuilder,
    render_pass::{RenderPassColorAttachmentBuilder, RenderPassDescriptorBuilder},
    render_pipeline::RenderPipeline,
    vertex::Vertex,
};
use wgpu::util::DrawIndexedIndirectArgs;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, BufferAddress, BufferUsages, PipelineLayoutDescriptor,
    RenderPass, ShaderStages, VertexAttribute, VertexStepMode, hal::Instance,
    wgc::command::bundle_ffi::wgpu_render_bundle_push_debug_group,
};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, Event, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct ChangePackage {
    id: u32,
    color: Color,
    matrix: [[f32; 4]; 4],
}
impl ChangePackage {
    pub fn new(id: u32, color: Color, matrix: Matrix4<f32>) -> Self {
        ChangePackage {
            id,
            color,
            matrix: matrix.into(),
        }
    }
    pub fn from_package(package: Package) -> Self {
        let mut package = package;
        let id = package.get_u32();
        let color = package.get_cons_length::<16>();
        let color = Color::from_bytes(color);
        let matrix = package.get_cons_length::<64>();
        let matrix: &[[f32; 4]; 4] = bytemuck::from_bytes(&matrix);
        ChangePackage {
            id,
            color,
            matrix: *matrix,
        }
    }
    pub fn to_bytes(self) -> Vec<u8> {
        let id = self.id.to_be_bytes();
        let col = self.color.to_bytes();
        let matrix: &[u8] = bytemuck::cast_slice(&self.matrix);

        let mut bytes = Vec::new();

        bytes.append(&mut id.to_vec());
        bytes.append(&mut col.to_vec());
        bytes.append(&mut matrix.to_vec());

        bytes
    }
}
impl ToPackage for ChangePackage {
    fn to_package(self) -> Package {
        let bytes = self.to_bytes();
        Package::new("change.color", bytes.to_vec())
    }
}
struct DrawRangePackage {
    end: u32,
    start: u32,
}
impl DrawRangePackage {
    pub fn from_package(package: Package) -> DrawRangePackage {
        let mut package = package;
        let start = package.get_u32();
        let end = package.get_u32();

        DrawRangePackage { end, start }
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let end = self.end.to_be_bytes();
        let start = self.start.to_be_bytes();
        let mut bytes = Vec::new();
        bytes.append(&mut start.to_vec());
        bytes.append(&mut end.to_vec());

        bytes
    }

    pub fn to_range(self) -> Range<u32> {
        self.start..self.end
    }
}
impl ToPackage for DrawRangePackage {
    fn to_package(self) -> Package {
        let bytes = self.to_bytes();

        Package::new("change.range", bytes)
    }
}

struct BgChangePackage {
    color: Color,
}
impl BgChangePackage {
    pub fn from_package(package: Package) -> BgChangePackage {
        let mut package = package;
        let color = Color::from_bytes(package.get_cons_length::<16>());
        BgChangePackage { color }
    }
    pub fn to_bytes(self) -> Vec<u8> {
        let bytes = self.color.to_bytes();
        bytes.to_vec()
    }
    pub fn new(color: Color) -> Self {
        BgChangePackage { color }
    }
}
impl ToPackage for BgChangePackage {
    fn to_package(self) -> Package {
        let bytes = self.to_bytes();
        Package::new("change.bg", bytes)
    }
}

struct InstanceBuffer<'a> {
    buffer: Buffer,
    renderer: Arc<RwLock<Renderer<'a>>>,
}

use steamengine_client::render::SimpleBuffer;
impl<'a> SimpleBuffer<'a, RawEntity> for InstanceBuffer<'a> {
    fn new(renderer: Arc<RwLock<Renderer<'a>>>, limit: u64) -> Self {
        let lock = renderer.clone();
        let lock = lock.read().expect("Cannot read renderer");
        let buffer = lock.create_buffer(
            "Instance Buffer",
            BufferUsages::VERTEX | BufferUsages::COPY_DST,
            limit * std::mem::size_of::<RawEntity>() as u64,
        );
        Self { buffer, renderer }
    }
    fn insert(&self, index: u64, data: RawEntity) {
        let lock = self.renderer.clone();
        let lock = lock.read().expect("Cannot read renderer");
        lock.update_buffer_entry(&self.buffer, index, data);
    }
}
use steamengine_client::camera::prespective::PrespectiveCamera;
use steamengine_client::resources::model::Models;
#[derive(Default)]
struct State<'a> {
    window: Option<Arc<Window>>,
    renderer: Option<Arc<Renderer<'a>>>,
    models: Option<Models>,
    models_keys: Option<HashMap<Identifier, DrawIndexedIndirectArgs>>,
    pipeline: Option<wgpu::RenderPipeline>,
    instances: Option<Arc<InstanceBuffer<'a>>>,
    bg_color: Arc<RwLock<Color>>,
    commands: Option<Arc<DrawQueueBuffer<'a>>>,
    camera: PrespectiveCamera,
    camera_bind_group: Option<(BindGroupLayout, BindGroup)>,
    view_camera_buffer: Option<Buffer>,
    projection_camera_buffer: Option<Buffer>,
    yaw: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}
impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }
    pub fn to_bytes(self) -> [u8; 16] {
        let r = self.r.to_be_bytes();
        let g = self.g.to_be_bytes();
        let b = self.b.to_be_bytes();
        let a = self.a.to_be_bytes();

        [
            r[0], r[1], r[2], r[3], g[0], g[1], g[2], g[3], b[0], b[1], b[2], b[3], a[0], a[1],
            a[2], a[3],
        ]
    }
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        let r: &[u8] = &bytes[..4];
        let r: [u8; 4] = r.try_into().unwrap();
        let r: f32 = f32::from_be_bytes(r);

        let g: &[u8] = &bytes[4..8];
        let g: [u8; 4] = g.try_into().unwrap();
        let g: f32 = f32::from_be_bytes(g);

        let b: &[u8] = &bytes[8..12];
        let b: [u8; 4] = b.try_into().unwrap();
        let b: f32 = f32::from_be_bytes(b);

        let a: &[u8] = &bytes[12..16];
        let a: [u8; 4] = a.try_into().unwrap();
        let a: f32 = f32::from_be_bytes(a);

        Self { r, g, b, a }
    }
}
#[repr(C)]
#[derive(Debug, bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub struct UniformMatrix {
    matrix: [[f32; 4]; 4],
}
impl UniformMatrix {
    pub fn from_matrix(matrix: Matrix4<f32>) -> Self {
        let matrix: [[f32; 4]; 4] = matrix.into();
        UniformMatrix { matrix }
    }
}

use cgmath::*;
impl ApplicationHandler<()> for State<'static> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        let window = Arc::new(window);
        let size = window.inner_size();
        let size = (size.width, size.height);
        let renderer =
            pollster::block_on(RendererBuilder::new().build(window.clone(), size)).unwrap();

        *self.camera.aspect_ratio() = (size.0 as f32 / size.1 as f32) as f32;
        println!("DIV {}", (size.0 as f32 / size.1 as f32));
        println!("ASPECT RATIO : {}", self.camera.aspect_ratio());

        let renderer = Arc::new(renderer);

        let renderer_clone = renderer.clone();
        //let instance_buffer_clone = instance_buffer.clone();
        let color = self.bg_color.clone();
        //std::thread::spawn(move || {
        //    let mut tcp = TcpStream::connect("127.0.0.1:8090").unwrap();
        //    let renderer_th = renderer_clone;
        //    let instance_buffer_th = instance_buffer_clone;
        //
        //    loop {
        //        let package = tcp.read_package().unwrap();
        //        match package.name.as_str() {
        //            "change" => {
        //                let renderer_th = renderer_th.read().unwrap();
        //                let package = ChangePackage::from_package(package);
        //
        //                let matrix = package.matrix;
        //                let matrix = Matrix4::from(matrix);

        //                renderer_th.update_buffer_entry(
        //                    &instance_buffer_th,
        //                    package.id as u64,
        //                    Entity::new(package.color, matrix).to_raw(),
        //                );
        //            }
        //            "change.range" => {
        //                let package = DrawRangePackage::from_package(package);
        //                let mut range = range.write().unwrap();
        //                *range = package.to_range();
        //            }
        //            "change.bg" => {
        //                let package = BgChangePackage::from_package(package);
        //                let mut color = color.write().unwrap();
        //                *color = package.color;
        //            }
        //            _ => {}
        //        }
        //    }
        //});
        //

        let view_camera_buffer = rendere.init_buffer(
            "View Camera Buffer",
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            &[UniformMatrix::from_matrix(Matrix4::identity())],
        );
        let projection_camera_buffer = renderer.init_buffer(
            "Projecttion Camera Buffer",
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            &[UniformMatrix::from_matrix(Matrix4::identity())],
        );
        let (camera_bind_group, camera_bind_group_layout) = renderer.bind_group(
            "Camera Bind Group",
            &[
                BindGroupEntryBuilder::new(0)
                    .uniform()
                    .with(wgpu::BindingResource::Buffer(
                        view_camera_buffer.as_entire_buffer_binding(),
                    ))
                    .on(ShaderStages::VERTEX),
                BindGroupEntryBuilder::new(1)
                    .uniform()
                    .with(wgpu::BindingResource::Buffer(
                        projection_camera_buffer.as_entire_buffer_binding(),
                    ))
                    .on(ShaderStages::VERTEX),
            ],
        );
        /*
        let view = self.camera.view();
        renderer
            .read()
            .unwrap()
            .update_buffer(&view_camera_buffer, &[UniformMatrix::from_matrix(view)]);

        let projection = self.camera.projection();
        renderer.read().unwrap().update_buffer(
            &projection_camera_buffer,
            &[UniformMatrix::from_matrix(projection)],
        );
        */

        let pipeline = AppRenderPipeline::new(&[&camera_bind_group_layout]).to_wgpu(&renderer);

        let (models, models_keys) = ModelResourceLoader::new()
            .load_to_buffers("resources/models", renderer.clone())
            .expect("Failed to read models");

        let models_keys: HashMap<Identifier, DrawIndexedIndirectArgs> = models_keys
            .par_iter()
            .map(|(k, v)| (k.clone(), v.first().unwrap().clone()))
            .collect();

        println!("CREATING INSTANCE BUFFER");
        let instances = Arc::new(InstanceBuffer::new(renderer.clone(), 90));
        println!("CREATING COMMANDS BUFFER");
        let commands = Arc::new(DrawQueueBuffer::new(renderer.clone(), 90));

        *self.bg_color.write().unwrap() = Color::new(1.0, 1.0, 1.0, 1.0);

        instances.insert(
            0,
            Entity::new(Color::new(1.0, 0.0, 0.0, 1.0), Matrix4::identity()).to_raw(),
        );

        let mut model = models_keys
            .get(&Identifier::parse_from_str("resources/models/quit.obj"))
            .unwrap()
            .clone();
        model.instance_count = 1;
        model.first_instance = 0;
        commands.insert(0, model);
        self.renderer = Some(renderer);
        self.window = Some(window);
        self.pipeline = Some(pipeline);
        self.models = Some(models);
        self.models_keys = Some(models_keys);
        self.instances = Some(instances);
        self.commands = Some(commands);
        self.camera_bind_group = Some((camera_bind_group_layout, camera_bind_group));
        self.view_camera_buffer = Some(view_camera_buffer);
        self.projection_camera_buffer = Some(projection_camera_buffer);
    }
    fn window_event(&mut self, _el: &ActiveEventLoop, _wid: WindowId, event: WindowEvent) {
        if let WindowEvent::CloseRequested = event {
            // Aquí controlarías cierre
            exit(0);
        }
        if let WindowEvent::RedrawRequested = event {
            self.redraw(_el);
        }
        if let WindowEvent::Resized(ph) = event {
            let w = ph.width;
            let h = ph.height;
            let size = (w, h);
            let renderer = self.renderer.clone().unwrap();
            let mut renderer = renderer.write().unwrap();
            renderer.resize(&size);
        }
    }
}
impl<'a> State<'a> {
    pub fn redraw(&mut self, event_loop: &ActiveEventLoop) {
        let target = self.camera.target().clone();
        let eye = self.camera.eye();
        let radius = (eye.to_vec() - target).magnitude();
        let theta = Rad(self.yaw);
        let phi = Rad(0.0);
        /*
        *self.camera.eye() = Point3::new(
            target.x + radius * phi.cos() * theta.sin(),
            target.y + radius * phi.sin(),
            target.z + radius * phi.cos() * theta.cos(),
        );

        let view = self.camera.view();
        dbg!(view);
        self.renderer
            .as_ref()
            .unwrap()
            .read()
            .unwrap()
            .update_buffer(
                self.view_camera_buffer.as_ref().unwrap(),
                &[UniformMatrix::from_matrix(view)],
            );

        let projection = self.camera.projection();
        dbg!(projection);
        self.renderer
            .as_ref()
            .unwrap()
            .read()
            .unwrap()
            .update_buffer(
                self.projection_camera_buffer.as_ref().unwrap(),
                &[UniformMatrix::from_matrix(projection)],
            );
        */
        //println!("redraw");
        let window = self.window.as_ref().unwrap();
        let renderer = self.renderer.as_ref().unwrap();
        let renderer = renderer.read().unwrap();
        let pipeline = self.pipeline.as_ref().unwrap();
        let instances = self.instances.as_ref().unwrap();
        let models = self.models.as_ref().unwrap();
        let color = self.bg_color.read().unwrap();
        let commands = self.commands.as_ref().unwrap();
        let (_, camera_bind_group) = self.camera_bind_group.as_ref().unwrap();

        let (mut encoder, view, output) = renderer.create_encoder().unwrap();
        //println!("encored");
        let color = RenderPassColorAttachmentBuilder::from_color(
            color.r as f64,
            color.g as f64,
            color.b as f64,
            1.0,
        )
        .build(&view);
        {
            let mut render_pass = encoder.begin_render_pass(
                &RenderPassDescriptorBuilder::new("render pass")
                    .with_colors(&[Some(color)])
                    .build(),
            );

            render_pass.set_pipeline(pipeline);
            render_pass.set_models(models);
            render_pass.set_vertex_buffer(1, instances.buffer.slice(..));
            render_pass.set_bind_group(0, camera_bind_group, &[]);

            render_pass.draw_indexed_indirect(&commands.buffer, 0);
        }

        renderer.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        window.request_redraw();
        self.yaw = self.yaw + 0.001;
    }
}
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt};
fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env()) // usa RUST_LOG si está definida
        .with(fmt::layer()) // salida formateada en consola
        .init(); // inicia como el subscriber global

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut state = State::default();
    event_loop.run_app(&mut state)?;
    Ok(())
}

pub struct AppRenderPipeline<'a> {
    bind_group_layouts: &'a [&'a BindGroupLayout],
}
impl<'a> AppRenderPipeline<'a> {
    pub fn new(bind_group_layouts: &'a [&'a BindGroupLayout]) -> Self {
        AppRenderPipeline { bind_group_layouts }
    }
}
impl<'a> steamengine_wgpu_core::render::render_pipeline::RenderPipeline for AppRenderPipeline<'a> {
    fn label(&self) -> &str {
        "Render pipeline"
    }

    fn source(&self) -> &str {
        r#"
        // Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normals: vec3<f32>,
    @location(5) model_0: vec4<f32>,
    @location(6) model_1: vec4<f32>,
    @location(7) model_2: vec4<f32>,
    @location(8) model_3: vec4<f32>,
    @location(9) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> view: mat4x4<f32>;

@group(0) @binding(1)
var<uniform> projection: mat4x4<f32>;

@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    let model = mat4x4<f32>(
        input.model_0,
        input.model_1,
        input.model_2,
        input.model_3,
    );

    var out: VertexOutput;
    out.color = input.color;
    out.clip_position = view * projection * model * vec4<f32>(input.position, 1.0);
    return out;
}

// Fragment

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
        "#
    }

    fn buffers(&self) -> Vec<wgpu::VertexBufferLayout> {
        vec![
            steamengine_client::resources::model::Vertex::desc(),
            RawEntity::desc(),
        ]
    }
    fn layout(&self) -> wgpu::PipelineLayoutDescriptor {
        PipelineLayoutDescriptor {
            label: Some("Render pipeline layout"),
            bind_group_layouts: self.bind_group_layouts,
            push_constant_ranges: &[],
        }
    }
}

const INSTANCE_ATTRIBS: [VertexAttribute; 5] = wgpu::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4, 9 => Float32x4];

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct RawEntity {
    matrix: [[f32; 4]; 4],
    color: [f32; 4],
}

impl Vertex for RawEntity {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: (std::mem::size_of::<Self>()) as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &INSTANCE_ATTRIBS,
        }
    }
}

struct Entity {
    color: Color,
    matrix: Matrix4<f32>,
}
impl Entity {
    pub fn new(color: Color, matrix: Matrix4<f32>) -> Self {
        Self { color, matrix }
    }
    pub fn render<'a>(
        &self,
        render_pass: &RenderPass,
        renderer: &Renderer<'a>,
        color_buffer: &Buffer,
    ) {
        renderer.update_buffer(color_buffer, &[self.color]);
        //renderer.update_buffer(matrix_buffer, &[self.matrix]);
        //render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
    }
    pub fn to_raw(self) -> RawEntity {
        let matrix: [[f32; 4]; 4] = self.matrix.into();
        let color: [f32; 4] = [self.color.r, self.color.g, self.color.b, self.color.a];

        RawEntity { color, matrix }
    }
}
