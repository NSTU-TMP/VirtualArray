use super::Storage;

#[derive(Debug)]
pub(crate) struct Metadata<const SIGNATURE_SIZE: usize> {
    pub signature: [u8; SIGNATURE_SIZE],
    pub page_size: usize,
    pub array_size: usize,
}

impl<const SIGNATURE_SIZE: usize> Metadata<SIGNATURE_SIZE> {
    pub fn write<'storage, S: Storage>(
        &self,
        storage: &'storage mut S,
    ) -> Result<(), std::io::Error> {
        storage.write_all(&self.signature)?;
        storage.write_all(self.page_size.to_ne_bytes().as_slice())?;
        storage.write_all(self.array_size.to_ne_bytes().as_slice())
    }

    pub fn read<'storage, S: Storage>(
        storage: &'storage mut S,
    ) -> Result<Self, std::io::Error> {
        use std::mem::size_of;

        let mut buff = [0u8; SIGNATURE_SIZE];
        storage.read_exact(&mut buff)?;
        let signature = buff;

        let mut buff = [0u8; size_of::<usize>()];

        storage.read_exact(&mut buff)?;
        let page_size = usize::from_ne_bytes(buff);

        storage.read_exact(&mut buff)?;
        let array_size = usize::from_ne_bytes(buff);

        Ok(Self {
            signature,
            page_size,
            array_size,
        })
    }

    pub fn count_elements_on_page<T>(&self) -> usize {
        self.page_size / std::mem::size_of::<T>()
    }
}
