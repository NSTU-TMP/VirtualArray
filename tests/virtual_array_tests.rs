use virtual_array::VirtualArrayBuilder;

#[derive(Debug, Default, PartialEq, Clone, Copy)]
struct Test {
    field_1: u8,
    field_2: u8,
}

fn remove_file(file_name: &str) {
    use std::fs::remove_file;
    if std::path::Path::new(file_name).exists() {
        remove_file(file_name).unwrap();
    }
}

#[test]
fn test_with_float_items() {
    const FILE_NAME: &str = "test_with_float_items.bin";
    remove_file(FILE_NAME);

    {
        let mut va = VirtualArrayBuilder::new()
            .file_name(FILE_NAME)
            .buffer_size(3)
            .array_size(100000000)
            .desired_page_size(512)
            .create::<f32>();

        va.set_element(0, 1.0);
        va.set_element(13, 2.0);
        va.set_element(0, 3.0);
        va.set_element(512, 4.0);
        va.set_element(1024, 5.0);
        va.set_element(2048, 6.0);
        va.set_element(99999999, 7.0);
    }

    {
        let mut va = VirtualArrayBuilder::new()
            .file_name(FILE_NAME)
            .buffer_size(3)
            .open::<f32>();

        va.remove_element(1024);
    }

    {
        let mut va = VirtualArrayBuilder::new()
            .file_name(FILE_NAME)
            .buffer_size(3)
            .open::<f32>();

        assert_eq!(Some(&3.0), va.get_element(0));
        assert_eq!(Some(&2.0), va.get_element(13));
        assert_eq!(Some(&4.0), va.get_element(512));
        assert_eq!(None, va.get_element(1024));
        assert_eq!(Some(&6.0), va.get_element(2048));
        assert_eq!(Some(&7.0), va.get_element(99999999));
    }
}

#[test]
fn test_with_struct_items() {
    const FILE_NAME: &str = "test_with_struct_items.bin";
    remove_file(FILE_NAME);

    let value_1 = Test {
        field_1: 5,
        field_2: 3,
    };

    let value_2 = Test {
        field_1: 11,
        field_2: 16,
    };

    {
        let mut va = VirtualArrayBuilder::new()
            .file_name(FILE_NAME)
            .buffer_size(1)
            .array_size(10)
            .desired_page_size(18)
            .create::<Test>();

        for i in 0..10 {
            va.set_element(i, if i % 2 == 0 { value_1 } else { value_2 });
        }
    }

    {
        let mut va = VirtualArrayBuilder::new()
            .file_name(FILE_NAME)
            .buffer_size(1)
            .open::<Test>();

        va.set_element(0, value_2);
        va.remove_element(9);
    }

    {
        let mut va = VirtualArrayBuilder::new()
            .file_name(FILE_NAME)
            .buffer_size(1)
            .open::<Test>();

        assert_eq!(va.get_element(0), Some(&value_2));
        assert_eq!(va.get_element(9), None);

        for i in 1..9 {
            assert_eq!(
                va.get_element(i),
                Some(&if i % 2 == 0 { value_1 } else { value_2 })
            );
        }
    }
}

#[test]
fn test_with_u8_items() {
    const FILE_NAME: &str = "test_with_u8_items.bin";
    remove_file(FILE_NAME);

    {
        let mut va = VirtualArrayBuilder::new()
            .file_name(FILE_NAME)
            .buffer_size(1000)
            .array_size(40)
            .desired_page_size(20)
            .create::<u8>();

        va.set_element(0, 123);
        va.set_element(35, 99);
        va.set_element(39, 11);
    }

    {
        let mut va = VirtualArrayBuilder::new()
            .file_name(FILE_NAME)
            .buffer_size(1000)
            .open::<u8>();

        va.remove_element(35);
        va.set_element(39, 15);
    }

    {
        let mut va = VirtualArrayBuilder::new()
            .file_name(FILE_NAME)
            .buffer_size(1000)
            .open::<u8>();

        assert_eq!(va.get_element(0), Some(&123));
        assert_eq!(va.get_element(35), None);
        assert_eq!(va.get_element(39), Some(&15));
    }
}
