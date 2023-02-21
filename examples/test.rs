use virtual_array::VirtualArray;

fn main() {
    let mut va = VirtualArray::<u8>::new("test.bin", 20, 1, 20);
    // va.insert_element(0, 10);
    // va.insert_element(1, 1);
    dbg!(va.get_element(0));
    dbg!(va.get_element(1));
}
