use super::base64_encode;

macro_rules! test_base64_encode {
    ($label:ident : $in:literal => $out:expr) => {
      #[test]
      fn ${concat(test_base64_encode_, $label)}() {
        assert_eq!(base64_encode($in.as_bytes()), $out);
      }
    };
}

test_base64_encode!(empty: "" => "");
test_base64_encode!(no_pad: "abcdef" => "YWJjZGVm");
test_base64_encode!(pad_one: "gobledygook" => "Z29ibGVkeWdvb2s=");
test_base64_encode!(pad_two: "abcdefg" => "YWJjZGVmZw==");
test_base64_encode!(with_nulls: "foo\0bar\0bazz\0" => "Zm9vAGJhcgBiYXp6AA==");
