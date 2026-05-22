use std::io::{self, Error, ErrorKind, Write};

/// A wrapper around any `std::io::Write` instance which applies a transformation function
/// to each `\n`-terminated line of input before passing along to the underlying writer.
///
/// The Strings passed to `line_transform` *include* any trailing newline; if such newline
/// is missing, it means this is the last write before EOF.
///
/// If any bytes written to this wrapper are not valid UTF-8, an `io::Error` with
/// `io::ErrorKind::InvalidData` will be the result.
pub struct LineTransformWriter<W: Write, F: Fn(&mut String)> {
    pub inner: W,
    pub line_transform: F,
    partial_line: Vec<u8>,
}

impl<W: Write, F: Fn(&mut String)> LineTransformWriter<W, F> {
    pub fn new(inner: W, line_transform: F) -> Self {
        Self {
            inner,
            line_transform,
            partial_line: vec![],
        }
    }

    /// Transforms the given line and then writes it to `inner`. The given `line` must not
    /// have any interior newline (`\n`) characters, but may end with one. `self` also
    /// must not have any unwritten partial line fragments stored (which will only be the
    /// case if `io::Write` methods have been called directly).
    pub fn write_line_transformed<'a>(
        &mut self,
        line: &'a mut String,
    ) -> io::Result<&'a mut String> {
        if !self.partial_line.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "non-empty `partial_line` state!",
            ));
        }
        if let Some(nl) = line.find('\n')
            && nl < line.len() - 1
        {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Interior newline found at {nl}!"),
            ));
        }
        (*&self.line_transform)(line);
        self.inner.write_all(line.as_bytes()).map(|_| line)
    }

    /// Internal method for implementing `io::Write`
    fn flush_partial_line(&mut self) -> io::Result<()> {
        let line_res = String::from_utf8(self.partial_line.split_off(0));
        let mut line = line_res.map_err(|u8err| Error::new(ErrorKind::InvalidData, u8err))?;
        self.write_line_transformed(&mut line)?;
        Ok(())
    }
}

impl<W: Write, F: Fn(&mut String)> Write for LineTransformWriter<W, F> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut n_written = 0;
        for line_part in buf.split_inclusive(|b| *b == b'\n') {
            self.partial_line.extend(line_part);
            if self.partial_line.last().copied() == Some(b'\n') {
                n_written += self.partial_line.len();
                self.flush_partial_line()?;
            }
        }
        Ok(n_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_partial_line()?;
        self.inner.flush()
    }
}
