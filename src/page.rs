use std::{
    io::{Read, Write},
    mem, slice,
    time::SystemTime,
};

use crate::bitmap::Bitmap;

#[derive(Clone)]
pub(crate) struct Page<T: Clone> {
    bitmap: Bitmap,
    data: Vec<T>,
    pub handling_time: SystemTime,
    pub is_modified: bool,
    elements_count_on_page: usize,
    pub page_index: usize,
}

impl<T: Clone> Page<T> {
    pub fn new(page_index: usize, elements_count_on_page: usize) -> Self {
        Self {
            page_index,
            elements_count_on_page,
            bitmap: Bitmap::new(elements_count_on_page),
            data: Vec::with_capacity(elements_count_on_page),
            handling_time: SystemTime::now(),
            is_modified: false,
        }
    }

    // pub fn get_handling_time(&self) -> SystemTime {
    //     self.handling_time
    // }
    //
    // pub fn is_modfied(&self) -> bool {
    //     self.is_modified
    // }



    pub fn insert(&mut self, index_on_page: usize, value: T) {
        debug_assert!(index_on_page < self.elements_count_on_page);

        self.is_modified = true;
        self.handling_time = SystemTime::now();
        self.data.insert(index_on_page, value);
        self.bitmap.set(index_on_page);
    }

    pub fn get(&self, index_on_page: usize) -> Option<T> {
        debug_assert!(index_on_page < self.elements_count_on_page);

        if !self.bitmap.get(index_on_page) {
            return None;
        }

        Some(self.data[index_on_page].clone())
    }

    pub fn remove(&mut self, index_on_page: usize) {
        debug_assert!(index_on_page < self.elements_count_on_page);

        self.is_modified = true;
        self.handling_time = SystemTime::now();
        self.bitmap.unset(index_on_page);
    }

    pub fn write<W: Write>(&self, writer: &mut W) {
        self.bitmap.write(writer);

        let data_as_bytes = unsafe {
            slice::from_raw_parts(
                self.data.as_slice().as_ptr() as *const u8,
                mem::size_of::<Page<T>>(),
            )
        };

        writer.write_all(data_as_bytes);
    }

    pub fn read<R: Read>(page_index: usize, elements_count_on_page: usize, reader: &mut R) -> Self {
        let bitmap = Bitmap::read(elements_count_on_page, reader);

        let mut buffer = vec![0; elements_count_on_page];
        reader.read_exact(&mut buffer);

        let data: &[T] = unsafe { mem::transmute(buffer.as_slice()) };

        Self {
            bitmap,
            page_index,
            elements_count_on_page,
            handling_time: SystemTime::now(),
            is_modified: false,
            data: data.to_vec(),
        }
    }
}
