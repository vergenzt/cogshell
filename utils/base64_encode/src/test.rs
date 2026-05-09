use super::base64_encode;

#[test]
fn test_base64_encode() {
    assert_eq!(base64_encode(b"abcdefg"), "YWJjZGVmZw==");
}
