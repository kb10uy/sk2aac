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

    /// Extends current instance and indents.
    pub fn indent(&mut self) -> CodeWriter<W> {
        CodeWriter {
            writer: self.writer,
            indent_width: self.indent_width,
            indent_spaces: format!("{}{}", self.indent_spaces, " ".repeat(self.indent_width)),
            termination: None,
        }
    }

    /// Executes function with indented.
    pub fn with_indent<'f, T, F>(&'f mut self, f: F) -> Result<T, IoError>
    where
        F: FnOnce(CodeWriter<'f, W>) -> Result<T, IoError>,
    {
        let inner = self.indent();
        let returned = f(inner)?;
        Ok(returned)
    }

    /// Extends current instance and indents with block.
    pub fn indent_with_block(&mut self) -> Result<CodeWriter<W>, IoError> {
        self.write("{")?;
        Ok(CodeWriter {
            writer: self.writer,
            indent_width: self.indent_width,
            indent_spaces: format!("{}{}", self.indent_spaces, " ".repeat(self.indent_width)),
            termination: Some(("}", true)),
        })
    }

    /// Executes function with indented block.
    pub fn with_block<'f, T, F>(&'f mut self, f: F) -> Result<T, IoError>
    where
        F: FnOnce(CodeWriter<'f, W>) -> Result<T, IoError>,
    {
        let inner = self.indent_with_block()?;
        let returned = f(inner)?;
        Ok(returned)
    }

    /// Extends current instance and wraps with ifdef.
    pub fn wrap_ifdef(&mut self, identifier: &str) -> Result<CodeWriter<W>, IoError> {
        self.write_head(format_args!("#ifdef {identifier}"))?;
        Ok(CodeWriter {
            writer: self.writer,
            indent_width: self.indent_width,
            indent_spaces: self.indent_spaces.clone(),
            termination: Some(("#endif", false)),
        })
    }

    /// Executes function with ifdef.
    pub fn with_ifdef<'f, T, F>(&'f mut self, identifier: &str, f: F) -> Result<T, IoError>
    where
        F: FnOnce(CodeWriter<'f, W>) -> Result<T, IoError>,
    {
        let inner = self.wrap_ifdef(identifier)?;
        let returned = f(inner)?;
        Ok(returned)
    }

    /// Writes a line.
    pub fn write<D: Display>(&mut self, line: D) -> Result<(), IoError> {
        write!(self.writer, "{}{line}", self.indent_spaces)?;
        writeln!(self.writer)?;
        Ok(())
    }

    /// Writes a line without current indent.
    pub fn write_head<D: Display>(&mut self, line: D) -> Result<(), IoError> {
        write!(self.writer, "{line}")?;
        writeln!(self.writer)?;
        Ok(())
    }

    /// Writes a blank line.
    pub fn write_empty(&mut self) -> Result<(), IoError> {
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
            self.write_head(text)?;
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
