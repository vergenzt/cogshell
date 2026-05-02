#[cfg(all(target_family = "wasm", not(target_os = "wasi")))]
use wasm_bindgen_test::wasm_bindgen_test as test;

#[cfg(all(target_family = "wasm", not(target_os = "wasi")))]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);


fn bool() {
    for x in &[false, true] {
        while fastrand::bool() != *x {}
    }
}


fn u8() {
    for x in 0..10 {
        while fastrand::u8(..10) != x {}
    }

    for x in 200..=u8::MAX {
        while fastrand::u8(200..) != x {}
    }
}


fn i8() {
    for x in -128..-120 {
        while fastrand::i8(..-120) != x {}
    }

    for x in 120..=127 {
        while fastrand::i8(120..) != x {}
    }
}


fn u32() {
    for n in 1u32..10_000 {
        let n = n.wrapping_mul(n);
        let n = n.wrapping_mul(n);
        if n != 0 {
            for _ in 0..1000 {
                assert!(fastrand::u32(..n) < n);
            }
        }
    }
}


fn u64() {
    for n in 1u64..10_000 {
        let n = n.wrapping_mul(n);
        let n = n.wrapping_mul(n);
        let n = n.wrapping_mul(n);
        if n != 0 {
            for _ in 0..1000 {
                assert!(fastrand::u64(..n) < n);
            }
        }
    }
}


fn u128() {
    for n in 1u128..10_000 {
        let n = n.wrapping_mul(n);
        let n = n.wrapping_mul(n);
        let n = n.wrapping_mul(n);
        let n = n.wrapping_mul(n);
        if n != 0 {
            for _ in 0..1000 {
                assert!(fastrand::u128(..n) < n);
            }
        }
    }
}


fn f32() {
    let mut r = fastrand::Rng::with_seed(0);
    let tiny = (-24.0f32).exp2();
    let mut count_tiny_nonzero = 0;
    let mut count_top_half = 0;
    for _ in 0..100_000_000 {
        let x = r.f32();
        assert!((0.0..1.0).contains(&x));
        if x > 0.0 && x < tiny {
            count_tiny_nonzero += 1;
        } else if x > 0.5 {
            count_top_half += 1;
        }
    }
    assert!(count_top_half >= 49_000_000);
    assert!(count_tiny_nonzero > 0);
}


fn f32_inclusive() {
    let mut r = fastrand::Rng::with_seed(0);
    let tiny = (-24.0f32).exp2();
    let mut count_top_half = 0;
    let mut count_tiny_nonzero = 0;
    let mut count_one = 0;
    for _ in 0..100_000_000 {
        let x = r.f32_inclusive();
        assert!((0.0..=1.0).contains(&x));
        if x == 1.0 {
            count_one += 1;
        } else if x > 0.5 {
            count_top_half += 1;
        } else if x > 0.0 && x < tiny {
            count_tiny_nonzero += 1;
        }
    }
    assert!(count_top_half >= 49_000_000);
    assert!(count_one > 0);
    assert!(count_tiny_nonzero > 0);
}


fn f64() {
    let mut r = fastrand::Rng::with_seed(0);
    let mut count_top_half = 0;
    for _ in 0..100_000_000 {
        let x = r.f64();
        assert!((0.0..1.0).contains(&x));
        if x > 0.5 {
            count_top_half += 1;
        }
    }
    assert!(count_top_half >= 49_000_000);
}


fn f64_inclusive() {
    let mut r = fastrand::Rng::with_seed(0);
    let mut count_top_half = 0;
    for _ in 0..100_000_000 {
        let x = r.f64_inclusive();
        assert!((0.0..=1.0).contains(&x));
        if x > 0.5 {
            count_top_half += 1;
        }
    }
    assert!(count_top_half >= 49_000_000);
}


fn digit() {
    for base in 1..36 {
        let result = fastrand::digit(base);
        assert!(result.is_ascii_digit() || result.is_ascii_lowercase());
    }
}


fn global_rng_choice() {
    let items = [1, 4, 9, 5, 2, 3, 6, 7, 8, 0];

    for item in &items {
        while fastrand::choice(&items).unwrap() != item {}
    }
}


fn global_rng_alphabetic() {
    for _ in 0..1000 {
        let result = fastrand::alphabetic();
        assert!(result.is_ascii_alphabetic())
    }
}


fn global_rng_lowercase() {
    for _ in 0..1000 {
        let result = fastrand::lowercase();
        assert!(result.is_ascii_lowercase())
    }
}


fn global_rng_uppercase() {
    for _ in 0..1000 {
        let result = fastrand::uppercase();
        assert!(result.is_ascii_uppercase())
    }
}


fn fill() {
    let mut r = fastrand::Rng::new();
    let mut a = [0u8; 64];
    let mut b = [0u8; 64];

    r.fill(&mut a);
    r.fill(&mut b);

    assert_ne!(a, b);

    let mut a = [0u8; 9];
    let mut b = [0u8; 9];

    r.fill(&mut a);
    r.fill(&mut b);

    assert_ne!(a, b);
}


fn global_rng_fill() {
    let mut a = [0u8; 64];
    let mut b = [0u8; 64];

    fastrand::fill(&mut a);
    fastrand::fill(&mut b);

    assert_ne!(a, b);
}


fn rng() {
    let mut r = fastrand::Rng::new();

    assert_ne!(r.u64(..), r.u64(..));

    r.seed(7);
    let a = r.u64(..);
    r.seed(7);
    let b = r.u64(..);
    assert_eq!(a, b);
}


fn rng_init() {
    let mut a = fastrand::Rng::new();
    let mut b = fastrand::Rng::new();
    assert_ne!(a.u64(..), b.u64(..));

    a.seed(7);
    b.seed(7);
    assert_eq!(a.u64(..), b.u64(..));
}


fn rng_digit() {
    let mut rng = fastrand::Rng::new();
    for base in 1..36 {
        let result = rng.digit(base);
        assert!(result.is_ascii_digit() || result.is_ascii_lowercase());
    }
}


#[should_panic]
fn rng_digit_panic_1() {
    let mut rng = fastrand::Rng::new();
    let _result = rng.digit(0);
}


#[should_panic]
fn rng_digit_panic_2() {
    let mut rng = fastrand::Rng::new();
    let base = rng.u32(37..);
    let _result = rng.digit(base);
}


fn with_seed() {
    let mut a = fastrand::Rng::with_seed(7);
    let mut b = fastrand::Rng::new();
    b.seed(7);
    assert_eq!(a.u64(..), b.u64(..));
}


fn choose_multiple() {
    let mut a = fastrand::Rng::new();
    let mut elements = (0..20).collect::<Vec<_>>();

    while !elements.is_empty() {
        let chosen = a.choose_multiple(0..20, 5);
        for &x in &chosen {
            elements.retain(|&y| y != x);
        }
    }

    let empty_elements: Vec<i32> = Vec::new();
    let empty_result = a.choose_multiple(empty_elements, 5);
    assert!(empty_result.is_empty());
}


fn choice() {
    let items = [1, 4, 9, 5, 2, 3, 6, 7, 8, 0];
    let mut r = fastrand::Rng::new();

    for item in &items {
        while r.choice(&items).unwrap() != item {}
    }
}


fn choice_empty() {
    let mut rng = fastrand::Rng::new();
    let data: Vec<i32> = Vec::new();
    let result = rng.choice(data);
    assert!(result.is_none());
}


fn lowercase() {
    let mut rng = fastrand::Rng::new();
    for _ in 0..1000 {
        let result = rng.lowercase();
        assert!(result.is_ascii_lowercase())
    }
}


fn alphabetic() {
    let mut rng = fastrand::Rng::new();
    for _ in 0..1000 {
        let result = rng.alphabetic();
        assert!(result.is_ascii_alphabetic())
    }
}


fn uppercase() {
    let mut rng = fastrand::Rng::new();
    for _ in 0..1000 {
        let result = rng.uppercase();
        assert!(result.is_ascii_uppercase())
    }
}


#[should_panic]
fn char_panic() {
    let mut rng = fastrand::Rng::new();
    let _result = rng.char('z'..='a');
}


fn char() {
    use core::ops::Bound;
    let mut rng = fastrand::Rng::new();

    let result = rng.char(..);
    assert!(result >= 0 as char && result <= core::char::MAX);

    let result = rng.char((Bound::Excluded('0'), Bound::Excluded('9')));
    for _ in 0..1000 {
        assert!(result > '0' && result < '9');
    }
}
