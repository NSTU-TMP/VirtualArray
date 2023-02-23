use std::{
    // fmt::Debug,
    fs::{File, OpenOptions},
    io::{Seek, Write},
    mem,
    path::Path,
};

use crate::{bitmap::calc_bitmap_byte_size, page::Page};

pub struct VirtualArray {
    file: File,
    pages: Vec<Page>,
    array_size: usize,
    page_size: usize,
    buffer_size: usize,
    count_of_elements_on_page: usize,
}

impl VirtualArray {
    const VM_SIGNATURE_SIZE: usize = 2 * mem::size_of::<u8>();

    pub fn new<'file_name>(
        file_name: &'file_name str,
        array_size: usize,
        buffer_size: usize,
        desired_page_size: usize,
    ) -> Self {
        let path = Path::new(file_name);

        let file = if !path.exists() {
            let mut f = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open(Path::new(file_name))
                .unwrap();

            f.seek(std::io::SeekFrom::Start(0)).unwrap();
            f.write_all(b"VM").unwrap();

            f
        } else {
            OpenOptions::new()
                .write(true)
                .read(true)
                .open(Path::new(file_name))
                .unwrap()
        };

        let page_size = if desired_page_size % mem::size_of::<u8>() == 0 {
            desired_page_size
        } else {
            desired_page_size + desired_page_size % mem::size_of::<u8>()
        };

        let count_of_elements_on_page = page_size / mem::size_of::<u8>();

        // dbg!(page_size);

        Self {
            file,
            buffer_size,
            array_size,
            pages: Vec::with_capacity(buffer_size),
            page_size,
            count_of_elements_on_page,
        }
    }

    pub fn set_element(&mut self, element_index: usize, value: u8) {
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

    fn insert_page(&mut self, page: Page) {
        for i in 0..self.pages.len() {
            if self.pages[i].page_index == page.page_index {
                self.pages[i] = page;
                return;
            }
        }

        self.remove_oldest_page();
        self.pages.push(page);
    }

    pub fn get_element(&mut self, element_index: usize) -> Option<u8> {
        debug_assert!(element_index < self.array_size);

        let element_index_on_page = self.get_element_index_on_page(element_index);

        let page = self.get_page(element_index)?;
        page.get(element_index_on_page)
    }

    pub fn remove_element(&mut self, element_index: usize) {
        debug_assert!(element_index < self.array_size);

        let page_index = self.get_page_index_by_element_index(element_index);
        let element_index_on_page = self.get_element_index_on_page(element_index);

        let mut page = match self.get_page(element_index) {
            Some(page) => page,
            None => Page::new(page_index, self.count_of_elements_on_page),
        };

        page.remove(element_index_on_page);
        self.insert_page(page);
    }

    fn get_page(&mut self, element_index: usize) -> Option<Page> {
        let page_index = self.get_page_index_by_element_index(element_index);

        match self.get_page_if_in_memory(page_index) {
            Some(page) => Some(page),
            None => self.read_page(page_index),
        }
    }

    fn read_page(&mut self, page_index: usize) -> Option<Page> {
        let offset = self.get_page_offset(page_index);
        self.file
            .seek(std::io::SeekFrom::Start(offset as u64))
            .unwrap();

        let page = Page::read(page_index, self.count_of_elements_on_page, &mut self.file)?;
        self.insert_page(page.clone());
        // print!("read\n");
        // dbg!(page.clone());

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

        // print!("save\n");
        // dbg!(page.clone());
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

    fn get_page_if_in_memory(&self, page_index: usize) -> Option<Page> {
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

impl Drop for VirtualArray {
    fn drop(&mut self) {
        for i in 0..self.pages.len() {
            self.save_page(i);
        }
    }
}
