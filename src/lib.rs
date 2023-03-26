mod builder;
pub mod metadata;
pub mod page;

pub use builder::VirtualArrayBuilder;

use std::{
    error::Error,
    fmt::{Debug, Display},
    fs::File,
    io::{Read, Seek, Write},
};

type BytesCount = usize;

pub trait Storage: Read + Write + Seek + Debug {
    fn seek_to_start(&mut self) -> std::io::Result<()> {
        self.seek(std::io::SeekFrom::Start(0))?;
        Ok(())
    }

    fn get_page_offset<Item, PSerializer, MSerializer>(
        page_index: usize,
        metadata: &metadata::Metadata,
    ) -> u64
    where
        Item: Default,
        PSerializer: page::Serializer<Item>,
        MSerializer: metadata::Serializer,
    {
        let count_of_elements_on_page = metadata.count_elements_on_page::<Item>();
        let metadata_size_in_bytes = MSerializer::get_metadata_size_in_bytes(metadata);
        let page_size_in_bytes = PSerializer::get_page_size_in_bytes(count_of_elements_on_page);

        (metadata_size_in_bytes + page_index * page_size_in_bytes) as u64
    }

    fn seek_to_page<Item, PSerializer, MSerializer>(
        &mut self,
        page_index: usize,
        metadata: &metadata::Metadata,
    ) -> std::io::Result<()>
    where
        Item: Default,
        PSerializer: page::Serializer<Item>,
        MSerializer: metadata::Serializer,
    {
        let offset = <Self as Storage>::get_page_offset::<Item, PSerializer, MSerializer>(
            page_index, metadata,
        );
        self.seek(std::io::SeekFrom::Start(offset))?;
        Ok(())
    }
}

use crate::page::Page;

impl Storage for File {}

const DEFAULT_SIGNATURE: &[u8] = &[b'V', b'M'];

#[derive(Debug)]
pub struct VirtualArray<'metadata, Item, Store, PSerializer, MSerializer>
where
    Item: Default,
    Store: Storage,
    PSerializer: page::Serializer<Item>,
    MSerializer: metadata::Serializer,
{
    metadata: metadata::Metadata<'metadata>,
    storage: Store,
    page_serializer: PSerializer,
    metadata_serializer: MSerializer,
    pages: Vec<Page<Item>>,
    buffer_size: usize,
}

impl<'metadata, Item, Store, PSerializer, MSerializer>
    VirtualArray<'metadata, Item, Store, PSerializer, MSerializer>
where
    Item: Default,
    Store: Storage,
    PSerializer: page::Serializer<Item>,
    MSerializer: metadata::Serializer,
{
    pub fn set(&mut self, element_index: usize, value: Item) -> Result<()> {
        let index_on_page = self.get_index_on_page(element_index);
        let page = self.get_page_by_element_index(element_index)?;

        page.set(index_on_page, value);
        self.save()
    }

    pub fn get(&mut self, element_index: usize) -> Result<Option<&Item>> {
        let index_on_page = self.get_index_on_page(element_index);
        let page = self.get_page_by_element_index(element_index)?;

        Ok(page.get(index_on_page))
    }

    pub fn delete(&mut self, element_index: usize) -> Result<()> {
        let index_on_page = self.get_index_on_page(element_index);
        let page = self.get_page_by_element_index(element_index)?;

        page.delete(index_on_page);
        self.save()
    }

    fn get_page_by_element_index(&mut self, element_index: usize) -> Result<&mut Page<Item>> {
        let page_index = self.get_page_index(element_index);
        self.get_page(page_index)
    }

    fn get_page(&mut self, page_index: usize) -> Result<&mut Page<Item>> {
        let buff_index = if let Some(found_page_index) =
            self.pages.iter().position(|page| page.index == page_index)
        {
            found_page_index
        } else {
            let readed_page = self.read_page(page_index)?;
            self.insert_page(readed_page)
        };

        Ok(&mut self.pages[buff_index])
    }

    fn read_page(&mut self, page_index: usize) -> Result<Page<Item>> {
        self.storage
            .seek_to_page::<Item, PSerializer, MSerializer>(page_index, &self.metadata)?;

        Ok(PSerializer::deserialize(
            &mut self.storage,
            page_index,
            self.metadata.count_elements_on_page::<Item>(),
        )?)
    }

    fn insert_page(&mut self, page_to_insert: Page<Item>) -> usize {
        if self.pages.len() < self.buffer_size {
            self.pages.push(page_to_insert);
            self.pages.len() - 1
        } else {
            let max_priority_pos = self.get_buff_max_priority_pos().unwrap();
            self.pages[max_priority_pos] = page_to_insert;

            max_priority_pos
        }
    }

    fn get_buff_max_priority_pos(&self) -> Option<usize> {
        self.pages
            .iter()
            .enumerate()
            .max_by(|(_, x), (_, y)| x.cmp_priorities(&y))
            .map(|(index, _)| index)
    }

    fn get_page_index(&self, element_index: usize) -> usize {
        element_index / self.metadata.count_elements_on_page::<Item>()
    }

    fn get_index_on_page(&self, element_index: usize) -> usize {
        element_index % self.metadata.count_elements_on_page::<Item>()
    }

    fn save(&mut self) -> Result<()> {
        for page in self.pages.iter().filter(|page| page.should_be_saved()) {
            self.storage
                .seek_to_page::<Item, PSerializer, MSerializer>(page.index, &self.metadata)?;
            PSerializer::serialize(&mut self.storage, page)?;
        }

        Ok(())
    }
}

impl<'metadata, Item, Store, PSerializer, MSerializer> Drop
    for VirtualArray<'metadata, Item, Store, PSerializer, MSerializer>
where
    Item: Default,
    Store: Storage,
    PSerializer: page::Serializer<Item>,
    MSerializer: metadata::Serializer,
{
    fn drop(&mut self) {
        self.save().unwrap();
    }
}

#[derive(Debug)]
pub enum VirtualArrayError {
    MetadataSerializationError(metadata::SerializationError),
    PageSerializationError(page::SerializationError),
    ConstructMetadataError(metadata::ConstructError),
    IoError(std::io::Error),
}

pub type Result<T> = std::result::Result<T, VirtualArrayError>;

impl Display for VirtualArrayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MetadataSerializationError(error) => Display::fmt(&error, f),
            Self::PageSerializationError(error) => Display::fmt(&error, f),
            Self::IoError(error) => Display::fmt(&error, f),
            Self::ConstructMetadataError(error) => Display::fmt(&error, f),
        }
    }
}

impl Error for VirtualArrayError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MetadataSerializationError(error) => Some(error),
            Self::PageSerializationError(error) => Some(error),
            Self::IoError(error) => Some(error),
            Self::ConstructMetadataError(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for VirtualArrayError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<metadata::SerializationError> for VirtualArrayError {
    fn from(error: metadata::SerializationError) -> Self {
        Self::MetadataSerializationError(error)
    }
}

impl From<page::SerializationError> for VirtualArrayError {
    fn from(error: page::SerializationError) -> Self {
        Self::PageSerializationError(error)
    }
}

impl From<metadata::ConstructError> for VirtualArrayError {
    fn from(error: metadata::ConstructError) -> Self {
        Self::ConstructMetadataError(error)
    }
}
