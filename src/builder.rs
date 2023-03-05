use crate::{Storage, VirtualArray};

pub struct VirtualArrayBuilder<FileName, Storage, ArraySize, BufferSize, PageSize> {
    file_name: FileName,
    storage: Storage,
    array_size: ArraySize,
    buffer_size: BufferSize,
    page_size: PageSize,
}

struct NoneValue;

impl VirtualArrayBuilder<NoneValue, NoneValue, NoneValue, NoneValue, NoneValue> {
    pub fn new() -> Self {
        Self {
            file_name: NoneFileName,
            storage: NoneStorage,
            array_size: NoneSize,
            buffer_size: NoneSize,
            page_size: NoneSize,
        }
    }
}

impl<ArraySize, BufferSize, PageSize>
    VirtualArrayBuilder<NoneValue, NoneValue, ArraySize, BufferSize, PageSize>
{
    pub fn file_name<'file_name>(
        self,
        value: &'file_name str,
    ) -> VirtualArrayBuilder<SomeFileName<'file_name>, NoneValue, ArraySize, BufferSize, PageSize>
    {
        VirtualArrayBuilder {
            file_name: SomeFileName(value),
            storage: NoneValue,
            array_size: self.array_size,
            buffer_size: self.buffer_size,
            page_size: self.page_size,
        }
    }
}

impl<ArraySize, BufferSize, PageSize>
    VirtualArrayBuilder<NoneValue, NoneValue, ArraySize, BufferSize, PageSize>
{
    pub fn storage<S: Storage>(
        self,
        value: S,
    ) -> VirtualArrayBuilder<NoneValue, SomeStorage<S>, ArraySize, BufferSize, PageSize>
    {
        VirtualArrayBuilder {
            file_name: NoneValue,
            storage: SomeStorage(value),
            array_size: self.array_size,
            buffer_size: self.buffer_size,
            page_size: self.page_size,
        }
    }
}

impl<FileName, Storage, BufferSize, PageSize>
    VirtualArrayBuilder<FileName, Storage, NoneValue, BufferSize, PageSize>
{
    pub fn array_size(
        self,
        value: usize,
    ) -> VirtualArrayBuilder<FileName, Storage, SomeSize, BufferSize, PageSize>
    {
        VirtualArrayBuilder {
            file_name: self.file_name,
            storage: self.storage,
            array_size: SomeSize(value),
            buffer_size: self.buffer_size,
            page_size: self.page_size,
        }
    }
}

impl<FileName, Storage, ArraySize, PageSize>
    VirtualArrayBuilder<FileName, Storage, ArraySize, NoneValue, PageSize>
{
    pub fn buffer_size(
        self,
        value: usize,
    ) -> VirtualArrayBuilder<FileName, Storage, ArraySize, SomeSize, PageSize>
    {
        VirtualArrayBuilder {
            file_name: self.file_name,
            storage: self.storage,
            array_size: self.array_size,
            buffer_size: SomeSize(value),
            page_size: self.page_size,
        }
    }
}

impl<FileName, Storage, ArraySize, BufferSize>
    VirtualArrayBuilder<FileName, Storage, ArraySize, BufferSize, NoneValue>
{
    pub fn page_size(
        self,
        value: usize,
    ) -> VirtualArrayBuilder<FileName, Storage, ArraySize, BufferSize, SomeSize>
    {
        VirtualArrayBuilder {
            file_name: self.file_name,
            storage: self.storage,
            array_size: self.array_size,
            buffer_size: self.buffer_size,
            page_size: SomeSize(value),
        }
    }
}

// impl VirtualArrayBuilder<SomeFileName, NoneStorage, SomeSize, SomeSize, SomeSize>
// {
//     pub fn create(self) -> VirtualArray<std::fs::File, >
// }

fn test() {
    let b = VirtualArrayBuilder::new();
    let f = std::fs::File::open(std::path::Path::new("sdfsf")).unwrap();
    // b.file_name("test").page_size(23).buffer_size(32).array_size(3);
    b.storage(f).page_size(23).buffer_size(32).array_size(3);
}
