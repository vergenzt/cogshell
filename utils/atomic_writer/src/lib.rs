#![cfg_attr(test, feature(macro_metavar_expr_concat))]

#[cfg(test)]
mod test;

use std::{
    fs::write,
    io::{Cursor, Write},
    ops::Deref,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

pub static ERROR_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Accumulates writes into a buffer, then only writes to the output file when dropped.
pub struct AtomicFileWriter {
    curs: Cursor<Vec<u8>>,
    path: PathBuf,
}

impl Write for AtomicFileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.curs.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.curs.flush()
    }
}

impl AtomicFileWriter {
    pub fn new(path: PathBuf) -> Self {
        let buf = Vec::new();
        let curs = Cursor::new(buf);
        Self { curs, path }
    }
}

impl Deref for AtomicFileWriter {
    type Target = dyn Write;
    fn deref(&self) -> &Self::Target {
        &self.curs
    }
}

impl Drop for AtomicFileWriter {
    fn drop(&mut self) {
        let result = write(&self.path, self.curs.get_ref());
        if let Err(err) = result {
            crate::ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
            eprint!("{err}")
        }
    }
}
