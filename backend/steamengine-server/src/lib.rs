use redis::Client;
use redis::aio::MultiplexedConnection;
use steamengine_communication::Package;
use steamengine_communication::tokio::AsyncReadPackageImpl;
use steamengine_communication::tokio::AsyncWritePackageImpl;
use steamengine_persistent::VKSPersistent;
use tokio::io::AsyncWriteExt;
use tokio::net::ToSocketAddrs;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::Sender;
use tokio::{
    net::{TcpListener, tcp::OwnedWriteHalf},
    task,
};
use tracing::info;

use crate::errors::ServerError;

pub mod errors;
pub mod systems;

const BROADCAST_BUFFER_SIZE: usize = 32;

#[async_trait::async_trait]
pub trait UserSessionHandler: Clone + Send + Sync {
    async fn execute<R, W, P>(
        &mut self,
        reader: &mut R,
        writer: &mut W,
        persistent: &mut P,
        broadcast: &mut Receiver<Package>,
    ) where
        R: AsyncReadPackageImpl + Send,
        W: AsyncWritePackageImpl + ConnectionTest + Send,
        P: VKSPersistent + Send;
}
#[async_trait::async_trait]
pub trait GlobalServerHandler: Clone + Send + Sync {
    async fn execute<'a, T: VKSPersistent>(
        &mut self,
        persistent: &mut T,
        broadcast: &mut Sender<Package>,
    );
}

pub async fn exec_offline<U, G>(
    unic_handler: U,
    global_handler: G,
    persistent: Client,
    receiver: tokio::sync::mpsc::Receiver<Package>,
    sender: tokio::sync::mpsc::Sender<Package>,
) -> Result<(), ServerError>
where
    U: UserSessionHandler + 'static,
    G: GlobalServerHandler + 'static,
{
    let (mut tx, _) = broadcast::channel::<Package>(BROADCAST_BUFFER_SIZE);
    let unic_persistent = persistent_from_client(&persistent).await?;
    exec_unic(unic_handler, receiver, sender, unic_persistent, &tx).await;
    let global_persistent = persistent_from_client(&persistent).await?;
    exec_global(global_handler, &mut tx, global_persistent).await;

    Ok(())
}
pub async fn exec_online<U, G>(
    unic_handler: U,
    global_handler: G,
    persistent: Client,
    addrs: impl ToSocketAddrs,
) -> Result<(), ServerError>
where
    U: UserSessionHandler + 'static,
    G: GlobalServerHandler + 'static,
{
    let tpc = TcpListener::bind(addrs).await?;
    let (mut tx, _) = broadcast::channel::<Package>(BROADCAST_BUFFER_SIZE);
    let tx_users = tx.clone();
    let persistent_user = persistent.clone();
    let users = task::spawn(async move {
        loop {
            let (socket, addr) = tpc.accept().await?;
            info!("Connect user {} to server", addr.ip());
            let (reader, writer) = socket.into_split();
            let handler = unic_handler.clone();
            let persistent = persistent_from_client(&persistent_user).await?;
            exec_unic::<U, _, _, _>(handler, reader, writer, persistent, &tx_users).await;
        }
    });
    let handler = global_handler.clone();
    let persistent = persistent_from_client(&persistent).await?;
    exec_global(handler, &mut tx, persistent).await;
    let res = tokio::join!(users);
    let res: Result<(), ServerError> = res.0?;
    res
}
pub async fn exec_global<G, P>(mut handler: G, writer: &mut Sender<Package>, mut persistent: P)
where
    G: GlobalServerHandler + 'static,
    P: VKSPersistent + Send + 'static,
{
    let _ = handler.execute(&mut persistent, writer).await;
}
pub async fn exec_unic<U, R, W, P>(
    mut handler: U,
    mut reader: R,
    mut writer: W,
    mut persistent: P,
    broadcast: &Sender<Package>,
) where
    U: UserSessionHandler + 'static,
    R: AsyncReadPackageImpl + Send + 'static,
    W: AsyncWritePackageImpl + Send + ConnectionTest + 'static,
    P: VKSPersistent + Send + 'static,
{
    let mut rx = broadcast.subscribe();
    tokio::spawn(async move {
        let _ = handler
            .execute(&mut reader, &mut writer, &mut persistent, &mut rx)
            .await;
    });
}

pub enum ConnectionStatus {
    Connected,
    Disconnected,
}

#[async_trait::async_trait]
pub trait ConnectionTest {
    async fn ping(&mut self) -> ConnectionStatus;
}

#[async_trait::async_trait]
impl ConnectionTest for OwnedWriteHalf {
    async fn ping(&mut self) -> ConnectionStatus {
        match self.write_all(b"ping\n").await {
            Ok(_) => return ConnectionStatus::Connected,
            Err(_) => return ConnectionStatus::Disconnected,
        }
    }
}

#[async_trait::async_trait]
impl ConnectionTest for tokio::sync::mpsc::Sender<Package> {
    async fn ping(&mut self) -> ConnectionStatus {
        ConnectionStatus::Connected
    }
}

async fn persistent_from_client<'a>(
    client: &'a Client,
) -> Result<MultiplexedConnection, redis::RedisError> {
    Ok(client.create_multiplexed_tokio_connection().await?.0)
}
