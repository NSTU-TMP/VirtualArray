use std::{
    fmt::Debug,
    io::{Read, Write},
    mem, slice,
    time::SystemTime,
};

use crate::bitmap::Bitmap;

#[derive(Clone, Debug)]
pub(crate) struct Page {
    bitmap: Bitmap,
    data: Vec<u8>,
    pub handling_time: SystemTime,
    pub is_modified: bool,
    elements_count_on_page: usize,
    pub page_index: usize,
}

impl Page {
    pub fn new(page_index: usize, elements_count_on_page: usize) -> Self {

        Self {
            page_index,
            elements_count_on_page,
            bitmap: Bitmap::new(elements_count_on_page),
            data: vec![0; elements_count_on_page],
            handling_time: SystemTime::now(),
            is_modified: false,
        }
    }

    pub fn insert(&mut self, index_on_page: usize, value: u8) {
        debug_assert!(index_on_page < self.elements_count_on_page);

        self.is_modified = true;
        self.handling_time = SystemTime::now();
        self.data.insert(index_on_page, value);
        self.bitmap.set(index_on_page);
    }

    pub fn get(&self, index_on_page: usize) -> Option<u8> {
        debug_assert!(index_on_page < self.elements_count_on_page);

        if !self.bitmap.get(index_on_page) {
            return None;
        }

        dbg!(self.data[index_on_page].clone());
        Some(self.data[index_on_page].clone())
    }

    pub fn remove(&mut self, index_on_page: usize) {
        debug_assert!(index_on_page < self.elements_count_on_page);

        self.is_modified = true;
        self.handling_time = SystemTime::now();
        self.bitmap.unset(index_on_page);
    }

    pub fn write<W: Write>(&self, writer: &mut W) {
        writer.write_all(self.data.as_slice()).unwrap();

        self.bitmap.write(writer);
    }

    pub fn read<R: Read>(page_index: usize, elements_count_on_page: usize, reader: &mut R) -> Self {
        let mut buffer = vec![0; elements_count_on_page];
        reader.read_exact(&mut buffer);
        buffer.reverse();

        let bitmap = Bitmap::read(elements_count_on_page, reader);

        Self {
            bitmap,
            page_index,
            elements_count_on_page,
            handling_time: SystemTime::now(),
            is_modified: false,
            data: buffer,
        }
    }
}
