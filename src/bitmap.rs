use std::{
    io::{Read, Write},
    marker::PhantomData,
    mem,
};

pub struct Bitmap<T> {
    elements_count: usize,
    bytes: Vec<u8>,
    data_type: PhantomData<T>,
}

pub(crate) fn calc_bitmap_byte_size<T>(count_of_elements: usize) -> usize {
    let count_of_bytes = count_of_elements / (mem::size_of::<T>() * 8);

    if count_of_elements % (mem::size_of::<T>() * 8) != 0 {
        count_of_bytes + 1
    } else {
        count_of_bytes
    }
}

impl<T> Bitmap<T> {
    pub fn new(elements_count: usize) -> Self {
        dbg!(elements_count);
        dbg!(calc_bitmap_byte_size::<T>(elements_count));
        Self {
            elements_count,
            bytes: vec![0; calc_bitmap_byte_size::<T>(elements_count)],
            data_type: PhantomData,
        }
    }

    pub fn set(&mut self, index: usize) {
        debug_assert!(index < self.elements_count);

        let (byte_index, bit_index) = self.get_byte_bit_indices(index);

        self.bytes[byte_index] = self.bytes[byte_index] | (1 << bit_index);
    }

    pub fn unset(&mut self, index: usize) {
        debug_assert!(index < self.elements_count);

        let (byte_index, bit_index) = self.get_byte_bit_indices(index);
        self.bytes[byte_index] = self.bytes[byte_index] & !(1 << bit_index);
    }

    pub fn get(&self, index: usize) -> bool {
        debug_assert!(index < self.elements_count);

        let (byte_index, bit_index) = self.get_byte_bit_indices(index);
        self.bytes[byte_index] & (1 << bit_index) != 0
    }

    fn get_byte_bit_indices(&self, index: usize) -> (usize, usize) {
        (index / mem::size_of::<T>(), index % mem::size_of::<T>())
    }

    pub fn write<W: Write>(&self, writer: &mut W) {
        writer.write_all(self.bytes.as_slice()).unwrap();
    }

    pub fn read<R: Read>(elements_count: usize, reader: &mut R) -> Option<Self> {
        let mut buffer = vec![0; calc_bitmap_byte_size::<T>(elements_count)];
        if let Err(_) = reader.read_exact(&mut buffer) {
            return None;
        }

        Some(Self {
            elements_count,
            bytes: buffer,
            data_type: PhantomData,
        })
    }
}
