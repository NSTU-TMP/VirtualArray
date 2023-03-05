use super::Page;
use crate::bitmap::BitmapReaderWriter;
use std::{
    fmt::Debug,
    io::{Read, Write},
    mem, slice,
};

pub trait PageReaderWriter<BitmapRW, Item>
where
    BitmapRW: BitmapReaderWriter,
    Item: Clone + Debug + Default,
{
    fn write<Writer: Write>(writer: &mut Writer, page: &Page<Item>);

    fn read<Reader: Read>(
        reader: &mut Reader,
        page_index: usize,
        elements_count_on_page: usize,
    ) -> Option<Page<Item>>;
}

pub struct DefaultPageReaderWriter;

impl<BitmapRW, Item> PageReaderWriter<BitmapRW, Item> for DefaultPageReaderWriter
where
    BitmapRW: BitmapReaderWriter,
    Item: Clone + Debug + Default,
{
    fn write<Writer: Write>(writer: &mut Writer, page: &Page<Item>) {
        let data_as_bytes = unsafe {
            slice::from_raw_parts(
                page.as_ptr() as *const u8,
                mem::size_of::<Item>() * page.len(),
            )
        };

        writer.write_all(data_as_bytes).unwrap();
        BitmapRW::write(writer, page.bitmap());
    }

    fn read<Reader: Read>(
        reader: &mut Reader,
        page_index: usize,
        elements_count_on_page: usize,
    ) -> Option<Page<Item>> {
        let mut buffer = vec![0; elements_count_on_page * mem::size_of::<Item>()];

        if let Err(_) = reader.read_exact(&mut buffer) {
            return None;
        }

        let data = unsafe {
            slice::from_raw_parts(
                buffer.clone().as_ptr() as *const Item,
                elements_count_on_page,
            )
            .to_vec()
        };

        Some(Page::new(
            page_index,
            elements_count_on_page,
            BitmapRW::read(reader, elements_count_on_page)?,
            data,
        ))
    }
}
