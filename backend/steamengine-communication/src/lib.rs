use std::io::{Read, Write};
#[cfg(feature = "tcp")]
use std::net::TcpStream;

use rkyv::{
    access, api::{
        high::HighValidator,
        low::access_pos,
        root_position,
    }, bytecheck::CheckBytes, de::Pool, rancor::{Failure, Panic, Source, Strategy}, to_bytes, with::Skip, Archive, Deserialize, Serialize
};

use crate::errors::PCSError;

macro_rules! define_get {
    ($name:ident, $type:ty) => {
        pub fn $name(&mut self) -> $type {
            const SIZE: usize = std::mem::size_of::<$type>();
            let bytes = self.get_cons_length::<{ SIZE }>();
            <$type>::from_be_bytes(bytes)
        }
    };
}


#[cfg(feature = "tokio-comp")]
/// Module with implementations for tokio
/// Enable it with feature `tokio-comp`
pub mod tokio;
/// Module with errors
pub mod errors;

/// this trait is for packet data, all packets to be sent have to implement this trait, it contains a function to convert the object to an instance of Package.
pub trait ToPackage: Send {
    fn to_package(self) -> Package;
}

/// This struct is the representation of a packet, it contains the data of the original packet in binary in its own buffer and can be read in parts for faster processing, as long as its encryption allows it.
#[derive(Serialize, Deserialize, Archive, Debug, Clone)]
pub struct Package {
    /// this is the target of the package, it is important that all package types are identified by this value
    pub name: String,
    /// Contains the original data in binary
    pub value: Vec<u8>,
    #[rkyv(with = Skip)]
    /// Used to read the binary in parts
    index: usize,
}
impl<'a> Package {
    /// this function deserializes the data in binary to an object as long as you know its type
    pub fn get<T, E>(&'a self) -> &'a T::Archived
    where
        T: Archive,
        T::Archived: for<'b> CheckBytes<HighValidator<'b, E>> + Deserialize<T, Strategy<Pool, E>>,
        E: Source,
    {
        access::<T::Archived, E>(&self.value).expect("Error deserializing a package")
    }
    define_get!(get_u32, u32);
    define_get!(get_u64, u64);
    define_get!(get_usize, usize);
    define_get!(get_f32, f32);
    define_get!(get_f64, f64);
    define_get!(get_i32, i32);
    define_get!(get_i64, i64);
    define_get!(get_isize, isize);
    define_get!(get_u8, u8);
    pub fn get_char(&mut self) -> char {
        let num = self.get_u32();
        char::from_u32(num).expect("Attempt to read a char failed")
    }
    pub fn get_str(&mut self, length: usize) -> String {
        let mut str = String::new();
        for i in 0..length {
            let ch = self.get_char();
            str.insert(i, ch);
        }
        str
    }
    /// allows to obtain a piece of the binary with dynamic length
    pub fn get_var_length(&mut self, length: usize) -> Vec<u8> {
        let start = self.index;
        let end = self.index + length;
        let value = &self.value[start..end];
        let value = value.to_vec();
        self.index = end;
        value
    }
    /// allows you to obtain a piece of the binary with a constant length
    pub fn get_cons_length<const N: usize>(&mut self) -> [u8; N] {
        let start = self.index;
        let end = self.index + N;
        let value = &self.value[start..end];
        let value: [u8; N] = value.try_into().expect("Fail to convert a slice to a vector");
        self.index = end;
        value
    }
    pub fn pop(&'a mut self) -> Option<u8> {
        self.value.pop()
    }
    pub fn new(name: &str, value: Vec<u8>) -> Self {
        Self {
            name: name.to_owned(),
            value,
            index: 0
        }
    }
}
/// reads a package to everything you implement read
pub fn read(reader: &mut impl Read) -> Result<Package, PCSError> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    let pos = root_position::<ArchivedPackage>(buf.len());
    let message_wrapper: &ArchivedPackage = access_pos::<_, Panic>(&buf, pos).expect("Error deserializing the package, package corrupt or incomplete");
    let mut name = String::new();
    message_wrapper.name.clone_into(&mut name);
    let mut value = Vec::new();
    message_wrapper.value.clone_into(&mut value);
    Ok(Package { name, value, index: 0 })
}

/// to everything that implements write
pub fn write(writer: &mut impl Write, package: Package) -> Result<(), PCSError> {
    let bytes = to_bytes::<Failure>(&package).expect("Error serilizing a package");
    writer.write_all(&bytes)?;
    Ok(())
}

/// trait containing a function to read a packet, TpcStream implements it if you have the tcp feature enabled.
pub trait ReadPackageImpl {
    fn read_package(&mut self) -> Result<Package, PCSError>;
}
/// trait containing a function to write a packet, TpcStream implements it if you have the tcp feature enabled.
pub trait WritePackageImpl {
    fn write_package<T: ToPackage>(&mut self, package: T) -> Result<(), PCSError>;
    fn write_raw_package(&mut self, package: Package) -> Result<(), PCSError>;
}

#[cfg(feature = "tcp")]
impl ReadPackageImpl for TcpStream {
    fn read_package(&mut self) -> Result<Package, PCSError> {
        read(self)
    }
}
#[cfg(feature = "tcp")]
impl WritePackageImpl for TcpStream {
    fn write_package<T: ToPackage>(&mut self, arena: &mut Arena, package: T) -> Result<(), PCSError> {
        let package = package.to_package(arena.acquire());
        write(self, arena, package)
    }
    fn write_raw_package(&mut self, arena: &mut Arena, package: Package) -> Result<(), PCSError> {
        write(self, arena, package)
    }
}
