use std::{
    fs::{File, OpenOptions},
    marker::PhantomData,
};

use crate::Storage;

use super::{
    metadata::{self, Metadata},
    page, Result, VirtualArray, DEFAULT_SIGNATURE,
};

pub struct VirtualArrayBuilder<'signature, Source, Item, PSerializer, MSerializer, BufferSize> {
    source: Source,
    signature: &'signature [u8],
    page_serializer: PSerializer,
    metadata_serializer: MSerializer,
    buffer_size: BufferSize,
    _item_marker: PhantomData<Item>,
}

pub struct NoneType;

impl<'signature> VirtualArrayBuilder<'signature, NoneType, NoneType, NoneType, NoneType, NoneType> {
    pub fn from_storage(
        storage: impl Storage,
    ) -> VirtualArrayBuilder<
        'signature,
        impl Storage,
        NoneType,
        page::DefaultSerializer,
        metadata::DefaultSerializer,
        NoneType,
    > {
        VirtualArrayBuilder {
            source: storage,
            page_serializer: page::DefaultSerializer,
            metadata_serializer: metadata::DefaultSerializer,
            signature: DEFAULT_SIGNATURE,
            buffer_size: NoneType,
            _item_marker: PhantomData,
        }
    }

    pub fn from_file_name(
        file_name: &str,
    ) -> VirtualArrayBuilder<
        'signature,
        &str,
        NoneType,
        page::DefaultSerializer,
        metadata::DefaultSerializer,
        NoneType,
    > {
        VirtualArrayBuilder {
            source: file_name,
            page_serializer: page::DefaultSerializer,
            metadata_serializer: metadata::DefaultSerializer,
            signature: DEFAULT_SIGNATURE,
            buffer_size: NoneType,
            _item_marker: PhantomData,
        }
    }
}

impl<'signature, Source, Item, MSerializer, BufferSize>
    VirtualArrayBuilder<'signature, Source, Item, page::DefaultSerializer, MSerializer, BufferSize>
where
    Item: Default,
{
    pub fn page_serializer(
        self,
        page_serializer: impl page::Serializer<Item>,
    ) -> VirtualArrayBuilder<
        'signature,
        Source,
        Item,
        impl page::Serializer<Item>,
        MSerializer,
        BufferSize,
    > {
        VirtualArrayBuilder {
            source: self.source,
            signature: self.signature,
            page_serializer,
            metadata_serializer: self.metadata_serializer,
            buffer_size: self.buffer_size,
            _item_marker: PhantomData,
        }
    }
}

impl<'signature, Source, Item, PSerializer, BufferSize>
    VirtualArrayBuilder<
        'signature,
        Source,
        Item,
        PSerializer,
        metadata::DefaultSerializer,
        BufferSize,
    >
{
    pub fn metadata_serializer(
        self,
        metadata_serializer: impl metadata::Serializer,
    ) -> VirtualArrayBuilder<
        'signature,
        Source,
        Item,
        PSerializer,
        impl metadata::Serializer,
        BufferSize,
    > {
        VirtualArrayBuilder {
            source: self.source,
            signature: self.signature,
            page_serializer: self.page_serializer,
            metadata_serializer,
            buffer_size: self.buffer_size,
            _item_marker: PhantomData,
        }
    }
}

impl<'signature, Source, PSerializer, MSerializer, Item, BufferSize>
    VirtualArrayBuilder<'signature, Source, Item, PSerializer, MSerializer, BufferSize>
{
    pub fn signature(
        self,
        signature: &'signature [u8],
    ) -> VirtualArrayBuilder<'signature, Source, Item, PSerializer, MSerializer, BufferSize> {
        VirtualArrayBuilder {
            source: self.source,
            signature,
            page_serializer: self.page_serializer,
            metadata_serializer: self.metadata_serializer,
            buffer_size: self.buffer_size,
            _item_marker: PhantomData,
        }
    }
}

impl<'signature, Source, Item, PSerializer, MSerializer>
    VirtualArrayBuilder<'signature, Source, Item, PSerializer, MSerializer, NoneType>
{
    pub fn buffer_size(
        self,
        buffer_size: usize,
    ) -> VirtualArrayBuilder<'signature, Source, Item, PSerializer, MSerializer, usize> {
        VirtualArrayBuilder {
            source: self.source,
            signature: self.signature,
            page_serializer: self.page_serializer,
            metadata_serializer: self.metadata_serializer,
            buffer_size,
            _item_marker: PhantomData,
        }
    }
}

impl<'signature, Source, Item, PSerializer, MSerializer, BufferSize>
    VirtualArrayBuilder<'signature, Source, Item, PSerializer, MSerializer, BufferSize>
{
    pub fn item_type<I: Default>(
        self,
    ) -> VirtualArrayBuilder<'signature, Source, I, PSerializer, MSerializer, BufferSize> {
        VirtualArrayBuilder {
            source: self.source,
            signature: self.signature,
            page_serializer: self.page_serializer,
            metadata_serializer: self.metadata_serializer,
            buffer_size: self.buffer_size,
            _item_marker: PhantomData,
        }
    }
}

impl<'signature, Source, Item, PSerializer, MSerializer>
    VirtualArrayBuilder<'signature, Source, Item, PSerializer, MSerializer, usize>
where
    Source: Storage,
    Item: Default,
    PSerializer: page::Serializer<Item>,
    MSerializer: metadata::Serializer,
{
    pub fn create(
        mut self,
        array_size: usize,
        data_chunk_size: usize,
    ) -> Result<VirtualArray<'signature, Item, Source, PSerializer, MSerializer>> {
        let metadata = Metadata::new::<Item>(self.signature, data_chunk_size, array_size)?;
        MSerializer::serialize(&mut self.source, &metadata)?;
        self.source.flush()?;

        let elements_count_on_page = metadata.count_elements_on_page::<Item>();

        for i in 0..(array_size / elements_count_on_page + 1) {
            self.source
                .seek_to_page::<Item, PSerializer, MSerializer>(i, &metadata)?;
            PSerializer::serialize_zeroed(&mut self.source, elements_count_on_page)?;
            self.source.flush()?;
        }

        let virtual_array = VirtualArray {
            pages: Vec::with_capacity(self.buffer_size),
            metadata,
            buffer_size: self.buffer_size,
            storage: self.source,
            page_serializer: self.page_serializer,
            metadata_serializer: self.metadata_serializer,
        };

        Ok(virtual_array)
    }

    pub fn open(
        mut self,
    ) -> Result<VirtualArray<'signature, Item, Source, PSerializer, MSerializer>> {
        let metadata = MSerializer::deserialize::<Source, Item>(&mut self.source, self.signature)?;

        let virtual_array = VirtualArray {
            pages: Vec::with_capacity(self.buffer_size),
            metadata,
            buffer_size: self.buffer_size,
            storage: self.source,
            page_serializer: self.page_serializer,
            metadata_serializer: self.metadata_serializer,
        };

        Ok(virtual_array)
    }
}

impl<'signature, 'file_name, Item, PSerializer, MSerializer>
    VirtualArrayBuilder<'signature, &'file_name str, Item, PSerializer, MSerializer, usize>
where
    Item: Default,
    PSerializer: page::Serializer<Item>,
    MSerializer: metadata::Serializer,
{
    pub fn create(
        self,
        array_size: usize,
        data_chunk_size: usize,
    ) -> Result<VirtualArray<'signature, Item, File, PSerializer, MSerializer>> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(self.source)?;

        VirtualArrayBuilder {
            source: file,
            signature: self.signature,
            page_serializer: self.page_serializer,
            metadata_serializer: self.metadata_serializer,
            buffer_size: self.buffer_size,
            _item_marker: PhantomData,
        }
        .create(array_size, data_chunk_size)
    }

    pub fn open(self) -> Result<VirtualArray<'signature, Item, File, PSerializer, MSerializer>> {
        let file = OpenOptions::new()
            .create(false)
            .write(true)
            .read(true)
            .open(self.source)?;

        VirtualArrayBuilder {
            source: file,
            signature: self.signature,
            page_serializer: self.page_serializer,
            metadata_serializer: self.metadata_serializer,
            buffer_size: self.buffer_size,
            _item_marker: PhantomData,
        }
        .open()
    }
}
