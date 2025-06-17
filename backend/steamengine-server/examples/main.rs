use std::pin::Pin;

use redis::{Client, Connection};
use rkyv::{ser::allocator::Arena, util::AlignedVec};
use steamengine_pcs::{
    Package, ToPackage,
    tokio::{AsyncReadPackageImpl, AsyncWritePackageImpl},
};
use steamengine_server::{ConnectionTest, GlobalServerHandler, UserSessionHandler};
use steamengine_vks::VKSPersistent;
use tokio::{
    runtime::Runtime,
    sync::broadcast::{self, Receiver},
};
use tracing::{error, info, trace, Level};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
           .expect("setting default subscriber failed");

    let redis = Client::open("redis://127.0.0.1:7060").unwrap();
    steamengine_server::exec_online(UnicServer {}, GlobalServer {}, redis, "127.0.0.1:8090").await?;

    Ok(())
}

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

#[derive(Clone)]
pub struct UnicServer;

#[async_trait::async_trait]
impl UserSessionHandler for UnicServer {
    async fn execute<R, W, P>(
        &mut self,
        reader: &mut R,
        writer: &mut W,
        persistent: &mut P,
        broadcast: &mut Receiver<Package>,
    ) where
        R: AsyncReadPackageImpl + Send,
        W: AsyncWritePackageImpl + ConnectionTest + Send,
        P: VKSPersistent + Send,
    {

            writer
            .write_package(
                LegiblePackage::from_str("Connected successful to server"),
            )
            .await
            .unwrap();
            loop {

                if let Ok(package) = reader.read_package().await {
                    match package.name.as_str() {
                        "test.legible" => {
                            let message = String::from_utf8(package.value).unwrap();
                            info!("RECV {}", message);
                        },
                        _ => {}
                    }
                }

                let rec = broadcast.try_recv();
                if let Ok(package) = rec {
                    writer.write_raw_package(package).await.unwrap();
                }
            }
    }
}

#[derive(Clone)]
pub struct GlobalServer;

#[async_trait::async_trait]
impl GlobalServerHandler for GlobalServer {
    async fn execute<'a, T: steamengine_vks::VKSPersistent>(
        &mut self,
        persistent: &mut T,
        broadcast: &mut broadcast::Sender<Package>,
    ) {
        loop {
            if broadcast.receiver_count() > 0 {
                broadcast.send(LegiblePackage::from_str("broadcast").to_package()).unwrap();
            }
        }
    }
}
