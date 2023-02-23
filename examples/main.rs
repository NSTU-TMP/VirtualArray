use virtual_array::VirtualArray;

fn main() {
    let mut va = VirtualArray::new("test.bin", 20, 1, 20);
    va.set_element(0, 123);
    va.set_element(2, 99);
    dbg!(va.get_element(0));
    dbg!(va.get_element(2));
    // va.remove_element(2);
}
