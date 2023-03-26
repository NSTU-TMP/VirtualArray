use std::{
    error::Error,
    fmt::Display,
    io::{Read, Write},
    mem,
};

use crate::{
    metadata::{ConstructError, Metadata},
    BytesCount,
};

pub trait Serializer {
    fn serialize<Writer: Write>(
        writer: &mut Writer,
        metadata: &Metadata,
    ) -> SerializationResult<()>;

    fn deserialize<'signature, Reader: Read, Item>(
        reader: &mut Reader,
        signature: &'signature [u8],
    ) -> SerializationResult<Metadata<'signature>>;

    fn get_metadata_size_in_bytes(metadata: &Metadata) -> BytesCount;
}

#[derive(Debug)]
pub enum SerializationError {
    InvalidSignature { expected: Vec<u8>, found: Vec<u8> },
    IoError(std::io::Error),
    ConstructError(ConstructError),
}

pub type SerializationResult<T> = Result<T, SerializationError>;

#[derive(Debug)]
pub struct DefaultSerializer;

impl Serializer for DefaultSerializer {
    fn serialize<Writer: Write>(
        writer: &mut Writer,
        metadata: &Metadata,
    ) -> SerializationResult<()> {
        writer.write_all(metadata.signature)?;
        writer.write_all(metadata.data_chunk_size.to_ne_bytes().as_slice())?;
        writer.write_all(metadata.array_size.to_ne_bytes().as_slice())?;

        Ok(())
    }

    fn deserialize<'signature, Reader, Item>(
        reader: &mut Reader,
        signature: &'signature [u8],
    ) -> SerializationResult<Metadata<'signature>>
    where
        Reader: Read,
    {
        use std::mem::size_of;

        let mut buff = vec![0; signature.len()];
        reader.read_exact(&mut buff)?;

        if buff != signature {
            return Err(SerializationError::InvalidSignature {
                expected: signature.to_vec(),
                found: buff,
            });
        }

        let mut buff = [0u8; size_of::<usize>()];

        reader.read_exact(&mut buff)?;
        let data_chunk_size = usize::from_ne_bytes(buff);

        reader.read_exact(&mut buff)?;
        let array_size = usize::from_ne_bytes(buff);

        let metadata = Metadata::new::<Item>(signature, data_chunk_size, array_size)?;
        Ok(metadata)
    }

    fn get_metadata_size_in_bytes(metadata: &Metadata) -> BytesCount {
        mem::size_of::<u8>() * metadata.signature.len() + mem::size_of::<usize>() * 2
    }
}

impl From<std::io::Error> for SerializationError {
    fn from(io_error: std::io::Error) -> Self {
        Self::IoError(io_error)
    }
}

impl From<ConstructError> for SerializationError {
    fn from(construct_error: ConstructError) -> Self {
        Self::ConstructError(construct_error)
    }
}

impl Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSignature { expected, found } => write!(
                f,
                "invalid signature value (expected: {:?}, found: {:?})",
                expected, found
            ),
            Self::IoError(io_error) => io_error.fmt(f),
            Self::ConstructError(construct_error) => construct_error.fmt(f),
        }
    }
}

impl Error for SerializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidSignature { .. } => None,
            Self::ConstructError(_) => None,
            Self::IoError(io_error) => Some(io_error),
        }
    }
}
