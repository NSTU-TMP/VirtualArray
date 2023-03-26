use virtual_array::VirtualArrayBuilder;

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
        let mut va = VirtualArrayBuilder::from_file_name(FILE_NAME)
            .item_type::<f32>()
            .buffer_size(3)
            .create(100000000, 512)
            .unwrap();

        va.set(0, 1.0).unwrap();
        va.set(13, 2.0).unwrap();
        va.set(0, 3.0).unwrap();
        va.set(512, 4.0).unwrap();
        va.set(1024, 5.0).unwrap();
        va.set(2048, 6.0).unwrap();
        va.set(99999999, 7.0).unwrap();
    }

    {
        let mut va = VirtualArrayBuilder::from_file_name(FILE_NAME)
            .item_type::<f32>()
            .buffer_size(3)
            .open()
            .unwrap();

        va.delete(1024).unwrap();
    }

    {
        let mut va = VirtualArrayBuilder::from_file_name(FILE_NAME)
            .item_type::<f32>()
            .buffer_size(3)
            .open()
            .unwrap();

        assert_eq!(Some(&3.0), va.get(0).unwrap());
        assert_eq!(Some(&2.0), va.get(13).unwrap());
        assert_eq!(Some(&4.0), va.get(512).unwrap());
        assert_eq!(None, va.get(1024).unwrap());
        assert_eq!(Some(&6.0), va.get(2048).unwrap());
        assert_eq!(Some(&7.0), va.get(99999999).unwrap());
    }
}

#[test]
fn test_with_struct_items() {
    const FILE_NAME: &str = "test_with_struct_items.bin";
    remove_file(FILE_NAME);

    #[derive(Debug, Default, PartialEq, Clone, Copy)]
    struct Test {
        field_1: u8,
        field_2: u8,
    }

    let value_1 = Test {
        field_1: 5,
        field_2: 3,
    };

    let value_2 = Test {
        field_1: 11,
        field_2: 16,
    };

    {
        let mut va = VirtualArrayBuilder::from_file_name(FILE_NAME)
            .item_type::<Test>()
            .buffer_size(1)
            .create(10, 18)
            .unwrap();

        for i in 0..10 {
            va.set(i, if i % 2 == 0 { value_1 } else { value_2 })
                .unwrap();
        }
    }

    {
        let mut va = VirtualArrayBuilder::from_file_name(FILE_NAME)
            .item_type::<Test>()
            .buffer_size(1)
            .open()
            .unwrap();

        va.set(0, value_2).unwrap();
        va.delete(9).unwrap();
    }

    {
        let mut va = VirtualArrayBuilder::from_file_name(FILE_NAME)
            .item_type::<Test>()
            .buffer_size(1)
            .open()
            .unwrap();

        assert_eq!(va.get(0).unwrap(), Some(&value_2));
        assert_eq!(va.get(9).unwrap(), None);

        for i in 1..9 {
            assert_eq!(
                va.get(i).unwrap(),
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
        let mut va = VirtualArrayBuilder::from_file_name(FILE_NAME)
            .item_type::<u8>()
            .buffer_size(10)
            .create(40, 20)
            .unwrap();

        va.set(0, 123).unwrap();
        va.set(7, 123).unwrap();
        va.set(35, 99).unwrap();
        va.set(38, 1).unwrap();
    }

    {
        let mut va = VirtualArrayBuilder::from_file_name(FILE_NAME)
            .item_type::<u8>()
            .buffer_size(10)
            .open()
            .unwrap();

        va.delete(35).unwrap();
        va.set(38, 15).unwrap();
    }

    {
        let mut va = VirtualArrayBuilder::from_file_name(FILE_NAME)
            .item_type::<u8>()
            .buffer_size(10)
            .open()
            .unwrap();

        assert_eq!(va.get(0).unwrap(), Some(&123));
        assert_eq!(va.get(7).unwrap(), Some(&123));
        assert_eq!(va.get(35).unwrap(), None);
        assert_eq!(va.get(38).unwrap(), Some(&15));
    }
}
