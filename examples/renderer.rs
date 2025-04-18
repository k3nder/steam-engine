use std::iter;

use rand::{
    Rng, SeedableRng,
    rngs::{StdRng, ThreadRng},
};
use steamengine::{
    color_render_pass, exec,
    render::{Renderer, RendererBuilder},
    thread,
    threads::channel::{CommManager, Event, Message},
    windows::AppHandle,
};
use wgpu::QuerySet;
use winit::{
    event_loop::EventLoopWindowTarget, platform::x11::WindowBuilderExtX11, window::WindowBuilder,
};

struct App {
    threads: CommManager,
    color: (f64, f64, f64),
    rng: ThreadRng,
}
impl AppHandle for App {
    fn redraw(
        &mut self,
        renderer: &Renderer,
        control: &EventLoopWindowTarget<()>,
    ) -> Result<(), wgpu::SurfaceError> {
        let (mut encoder, view, output) = renderer.create_encoder().unwrap();
        {
            let mut render_pass = encoder.begin_render_pass(&color_render_pass!(
                self.color.0,
                self.color.1,
                self.color.2,
                view
            ));
        }

        renderer.queue().submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn update(&mut self, control: &EventLoopWindowTarget<()>) {
        self.color.0 = self.rng.r#gen();
        self.color.1 = self.rng.r#gen();
        self.color.2 = self.rng.r#gen();
    }
    fn on_resize(
        &mut self,
        new_size: &winit::dpi::PhysicalSize<u32>,
        renderer: &mut Renderer,
        _: &EventLoopWindowTarget<()>,
    ) -> bool {
        renderer.resize(new_size);
        true
    }
    fn on_close(&mut self, control: &EventLoopWindowTarget<()>) -> bool {
        self.threads.broadcast(Message::Event(Event::Exit)).unwrap();
        control.exit();
        true
    }
    fn on_keyboard(
        &mut self,
        key: winit::keyboard::Key,
        control: &EventLoopWindowTarget<()>,
    ) -> bool {
        let mut rng = rand::thread_rng();
        match key {
            winit::keyboard::Key::Character(c) => {
                self.color.0 = rng.r#gen();
                self.color.1 = rng.r#gen();
                self.color.2 = rng.r#gen();
                true
            }
            winit::keyboard::Key::Named(c) => {
                match c {
                    winit::keyboard::NamedKey::Escape => {
                        self.on_close(control);
                    }
                    _ => {}
                }
                true
            }
            _ => false,
        }
    }
    fn window(&self) -> winit::window::WindowBuilder {
        WindowBuilder::new().with_title("Windows")
    }
}

fn main() {
    let comm_manager = CommManager::from_threads(0..1);
    let audio = thread!(
        0,
        comm_manager,
        |channel: steamengine::threads::channel::Channel| {
            loop {
                let rec = channel.recv();
                if let Ok(message) = rec {
                    match message {
                        Message::String(str) => {
                            println!("Playsound: {}", str);
                        }
                        Message::Event(steamengine::threads::channel::Event::Exit) => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    );
    let app = App {
        threads: comm_manager,
        color: (0.0, 0.0, 0.0),
        rng: rand::thread_rng(),
    };
    pollster::block_on(exec!(app, RendererBuilder::new()));
    audio.join().unwrap();
}
