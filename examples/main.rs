use core::slice;

use virtual_array::VirtualArray;

#[derive(Debug)]
struct Test {
    name: i64,
    surname: i64,
}
fn main() {
    let mut va: VirtualArray<std::fs::File, f64> =
        VirtualArray::from_file_name("float_tets.bin", 100, 3, 512).unwrap();
    va.set_element(0, 1.0);
    va.set_element(13, 2.0);
    va.set_element(0, 3.0);
    va.set_element(70, 4.0);
    va.set_element(72, 5.0);
    va.set_element(75, 6.0);
    va.set_element(98, 6.0);
    va.remove_element(90);

    //thread::sleep(time::Duration::from_millis(1000000));
    // assert_eq!(Some(&3.0), va.get_element(0));
    // assert_eq!(Some(&4.0), va.get_element(512));
    // assert_eq!(Some(&6.0), va.get_element(2048));
}
