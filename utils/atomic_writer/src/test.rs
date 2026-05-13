use super::{AtomicFileWriter, ERROR_COUNT};
use std::{fs, io::Write, path::PathBuf, sync::Mutex, sync::atomic::Ordering};
use tempfile::TempDir;

/// Serializes tests that read/modify the global `ERROR_COUNT`.
static ERROR_COUNT_LOCK: Mutex<()> = Mutex::new(());

fn unwritable_path() -> PathBuf {
    PathBuf::from("/nonexistent_dir_for_atomic_writer_tests/file")
}

fn temp_path(label: &str) -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join(label);
    (dir, path)
}

macro_rules! test_atomic_writer {
    ($label:ident : [$($write:expr),* $(,)?] => $expected:expr) => {
        #[test]
        fn ${concat(test_atomic_writer_, $label)}() {
            let (_dir, path) = temp_path(stringify!($label));
            {
                #[allow(unused_mut, unused_variables)]
                let mut writer = AtomicFileWriter::new(path.clone());
                $(writer.write_all($write).unwrap();)*
            }
            let expected: &[u8] = $expected;
            assert_eq!(fs::read(&path).unwrap(), expected);
        }
    };
}

test_atomic_writer!(empty: [] => b"");
test_atomic_writer!(single: [b"hello"] => b"hello");
test_atomic_writer!(multiple: [b"foo", b"bar", b"baz"] => b"foobarbaz");
test_atomic_writer!(with_nulls: [b"foo\0bar\0"] => b"foo\0bar\0");
test_atomic_writer!(binary: [&[0u8, 1, 2, 254, 255]] => &[0u8, 1, 2, 254, 255]);

#[test]
fn test_atomic_writer_not_written_until_drop() {
    let (_dir, path) = temp_path("not_written_until_drop");
    let mut writer = AtomicFileWriter::new(path.clone());
    writer.write_all(b"hello").unwrap();
    assert!(!path.exists(), "file should not exist before drop");
    drop(writer);
    assert_eq!(fs::read(&path).unwrap(), b"hello");
}

#[test]
fn test_atomic_writer_overwrites_existing() {
    let (_dir, path) = temp_path("overwrites_existing");
    fs::write(&path, b"original contents").unwrap();
    {
        let mut writer = AtomicFileWriter::new(path.clone());
        writer.write_all(b"new").unwrap();
    }
    assert_eq!(fs::read(&path).unwrap(), b"new");
}

macro_rules! test_error_count_delta {
    ($label:ident : $body:block => $delta:expr) => {
        #[test]
        fn ${concat(test_atomic_writer_error_count_, $label)}() {
            let _guard = ERROR_COUNT_LOCK.lock().unwrap();
            let before = ERROR_COUNT.load(Ordering::Relaxed);
            $body
            assert_eq!(ERROR_COUNT.load(Ordering::Relaxed), before + $delta);
        }
    };
}

test_error_count_delta!(single_failure: {
    let mut writer = AtomicFileWriter::new(unwritable_path());
    writer.write_all(b"hello").unwrap();
} => 1);

test_error_count_delta!(success_unchanged: {
    let (_dir, path) = temp_path("error_count_unchanged");
    let mut writer = AtomicFileWriter::new(path);
    writer.write_all(b"hello").unwrap();
} => 0);

test_error_count_delta!(multiple_failures: {
    for _ in 0..3 {
        drop(AtomicFileWriter::new(unwritable_path()));
    }
} => 3);
