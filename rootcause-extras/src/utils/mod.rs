use core::str;
use std::{fmt, io};

use rootcause::handlers::{AttachmentHandler, ContextHandler};

#[derive(Clone, Copy)]
pub struct FormatFunctions<'a, Data> {
    data: &'a Data,
    display: fn(&Data, &mut fmt::Formatter) -> fmt::Result,
    debug: fn(&Data, &mut fmt::Formatter) -> fmt::Result,
}

impl<'a, Data> FormatFunctions<'a, Data> {
    fn attachment<H>(data: &'a Data) -> Self
    where
        H: AttachmentHandler<Data>,
    {
        Self {
            data,
            display: H::display,
            debug: H::debug,
        }
    }

    fn context<H>(data: &'a Data) -> Self
    where
        H: ContextHandler<Data>,
    {
        Self {
            data,
            display: H::display,
            debug: H::debug,
        }
    }

    fn new(data: &'a Data) -> Self
    where
        Data: fmt::Display + fmt::Debug,
    {
        Self {
            data,
            display: fmt::Display::fmt,
            debug: fmt::Debug::fmt,
        }
    }
}

impl<'a, Data> fmt::Display for FormatFunctions<'a, Data> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.display)(self.data, f)
    }
}

impl<'a, Data> fmt::Debug for FormatFunctions<'a, Data> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.debug)(self.data, f)
    }
}

pub struct FormatterWriter<'a, 'b: 'a>(&'a mut fmt::Formatter<'b>);

impl<'a, 'b: 'a> io::Write for FormatterWriter<'a, 'b> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let utf8 = str::from_utf8(buf).map_err(io::Error::other)?;
        self.0.write_str(utf8).map_err(io::Error::other)?;
        Ok(utf8.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a, 'b: 'a> FormatterWriter<'a, 'b> {
    pub fn new(fmt: &'a mut fmt::Formatter<'b>) -> Self {
        Self(fmt)
    }
}
