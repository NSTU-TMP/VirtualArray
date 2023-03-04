use std::{
    fmt::Debug,
    io::{Read, Write},
    mem, slice,
    time::SystemTime,
};

use crate::bitmap::Bitmap;

#[derive(Debug)]
pub(crate) struct Page<T: Debug + Default + Clone> {
    bitmap: Bitmap,
    data: Vec<T>,
    pub handling_time: SystemTime,
    pub is_modified: bool,
    elements_count_on_page: usize,
    pub page_index: usize,
}

impl<T: Debug + Default + Clone> Page<T> {
    pub fn new(page_index: usize, elements_count_on_page: usize) -> Self {
        let mut data = Vec::with_capacity(elements_count_on_page);

        for _ in 0..elements_count_on_page {
            data.push(T::default());
        }

        Self {
            page_index,
            elements_count_on_page,
            bitmap: Bitmap::new(elements_count_on_page),
            data,
            handling_time: SystemTime::now(),
            is_modified: false,
        }
    }

    pub fn set(&mut self, index_on_page: usize, value: T) {
        debug_assert!(index_on_page < self.elements_count_on_page);
        self.is_modified = true;
        self.handling_time = SystemTime::now();

        self.data[index_on_page] = value;
        self.bitmap.set(index_on_page);
    }

    pub fn get(&self, index_on_page: usize) -> Option<&T> {
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
        let data_as_bytes = unsafe {
            let a = slice::from_raw_parts(
                // self.data.as_slice().as_ptr() as *const u8,
                self.data.as_ptr() as *const u8,
                mem::size_of::<T>() * self.data.len(),
            );

            a
        };

        writer.write_all(data_as_bytes).unwrap();

        self.bitmap.write(writer);
    }

    pub fn read<R: Read>(
        page_index: usize,
        elements_count_on_page: usize,
        reader: &mut R,
    ) -> Option<Self> {
        let mut buffer = vec![0; elements_count_on_page * mem::size_of::<T>()];

        if let Err(_) = reader.read_exact(&mut buffer) {
            return None;
        }

        let data = unsafe {
            slice::from_raw_parts(buffer.clone().as_ptr() as *const T, elements_count_on_page).to_vec()
        };

        Some(Self {
            page_index,
            bitmap: Bitmap::read(elements_count_on_page, reader)?,
            elements_count_on_page,
            handling_time: SystemTime::now(),
            is_modified: false,
            data,
        })
    }
}
