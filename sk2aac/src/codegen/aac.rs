use crate::{
    codegen::{
        aac_object::{BehaviourClass, CustomEditorClass, Preamble},
        AacObject, CodeWriter,
    },
    descriptor::Descriptor,
};

use std::io::{Result as IoResult, Write};

/// Generates AaC code.
pub struct AacCodeGenerator<'a, W>
where
    W: Write,
{
    writer: CodeWriter<'a, W>,
    avatar_name: String,
}

impl<'a, W> AacCodeGenerator<'a, W>
where
    W: Write,
{
    /// Creates new generator.
    pub fn new<S: Into<String>>(writer: &'a mut W, avatar_name: S) -> IoResult<Self> {
        let writer = CodeWriter::new(writer, 4);
        Ok(AacCodeGenerator {
            writer,
            avatar_name: avatar_name.into(),
        })
    }

    pub fn class_name(&self) -> String {
        format!("SK2AACGenerator_{}", self.avatar_name)
    }
}

/// Code generation functions.
impl<'a, W> AacCodeGenerator<'a, W>
where
    W: Write,
{
    /// Emits the whole code.
    pub fn emit_code(&mut self, descriptor: Descriptor) -> IoResult<()> {
        let class_name = self.class_name();
        let editor_class_name = format!("{}_Editor", self.class_name());

        Preamble.write_into(&mut self.writer)?;
        self.writer.write_empty()?;
        CustomEditorClass::new(editor_class_name).write_into(&mut self.writer)?;
        self.writer.write_empty()?;
        BehaviourClass::new(class_name, descriptor).write_into(&mut self.writer)
    }
}
