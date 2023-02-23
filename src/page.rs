use std::{
    fmt::Debug,
    io::{Read, Write},
    time::SystemTime,
};

use crate::bitmap::Bitmap;

#[derive(Debug)]
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

        self.data[index_on_page] = value;
        self.bitmap.set(index_on_page);
    }

    pub fn get(&self, index_on_page: usize) -> Option<&u8> {
        debug_assert!(index_on_page < self.elements_count_on_page);

        if !self.bitmap.get(index_on_page) {
            return None;
        }

        self.data.get(index_on_page)
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

    pub fn read<R: Read>(
        page_index: usize,
        elements_count_on_page: usize,
        reader: &mut R,
    ) -> Option<Self> {
        let mut buffer = vec![0; elements_count_on_page];

        if let Err(_) = reader.read_exact(&mut buffer) {
            return None;
        }

        Some(Self {
            page_index,
            bitmap: Bitmap::read(elements_count_on_page, reader)?,
            elements_count_on_page,
            handling_time: SystemTime::now(),
            is_modified: false,
            data: buffer,
        })
    }
}
