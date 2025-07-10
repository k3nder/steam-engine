use crate::buffers::*;
use crate::camera_controler::CameraController;
use crate::color::*;
use crate::pipeline::AppRenderPipeline;
use cgmath::*;
use hashbrown::HashMap;
use rayon::prelude::*;
use std::sync::Arc;
use std::sync::RwLock;
use steamengine_renderer::Renderer;
use steamengine_renderer::RendererBuilder;
use steamengine_renderer::render_pass::RenderPassColorAttachmentBuilder;
use steamengine_renderer::render_pass::RenderPassDescriptorBuilder;
use steamengine_renderer::render_pipeline::RenderPipeline;
use steamengine_renderer_util::bindings::Bindings;
use steamengine_renderer_util::bindings::CreateBindings;
use steamengine_renderer_util::camera::Camera;
use steamengine_renderer_util::camera::CameraBuffer;
use steamengine_renderer_util::camera::prespective::PrespectiveCamera;
use steamengine_renderer_util::depth_texture::DefaultDepthTexture;
use steamengine_renderer_util::depth_texture::DepthTexture;
use steamengine_renderer_util::depth_texture::RenderPassCreateDepthTexture;
use steamengine_renderer_util::resources::Identifier;
use steamengine_renderer_util::resources::model::ModelResourceLoader;
use steamengine_renderer_util::resources::model::Models;
use steamengine_renderer_util::resources::model::RenderPassAttachModels;
use steamengine_renderer_util::resources::texture::TextureResourceLoader;
use steamengine_renderer_util::simple_buffer::DrawQueueBuffer;
use steamengine_renderer_util::simple_buffer::SimpleBuffer;
use wgpu::BindGroup;
use wgpu::BindGroupLayout;
use wgpu::Buffer;
use wgpu::util::DrawIndexedIndirectArgs;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub struct State<'a> {
    window: Option<Arc<Window>>,
    renderer: Option<Arc<Renderer<'a>>>,
    atlas_bindings: Option<Bindings>,
    models: Option<Models>,
    models_keys: Option<HashMap<Identifier, DrawIndexedIndirectArgs>>,
    pipeline: Option<wgpu::RenderPipeline>,
    instances: Option<Arc<InstanceBuffer<'a>>>,
    bg_color: Arc<RwLock<Color>>,
    commands: Option<Arc<DrawQueueBuffer<'a>>>,
    camera_buffer: Option<CameraBuffer<'a, PrespectiveCamera>>,
    camera: PrespectiveCamera,
    camera_bindings: Option<Bindings>,
    camera_controler: CameraController,
    depth_texture: Option<steamengine_renderer_util::depth_texture::DefaultDepthTexture>,
}

impl Default for State<'_> {
    fn default() -> Self {
        Self {
            bg_color: Arc::new(RwLock::new(BLACK)),
            window: None,
            renderer: None,
            models: None,
            models_keys: None,
            pipeline: None,
            instances: None,
            commands: None,
            camera_buffer: None,
            camera: PrespectiveCamera::default(),
            camera_bindings: None,
            atlas_bindings: None,
            camera_controler: CameraController::new(0.2),
            depth_texture: None,
        }
    }
}

impl ApplicationHandler<()> for State<'static> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        let window = Arc::new(window);
        let size = window.inner_size();
        let size = (size.width, size.height);
        let renderer = pollster::block_on(
            RendererBuilder::new()
                .required_features(wgpu::Features::MULTI_DRAW_INDIRECT)
                .build(window.clone(), size),
        )
        .unwrap();

        self.camera = PrespectiveCamera::default();

        *self.camera.aspect_ratio() = (size.0 as f32 / size.1 as f32) as f32;

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
        let camera_buffer = CameraBuffer::<'_, PrespectiveCamera>::new(renderer.clone(), 1);
        let camera_bindings = steamengine_renderer_util::camera::create_bindings(
            renderer.clone(),
            camera_buffer.as_entrie(),
        );

        camera_buffer.set(0, &self.camera);
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

        let (models, models_keys) = ModelResourceLoader::new()
            .load_to_buffers("resources/models", renderer.clone())
            .expect("Failed to read models");

        let (textures, textures_keys) =
            TextureResourceLoader::new().load_to_atlas("resources/textures", renderer.clone());
        let tree_bounds = textures_keys
            .get(&Identifier::parse_from_str("resources/textures/tree.png"))
            .unwrap()
            .clone();

        let models_keys: HashMap<Identifier, DrawIndexedIndirectArgs> = models_keys
            .par_iter()
            .map(|(k, v)| (k.clone(), v.first().unwrap().clone()))
            .collect();

        let pipeline = AppRenderPipeline::new(&[&camera_bindings.layout(), &textures.layout()])
            .to_wgpu(&renderer);

        println!("CREATING INSTANCE BUFFER");
        let instances = Arc::new(InstanceBuffer::new(renderer.clone(), 90));
        println!("CREATING COMMANDS BUFFER");
        let commands = Arc::new(DrawQueueBuffer::new(renderer.clone(), 90));

        *self.bg_color.write().unwrap() = Color::new(1.0, 1.0, 1.0, 1.0);

        instances.set(
            0,
            Instance::new(
                Color::new(1.0, 0.0, 0.0, 1.0),
                Matrix4::from_translation(Vector3::new(0.0, 0.0, 1.0)),
                tree_bounds,
            ),
        );
        instances.set(
            1,
            Instance::new(
                Color::new(0.0, 1.0, 0.0, 1.0),
                Matrix4::from_translation(Vector3::new(3.0, 0.0, 0.0)),
                tree_bounds,
            ),
        );
        instances.set(
            2,
            Instance::new(
                WHITE,
                Matrix4::from_translation(vec3(3.0, 2.0, 1.0)),
                tree_bounds,
            ),
        );

        let mut model = models_keys
            .get(&Identifier::parse_from_str("resources/models/quit.obj"))
            .unwrap()
            .clone();

        model.instance_count = 2;
        model.first_instance = 0;
        commands.set(0, model);

        let mut model = models_keys
            .get(&Identifier::parse_from_str("resources/models/triangle.obj"))
            .unwrap()
            .clone();

        model.instance_count = 3;
        model.first_instance = 2;
        commands.set(1, model);

        let depth_texture = renderer.create_depth_texture::<DefaultDepthTexture>();

        self.renderer = Some(renderer);
        self.window = Some(window);
        self.pipeline = Some(pipeline);
        self.models = Some(models);
        self.models_keys = Some(models_keys);
        self.instances = Some(instances);
        self.commands = Some(commands);
        self.camera_bindings = Some(camera_bindings);
        self.camera_buffer = Some(camera_buffer);
        self.atlas_bindings = Some(textures);
        self.depth_texture = Some(depth_texture);
    }
    fn window_event(
        &mut self,
        _el: &ActiveEventLoop,
        _wid: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::CloseRequested = event {
            // Aquí controlarías cierre
            std::process::exit(0);
        }
        if let WindowEvent::RedrawRequested = event {
            self.redraw(_el);
        }
        if let WindowEvent::Resized(ph) = event {
            let w = ph.width;
            let h = ph.height;
            let size = (w, h);
            let renderer = self.renderer.clone().unwrap();
            renderer.resize(&size);

            self.depth_texture = Some(renderer.create_depth_texture::<DefaultDepthTexture>())
        }
        self.camera_controler.process_events(&event);
    }
}
impl<'a> State<'a> {
    pub fn redraw(&mut self, event_loop: &ActiveEventLoop) {
        //println!("redraw");
        let window = self.window.as_ref().unwrap();
        let renderer = self.renderer.as_ref().unwrap();
        let renderer = renderer.clone();
        let pipeline = self.pipeline.as_ref().unwrap();
        let instances = self.instances.as_ref().unwrap();
        let models = self.models.as_ref().unwrap();
        let color = self.bg_color.read().unwrap();
        let commands = self.commands.as_ref().unwrap();
        let binding = self.camera_bindings.as_ref().unwrap();
        let atlas_binding = self.atlas_bindings.as_ref().unwrap();
        let depth_texture = self.depth_texture.as_ref().unwrap();

        self.camera_controler.update_camera(&mut self.camera);
        self.camera_buffer.as_ref().unwrap().set(0, &self.camera);

        let (mut encoder, view, output) = renderer.create_encoder().unwrap();
        //println!("encored");
        let color = RenderPassColorAttachmentBuilder::from_color(
            color.x as f64,
            color.y as f64,
            color.z as f64,
            1.0,
        )
        .build(&view);
        {
            let mut render_pass = encoder.begin_render_pass(
                &RenderPassDescriptorBuilder::new("render pass")
                    .with_colors(&[Some(color)])
                    .with_depth(depth_texture.stencil_attachment())
                    .build(),
            );

            render_pass.set_pipeline(pipeline);
            render_pass.set_models(models);
            render_pass.set_vertex_buffer(1, instances.buffer().slice(..));
            render_pass.set_bind_group(0, binding.bind().clone().as_ref(), &[]);
            render_pass.set_bind_group(1, atlas_binding.bind().clone().as_ref(), &[]);

            render_pass.multi_draw_indexed_indirect(&commands.buffer(), 0, 2);
        }

        renderer.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        window.request_redraw();
    }
}
