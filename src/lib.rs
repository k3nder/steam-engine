#[macro_use]
pub mod threads;
#[macro_use]
pub mod windows;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread, time::Duration};

    use winit::event::WindowEvent;

    use crate::{
        threads::channel::{self, Channel, CommManager, Message},
        windows::AppHandle,
    };

    use super::*;

    #[test]
    fn test() {
        struct App;
        impl AppHandle for App {
            fn redraw(&mut self) {
                todo!()
            }

            fn update(&mut self) {
                todo!()
            }

            fn on(&mut self, event: &WindowEvent) -> bool {
                false
            }
        }
        impl App {
            fn new() -> Self {
                Self
            }
        }
        exec!(App::new());
    }
}
