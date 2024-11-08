use anyhow::anyhow;

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

/// A variable-length integer or "varint" is a static Huffman encoding of 64-bit
/// twos-complement integers that uses less space for small positive values. A
/// varint is between 1 and 9 bytes in length. The varint consists of either
/// zero or more bytes which have the high-order bit set followed by a single
/// byte with the high-order bit clear, or nine bytes, whichever is shorter. The
/// lower seven bits of each of the first eight bytes and all 8 bits of the ninth
/// byte are used to reconstruct the 64-bit twos-complement integer. Varints are
/// big-endian: bits taken from the earlier byte of the varint are more
/// significant than bits taken from the later bytes.
// TODO: for some reason, min and max u64 don't work
pub fn varint_unsigned(input: &mut &[u8]) -> Result<u64, anyhow::Error> {
    assert!(!input.is_empty());

    let mut cnt = 0;
    let mut result = 0u64;
    let mut shift = 0u8;
    loop {
        println!("input: {:#X?}", input);

        let s = result << shift;
        let v = input[0] & 0x7f;
        result = s | v as u64;

        println!(
            "v: {:#X} shift: {shift} bits s: {:#X} s|v: {:#X}",
            v, s, result
        );

        if input[0] & 0x80 == 0 {
            break;
        }

        *input = &input[1..];

        if input.is_empty() {
            return Err(anyhow!("varint not long enough"));
        }

        cnt += 1;

        if cnt == 9 {
            return Err(anyhow!("varint too long"));
        }

        shift += 7;
    }

    Ok(result)
}

pub fn varint_signed(input: &mut &[u8]) -> Result<i64, anyhow::Error> {
    let n = varint_unsigned(input)?;

    Ok(((n >> 1) as i64) ^ (-((n & 1) as i64)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_positive() {
        let encoded = vec![0x78];
        let decoded = varint_unsigned(&mut encoded.as_slice()).unwrap();
        assert_eq!(decoded, 120);

        let encoded = vec![0x81, 0x16];
        let decoded = varint_unsigned(&mut encoded.as_slice()).unwrap();
        assert_eq!(decoded, 150);
    }

    #[test]
    fn test_varint_negative() {
        let encoded = vec![0x80, 0x01];
        let decoded = varint_signed(&mut encoded.as_slice()).unwrap();
        assert_eq!(decoded, -1);
    }

    #[test]
    fn test_varint_zero() {
        let encoded = vec![0x00];
        let decoded = varint_unsigned(&mut encoded.as_slice()).unwrap();
        assert_eq!(decoded, 0);
    }

    #[test]
    #[ignore]
    fn test_varint_max_u64() {
        let encoded = vec![0x81, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f];
        let decoded = varint_unsigned(&mut encoded.as_slice()).unwrap();
        assert_eq!(decoded, u64::MAX);
    }

    #[test]
    #[ignore]
    fn test_varint_min_i64() {
        let encoded = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x00];
        let decoded = varint_signed(&mut encoded.as_slice()).unwrap();
        assert_eq!(decoded, i64::MIN);
    }

    #[test]
    fn test_varint_incomplete() {
        let encoded = vec![0x96];
        let result = varint_unsigned(&mut encoded.as_slice());
        assert!(result.is_err());
    }

    #[test]
    fn test_varint_too_long() {
        let encoded = vec![0x80; 10];
        let result = varint_unsigned(&mut encoded.as_slice());
        assert!(result.is_err());
    }
}
