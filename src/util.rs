pub fn read_be_u32(input: &mut &[u8]) -> u32 {
    let (int_bytes, rest) = input.split_at(size_of::<u32>());
    *input = rest;
    u32::from_be_bytes(int_bytes.try_into().unwrap())
}

pub fn read_len<'a>(input: &mut &'a [u8], len: usize) -> &'a [u8] {
    let (read, rest) = input.split_at(len);
    *input = rest;
    read
}
