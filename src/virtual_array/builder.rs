use super::{Metadata, Storage, VirtualArray, DEFAULT_SIGNATURE, DEFAULT_SIGNATURE_SIZE};
use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::path::Path;

pub struct VirtualArrayBuilder<FileName, Storage, ArraySize, BufferSize, PageSize, Signature> {
    file_name: FileName,
    storage: Storage,
    array_size: ArraySize,
    buffer_size: BufferSize,
    desired_page_size: PageSize,
    signature: Signature,
}

pub struct NoneValue;

impl VirtualArrayBuilder<NoneValue, NoneValue, NoneValue, NoneValue, NoneValue, NoneValue> {
    pub fn new() -> Self {
        Self {
            file_name: NoneValue,
            storage: NoneValue,
            array_size: NoneValue,
            buffer_size: NoneValue,
            desired_page_size: NoneValue,
            signature: NoneValue,
        }
    }
}

impl<ArraySize, BufferSize, PageSize, Signature>
    VirtualArrayBuilder<NoneValue, NoneValue, ArraySize, BufferSize, PageSize, Signature>
{
    pub fn file_name<'file_name>(
        self,
        value: &'file_name str,
    ) -> VirtualArrayBuilder<&'file_name str, NoneValue, ArraySize, BufferSize, PageSize, Signature>
    {
        VirtualArrayBuilder {
            file_name: value,
            storage: NoneValue,
            array_size: self.array_size,
            buffer_size: self.buffer_size,
            desired_page_size: self.desired_page_size,
            signature: self.signature,
        }
    }
}

impl<ArraySize, BufferSize, PageSize, Signature>
    VirtualArrayBuilder<NoneValue, NoneValue, ArraySize, BufferSize, PageSize, Signature>
{
    pub fn storage<S: Storage>(
        self,
        value: S,
    ) -> VirtualArrayBuilder<NoneValue, S, ArraySize, BufferSize, PageSize, Signature> {
        VirtualArrayBuilder {
            file_name: NoneValue,
            storage: value,
            array_size: self.array_size,
            buffer_size: self.buffer_size,
            desired_page_size: self.desired_page_size,
            signature: self.signature,
        }
    }
}

impl<FileName, Storage, BufferSize, PageSize, Signature>
    VirtualArrayBuilder<FileName, Storage, NoneValue, BufferSize, PageSize, Signature>
{
    pub fn array_size(
        self,
        value: usize,
    ) -> VirtualArrayBuilder<FileName, Storage, usize, BufferSize, PageSize, Signature> {
        VirtualArrayBuilder {
            file_name: self.file_name,
            storage: self.storage,
            array_size: value,
            buffer_size: self.buffer_size,
            desired_page_size: self.desired_page_size,
            signature: self.signature,
        }
    }
}

impl<FileName, Storage, ArraySize, PageSize, Signature>
    VirtualArrayBuilder<FileName, Storage, ArraySize, NoneValue, PageSize, Signature>
{
    pub fn buffer_size(
        self,
        value: usize,
    ) -> VirtualArrayBuilder<FileName, Storage, ArraySize, usize, PageSize, Signature> {
        VirtualArrayBuilder {
            file_name: self.file_name,
            storage: self.storage,
            array_size: self.array_size,
            buffer_size: value,
            desired_page_size: self.desired_page_size,
            signature: self.signature,
        }
    }
}

impl<FileName, Storage, ArraySize, BufferSize, Signature>
    VirtualArrayBuilder<FileName, Storage, ArraySize, BufferSize, NoneValue, Signature>
{
    pub fn desired_page_size(
        self,
        value: usize,
    ) -> VirtualArrayBuilder<FileName, Storage, ArraySize, BufferSize, usize, Signature> {
        VirtualArrayBuilder {
            file_name: self.file_name,
            storage: self.storage,
            array_size: self.array_size,
            buffer_size: self.buffer_size,
            desired_page_size: value,
            signature: self.signature,
        }
    }
}

impl<FileName, Storage, ArraySize, BufferSize, PageSize>
    VirtualArrayBuilder<FileName, Storage, ArraySize, BufferSize, PageSize, NoneValue>
{
    pub fn signature<const SIGNATURE_SIZE: usize>(
        self,
        value: [u8; SIGNATURE_SIZE],
    ) -> VirtualArrayBuilder<FileName, Storage, ArraySize, BufferSize, PageSize, [u8; SIGNATURE_SIZE]>
    {
        VirtualArrayBuilder {
            file_name: self.file_name,
            storage: self.storage,
            array_size: self.array_size,
            buffer_size: self.buffer_size,
            desired_page_size: self.desired_page_size,
            signature: value,
        }
    }
}

impl<'file_name>
    VirtualArrayBuilder<&'file_name str, NoneValue, usize, usize, usize, NoneValue>
{
    pub fn create<T: Default + Debug + Clone>(
        self,
    ) -> VirtualArray<DEFAULT_SIGNATURE_SIZE, std::fs::File, T> {
        self.signature(DEFAULT_SIGNATURE).use_file_as_storage(true).create::<T>()
    }
}

impl<'file_name>
    VirtualArrayBuilder<&'file_name str, NoneValue, NoneValue, usize, NoneValue, NoneValue>
{
    pub fn open<T: Default + Debug + Clone>(
        self,
    ) -> VirtualArray<DEFAULT_SIGNATURE_SIZE, std::fs::File, T> {
        self.signature(DEFAULT_SIGNATURE).use_file_as_storage(false).open::<T>()
    }
}

impl<'file_name, const SIGNATURE_SIZE: usize>
    VirtualArrayBuilder<&'file_name str, NoneValue, usize, usize, usize, [u8; SIGNATURE_SIZE]>
{
    pub fn create<T: Default + Debug + Clone>(
        self,
    ) -> VirtualArray<SIGNATURE_SIZE, std::fs::File, T> {
        self.use_file_as_storage(true).create::<T>()
    }
}

impl<'file_name, const SIGNATURE_SIZE: usize>
    VirtualArrayBuilder<&'file_name str, NoneValue, NoneValue, usize, NoneValue, [u8; SIGNATURE_SIZE]>
{
    pub fn open<T: Default + Debug + Clone>(
        self,
    ) -> VirtualArray<SIGNATURE_SIZE, std::fs::File, T> {
        self.use_file_as_storage(false).open::<T>()
    }
}

impl<'file_name, ArraySize, BufferSize, PageSize, Signature>
    VirtualArrayBuilder<&'file_name str, NoneValue, ArraySize, BufferSize, PageSize, Signature>
{
    pub fn use_file_as_storage(
        self,
        create: bool,
    ) -> VirtualArrayBuilder<NoneValue, File, ArraySize, BufferSize, PageSize, Signature> {
        let file = OpenOptions::new()
            .create(create)
            .write(true)
            .read(true)
            .open(Path::new(self.file_name))
            .unwrap();

        VirtualArrayBuilder {
            file_name: NoneValue,
            storage: file,
            buffer_size: self.buffer_size,
            desired_page_size: self.desired_page_size,
            array_size: self.array_size,
            signature: self.signature,
        }
    }
}

impl<S: Storage> VirtualArrayBuilder<NoneValue, S, usize, usize, usize, NoneValue> {
    pub fn create<T: Default + Debug + Clone>(self) -> VirtualArray<DEFAULT_SIGNATURE_SIZE, S, T> {
        self.signature::<DEFAULT_SIGNATURE_SIZE>(DEFAULT_SIGNATURE)
            .create()
    }
}

impl<S: Storage> VirtualArrayBuilder<NoneValue, S, NoneValue, usize, NoneValue, NoneValue> {
    pub fn open<T: Default + Debug + Clone>(self) -> VirtualArray<DEFAULT_SIGNATURE_SIZE, S, T> {
        self.signature::<DEFAULT_SIGNATURE_SIZE>(DEFAULT_SIGNATURE)
            .open()
    }
}

impl<const SIGNATURE_SIZE: usize, S: Storage>
    VirtualArrayBuilder<NoneValue, S, usize, usize, usize, [u8; SIGNATURE_SIZE]>
{
    pub fn create<T: Default + Debug + Clone>(mut self) -> VirtualArray<SIGNATURE_SIZE, S, T> {
        let metadata = Metadata {
            array_size: self.array_size,
            signature: self.signature,
            page_size: VirtualArray::<SIGNATURE_SIZE, S, T>::count_page_size(
                self.desired_page_size,
            ),
        };

        self.storage.seek_to_start().unwrap();
        metadata.write(&mut self.storage).unwrap();
        self.storage.flush().unwrap();

        let count_of_elements_on_page = metadata.count_elements_on_page::<T>();
        let page = crate::page::Page::<T>::new(0, count_of_elements_on_page);

        for i in 0..(self.array_size / count_of_elements_on_page + 1) {
            self.storage
                .seek_to_page(i, metadata.page_size, count_of_elements_on_page)
                .unwrap();

            page.write(&mut self.storage);
            self.storage.flush().unwrap();
        }

        VirtualArray {
            storage: self.storage,
            buffer_size: self.buffer_size,
            metadata,
            pages: Vec::with_capacity(self.buffer_size),
            count_of_elements_on_page,
        }
    }
}

impl<const SIGNATURE_SIZE: usize, S: Storage>
    VirtualArrayBuilder<NoneValue, S, NoneValue, usize, NoneValue, [u8; SIGNATURE_SIZE]>
{
    pub fn open<T: Default + Debug + Clone>(mut self) -> VirtualArray<SIGNATURE_SIZE, S, T> {
        self.storage.seek_to_start().unwrap();
        let metadata = Metadata::read(&mut self.storage).unwrap();
        let count_of_elements_on_page = metadata.count_elements_on_page::<T>();

        VirtualArray {
            storage: self.storage,
            buffer_size: self.buffer_size,
            metadata,
            pages: Vec::with_capacity(self.buffer_size),
            count_of_elements_on_page,
        }
    }
}
