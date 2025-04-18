use std::iter;

use rand::Rng;
use steamengine::{
    color_render_pass, exec,
    render::{Renderer, RendererBuilder},
    thread,
    threads::channel::{CommManager, Event, Message},
    windows::AppHandle,
};
use winit::{event_loop::EventLoopWindowTarget, window::WindowBuilder};

struct App {
    threads: CommManager,
    color: (f64, f64, f64),
}
impl AppHandle for App {
    fn redraw(
        &mut self,
        renderer: &Renderer,
        _control: &EventLoopWindowTarget<()>,
    ) -> Result<(), wgpu::SurfaceError> {
        let (mut encoder, view, output) = renderer.create_encoder().unwrap();
        {
            let mut _render_pass = encoder.begin_render_pass(&color_render_pass!(
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

    fn update(&mut self, _control: &EventLoopWindowTarget<()>) {
        self.threads.send_to(1, Message::Int(3)).unwrap();

        self.color.0 = if let Ok(Message::Float(color)) = self.threads.try_recv() {
            color as f64
        } else {
            self.color.0
        };
        self.color.1 = if let Ok(Message::Float(color)) = self.threads.try_recv() {
            color as f64
        } else {
            self.color.1
        };
        self.color.2 = if let Ok(Message::Float(color)) = self.threads.try_recv() {
            color as f64
        } else {
            self.color.2
        };
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
        self.threads.send_to(1, Message::Int(3)).unwrap();
        match key {
            winit::keyboard::Key::Character(_) => {
                self.color.0 = if let Ok(Message::Float(color)) = self.threads.recv() {
                    color as f64
                } else {
                    self.color.0
                };
                self.color.1 = if let Ok(Message::Float(color)) = self.threads.recv() {
                    color as f64
                } else {
                    self.color.1
                };
                self.color.2 = if let Ok(Message::Float(color)) = self.threads.recv() {
                    color as f64
                } else {
                    self.color.2
                };
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
    env_logger::init();
    let mut comm_manager = CommManager::new();
    let random = thread!(
        1,
        comm_manager,
        |channel: steamengine::threads::channel::Channel| {
            let mut rng = rand::thread_rng();
            loop {
                let rec = channel.recv();
                if let Ok(message) = rec {
                    match message {
                        Message::Int(num) => {
                            for _ in 0..num {
                                channel.send(0, Message::Float(rng.r#gen())).unwrap();
                            }
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
    };
    pollster::block_on(exec!(app, RendererBuilder::new()));
    random.join().unwrap();
}
