#[derive(Debug)]
pub struct DataChunk<Item> {
    source: Vec<Item>,
}

impl<Item> From<Vec<Item>> for DataChunk<Item> {
    fn from(items: Vec<Item>) -> Self {
        DataChunk { source: items }
    }
}

impl<Item> AsRef<[Item]> for DataChunk<Item> {
    fn as_ref(&self) -> &[Item] {
        &self.source
    }
}

impl<Item> DataChunk<Item> {
    pub(super) fn set(&mut self, index: usize, value: Item) {
        debug_assert!(index < self.source.len());
        self.source[index] = value
    }

    pub(super) fn get(&self, index_on_page: usize) -> &Item {
        debug_assert!(index_on_page < self.source.len());
        self.source.get(index_on_page).unwrap()
    }
}
