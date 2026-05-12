#![cfg_attr(test, feature(macro_metavar_expr_concat))]

#[cfg(test)]
mod test;

/// Base64-encodes the input bytes
/// https://datatracker.ietf.org/doc/html/rfc4648#section-4
pub fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    const MASK: u32 = 0b111_111;
    let outlen = (input.len() * 4).div_ceil(3) + 2;
    let mut out = String::with_capacity(outlen);
    for chunk in input.chunks(3) {
        let word = u32::from_be_bytes({
            let mut arr = [0, 0, 0, 0];
            arr[1..=chunk.len()].copy_from_slice(chunk);
            arr
        });
        for (offset, byte_idx) in [(18, 0), (12, 0), (6, 1), (0, 2)] {
            out.push(if chunk.len() > byte_idx {
                CHARS[((word & (MASK << offset)) >> offset) as usize] as char
            } else {
                '='
            });
        }
        #[cfg(test)]
        assert_eq!(out.capacity(), outlen);
    }
    out
}
