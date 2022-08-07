use std::io::{Error as IoError, Write};

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

    /// Extends current instance and wraps with ifdef.
    pub fn wrap_ifdef(&mut self, identifier: &str) -> Result<CodeWriter<W>, IoError> {
        self.write_without_indent(&format!("#ifdef {identifier}"))?;
        Ok(CodeWriter {
            writer: self.writer,
            indent_width: self.indent_width,
            indent_spaces: self.indent_spaces.clone(),
            termination: Some(("#endif", false)),
        })
    }

    /// Writes a line.
    pub fn write(&mut self, line: &str) -> Result<(), IoError> {
        write!(self.writer, "{}", self.indent_spaces)?;
        write!(self.writer, "{line}")?;
        writeln!(self.writer)?;
        Ok(())
    }

    /// Writes a line without current indent.
    pub fn write_without_indent(&mut self, line: &str) -> Result<(), IoError> {
        write!(self.writer, "{line}")?;
        writeln!(self.writer)?;
        Ok(())
    }

    /// Flushes current content.
    pub fn flush(&mut self) -> Result<(), IoError> {
        if let Some((text, with_indent)) = self.termination {
            if with_indent {
                self.write(text)?;
            } else {
                self.write_without_indent(text)?;
            }
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
