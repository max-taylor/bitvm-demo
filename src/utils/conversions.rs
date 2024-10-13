pub fn number_to_bool_array(number: usize, length: usize) -> Vec<bool> {
    let mut v = Vec::new();
    for i in 0..length {
        v.push(0 != number & (1 << i));
    }
    v
}

pub fn bool_array_to_number(bool_array: Vec<bool>) -> usize {
    let mut a = 0;
    for b in bool_array.iter().rev() {
        a *= 2;
        if *b {
            a += 1;
        }
    }
    a
}
