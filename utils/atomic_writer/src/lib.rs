#![cfg_attr(test, feature(macro_metavar_expr_concat))]

#[cfg(test)]
mod test;

use std::{
    fs::write,
    io::{Cursor, Write},
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

pub static ERROR_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Accumulates writes into a buffer, then only writes to the output file when dropped.
pub struct AtomicFileWriter<'a> {
    curs: Cursor<Vec<u8>>,
    path: &'a PathBuf,
}

impl<'a> Write for AtomicFileWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.curs.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.curs.flush()
    }
}

impl<'a> AtomicFileWriter<'a> {
    pub fn new(path: &'a PathBuf) -> Self {
        let buf = Vec::new();
        let curs = Cursor::new(buf);
        Self { curs, path }
    }
}

impl Drop for AtomicFileWriter<'_> {
    fn drop(&mut self) {
        let result = write(&self.path, self.curs.get_ref());
        if let Err(err) = result {
            crate::ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
            eprint!("{err}")
        }
    }
}
