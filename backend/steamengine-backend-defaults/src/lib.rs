use rkyv::{Archive, Deserialize, Serialize, rancor::Panic, to_bytes};
use steamengine_communication::ToPackage;

const RENDER: &str = "view.render";
const PLAYER_TRANSFORM: &str = "update.player.transform";
const PLAYER_EXIT: &str = "update.player.exit";
const PLAYER_INTERACT: &str = "update.player.interact";

#[derive(Serialize, Deserialize, Archive, Debug)]
pub struct RenderPackage {
    vertices: String,
    model: [[u8; 4]; 4],
    texture: String,
    shader: String,
}
impl ToPackage for RenderPackage {
    fn to_package(self) -> steamengine_communication::Package {
        let bytes = to_bytes::<Panic>(&self).unwrap().to_vec();
        steamengine_communication::Package::new(RENDER, bytes)
    }
}

#[derive(Serialize, Deserialize, Archive, Debug)]
pub struct TransformPlayerPackage {
    matrix: [[u8; 4]; 4],
}
impl ToPackage for TransformPlayerPackage {
    fn to_package(self) -> steamengine_communication::Package {
        let bytes = to_bytes::<Panic>(&self).unwrap().to_vec();
        steamengine_communication::Package::new(PLAYER_TRANSFORM, bytes)
    }
}

pub struct ExitPlayerPackage;
impl ToPackage for ExitPlayerPackage {
    fn to_package(self) -> steamengine_communication::Package {
        steamengine_communication::Package::new(PLAYER_EXIT, vec![])
    }
}

pub struct PlayerInteractPackage {
    key: char,
}
impl ToPackage for PlayerInteractPackage {
    fn to_package(self) -> steamengine_communication::Package {
        let bytes = self.key as u32;
        let bytes = bytes.to_be_bytes();
        steamengine_communication::Package::new(PLAYER_INTERACT, bytes.to_vec())
    }
}
impl PlayerInteractPackage {
    pub fn from_package(package: steamengine_communication::Package) -> PlayerInteractPackage {
        let chr = package.clone().get_char();

        PlayerInteractPackage { key: chr }
    }
}
