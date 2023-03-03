use core::slice;
use std::fs::remove_file;

use virtual_array::VirtualArray;

#[derive(Debug, PartialEq, Clone, Copy)]
struct Test {
    name: i64,
    surname: i64,
}
fn main() {
    let test_struct_1 = Test {
        name: 123,
        surname: 456,
    };

    let test_struct_2 = Test {
        name: 789,
        surname: 101112,
    };

    // let mut va: VirtualArray<std::fs::File, Test> =
    //     VirtualArray::from_file_name("test_add_with_struct.bin", 3, 1, 18).unwrap();
    //
    // va.set_element(0, test_struct_1);
    // va.set_element(1, test_struct_2);
    remove_file("test_add_with_struct.bin");
    {
        let mut va: VirtualArray<std::fs::File, Test> =
            VirtualArray::from_file_name("test_add_with_struct.bin", 3, 1, 18).unwrap();

        va.set_element(0, test_struct_1);
        va.set_element(1, test_struct_2);
    }

    {
        let mut va: VirtualArray<std::fs::File, Test> =
            VirtualArray::from_file_name("test_add_with_struct.bin", 3, 1, 18).unwrap();

        assert_eq!(va.get_element(0), Some(&test_struct_1));
        assert_eq!(va.get_element(1), Some(&test_struct_2));
    }
}
