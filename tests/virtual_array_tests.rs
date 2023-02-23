#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use virtual_array::VirtualArray;
    // use super::*;
    #[test]
    fn test_add() {
        let mut va = VirtualArray::new("test.bin", 20, 1, 20);
        va.set_element(0, 123);
        va.set_element(2, 99);
        assert_eq!(1, 1);

        assert_eq!(va.get_element(0), Some(123));
        assert_eq!(va.get_element(2), Some(99));
    }
}
