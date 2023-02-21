use virtual_array::VirtualArray;

fn main() {
    let mut va = VirtualArray::<i32>::new("test.bin", 100, 3, 150);
    va.insert_element(0, 10);
}
