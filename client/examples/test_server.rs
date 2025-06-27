use std::{
    fs::{File, read_to_string},
    io::Write,
    net::{TcpListener, TcpStream},
    ops::Range,
    sync::{Arc, Mutex},
};

use cgmath::Vector3;
use cgmath::{Matrix4, SquareMatrix};
use redis::Client;
use rhai::{Dynamic, Engine, Scope};
use rkyv::{Archive, Deserialize, Serialize, rancor::Panic, rend::f32_le, ser::allocator::Arena};
use rustyline::{Editor, error::ReadlineError};
use steamengine_communication::{Package, ToPackage, WritePackageImpl};
use steamengine_persistent::Persistent;

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
        Package::new("change", bytes.to_vec())
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

#[repr(C)]
#[derive(
    Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize, Archive,
)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}
impl Color {
    const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

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

#[derive(Deserialize, Serialize, Archive, Debug, Clone)]
struct RenderComponent {
    matrix: [[f32; 4]; 4],
    color: Color,
}
impl RenderComponent {
    pub fn new(matrix: Matrix4<f32>, color: Color) -> Self {
        let matrix: [[f32; 4]; 4] = matrix.into();
        Self { matrix, color }
    }
}
struct Connection<P: Persistent> {
    stream: TcpStream,
    persistent: P,
}
impl<P: Persistent> Connection<P> {
    pub fn new(stream: TcpStream, persistent: P) -> Self {
        Self { stream, persistent }
    }
    pub fn entity(&mut self, id: u32, color: Color, matrix: Matrix4<f32>) {
        self.stream
            .write_package(ChangePackage::new(id, color, matrix))
            .unwrap();
    }
    pub fn range(&mut self, start: u32, end: u32) {
        self.stream
            .write_package(DrawRangePackage::new(start, end))
            .unwrap();
    }
    pub fn bg(&mut self, color: Color) {
        self.stream
            .write_package(BgChangePackage::new(color))
            .unwrap();
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
    pub fn new(start: u32, end: u32) -> Self {
        DrawRangePackage { end, start }
    }
}
impl ToPackage for DrawRangePackage {
    fn to_package(self) -> Package {
        let bytes = self.to_bytes();

        Package::new("change.range", bytes)
    }
}

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8090")?;
    let persistent = Client::open("redis://127.0.0.1:7060")?;
    let mut arena = Arena::new();

    loop {
        let (stream, addrs) = listener.accept()?;
        println!("Connecting to {}", addrs);
        let persistent = persistent.get_connection().unwrap();
        std::thread::spawn(move || {
            let mut engine = Engine::new();

            engine.register_type_with_name::<Matrix4<f32>>("Matrix4");
            engine.register_type_with_name::<Vector3<f32>>("Vector3");
            engine.register_type_with_name::<Color>("Color");

            engine.register_fn("Rgb", |r: i64, g: i64, b: i64| {
                Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
            });
            engine.register_fn("Rgba", |r: i64, g: i64, b: i64, a: i64| {
                Color::new(
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    a as f32 / 255.0,
                )
            });
            engine.register_fn("white", || Color::WHITE);
            engine.register_fn("red", || Color::RED);
            engine.register_fn("green", || Color::GREEN);
            engine.register_fn("blue", || Color::BLUE);
            engine.register_fn("black", || Color::BLACK);

            engine.register_fn("to_bytes", Color::to_bytes);
            engine.register_fn("from_bytes", Color::from_bytes);

            engine.register_fn("deg", |angle: f32| cgmath::Deg(angle));
            engine.register_fn("rad", |angle: f32| cgmath::Rad(angle));

            engine.register_fn("mat4", Matrix4::<f32>::identity);
            engine.register_fn("move", Matrix4::<f32>::from_translation);

            engine.register_fn("axisX", Vector3::<f32>::unit_x);
            engine.register_fn("axisY", Vector3::<f32>::unit_y);
            engine.register_fn("axisZ", Vector3::<f32>::unit_z);

            engine.register_fn("rotate", |axis: Vector3<f32>, angle: f32| {
                let deg = cgmath::Deg(angle);
                Matrix4::<f32>::from_axis_angle(axis, deg)
            });
            engine.register_fn("scale", Matrix4::<f32>::from_scale);

            engine.register_fn("vec3", Vector3::<f32>::new);

            let conn = Arc::new(Mutex::new(Connection::new(stream, persistent)));

            let conn_clone = conn.clone();
            engine.register_fn(
                "entity",
                move |id: i64, color: Color, matrix: Matrix4<f32>| {
                    let mut guard = conn_clone.lock().unwrap();
                    guard.entity(id as u32, color, matrix);
                },
            );

            engine.register_fn("*", |a: Matrix4<f32>, b: Matrix4<f32>| -> Matrix4<f32> {
                a * b
            });

            let conn_clone = conn.clone();
            engine.register_fn("range", move |start: i64, end: i64| {
                let mut guard = conn_clone.lock().unwrap();
                guard.range(start as u32, end as u32);
            });

            let conn_clone = conn.clone();
            engine.register_fn("bg", move |color: Color| {
                let mut guard = conn_clone.lock().unwrap();
                guard.bg(color);
            });

            line(&mut engine).unwrap();
        });
    }
}
fn line(engine: &mut Engine) -> anyhow::Result<()> {
    // Crear un editor (interfaz readline)
    let mut rl = Editor::<(), _>::new()?;
    let mut scope = Scope::new();

    if rl.load_history("history.txt").is_err() {
        println!("No se encontró historial previo.");
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let args: Vec<&str> = line.trim().split_whitespace().collect();
                if args[0] == "exit" {
                    break;
                }
                if args[0] == "exec" {
                    let file = args[1].clone();
                    let file = match std::fs::read_to_string(file) {
                        Ok(str) => str,
                        Err(err) => {
                            println!("{}", err);
                            continue;
                        }
                    };
                    match engine.eval_with_scope::<Dynamic>(&mut scope, &file) {
                        Ok(_) => {}
                        Err(err) => {
                            println!("{}", err);
                        }
                    }
                }
                match engine.eval_with_scope::<Dynamic>(&mut scope, &line) {
                    Ok(value) => {
                        println!("{}", value)
                    }
                    Err(err) => {
                        println!("{}", err);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history("history.txt").unwrap();

    Ok(())
}
