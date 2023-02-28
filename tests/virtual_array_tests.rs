use virtual_array::VirtualArray;

#[derive(Debug, PartialEq, Clone, Copy)]
struct Test {
    name: i32,
    surname: i64,
}

#[test]
fn tet_add_with_struct() {
    let test_struct_1 = Test {
        name: 123,
        surname: 456,
    };

    let test_struct_2 = Test {
        name: 789,
        surname: 101112,
    };

    let mut va: VirtualArray<std::fs::File, Test> =
        VirtualArray::from_file_name("test_add_with_struct.bin", 10, 1, 18).unwrap();

    va.set_element(0, test_struct_1);
    va.set_element(9, test_struct_2);

    assert_eq!(va.get_element(0), Some(&test_struct_1));
    assert_eq!(va.get_element(9), Some(&test_struct_2));
}

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
