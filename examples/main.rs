use virtual_array::VirtualArrayBuilder;


fn main() {
    let mut va = VirtualArrayBuilder::from_file_name("main_example.bin")
        .item_type::<u8>()
        .buffer_size(10)
        .array_size(40)
        .desired_page_size(20)
        .create();
}
