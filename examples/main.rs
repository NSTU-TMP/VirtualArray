use virtual_array::VirtualArrayBuilder;

fn remove_file(file_name: &str) {
    use std::fs::remove_file;
    if std::path::Path::new(file_name).exists() {
        remove_file(file_name).unwrap();
    }
}

fn main() {
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

        for i in 0..1 {
            va.set(i, if i % 2 == 0 { value_1 } else { value_2 })
                .unwrap();
        }
    }

    // {
    //     let mut va = VirtualArrayBuilder::from_file_name(FILE_NAME)
    //         .item_type::<Test>()
    //         .buffer_size(1)
    //         .open()
    //         .unwrap();
    //
    //     va.set(0, value_2).unwrap();
    //     va.delete(9).unwrap();
    // }
    //
    // {
    //     let mut va = VirtualArrayBuilder::from_file_name(FILE_NAME)
    //         .item_type::<Test>()
    //         .buffer_size(1)
    //         .open()
    //         .unwrap();
    //
    //     assert_eq!(va.get(0).unwrap(), Some(&value_2));
    //     assert_eq!(va.get(9).unwrap(), None);
    //
    //     for i in 1..9 {
    //         assert_eq!(
    //             va.get(i).unwrap(),
    //             Some(&if i % 2 == 0 { value_1 } else { value_2 })
    //         );
    //     }
    // }
}
