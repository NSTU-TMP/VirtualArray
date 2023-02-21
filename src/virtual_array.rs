use std::{
    fs::{File, OpenOptions},
    io::{Seek, Write},
    fmt::Debug,
    mem,
    path::Path,
};

use crate::{bitmap::calc_bitmap_byte_size, page::Page};

pub struct VirtualArray<T: Clone + Debug> {
    file: File,
    pages: Vec<Page<T>>,
    array_size: usize,
    page_size: usize,
    buffer_size: usize,
    count_of_elements_on_page: usize,
}

impl<T: Clone + Debug> VirtualArray<T> {
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

        // let count_of_pages = array_size / count_of_elements_on_page;
        // for i in 0..count_of_pages {
        //     let offset = Self::VM_SIGNATURE_SIZE
        //         + i * (page_size + calc_bitmap_byte_size(count_of_elements_on_page));
        //
        //     file.seek(std::io::SeekFrom::Start(offset as u64)).unwrap();
        //     let mut buff = vec![0; page_size + calc_bitmap_byte_size(count_of_elements_on_page)];
        //     file.write_all(&mut buff).unwrap();
        // }

        dbg!(page_size);

        Self {
            file,
            buffer_size,
            array_size,
            pages: Vec::with_capacity(buffer_size),
            page_size,
            count_of_elements_on_page,
        }
    }

    pub fn insert_element(&mut self, element_index: usize, value: T) {
        debug_assert!(element_index < self.array_size);

        let page_index = self.get_page_index_by_element_index(element_index);
        let index_on_page = self.get_element_index_on_page(element_index);

        let mut page = match self.get_page(element_index) {
            Some(page) => page,
            None => Page::new(page_index, self.count_of_elements_on_page),
        };

        page.insert(index_on_page, value);
        self.insert_page(page);
    }

    fn get_page_index_by_element_index(&self, element_index: usize) -> usize {
        element_index / self.count_of_elements_on_page
    }

    fn get_element_index_on_page(&self, element_index: usize) -> usize {
        element_index % self.count_of_elements_on_page
    }

    fn insert_page(&mut self, page: Page<T>) {
        for i in 0..self.pages.len() {
            if self.pages[i].page_index == page.page_index {
                self.pages[i] = page;
                return;
            }
        }

        self.remove_oldest_page();
        self.pages.push(page);
    }

    pub fn get_element(&mut self, element_index: usize) -> Option<T> {
        debug_assert!(element_index < self.array_size);

        let element_index_on_page = self.get_element_index_on_page(element_index);

        let page = self.get_page(element_index)?;
        page.get(element_index_on_page)
    }

    fn get_page(&mut self, element_index: usize) -> Option<Page<T>> {
        let page_index = self.get_page_index_by_element_index(element_index);

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

        dbg!("read page", page.clone());
        Some(page)
    }

    fn save_page(&mut self, page_index_in_buffer: usize) {
        let page = self.pages[page_index_in_buffer].clone();

        if !page.is_modified {
            return;
        }

        let offset = self.get_page_offset(page.page_index);
        self.file
            .seek(std::io::SeekFrom::Start(offset as u64))
            .unwrap();

        page.write(&mut self.file);
    }

    fn remove_oldest_page(&mut self) {
        if self.pages.len() < self.buffer_size {
            return;
        }

        let Some(oldest_page_index_in_buffer) = self.get_oldest_page_index() else {
            return;
        };

        self.save_page(oldest_page_index_in_buffer);
        self.pages.remove(oldest_page_index_in_buffer);
    }

    fn get_oldest_page_index(&self) -> Option<usize> {
        if self.pages.len() == 0 {
            return None;
        }

        let mut oldest = 0;

        for i in 0..self.pages.len() {
            if self.pages[i].handling_time > self.pages[oldest].handling_time {
                oldest = i;
            }
        }

        Some(oldest)
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
        let value = Self::VM_SIGNATURE_SIZE
            + page_index * (self.page_size + calc_bitmap_byte_size(self.count_of_elements_on_page));
        value
    }
}

impl<T: Clone + Debug> Drop for VirtualArray<T> {
    fn drop(&mut self) {
        for i in 0..self.pages.len() {
            self.save_page(i);
        }
    }
}
