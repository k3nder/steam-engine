use cgmath::*;

pub type Color = Vector4<f32>;
pub type BytedColor = Vector4<u8>;

pub const WHITE: Color = Color {
    x: 1.0,
    y: 1.0,
    z: 1.0,
    w: 1.0,
};
pub const RED: Color = Color {
    x: 1.0,
    y: 0.0,
    z: 0.0,
    w: 1.0,
};
pub const GREEN: Color = Color {
    x: 0.0,
    y: 1.0,
    z: 0.0,
    w: 1.0,
};
pub const BLUE: Color = Color {
    x: 0.0,
    y: 0.0,
    z: 1.0,
    w: 1.0,
};
pub const BLACK: Color = Color {
    x: 0.0,
    y: 0.0,
    z: 0.0,
    w: 1.0,
};

pub fn byted_to_color(byted: BytedColor) -> Color {
    let x = byted.x as f32 / 255.0;
    let y = byted.y as f32 / 255.0;
    let z = byted.z as f32 / 255.0;
    let w = byted.w as f32 / 255.0;
    Color { x, y, z, w }
}
pub fn color_to_byted(color: Color) -> BytedColor {
    let x = (color.x * 255.0) as u8;
    let y = (color.y * 255.0) as u8;
    let z = (color.z * 255.0) as u8;
    let w = (color.w * 255.0) as u8;
    BytedColor { x, y, z, w }
}
pub fn brgba(r: u8, g: u8, b: u8, a: u8) -> BytedColor {
    BytedColor {
        x: r,
        y: b,
        z: g,
        w: a,
    }
}

pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color {
        x: r,
        y: b,
        z: g,
        w: a,
    }
}

pub fn brgb(r: u8, g: u8, b: u8) -> BytedColor {
    BytedColor {
        x: r,
        y: g,
        z: b,
        w: 255,
    }
}

pub fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color {
        x: r,
        y: g,
        z: b,
        w: 255.0,
    }
}
