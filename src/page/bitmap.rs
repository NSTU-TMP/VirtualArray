use crate::BytesCount;

#[derive(Debug)]
pub struct Bitmap {
    elements_count: usize,
    bytes: Vec<u8>,
}

struct Indices {
    byte: usize,
    bit: usize,
}

impl Bitmap {
    // pub fn zeroed(elements_count: usize) -> Self {
    //     let bytes = vec![0; Self::calc_bitmap_size(elements_count)];
    //
    //     Self {
    //         elements_count,
    //         bytes,
    //     }
    // }

    pub(super) fn calc_bitmap_size(count_of_elements: usize) -> BytesCount {
        let count_of_bytes = count_of_elements / 8;

        if count_of_elements % 8 != 0 {
            count_of_bytes + 1
        } else {
            count_of_bytes
        }
    }

    pub fn new(elements_count: usize, bytes: Vec<u8>) -> Self {
        Self {
            elements_count,
            bytes,
        }
    }

    pub(super) fn set(&mut self, index: usize, value: bool) {
        debug_assert!(index < self.elements_count);

        let Indices {
            byte: byte_index,
            bit: bit_index,
        } = self.get_indices(index);

        self.bytes[byte_index] = if value {
            self.bytes[byte_index] | (1 << bit_index)
        } else {
            self.bytes[byte_index] & !(1 << bit_index)
        };
    }

    pub(super) fn get(&self, index: usize) -> bool {
        debug_assert!(index < self.elements_count);

        let Indices {
            byte: byte_index,
            bit: bit_index,
        } = self.get_indices(index);

        self.bytes[byte_index] & (1 << bit_index) != 0
    }

    fn get_indices(&self, index: usize) -> Indices {
        Indices {
            byte: index / 8,
            bit: index % 8,
        }
    }
}

impl AsRef<[u8]> for Bitmap {
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}
