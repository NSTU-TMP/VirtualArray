mod bitmap;
mod page;
mod virtual_array;

use std::{
    fmt::Debug,
    fs::File,
    io::{Read, Seek, Write},
};

pub use crate::virtual_array::VirtualArray;

pub trait BufferStream: Read + Write + Seek {}
impl BufferStream for File {}
