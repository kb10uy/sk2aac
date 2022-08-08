use crate::{codegen::CodeWriter, descriptor::ShapeKeySwitch};

use std::io::{Result as IoResult, Write};

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
        let mut acg = AacCodeGenerator {
            writer,
            avatar_name: avatar_name.into(),
        };
        acg.emit_preamble()?;
        acg.emit_custom_editor()?;
        Ok(acg)
    }
}

/// Code generation functions.
impl<'a, W> AacCodeGenerator<'a, W>
where
    W: Write,
{
    /// Emits using statements.
    fn emit_preamble(&mut self) -> IoResult<()> {
        self.writer.write(r#"using UnityEngine;"#)?;
        self.writer.with_ifdef("UNITY_EDITOR", |mut cw| {
            cw.write(r#"using UnityEditor;"#)?;
            cw.write(r#"using UnityEditor.Animations;;"#)?;
            cw.write(r#"using VRC.SDK3.Avatars.Components;"#)?;
            cw.write(r#"using AnimatorAsCodeFramework.Examples;"#)
        })?;
        self.writer.write_empty()
    }

    /// Emits custom editor class.
    fn emit_custom_editor(&mut self) -> IoResult<()> {
        let avatar_name = &self.avatar_name;
        let class_name = format!("SK2AACGenerator_{avatar_name}");

        self.writer.with_ifdef("UNITY_EDITOR", |mut ce| {
            ce.write(format_args!(
                r#"public class {avatar_name}_Editor : Editor"#
            ))?;
            ce.with_block(|mut ce| {
                ce.write(r#"public override void OnInspectorGUI()"#)?;
                ce.with_block(|mut ce| {
                    ce.write(r#"base.OnInspectorGUI();"#)?;
                    ce.write(format_args!(r#"var executor = target as {class_name};"#))?;
                    ce.write(r#"if (GUILayout.Button("Generate"))"#)?;
                    ce.write(r#"{"#)?;
                    ce.write(r#"    executor.GenerateAnimator();"#)?;
                    ce.write(r#"}"#)
                })
            })
        })
    }

    fn emit_shape_key_switch(method: &mut CodeWriter<W>, switch: &ShapeKeySwitch) -> IoResult<()> {
        let switch_name = &switch.common.name;

        method.write(format_args!(r#"// Shape Key Switch "{switch_name}""#))?;
        method.with_block(|mut b| {
            b.write(format_args!(
                r#"var parameter = layer.BoolParameter("{switch_name}");"#
            ))?;
            // b.write(format_args!(r#"var stateDisabled = layer.NewState("Disabled").WithAnimation(aac.NewClip("{layer_name}_Disabled").BlendShape(renderer, "{shape_name}", 0.0f));"#))?;
            // StateDef("Disabled").blend_shape(&[("shape_name", 0.0)])

            b.write_empty()
        })?;

        method.write_empty()
    }
}
