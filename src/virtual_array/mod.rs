mod builder;
mod metadata;

use crate::{
    bitmap::BitmapReaderWriter,
    page::{Page, PageReaderWriter},
    seek_to_page, Repository,
};
use metadata::Metadata;
use std::fmt::Debug;

pub use builder::VirtualArrayBuilder;

const DEFAULT_SIGNATURE_SIZE: usize = 2;
const DEFAULT_SIGNATURE: [u8; DEFAULT_SIGNATURE_SIZE] = [b'V', b'M'];

#[derive(Debug)]
pub struct VirtualArray<
    const SIGNATURE_SIZE: usize,
    Item: Debug + Default + Clone,
    BitmapRW: BitmapReaderWriter,
    PageRW: PageReaderWriter<BitmapRW, Item>,
> {
    pub(in crate::virtual_array) pages: Vec<Page<Item>>,
    pub(in crate::virtual_array) metadata: Metadata<SIGNATURE_SIZE>,
    pub(in crate::virtual_array) buffer_size: usize,
    pub(in crate::virtual_array) count_of_elements_on_page: usize,
    pub(in crate::virtual_array) repository: Box<dyn Repository>,
    pub(in crate::virtual_array) page_rw: PageRW,
    pub(in crate::virtual_array) bitmap_rw: BitmapRW,
}

impl<
        const SIGNATURE_SIZE: usize,
        Item: Debug + Default + Clone,
        BitmapRW: BitmapReaderWriter,
        PageRW: PageReaderWriter<BitmapRW, Item>,
    > VirtualArray<SIGNATURE_SIZE, Item, BitmapRW, PageRW>
{
    pub fn set_element(&mut self, element_index: usize, value: Item) {
        debug_assert!(element_index < self.metadata.array_size);

        let page_index = self.get_page_index_by_element_index(element_index);

        let index_on_page = self.get_element_index_on_page(element_index);

        let page = self.get_page(page_index);

        page.set(index_on_page, value);
    }

    pub fn get_element(&mut self, element_index: usize) -> Option<&Item> {
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

    fn get_page<'a>(&'a mut self, page_index: usize) -> &'a mut Page<Item> {
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

    fn read_page(&mut self, page_index: usize) -> &mut Page<Item> {
        seek_to_page::<SIGNATURE_SIZE>(
            self.repository.as_mut(),
            page_index,
            self.metadata.page_size,
            self.count_of_elements_on_page,
        )
        .unwrap();

        let page = PageRW::read(
            &mut self.repository,
            page_index,
            self.count_of_elements_on_page,
        )
        .unwrap();

        let index = self.insert_page(page);
        self.pages.get_mut(index).unwrap()
    }

    fn insert_page(&mut self, page: Page<Item>) -> usize {
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

        seek_to_page::<SIGNATURE_SIZE>(
            self.repository.as_mut(),
            page.page_index,
            self.metadata.page_size,
            self.count_of_elements_on_page,
        )
        .unwrap();

        PageRW::write(&mut self.repository, page);
        self.repository.flush().unwrap();
    }

    fn get_page_index_by_element_index(&self, element_index: usize) -> usize {
        element_index / self.count_of_elements_on_page
    }

    fn get_element_index_on_page(&self, element_index: usize) -> usize {
        element_index % self.count_of_elements_on_page
    }
}

impl<
        const SIGNATURE_SIZE: usize,
        Item: Debug + Default + Clone,
        BitmapRW: BitmapReaderWriter,
        PageRW: PageReaderWriter<BitmapRW, Item>,
    > Drop for VirtualArray<SIGNATURE_SIZE, Item, BitmapRW, PageRW>
{
    fn drop(&mut self) {
        for i in 0..self.pages.len() {
            self.save_page(i);
        }
    }
}
