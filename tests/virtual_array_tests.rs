use std::{thread, time};
use virtual_array::VirtualArray;

#[derive(Debug, PartialEq, Clone, Copy)]
struct Test {
    name: i64,
    surname: i64,
}

#[test]
fn float_test() {
    let mut va: VirtualArray<std::fs::File, f64> =
        VirtualArray::from_file_name("float_tets.bin", 100000000, 3, 512).unwrap();
    va.set_element(0, 1.0);
    va.set_element(13, 2.0);
    va.set_element(0, 3.0);
    va.set_element(512, 4.0);
    va.set_element(1024, 5.0);
    va.set_element(2048, 6.0);
    va.set_element(99999999, 6.0);
    va.remove_element(1024);

    //thread::sleep(time::Duration::from_millis(1000000));
    assert_eq!(Some(&3.0), va.get_element(0));
    assert_eq!(Some(&4.0), va.get_element(512));
    assert_eq!(Some(&6.0), va.get_element(2048));
    // assert_eq!(Some(1.0), va.get_element(0));
    // assert_eq!(Some(1.0), va.get_element(0));
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
        VirtualArray::from_file_name("test_add_with_struct.bin", 3, 1, 18).unwrap();

    va.set_element(0, test_struct_1);
    va.set_element(1, test_struct_2);

    assert_eq!(va.get_element(0), Some(&test_struct_1));
    assert_eq!(va.get_element(1), Some(&test_struct_2));
}

#[test]
fn test_add() {
    let mut va: VirtualArray<std::fs::File, u8> =
        VirtualArray::from_file_name("test_add.bin", 40, 1000, 20).unwrap();
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
