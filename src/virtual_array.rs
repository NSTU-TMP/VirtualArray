use std::{
    fmt::Debug,
    fs::{File, OpenOptions},
    io::{Read, Write},
    mem::{self, size_of},
    path::Path,
};

use crate::{page::Page, BufferStream};

#[derive(Debug)]
pub struct VirtualArray<Storage: BufferStream, T: Debug + Default + Clone> {
    pages: Vec<Page<T>>,
    array_size: usize,
    page_size: usize,
    buffer_size: usize,
    count_of_elements_on_page: usize,
    storage: Storage,
}

impl<T: Debug + Default + Clone> VirtualArray<File, T> {
    pub fn from_file_name<'file_name>(
        file_name: &'file_name str,
        array_size: usize,
        buffer_size: usize,
        desired_page_size: usize,
    ) -> Option<Self> {
        let is_exist = Path::new(file_name).exists();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(Path::new(file_name))
            .unwrap();
        let mut page_size = Self::count_page_size(desired_page_size);

        if is_exist {
            file.seek_to_start().unwrap();
            let mut vm_buff = [0u8; 2];
            file.read_exact(&mut vm_buff).unwrap();

            if b"VM" != &vm_buff {
                panic!("File should start with VM signature");
            }

            let mut size_buff = [0u8; size_of::<usize>()];
            file.read_exact(&mut size_buff).unwrap();

            page_size = usize::from_be_bytes(size_buff);
        } else {
            let count_of_elements_on_page = Self::count_elements_on_page(page_size);

            file.seek_to_start().unwrap();
            file.write_all(b"VM").unwrap();

            file.write_all(&page_size.to_be_bytes()).unwrap();

            let page = Page::<T>::new(0, count_of_elements_on_page);

            for i in 0..(array_size / count_of_elements_on_page + 1) {
                file.seek_to_page(i, page_size, count_of_elements_on_page)
                    .unwrap();
                page.write(&mut file);
                file.flush().unwrap();
            }
            file.flush().unwrap();
        }

        Self::new(file, array_size, buffer_size, page_size)
    }
}

impl<Storage: BufferStream, T: Debug + Default + Clone> VirtualArray<Storage, T> {
    pub fn new(
        mut storage: Storage,
        array_size: usize,
        buffer_size: usize,
        desired_page_size: usize,
    ) -> Option<Self> {
        let page_size = Self::count_page_size(desired_page_size);

        let count_of_elements_on_page = Self::count_elements_on_page(page_size);

        storage.seek_to_start().unwrap();

        let mut buf: [u8; 2] = [0, 0];

        if let Err(_) = storage.read_exact(&mut buf) {
            None
        } else if buf[0] != 'V' as u8 || buf[1] != 'M' as u8 {
            None
        } else {
            Some(Self {
                storage,
                buffer_size,
                array_size,
                pages: Vec::with_capacity(buffer_size),
                page_size,
                count_of_elements_on_page,
            })
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
        debug_assert!(element_index < self.array_size);

        let page_index = self.get_page_index_by_element_index(element_index);

        let index_on_page = self.get_element_index_on_page(element_index);

        let page = self.get_page(page_index);

        page.set(index_on_page, value);
    }

    pub fn get_element(&mut self, element_index: usize) -> Option<&T> {
        debug_assert!(element_index < self.array_size);

        let element_index_on_page = self.get_element_index_on_page(element_index);
        let page_index = self.get_page_index_by_element_index(element_index);

        let page = self.get_page(page_index);
        page.get(element_index_on_page)
    }

    pub fn remove_element(&mut self, element_index: usize) {
        debug_assert!(element_index < self.array_size);

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
            .seek_to_page(page_index, self.page_size, self.count_of_elements_on_page)
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
                self.page_size,
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

impl<Storage: BufferStream, T: Debug + Default + Clone> Drop for VirtualArray<Storage, T> {
    fn drop(&mut self) {
        for i in 0..self.pages.len() {
            self.save_page(i);
        }
    }
}
