use rkyv::{api::{high::{access_pos}, root_position}, rancor::{Failure, Panic}, to_bytes};
use tokio::{io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt}, sync::mpsc::{Receiver, Sender}};
#[cfg(feature = "tokio-tcp")]
use tokio::net::{tcp::{OwnedReadHalf, OwnedWriteHalf}, TcpStream};

use crate::{errors::PCSError, ArchivedPackage, Package, ToPackage};

#[async_trait::async_trait]
pub trait AsyncReadPackageImpl {
    async fn read_package(&mut self) -> Result<Package, PCSError>;
}
#[async_trait::async_trait]
pub trait AsyncWritePackageImpl {
    async fn write_package<T: ToPackage + Send>(&mut self, package: T) -> Result<(), PCSError>;
    async fn write_raw_package(&mut self, package: Package) -> Result<(), PCSError>;
}

#[cfg(feature = "tokio-tcp")]
#[async_trait::async_trait]
impl AsyncReadPackageImpl for TcpStream {
    async fn read_package(&mut self) -> Result<Package, PCSError> {
        read(self).await
    }
}

#[cfg(feature = "tokio-tcp")]
#[async_trait::async_trait]
impl AsyncWritePackageImpl for TcpStream {
    async fn write_package<T: ToPackage + Send>(&mut self, package: T) -> Result<(), PCSError> {
        let package = package.to_package();
        write(self, package).await
    }
    async fn write_raw_package(&mut self, package: Package) -> Result<(), PCSError> {
        write(self, package).await
    }
}

#[cfg(feature = "tokio-tcp")]
#[async_trait::async_trait]
impl AsyncReadPackageImpl for OwnedReadHalf {
    async fn read_package(&mut self) -> Result<Package, PCSError> {
        read(self).await
    }
}

#[cfg(feature = "tokio-tcp")]
#[async_trait::async_trait]
impl AsyncWritePackageImpl for OwnedWriteHalf {
    async fn write_package<T: ToPackage + Send>(&mut self, package: T) -> Result<(), PCSError> {
        let package = package.to_package();
        write(self, package).await
    }
    async fn write_raw_package(&mut self, package: Package) -> Result<(), PCSError> {
        write(self, package).await
    }
}


pub async fn read<T: AsyncRead + AsyncReadExt + Unpin>(reader: &mut T) -> Result<Package, PCSError> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await?;
    let pos = root_position::<ArchivedPackage>(buf.len());
    let message_wrapper: &ArchivedPackage = access_pos::<_, Panic>(&buf, pos).expect("Error deserializing a package");
    let mut name = String::new();
    message_wrapper.name.clone_into(&mut name);
    let mut value = Vec::new();
    message_wrapper.value.clone_into(&mut value);
    Ok(Package { name, value, index: 0 })
}

pub async fn write<T: AsyncWrite + AsyncWriteExt + Unpin>(writer: &mut T, package: Package) -> Result<(), PCSError> {
    let bytes = to_bytes::<Failure>(&package).expect("Error serializing a package");
    writer.write_all(&bytes).await?;
    Ok(())
}

#[async_trait::async_trait]
impl AsyncWritePackageImpl for Sender<Package> {
    async fn write_package<T: ToPackage + Send>(&mut self, package: T) -> Result<(), PCSError> {
        let package = package.to_package();
        self.send(package).await?;
        Ok(())
    }
    async fn write_raw_package(&mut self, package: Package) -> Result<(), PCSError> {
        self.send(package).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl AsyncReadPackageImpl for Receiver<Package> {
    async fn read_package(&mut self) -> Result<Package, PCSError> {
        Ok(self.try_recv()?)
    }
}
