use virtual_array::VirtualArray;

#[test]
fn test_add() {
    let mut va: VirtualArray<std::fs::File, u8> =
        VirtualArray::from_file_name("test_add.bin", 40, 1, 20).unwrap();
    va.set_element(0, 123);
    va.set_element(35, 99);
    assert_eq!(va.get_element(0), Some(&123));
    assert_eq!(va.get_element(35), Some(&99));
}

#[test]
fn test_remove() {
    let mut va: VirtualArray<std::fs::File, u8> =
        VirtualArray::from_file_name("test_remove.bin", 20, 1, 20).unwrap();
    va.set_element(0, 123);

    va.remove_element(0);
    va.remove_element(1);
    assert_eq!(va.get_element(0), None);
    assert_eq!(va.get_element(1), None);
}
