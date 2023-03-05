mod bitmap;
mod page;
mod virtual_array;

use std::{
    fmt::Debug,
    fs::File,
    io::Error,
    io::{Read, Seek, Write},
    mem,
};

use bitmap::{BitmapReaderWriter, DefaultBitmapReaderWriter};

pub use crate::virtual_array::{VirtualArray, VirtualArrayBuilder};

type BytesCount = usize;

pub trait Repository: Read + Write + Seek + Debug {
    fn seek_to_start(&mut self) {
        self.seek(std::io::SeekFrom::Start(0)).unwrap();
    }
}

fn get_page_offset<const SIGNATURE_SIZE: usize>(
    page_index: usize,
    page_size: usize,
    count_of_elements_on_page: usize,
) -> usize {
    SIGNATURE_SIZE * mem::size_of::<u8>()
        + mem::size_of::<usize>()
        + page_index
            * (page_size + DefaultBitmapReaderWriter::calc_bitmap_size(count_of_elements_on_page))
}

fn seek_to_page<const SIGNATURE_SIZE: usize>(
    repo: &mut dyn Repository,
    page_index: usize,
    page_size: usize,
    count_of_elements_on_page: usize,
) -> Result<u64, Error> {
    let offset =
        get_page_offset::<SIGNATURE_SIZE>(page_index, page_size, count_of_elements_on_page);
    repo.seek(std::io::SeekFrom::Start(offset as u64))
}

impl Repository for File {}
