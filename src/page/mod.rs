mod bitmap;
mod data_chunk;
mod serializer;

pub use self::{bitmap::Bitmap, data_chunk::DataChunk, serializer::*};

use std::{error::Error, fmt::Display, time::SystemTime};

#[derive(Debug)]
pub struct Page<Item> {
    pub bitmap: Bitmap,
    pub data_chunk: DataChunk<Item>,
    pub(crate) index: usize,
    handling_time: SystemTime,
    is_modified: bool,
}

#[derive(Debug)]
pub enum PageError {
    BitmapTooSmall {
        bitmap_size: usize,
        number_of_items: usize,
    },
}

type PageResult<T> = Result<T, PageError>;

impl<Item> Page<Item> {
    pub fn new(index: usize, bitmap: Bitmap, data_chunk: DataChunk<Item>) -> PageResult<Self> {
        if Bitmap::calc_bitmap_size(data_chunk.as_ref().len()) != bitmap.as_ref().len() {
            return Err(PageError::BitmapTooSmall {
                bitmap_size: bitmap.as_ref().len(),
                number_of_items: data_chunk.as_ref().len(),
            });
        }

        Ok(Self {
            bitmap,
            data_chunk,
            handling_time: SystemTime::now(),
            is_modified: false,
            index,
        })
    }

    pub(crate) fn set(&mut self, index: usize, value: Item) {
        self.is_modified = true;
        self.handling_time = SystemTime::now();

        self.data_chunk.set(index, value);
        self.bitmap.set(index, true);
    }

    pub(crate) fn get(&self, index: usize) -> Option<&Item> {
        if !self.bitmap.get(index) {
            None
        } else {
            Some(self.data_chunk.get(index))
        }
    }

    pub(crate) fn delete(&mut self, index: usize) {
        self.is_modified = true;
        self.handling_time = SystemTime::now();
        self.bitmap.set(index, false);
    }

    pub(crate) fn cmp_priorities(&self, other: &Page<Item>) -> std::cmp::Ordering {
        self.handling_time.cmp(&other.handling_time)
    }

    pub(crate) fn should_be_saved(&self) -> bool {
        self.is_modified
    }
}

impl Display for PageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BitmapTooSmall {
                bitmap_size,
                number_of_items: size_of_items,
            } => write!(
                f,
                "bitmap ({} bytes or {} flags) is too small for {} items",
                bitmap_size,
                8 * bitmap_size,
                size_of_items
            ),
        }
    }
}

impl Error for PageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::BitmapTooSmall { .. } => None,
        }
    }
}
