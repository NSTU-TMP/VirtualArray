use virtual_array::VirtualArray;

fn main() {
    let mut va: VirtualArray<std::fs::File, u8> =
        VirtualArray::from_file_name("test.bin", 20, 1, 20).unwrap();
    va.set_element(0, 123);
    // let mut va = VirtualArray::from_file_name("test.bin", 20, 1, 20);
    // va.set_element(0, 123);
    // va.set_element(2, 99);
    // va.remove_element(2);
}
