mod reader_writer;

pub use reader_writer::{BitmapReaderWriter, DefaultBitmapReaderWriter};

#[derive(Debug, Default)]
pub struct Bitmap {
    elements_count: usize,
    bytes: Vec<u8>,
}

impl Bitmap {
    pub fn zeroed(elements_count: usize) -> Self {
        let mut bytes =
            vec![
                0;
                <DefaultBitmapReaderWriter as BitmapReaderWriter>::calc_bitmap_size(elements_count)
            ];

        Self {
            elements_count,
            bytes,
        }
    }

    pub fn new(elements_count: usize, bytes: Vec<u8>) -> Self {
        Self {
            elements_count,
            bytes,
        }
    }

    pub fn elements_count(&self) -> usize {
        self.elements_count
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
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
        (index / 8, index % 8)
    }
}
