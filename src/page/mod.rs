mod reader_writer;

use std::{fmt::Debug, time::SystemTime};

use crate::bitmap::{Bitmap, BitmapReaderWriter};
pub use reader_writer::{DefaultPageReaderWriter, PageReaderWriter};

#[derive(Debug)]
pub struct Page<Item>
where
    Item: Debug + Default + Clone,
{
    bitmap: Bitmap,
    data: Vec<Item>,
    pub(crate) handling_time: SystemTime,
    pub(crate) is_modified: bool,
    elements_count_on_page: usize,
    pub(crate) page_index: usize,
}

impl<Item> Page<Item>
where
    Item: Debug + Default + Clone,
{
    pub fn zeroed(page_index: usize, elements_count_on_page: usize) -> Self {
        let data = vec![Item::default(); elements_count_on_page];

        debug_assert_eq!(elements_count_on_page, data.len());

        Self {
            page_index,
            elements_count_on_page,
            bitmap: Bitmap::zeroed(elements_count_on_page),
            data,
            handling_time: SystemTime::now(),
            is_modified: false,
        }
    }
    pub fn new(
        page_index: usize,
        elements_count_on_page: usize,
        bitmap: Bitmap,
        data: Vec<Item>,
    ) -> Self {
        debug_assert_eq!(elements_count_on_page, data.len());

        Self {
            page_index,
            elements_count_on_page,
            bitmap,
            data,
            handling_time: SystemTime::now(),
            is_modified: false,
        }
    }

    pub fn as_ptr(&self) -> *const Item {
        self.data.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.elements_count_on_page
    }

    pub fn bitmap(&self) -> &Bitmap {
        &self.bitmap
    }

    pub fn set(&mut self, index_on_page: usize, value: Item) {
        debug_assert!(index_on_page < self.elements_count_on_page);
        self.is_modified = true;
        self.handling_time = SystemTime::now();

        self.data[index_on_page] = value;
        self.bitmap.set(index_on_page);
    }

    pub fn get(&self, index_on_page: usize) -> Option<&Item> {
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
}
