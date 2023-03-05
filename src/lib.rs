mod bitmap;
mod page;
mod virtual_array;
mod builder;
mod metadata;

use std::{
    fs::File,
    io::Error,
    io::{Read, Seek, Write},
    mem,
};

use bitmap::calc_bitmap_byte_size;

pub use crate::virtual_array::VirtualArray;

pub trait Storage: Read + Write + Seek {
    fn get_page_offset(
        page_index: usize,
        page_size: usize,
        count_of_elements_on_page: usize,
    ) -> usize;

    fn seek_to_start(&mut self) -> Result<u64, Error>;

    fn seek_to_page(
        &mut self,
        page_index: usize,
        page_size: usize,
        count_of_elements_on_page: usize,
    ) -> Result<u64, Error>;
}

type BytesCount = usize;

impl Storage for File {
    fn seek_to_start(&mut self) -> Result<u64, Error> {
        self.seek(std::io::SeekFrom::Start(0))
    }

    fn get_page_offset(
        page_index: usize,
        page_size: usize,
        count_of_elements_on_page: usize,
    ) -> usize {
        2 * mem::size_of::<u8>()
            + mem::size_of::<usize>()
            + page_index * (page_size + calc_bitmap_byte_size(count_of_elements_on_page))
    }

    fn seek_to_page(
        &mut self,
        page_index: usize,
        page_size: usize,
        count_of_elements_on_page: usize,
    ) -> Result<u64, Error> {
        let offset = File::get_page_offset(page_index, page_size, count_of_elements_on_page);
        self.seek(std::io::SeekFrom::Start(offset as u64))
    }
}
