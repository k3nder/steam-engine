use steamengine_pcs::{tokio::AsyncWritePackageImpl, Package, ToPackage};
use tokio::net::TcpStream;
use anyhow::Result;

pub struct LegiblePackage {
    message: String,
}
impl ToPackage for LegiblePackage {
    fn to_package(self) -> steamengine_pcs::Package {
        Package::new("test.legible", self.message.into_bytes())
    }
}
impl LegiblePackage {
    pub fn from_str(value: &str) -> Self {
        Self {
            message: value.to_owned(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut tpc = TcpStream::connect("127.0.0.1:8090").await?;

    tpc.write_package(LegiblePackage::from_str("VAA")).await?;

    Ok(())
}
