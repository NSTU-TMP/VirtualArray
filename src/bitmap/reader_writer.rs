use std::io::{Read, Write};

use super::Bitmap;
use crate::BytesCount;

pub trait BitmapReaderWriter {
    fn write<Reader: Write>(writer: &mut Reader, bitmap: &Bitmap);

    fn read<Writer: Read>(reader: &mut Writer, elements_count: usize) -> Option<Bitmap>;

    fn calc_bitmap_size(count_of_elements: usize) -> BytesCount
    {
        let count_of_bytes = count_of_elements / 8;

        if count_of_elements % 8 != 0 {
            count_of_bytes + 1
        } else {
            count_of_bytes
        }
    }
}

pub struct DefaultBitmapReaderWriter;

impl BitmapReaderWriter for DefaultBitmapReaderWriter {
    fn write<Writer: Write>(writer: &mut Writer, bitmap: &Bitmap) {
        writer.write_all(bitmap.bytes()).unwrap();
    }

    fn read<Reader: Read>(reader: &mut Reader, elements_count: usize) -> Option<Bitmap> {
        let mut buffer = vec![0; <DefaultBitmapReaderWriter as BitmapReaderWriter>::calc_bitmap_size(elements_count)];

        if let Err(_) = reader.read_exact(&mut buffer) {
            return None;
        }

        Some(Bitmap::new(elements_count, buffer))
    }
}
