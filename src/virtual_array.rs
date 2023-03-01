use std::{
    fmt::Debug,
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    mem,
    ops::IndexMut,
    path::Path,
};

use crate::{bitmap::calc_bitmap_byte_size, page::Page, BufferStream};

pub struct VirtualArray<Storage: BufferStream, T: Debug> {
    pages: Vec<Page<T>>,
    array_size: usize,
    page_size: usize,
    buffer_size: usize,
    count_of_elements_on_page: usize,
    storage: Storage,
}

impl<T: Debug> VirtualArray<File, T> {
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
        let mut size = Self::count_page_size(desired_page_size);
        let count_of_elements_on_page = size / mem::size_of::<T>();

        if !is_exist {
            file.seek(std::io::SeekFrom::Start(0)).unwrap();
            file.write_all(b"VM").unwrap();
            // file.seek(std::io::SeekFrom::Start(Self::VM_SIGNATURE_SIZE as u64))
            //     .unwrap();

            file.write(&size.to_be_bytes()).unwrap();
            let page = Page::<T>::new(0, count_of_elements_on_page);

            for i in 0..(array_size / count_of_elements_on_page + 1) {
                dbg!((array_size / count_of_elements_on_page + 1));
                let offset = Self::get_page_offset(i, size, count_of_elements_on_page);
                file.seek(std::io::SeekFrom::Start(offset as u64)).unwrap();
                page.write(&mut file);
                file.flush().unwrap();
            }
            file.flush().unwrap();
        } else {
            file.seek(std::io::SeekFrom::Start(
                Self::VM_SIGNATURE_SIZE as u64 + Self::VM_PAGE_SIZE_VALUE as u64,
            ))
            .unwrap();
            let mut x = [0; mem::size_of::<usize>()];

            file.read_exact(&mut x).unwrap();
            size = usize::from_be_bytes(x);
        }

        Self::new(file, array_size, buffer_size, size)
    }
}

impl<Storage: BufferStream, T: Debug> VirtualArray<Storage, T> {
    const VM_SIGNATURE_SIZE: usize = 2 * mem::size_of::<u8>();
    const VM_PAGE_SIZE_VALUE: usize = mem::size_of::<usize>();

    pub fn new(
        mut storage: Storage,
        array_size: usize,
        buffer_size: usize,
        desired_page_size: usize,
    ) -> Option<Self> {
        let page_size = Self::count_page_size(desired_page_size);

        let count_of_elements_on_page = page_size / mem::size_of::<T>();

        storage.seek(std::io::SeekFrom::Start(0)).unwrap();

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
            desired_page_size + desired_page_size % mem::size_of::<T>()
        }
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
        let offset =
            Self::get_page_offset(page_index, self.page_size, self.count_of_elements_on_page);
        self.storage
            .seek(std::io::SeekFrom::Start(offset as u64))
            .unwrap();

        let page = if let Some(page) = Page::read(
            page_index,
            self.count_of_elements_on_page,
            &mut self.storage,
        ) {
            page
        } else {
            Page::new(page_index, self.count_of_elements_on_page)
        };

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

        let offset = Self::get_page_offset(
            page.page_index,
            self.page_size,
            self.count_of_elements_on_page,
        );
        self.storage
            .seek(std::io::SeekFrom::Start(offset as u64))
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

    fn get_page_offset(
        page_index: usize,
        page_size: usize,
        count_of_elements_on_page: usize,
    ) -> usize {
        let value = Self::VM_SIGNATURE_SIZE
            + Self::VM_PAGE_SIZE_VALUE
            + page_index * (page_size + calc_bitmap_byte_size(count_of_elements_on_page));
        value
    }
}

impl<Storage: BufferStream, T: Debug> Drop for VirtualArray<Storage, T> {
    fn drop(&mut self) {
        for i in 0..self.pages.len() {
            self.save_page(i);
        }
    }
}
