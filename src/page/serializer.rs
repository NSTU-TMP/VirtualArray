use std::{
    error::Error,
    fmt::Display,
    io::{Read, Write},
    mem, slice,
};

use super::{Bitmap, DataChunk, Page, PageError};

pub trait Serializer<Item> {
    fn serialize<Writer: Write>(writer: &mut Writer, page: &Page<Item>) -> SerializationResult<()>;

    fn serialize_zeroed<Writer: Write>(
        writer: &mut Writer,
        count_of_elements: usize,
    ) -> SerializationResult<()>;

    fn deserialize<Reader: Read>(
        reader: &mut Reader,
        page_index: usize,
        count_of_elements_on_page: usize,
    ) -> SerializationResult<Page<Item>>;

    fn get_page_size_in_bytes(count_of_elements_on_page: usize) -> usize;
}

#[derive(Debug)]
pub enum SerializationError {
    IoError(std::io::Error),
    PageError(PageError),
}

pub type SerializationResult<T> = Result<T, SerializationError>;

#[derive(Debug)]
pub struct DefaultSerializer;

impl<Item: Default> Serializer<Item> for DefaultSerializer {
    fn serialize<Writer: Write>(
        writer: &mut Writer,
        page: &Page<Item>,
    ) -> Result<(), SerializationError> {
        let data_chunk_bytes = Self::convert_items_to_bytes::<Item>(page.data_chunk.as_ref());
        let bitmap_bytes = page.bitmap.as_ref();

        writer.write_all(data_chunk_bytes)?;
        writer.write_all(bitmap_bytes)?;

        Ok(())
    }

    fn serialize_zeroed<Writer: Write>(
        writer: &mut Writer,
        count_of_elements: usize,
    ) -> SerializationResult<()> {
        writer.write_all(&vec![0; count_of_elements * mem::size_of::<Item>()])?;
        writer.write_all(&vec![0; Bitmap::calc_bitmap_size(count_of_elements)])?;

        Ok(())
    }

    fn deserialize<Reader: Read>(
        reader: &mut Reader,
        page_index: usize,
        elements_count_on_page: usize,
    ) -> SerializationResult<Page<Item>> {
        let data_chunk = Self::deserialize_data_chunk(reader, elements_count_on_page)?;
        let bitmap = Self::deserialize_bitmap(reader, elements_count_on_page)?;

        Ok(Page::new(page_index, bitmap, data_chunk)?)
    }

    fn get_page_size_in_bytes(elements_count_on_page: usize) -> usize {
        Bitmap::calc_bitmap_size(elements_count_on_page)
            + mem::size_of::<Item>() * elements_count_on_page
    }
}

impl DefaultSerializer {
    fn deserialize_data_chunk<Item, Reader>(
        reader: &mut Reader,
        elements_count_on_page: usize,
    ) -> SerializationResult<DataChunk<Item>>
    where
        Reader: Read,
    {
        let mut buffer = vec![0; elements_count_on_page * mem::size_of::<Item>()];
        reader.read_exact(&mut buffer)?;
        let items = Self::convert_bytes_to_items::<Item>(buffer, elements_count_on_page);

        Ok(DataChunk::from(items))
    }

    fn deserialize_bitmap<Reader>(
        reader: &mut Reader,
        elements_count_on_page: usize,
    ) -> SerializationResult<Bitmap>
    where
        Reader: Read,
    {
        let mut buffer = vec![0; Bitmap::calc_bitmap_size(elements_count_on_page)];
        reader.read_exact(&mut buffer)?;
        Ok(Bitmap::new(elements_count_on_page, buffer))
    }

    fn convert_bytes_to_items<Item>(bytes: Vec<u8>, elements_count_on_page: usize) -> Vec<Item> {
        assert_eq!(bytes.len() % mem::size_of::<Item>(), 0);
        assert_eq!(bytes.capacity() % mem::size_of::<Item>(), 0);

        let items = unsafe { Self::unchecked_convert_bytes_to_items(bytes) };
        assert_eq!(items.len(), elements_count_on_page);

        items
    }

    unsafe fn unchecked_convert_bytes_to_items<Item>(bytes: Vec<u8>) -> Vec<Item> {
        let mut bytes = mem::ManuallyDrop::new(bytes);

        let items = bytes.as_mut_ptr() as *mut Item;
        let len = bytes.len() / mem::size_of::<Item>();
        let capacity = bytes.capacity() / mem::size_of::<Item>();

        Vec::from_raw_parts(items, len, capacity)
    }

    fn convert_items_to_bytes<Item>(items: &[Item]) -> &[u8] {
        unsafe { Self::unchecked_convert_items_to_bytes(items) }
    }

    unsafe fn unchecked_convert_items_to_bytes<Item>(items: &[Item]) -> &[u8] {
        slice::from_raw_parts(
            items.as_ptr() as *const u8,
            mem::size_of::<Item>() * items.len(),
        )
    }
}

impl From<PageError> for SerializationError {
    fn from(page_error: PageError) -> Self {
        Self::PageError(page_error)
    }
}

impl From<std::io::Error> for SerializationError {
    fn from(io_error: std::io::Error) -> Self {
        Self::IoError(io_error)
    }
}

impl Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(io_error) => write!(f, "io error: {}", io_error),
            Self::PageError(page_error) => write!(f, "page error: {}", page_error),
        }
    }
}

impl Error for SerializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IoError(io_error) => Some(io_error),
            Self::PageError(page_error) => Some(page_error),
        }
    }
}
