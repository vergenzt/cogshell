#![allow(clippy::let_underscore_untyped)]

#[rustversion::not(nightly)]
#[ignore = "requires nightly"]

fn test_backtrace() {}

#[rustversion::nightly]

fn test_backtrace() {
    use anyhow::anyhow;

    let error = anyhow!("oh no!");
    let _ = error.backtrace();
}
