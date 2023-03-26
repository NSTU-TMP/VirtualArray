mod serializer;

pub use serializer::*;
use std::{error::Error, fmt::Display, mem};

#[derive(Debug)]
pub struct Metadata<'signature> {
    pub signature: &'signature [u8],
    pub data_chunk_size: usize,
    pub array_size: usize,
    _private: (),
}

impl<'signature> Metadata<'signature> {
    pub fn new<Item>(
        signature: &'signature [u8],
        data_chunk_size: usize,
        array_size: usize,
    ) -> Result<Self, ConstructError> {
        let metadata = Metadata {
            signature,
            data_chunk_size,
            array_size,
            _private: (),
        };

        if metadata.data_chunk_size == 0 {
            return Err(ConstructError::ZeroDataChunkSize);
        }

        if metadata.count_elements_on_page::<Item>() == 0 {
            return Err(ConstructError::SmallDataChunkSize);
        }

        Ok(metadata)
    }

    pub(crate) fn count_elements_on_page<Item>(&self) -> usize {
        self.data_chunk_size / mem::size_of::<Item>()
    }
}

#[derive(Debug)]
pub enum ConstructError {
    ZeroDataChunkSize,
    SmallDataChunkSize,
}

impl Display for ConstructError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SmallDataChunkSize => write!(f, "page size is too small for at least one item"),
            Self::ZeroDataChunkSize => write!(f, "page must be non-zero size"),
        }
    }
}

impl Error for ConstructError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
