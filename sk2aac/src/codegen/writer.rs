use std::{
    fmt::Display,
    io::{Error as IoError, Write},
};

/// Helps write out indented codes.
pub struct CodeWriter<'a, W>
where
    W: Write,
{
    writer: &'a mut W,
    indent_width: usize,
    indent_spaces: String,
    termination: Option<(&'a str, bool)>,
}

impl<'a, W: Write> CodeWriter<'a, W> {
    /// Wraps writer for code generation.
    pub fn new(writer: &mut W, indent_width: usize) -> CodeWriter<W> {
        CodeWriter {
            writer,
            indent_width,
            indent_spaces: String::new(),
            termination: None,
        }
    }

    /// Executes function with indented.
    pub fn with_indent<'f, T, F>(&'f mut self, f: F) -> Result<T, IoError>
    where
        F: FnOnce(CodeWriter<'f, W>) -> Result<T, IoError>,
    {
        let inner = CodeWriter {
            writer: self.writer,
            indent_width: self.indent_width,
            indent_spaces: format!("{}{}", self.indent_spaces, " ".repeat(self.indent_width)),
            termination: None,
        };
        let returned = f(inner)?;
        Ok(returned)
    }

    /// Executes function with indented block.
    pub fn with_block<'f, T, F>(&'f mut self, f: F) -> Result<T, IoError>
    where
        F: FnOnce(CodeWriter<'f, W>) -> Result<T, IoError>,
    {
        self.write("{")?;

        let inner = CodeWriter {
            writer: self.writer,
            indent_width: self.indent_width,
            indent_spaces: format!("{}{}", self.indent_spaces, " ".repeat(self.indent_width)),
            termination: Some(("}", true)),
        };
        let returned = f(inner)?;
        Ok(returned)
    }

    /// Executes function with ifdef.
    pub fn with_ifdef<'f, T, F>(&'f mut self, identifier: &str, f: F) -> Result<T, IoError>
    where
        F: FnOnce(CodeWriter<'f, W>) -> Result<T, IoError>,
    {
        write!(self.writer, "#if {identifier}")?;
        writeln!(self.writer)?;

        let inner = CodeWriter {
            writer: self.writer,
            indent_width: self.indent_width,
            indent_spaces: self.indent_spaces.clone(),
            termination: Some(("#endif", false)),
        };
        let returned = f(inner)?;
        Ok(returned)
    }

    /// Writes a line.
    pub fn write<D: Display>(&mut self, line: D) -> Result<(), IoError> {
        write!(self.writer, "{}{line}", self.indent_spaces)?;
        writeln!(self.writer)?;
        Ok(())
    }

    /// Writes a blank line.
    pub fn write_empty(&mut self) -> Result<(), IoError> {
        writeln!(self.writer)?;
        Ok(())
    }

    /// Yields write process to given function.
    pub fn write_yield<F>(&mut self, f: F) -> Result<(), IoError>
    where
        F: FnOnce(&mut W) -> Result<(), IoError>,
    {
        write!(self.writer, "{}", self.indent_spaces)?;
        {
            let yielded = &mut self.writer;
            f(yielded)?;
        }
        writeln!(self.writer)?;
        Ok(())
    }

    /// Flushes current content.
    pub fn flush(&mut self) -> Result<(), IoError> {
        if let Some((text, with_indent)) = self.termination {
            if with_indent {
                let prev_indent = self.indent_spaces.len() - self.indent_width;
                let indent_space = &self.indent_spaces[..prev_indent];
                write!(self.writer, "{indent_space}")?;
            }
            write!(self.writer, "{text}")?;
            writeln!(self.writer)?;
            self.writer.flush()?;
            self.termination = None;
        }
        Ok(())
    }
}

impl<'a, W: Write> Drop for CodeWriter<'a, W> {
    fn drop(&mut self) {
        self.flush().expect("Flush failed");
    }
}
