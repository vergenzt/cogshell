#![allow(clippy::extra_unused_type_parameters)]

use anyhow::Error;
use std::panic::{RefUnwindSafe, UnwindSafe};


fn test_send() {
    fn assert_send<T: Send>() {}
    assert_send::<Error>();
}


fn test_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<Error>();
}


fn test_unwind_safe() {
    fn assert_unwind_safe<T: UnwindSafe>() {}
    assert_unwind_safe::<Error>();
}


fn test_ref_unwind_safe() {
    fn assert_ref_unwind_safe<T: RefUnwindSafe>() {}
    assert_ref_unwind_safe::<Error>();
}


fn test_unpin() {
    fn assert_unpin<T: Unpin>() {}
    assert_unpin::<Error>();
}
