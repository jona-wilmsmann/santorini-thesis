pub const fn assert_valid_bit_count(bit_count: usize) {
    match bit_count {
        1 | 2 | 4 | 8 => (),
        _ => panic!("N must be 1, 2, 4, or 8"),
    }
}