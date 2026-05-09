#[cfg(test)]
mod test;

pub fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut b64s = String::with_capacity((input.len() * 4).div_ceil(3));
    let mut wind = input[0] as u32;
    let mut mask = 0b_1111_1100 as u32;
    for i in 1..input.len().next_multiple_of(3) {
        wind <<= 8;
        wind |= *input.get(i).unwrap_or(&0) as u32;
        loop {
            let zers = mask.trailing_zeros();
            let indx = (wind & mask) >> zers;
            let byte = CHARS[indx as usize];
            b64s.push(byte as char);
            if zers < 6 {
                break;
            } else {
                mask >>= 6;
            }
        }
        mask <<= 8;
    }
    b64s.push_str(&"=".to_string().repeat(b64s.len().rem_euclid(3)));
    b64s
}
