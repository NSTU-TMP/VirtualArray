use std::{
    fs::{File, OpenOptions},
    io::{Seek, Write},
    mem,
    path::Path,
};

use crate::{bitmap::calc_bitmap_byte_size, page::Page};

pub struct VirtualArray<T: Clone> {
    file: File,
    pages: Vec<Page<T>>,
    array_size: usize,
    page_size: usize,
    buffer_size: usize,
    count_of_elements_on_page: usize,
}

impl<T: Clone> VirtualArray<T> {
    const VM_SIGNATURE_SIZE: usize = 2 * mem::size_of::<u8>();

    pub fn new<'file_name>(
        file_name: &'file_name str,
        array_size: usize,
        buffer_size: usize,
        desired_page_size: usize,
    ) -> Self {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(Path::new(file_name))
            .unwrap();

        file.seek(std::io::SeekFrom::Start(0)).unwrap();
        file.write_all(b"VM").unwrap();

        let page_size = if desired_page_size % mem::size_of::<T>() == 0 {
            desired_page_size
        } else {
            desired_page_size + desired_page_size % mem::size_of::<T>()
        };

        let count_of_elements_on_page = page_size / mem::size_of::<T>();

        Self {
            file,
            buffer_size,
            array_size,
            pages: Vec::with_capacity(buffer_size),
            page_size,
            count_of_elements_on_page,
        }
    }

    pub fn insert_element(&mut self, index: usize, value: T) {
        debug_assert!(index < self.array_size);

        let page_index = index / self.count_of_elements_on_page;
        let element_index_on_page = index % self.count_of_elements_on_page;

        let mut page = match self.get_page(page_index) {
            Some(page) => page,
            None => Page::new(page_index, self.count_of_elements_on_page),
        };

        page.insert(element_index_on_page, value);
        self.insert_page(page);
    }

    fn insert_page(&mut self, page: Page<T>) {
        for i in 0..self.pages.len() {
            if self.pages[i].page_index == page.page_index {
                self.pages[i] = page;
                return;
            }
        }

        if self.pages.len() < self.buffer_size {
            self.remove_oldest_page();
        }
        self.pages.push(page);
    }

    pub fn get_element(&mut self, index: usize) -> Option<T> {
        debug_assert!(index < self.array_size);

        let page_index = self.array_size / self.count_of_elements_on_page;
        let element_index_on_page = self.array_size % self.count_of_elements_on_page;

        let page = self.get_page(page_index)?;
        page.get(element_index_on_page)
    }

    fn get_page(&mut self, page_index: usize) -> Option<Page<T>> {
        match self.get_page_if_in_memory(page_index) {
            Some(page) => Some(page),
            None => self.read_page(page_index),
        }
    }

    fn read_page(&mut self, page_index: usize) -> Option<Page<T>> {
        let offset = self.get_page_offset(page_index);
        self.file
            .seek(std::io::SeekFrom::Start(offset as u64))
            .unwrap();

        let page = Page::<T>::read(page_index, self.count_of_elements_on_page, &mut self.file);
        self.insert_page(page.clone());

        Some(page)
    }

    fn save_page(&mut self, page_index: usize) {
        let page = self.pages[page_index].clone();

        if !page.is_modified {
            return;
        }

        let offset = self.get_page_offset(page_index);
        self.file
            .seek(std::io::SeekFrom::Start(offset as u64))
            .unwrap();

        page.write(&mut self.file);
    }

    fn remove_oldest_page(&mut self) {
        let oldest_page_index = self.get_oldest_page_index();

        self.save_page(oldest_page_index);
        self.pages.remove(oldest_page_index);
    }

    fn get_oldest_page_index(&self) -> usize {
        let mut oldest = 0;

        for i in 0..self.pages.len() {
            if self.pages[i].handling_time > self.pages[oldest].handling_time {
                oldest = i;
            }
        }

        oldest
    }

    fn get_page_if_in_memory(&self, page_index: usize) -> Option<Page<T>> {
        for page in self.pages.iter() {
            if page.page_index == page_index {
                return Some(page.clone());
            }
        }

        None
    }

    fn get_page_offset(&self, page_index: usize) -> usize {
        Self::VM_SIGNATURE_SIZE
            + page_index * (self.page_size + calc_bitmap_byte_size(self.count_of_elements_on_page))
    }
}

impl<T: Clone> Drop for VirtualArray<T> {
    fn drop(&mut self) {
        for i in 0..self.pages.len() {
            self.save_page(i);
        }
    }
}
