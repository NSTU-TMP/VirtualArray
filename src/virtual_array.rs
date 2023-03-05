use std::{
    fmt::Debug,
    fs::{File, OpenOptions},
    mem::{self, size_of},
    path::Path,
};

use crate::{metadata::Metadata, page::Page, Storage};

const SIGNATURE_SIZE: usize = 2;
const SIGNATURE: [u8; SIGNATURE_SIZE] = [b'V', b'M'];

#[derive(Debug)]
pub struct VirtualArray<S: Storage, T: Debug + Default + Clone> {
    pages: Vec<Page<T>>,
    metadata: Metadata<SIGNATURE_SIZE>,
    buffer_size: usize,
    count_of_elements_on_page: usize,
    storage: S,
}

impl<T: Debug + Default + Clone> VirtualArray<File, T> {
    pub fn create_from_file_name(
        file_name: &str,
        array_size: usize,
        buffer_size: usize,
        desired_page_size: usize,
    ) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(Path::new(file_name))
            .unwrap();

        Self::create(file, array_size, buffer_size, desired_page_size)
    }

    pub fn open_from_file_name(file_name: &str, buffer_size: usize) -> Self {
        let file = OpenOptions::new()
            .create(false)
            .write(true)
            .read(true)
            .open(Path::new(file_name))
            .unwrap();

        Self::open(file, buffer_size)
    }
}

impl<S: Storage, T: Debug + Default + Clone> VirtualArray<S, T> {
    pub fn create(
        mut storage: S,
        array_size: usize,
        buffer_size: usize,
        desired_page_size: usize,
    ) -> Self {
        let page_size = Self::count_page_size(desired_page_size);

        let count_of_elements_on_page = Self::count_elements_on_page(page_size);

        storage.seek_to_start().unwrap();

        let metadata = Metadata {
            array_size,
            signature: SIGNATURE,
            page_size,
        };

        metadata.write(&mut storage).unwrap();
        storage.flush();

        let page = Page::<T>::new(0, count_of_elements_on_page);

        for i in 0..(array_size / count_of_elements_on_page + 1) {
            storage
                .seek_to_page(i, page_size, count_of_elements_on_page)
                .unwrap();
            page.write(&mut storage);
            storage.flush().unwrap();
        }
        storage.flush().unwrap();

        Self {
            storage,
            buffer_size,
            metadata,
            pages: Vec::with_capacity(buffer_size),
            count_of_elements_on_page,
        }
    }

    pub fn open(mut storage: S, buffer_size: usize) -> Self {
        storage.seek_to_start().unwrap();

        let metadata = Metadata::read(&mut storage).unwrap();

        // let mut vm_buff = [0u8; 2];
        // storage.read_exact(&mut vm_buff).unwrap();
        //
        // if b"VM" != &vm_buff {
        //     panic!("File should start with VM signature");
        // }
        //
        // let mut size_buff = [0u8; size_of::<usize>()];
        // storage.read_exact(&mut size_buff).unwrap();
        //
        // let page_size = usize::from_be_bytes(size_buff);
        let count_of_elements_on_page = Self::count_elements_on_page(metadata.page_size);

        Self {
            storage,
            buffer_size,
            metadata,
            pages: Vec::with_capacity(buffer_size),
            count_of_elements_on_page,
        }
    }

    fn count_page_size(desired_page_size: usize) -> usize {
        if desired_page_size % mem::size_of::<T>() == 0 {
            desired_page_size
        } else {
            desired_page_size + (mem::size_of::<T>() - (desired_page_size % mem::size_of::<T>()))
        }
    }

    fn count_elements_on_page(page_size: usize) -> usize {
        page_size / mem::size_of::<T>()
    }

    pub fn set_element(&mut self, element_index: usize, value: T) {
        debug_assert!(element_index < self.metadata.array_size);

        let page_index = self.get_page_index_by_element_index(element_index);

        let index_on_page = self.get_element_index_on_page(element_index);

        let page = self.get_page(page_index);

        page.set(index_on_page, value);
    }

    pub fn get_element(&mut self, element_index: usize) -> Option<&T> {
        debug_assert!(element_index < self.metadata.array_size);

        let element_index_on_page = self.get_element_index_on_page(element_index);
        let page_index = self.get_page_index_by_element_index(element_index);

        let page = self.get_page(page_index);
        page.get(element_index_on_page)
    }

    pub fn remove_element(&mut self, element_index: usize) {
        debug_assert!(element_index < self.metadata.array_size);

        let page_index = self.get_page_index_by_element_index(element_index);
        let element_index_on_page = self.get_element_index_on_page(element_index);

        let page = self.get_page(page_index);
        page.remove(element_index_on_page);
    }

    fn get_page<'a>(&'a mut self, page_index: usize) -> &'a mut Page<T> {
        if let Some(index) = self.get_page_index_in_memory(page_index) {
            self.pages.get_mut(index).unwrap()
        } else {
            self.read_page(page_index)
        }
    }

    fn get_page_index_in_memory(&self, page_index: usize) -> Option<usize> {
        for (index, page) in self.pages.iter().enumerate() {
            if page.page_index == page_index {
                return Some(index);
            }
        }
        None
    }

    fn read_page(&mut self, page_index: usize) -> &mut Page<T> {
        self.storage
            .seek_to_page(
                page_index,
                self.metadata.page_size,
                self.count_of_elements_on_page,
            )
            .unwrap();

        let page = Page::read(
            page_index,
            self.count_of_elements_on_page,
            &mut self.storage,
        )
        .unwrap();

        let index = self.insert_page(page);
        self.pages.get_mut(index).unwrap()
    }

    fn insert_page(&mut self, page: Page<T>) -> usize {
        for i in 0..self.pages.len() {
            if self.pages[i].page_index == page.page_index {
                self.pages[i] = page;
                return i;
            }
        }

        self.remove_oldest_page();
        self.pages.push(page);
        self.pages.len() - 1
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

    fn save_page(&mut self, page_index_in_buffer: usize) {
        let page = self.pages.get(page_index_in_buffer).unwrap();
        if !page.is_modified {
            return;
        }
        self.storage
            .seek_to_page(
                page.page_index,
                self.metadata.page_size,
                self.count_of_elements_on_page,
            )
            .unwrap();
        page.write(&mut self.storage);
        self.storage.flush().unwrap();
    }

    fn get_page_index_by_element_index(&self, element_index: usize) -> usize {
        element_index / self.count_of_elements_on_page
    }

    fn get_element_index_on_page(&self, element_index: usize) -> usize {
        element_index % self.count_of_elements_on_page
    }
}

impl<S: Storage, T: Debug + Default + Clone> Drop for VirtualArray<S, T> {
    fn drop(&mut self) {
        for i in 0..self.pages.len() {
            self.save_page(i);
        }
    }
}
