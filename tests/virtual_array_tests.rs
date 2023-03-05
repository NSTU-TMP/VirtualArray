use std::fs::remove_file;
use virtual_array::VirtualArray;

#[derive(Debug, Default, PartialEq, Clone, Copy)]
struct Test {
    name: u8,
    surname: u8,
}

#[test]
fn float_test() {
    let mut va: VirtualArray<std::fs::File, f64> =
        VirtualArray::create_from_file_name("float_tets.bin", 100000000, 3, 512);
    va.set_element(0, 1.0);
    va.set_element(13, 2.0);
    va.set_element(0, 3.0);
    va.set_element(512, 4.0);
    va.set_element(1024, 5.0);
    va.set_element(2048, 6.0);
    va.set_element(99999999, 6.0);
    va.remove_element(1024);

    assert_eq!(Some(&3.0), va.get_element(0));
    assert_eq!(Some(&4.0), va.get_element(512));
    assert_eq!(Some(&6.0), va.get_element(2048));
}

#[test]
fn tet_add_with_struct() {
    let test_struct_1 = Test {
        name: 5,
        surname: 3,
    };

    let test_struct_2 = Test {
        name: 11,
        surname: 16,
    };

    #[deny(unused_must_use, unused, unused_results)]
    remove_file("test_add_with_struct.bin");

    {
        let mut va: VirtualArray<std::fs::File, Test> =
            VirtualArray::create_from_file_name("test_add_with_struct.bin", 9, 1, 18);
        for i in 0..9 {
            va.set_element(
                i,
                if i % 2 == 0 {
                    test_struct_1
                } else {
                    test_struct_2
                },
            );
        }
    }

    {
        let mut va: VirtualArray<std::fs::File, Test> =
            VirtualArray::open_from_file_name("test_add_with_struct.bin", 9, 1);

        for i in 0..9 {
            assert_eq!(
                va.get_element(i),
                Some(&if i % 2 == 0 {
                    test_struct_1
                } else {
                    test_struct_2
                })
            );
        }
    }
}

#[test]
fn test_add() {
    #[deny(unused_must_use, unused, unused_results)]
    remove_file("test_add.bin");

    {
        let mut va: VirtualArray<std::fs::File, u8> =
            VirtualArray::create_from_file_name("test_add.bin", 40, 1000, 20);
        va.set_element(0, 123);
        va.set_element(35, 99);
    }

    {
        let mut va: VirtualArray<std::fs::File, u8> =
            VirtualArray::open_from_file_name("test_add.bin", 40, 1000);
        assert_eq!(va.get_element(0), Some(&123));
        assert_eq!(va.get_element(35), Some(&99));
    }
}

#[test]
fn test_remove() {
    #[deny(unused_must_use, unused, unused_results)]
    remove_file("test_remove.bin");

    {
        let mut va: VirtualArray<std::fs::File, u8> =
            VirtualArray::create_from_file_name("test_remove.bin", 20, 1, 20);
        va.set_element(0, 123);
    }

    {
        let mut va: VirtualArray<std::fs::File, u8> =
            VirtualArray::open_from_file_name("test_remove.bin", 20, 1);
        va.remove_element(0);
        va.remove_element(1);
    }

    {
        let mut va: VirtualArray<std::fs::File, u8> =
            VirtualArray::open_from_file_name("test_remove.bin", 20, 1);
        assert_eq!(va.get_element(0), None);
        assert_eq!(va.get_element(1), None);
    }
}
