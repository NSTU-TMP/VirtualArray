use crate::bitmap::{BitmapReaderWriter, DefaultBitmapReaderWriter};
use crate::page::{DefaultPageReaderWriter, Page, PageReaderWriter};
use crate::{BytesCount, Repository};
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::marker::PhantomData;
use std::path::Path;

use super::metadata::Metadata;
use super::{VirtualArray, DEFAULT_SIGNATURE, DEFAULT_SIGNATURE_SIZE};

pub struct NoneValue;

enum BuilderScheme {
    FromRepository(Box<dyn Repository>),
    FromFileName(String),
}

pub struct VirtualArrayBuilder<
    const SIGNATURE_SIZE: usize,
    ArraySize,
    BufferSize,
    PageSize,
    Item,
    BitmapRW,
    PageRW,
> {
    scheme: BuilderScheme,
    array_size: ArraySize,
    buffer_size: BufferSize,
    desired_page_size: PageSize,
    signature: [u8; SIGNATURE_SIZE],
    bitmap_rw: BitmapRW,
    page_rw: PageRW,
    _item_marker: PhantomData<Item>,
}

impl<'file_name>
    VirtualArrayBuilder<
        { DEFAULT_SIGNATURE_SIZE },
        NoneValue,
        NoneValue,
        NoneValue,
        NoneValue,
        DefaultBitmapReaderWriter,
        DefaultPageReaderWriter,
    >
{
    pub fn from_repository(
        storage: Box<dyn Repository>,
    ) -> VirtualArrayBuilder<
        { DEFAULT_SIGNATURE_SIZE },
        NoneValue,
        NoneValue,
        NoneValue,
        NoneValue,
        DefaultBitmapReaderWriter,
        DefaultPageReaderWriter,
    > {
        VirtualArrayBuilder {
            scheme: BuilderScheme::FromRepository(storage),
            array_size: NoneValue,
            buffer_size: NoneValue,
            desired_page_size: NoneValue,
            signature: DEFAULT_SIGNATURE,
            page_rw: DefaultPageReaderWriter,
            bitmap_rw: DefaultBitmapReaderWriter,
            _item_marker: PhantomData,
        }
    }

    pub fn from_file_name(
        file_name: &'file_name str,
    ) -> VirtualArrayBuilder<
        { DEFAULT_SIGNATURE_SIZE },
        NoneValue,
        NoneValue,
        NoneValue,
        NoneValue,
        DefaultBitmapReaderWriter,
        DefaultPageReaderWriter,
    > {
        VirtualArrayBuilder {
            scheme: BuilderScheme::FromFileName(file_name.to_owned()),
            array_size: NoneValue,
            buffer_size: NoneValue,
            desired_page_size: NoneValue,
            signature: DEFAULT_SIGNATURE,
            page_rw: DefaultPageReaderWriter,
            bitmap_rw: DefaultBitmapReaderWriter,
            _item_marker: PhantomData,
        }
    }
}

impl<
        'file_name,
        const SIGNATURE_SIZE: usize,
        ArraySize,
        BufferSize,
        PageSize,
        Item,
        BitmapRW,
        PageRW,
    > VirtualArrayBuilder<SIGNATURE_SIZE, ArraySize, BufferSize, PageSize, Item, BitmapRW, PageRW>
{
    pub fn item_type<I: Clone + Default + Debug>(
        self,
    ) -> VirtualArrayBuilder<SIGNATURE_SIZE, ArraySize, BufferSize, PageSize, I, BitmapRW, PageRW>
    {
        VirtualArrayBuilder {
            scheme: self.scheme,
            array_size: self.array_size,
            buffer_size: self.buffer_size,
            desired_page_size: self.desired_page_size,
            signature: self.signature,
            page_rw: self.page_rw,
            bitmap_rw: self.bitmap_rw,
            _item_marker: PhantomData,
        }
    }
}

impl<'file_name, const SIGNATURE_SIZE: usize, BufferSize, PageSize, Item, BitmapRW, PageRW>
    VirtualArrayBuilder<SIGNATURE_SIZE, NoneValue, BufferSize, PageSize, Item, BitmapRW, PageRW>
{
    pub fn array_size(
        self,
        value: usize,
    ) -> VirtualArrayBuilder<SIGNATURE_SIZE, usize, BufferSize, PageSize, Item, BitmapRW, PageRW>
    {
        VirtualArrayBuilder {
            scheme: self.scheme,
            array_size: value,
            buffer_size: self.buffer_size,
            desired_page_size: self.desired_page_size,
            signature: self.signature,
            page_rw: self.page_rw,
            bitmap_rw: self.bitmap_rw,
            _item_marker: PhantomData,
        }
    }
}

impl<const SIGNATURE_SIZE: usize, ArraySize, PageSize, Item, BitmapRW, PageRW>
    VirtualArrayBuilder<SIGNATURE_SIZE, ArraySize, NoneValue, PageSize, Item, BitmapRW, PageRW>
{
    pub fn buffer_size(
        self,
        value: usize,
    ) -> VirtualArrayBuilder<SIGNATURE_SIZE, ArraySize, usize, PageSize, Item, BitmapRW, PageRW>
    {
        VirtualArrayBuilder {
            scheme: self.scheme,
            array_size: self.array_size,
            buffer_size: value,
            desired_page_size: self.desired_page_size,
            signature: self.signature,
            page_rw: self.page_rw,
            bitmap_rw: self.bitmap_rw,
            _item_marker: PhantomData,
        }
    }
}

impl<const SIGNATURE_SIZE: usize, ArraySize, BufferSize, Item, BitmapRW, PageRW>
    VirtualArrayBuilder<SIGNATURE_SIZE, ArraySize, BufferSize, NoneValue, Item, BitmapRW, PageRW>
{
    pub fn desired_page_size(
        self,
        value: usize,
    ) -> VirtualArrayBuilder<SIGNATURE_SIZE, ArraySize, BufferSize, usize, Item, BitmapRW, PageRW>
    {
        VirtualArrayBuilder {
            scheme: self.scheme,
            array_size: self.array_size,
            buffer_size: self.buffer_size,
            desired_page_size: value,
            signature: self.signature,
            page_rw: self.page_rw,
            bitmap_rw: self.bitmap_rw,
            _item_marker: PhantomData,
        }
    }
}

impl<const SIGNATURE_SIZE: usize, ArraySize, BufferSize, PageSize, Item, BitmapRW, PageRW>
    VirtualArrayBuilder<SIGNATURE_SIZE, ArraySize, BufferSize, PageSize, Item, BitmapRW, PageRW>
{
    pub fn signature(
        self,
        value: [u8; SIGNATURE_SIZE],
    ) -> VirtualArrayBuilder<SIGNATURE_SIZE, ArraySize, BufferSize, PageSize, Item, BitmapRW, PageRW>
    {
        VirtualArrayBuilder {
            scheme: self.scheme,
            array_size: self.array_size,
            buffer_size: self.buffer_size,
            desired_page_size: self.desired_page_size,
            signature: value,
            page_rw: self.page_rw,
            bitmap_rw: self.bitmap_rw,
            _item_marker: PhantomData,
        }
    }
}

impl<const SIGNATURE_SIZE: usize, ArraySize, BufferSize, PageSize, Item, BitmapRW, PageRW>
    VirtualArrayBuilder<SIGNATURE_SIZE, ArraySize, BufferSize, PageSize, Item, BitmapRW, PageRW>
where
    Item: Clone + Debug + Default,
    BitmapRW: BitmapReaderWriter,
    PageRW: PageReaderWriter<BitmapRW, Item>,
{
    pub fn page_reader_writer(
        self,
        value: PageRW,
    ) -> VirtualArrayBuilder<SIGNATURE_SIZE, ArraySize, BufferSize, PageSize, Item, BitmapRW, PageRW>
    {
        VirtualArrayBuilder {
            scheme: self.scheme,
            array_size: self.array_size,
            buffer_size: self.buffer_size,
            desired_page_size: self.desired_page_size,
            signature: self.signature,
            page_rw: value,
            bitmap_rw: self.bitmap_rw,
            _item_marker: PhantomData,
        }
    }
}

impl<const SIGNATURE_SIZE: usize, ArraySize, BufferSize, PageSize, Item, BitmapRW, PageRW>
    VirtualArrayBuilder<SIGNATURE_SIZE, ArraySize, BufferSize, PageSize, Item, BitmapRW, PageRW>
where
    Item: Clone + Debug + Default,
    BitmapRW: BitmapReaderWriter,
    PageRW: PageReaderWriter<BitmapRW, Item>,
{
    pub fn bitmap_reader_writer(
        self,
        value: BitmapRW,
    ) -> VirtualArrayBuilder<SIGNATURE_SIZE, ArraySize, BufferSize, PageSize, Item, BitmapRW, PageRW>
    {
        VirtualArrayBuilder {
            scheme: self.scheme,
            array_size: self.array_size,
            buffer_size: self.buffer_size,
            desired_page_size: self.desired_page_size,
            signature: self.signature,
            page_rw: self.page_rw,
            bitmap_rw: value,
            _item_marker: PhantomData,
        }
    }
}

impl<'file_name, const SIGNATURE_SIZE: usize, Item, BitmapRW, PageRW>
    VirtualArrayBuilder<SIGNATURE_SIZE, usize, usize, usize, Item, BitmapRW, PageRW>
where
    Item: Clone + Debug + Default,
    BitmapRW: BitmapReaderWriter,
    PageRW: PageReaderWriter<BitmapRW, Item>,
{
    pub fn create(self) -> VirtualArray<SIGNATURE_SIZE, Item, BitmapRW, PageRW> {
        let mut repository = match self.scheme {
            BuilderScheme::FromRepository(repository) => repository,
            BuilderScheme::FromFileName(file_name) => open_file(&file_name, true),
        };

        let metadata = Metadata {
            array_size: self.array_size,
            signature: self.signature,
            page_size: calc_page_size::<Item>(self.desired_page_size),
        };

        metadata.write(&mut repository).unwrap();
        repository.flush().unwrap();

        let count_of_elements_on_page = metadata.count_elements_on_page::<Item>();

        for i in 0..(self.array_size / count_of_elements_on_page + 1) {
            let page = Page::zeroed(i, count_of_elements_on_page);
            PageRW::write(&mut repository, &page);
            repository.flush().unwrap();
        }

        VirtualArray {
            repository,
            buffer_size: self.buffer_size,
            metadata,
            pages: Vec::with_capacity(self.buffer_size),
            count_of_elements_on_page,
            bitmap_rw: self.bitmap_rw,
            page_rw: self.page_rw,
        }
    }
}

impl<'file_name, const SIGNATURE_SIZE: usize, Item, BitmapRW, PageRW>
    VirtualArrayBuilder<SIGNATURE_SIZE, NoneValue, usize, NoneValue, Item, BitmapRW, PageRW>
where
    Item: Clone + Debug + Default,
    BitmapRW: BitmapReaderWriter,
    PageRW: PageReaderWriter<BitmapRW, Item>,
{
    pub fn open(self) -> VirtualArray<SIGNATURE_SIZE, Item, BitmapRW, PageRW> {
        let mut repository = match self.scheme {
            BuilderScheme::FromRepository(repository) => repository,
            BuilderScheme::FromFileName(file_name) => open_file(&file_name, false),
        };

        repository.seek_to_start();
        let metadata = Metadata::read(&mut repository).unwrap();

        let count_of_elements_on_page = metadata.count_elements_on_page::<Item>();

        VirtualArray {
            repository,
            buffer_size: self.buffer_size,
            metadata,
            pages: Vec::with_capacity(self.buffer_size),
            count_of_elements_on_page,
            bitmap_rw: self.bitmap_rw,
            page_rw: self.page_rw,
        }
    }
}

fn calc_page_size<Item>(desired_page_size: usize) -> BytesCount {
    if desired_page_size % std::mem::size_of::<Item>() == 0 {
        desired_page_size
    } else {
        desired_page_size
            + (std::mem::size_of::<Item>() - (desired_page_size % std::mem::size_of::<Item>()))
    }
}

fn open_file(file_name: &str, create: bool) -> Box<dyn Repository> {
    Box::new(
        OpenOptions::new()
            .create(create)
            .write(true)
            .read(true)
            .open(Path::new(file_name))
            .unwrap(),
    )
}
