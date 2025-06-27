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
