use crate::{
    codegen::{
        aac_object::{Cond, Expr, ParameterDefinition, RendererFetch, StateDefinition, Transition},
        AacObject, CodeWriter,
    },
    descriptor::{Descriptor, ShapeKeyGroup, ShapeKeySwitch},
};

use std::{
    collections::HashMap,
    io::{Result as IoResult, Write},
};

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
    pub fn emit_code(&mut self, descriptor: &Descriptor) -> IoResult<()> {
        self.emit_preamble()?;
        self.writer.write_empty()?;
        self.emit_custom_editor()?;
        self.writer.write_empty()?;
        self.emit_behaviour(descriptor)
    }

    /// Emits using statements.
    fn emit_preamble(&mut self) -> IoResult<()> {
        self.writer.write(r#"using UnityEngine;"#)?;
        self.writer.with_ifdef("UNITY_EDITOR", |mut cw| {
            cw.write(r#"using UnityEditor;"#)?;
            cw.write(r#"using UnityEditor.Animations;"#)?;
            cw.write(r#"using VRC.SDK3.Avatars.Components;"#)?;
            cw.write(r#"using AnimatorAsCodeFramework.Examples;"#)
        })
    }

    /// Emits custom editor class.
    fn emit_custom_editor(&mut self) -> IoResult<()> {
        let class_name = self.class_name();

        self.writer.with_ifdef("UNITY_EDITOR", |mut ce| {
            ce.write(format_args!(r#"public class {class_name}_Editor : Editor"#))?;
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

    fn emit_behaviour(&mut self, descriptor: &Descriptor) -> IoResult<()> {
        let avatar_name = &self.avatar_name;
        let class_name = format!("SK2AACGenerator_{avatar_name}");

        self.writer
            .write(format_args!(r#"public class {class_name} : MonoBehaviour"#))?;
        self.writer.with_block(|mut cw| {
            cw.write(r#"public void GenerateAnimator()"#)?;
            cw.with_block(|mut cw| {
                cw.write(r#"var avatarDescriptor = GetComponent<VRCAvatarDescriptor>();"#)?;
                cw.write(r#"var aac = AacExample.AnimatorAsCode("SK2AAC {avatar_name}", avatarDescriptor, TargetContainer, AssetKey, AacExample.Options().WriteDefaultsOff());"#)?;
                cw.write(r#"// var fxDefault = aac.CreateMainFxLayer();"#)?;

                for switch in &descriptor.shape_switches {
                    Self::emit_shape_key_switch(&mut cw, switch)?;
                }
                for group in &descriptor.shape_groups {
                    Self::emit_shape_key_group(&mut cw, group)?;
                }
                Ok(())
            })
        })
    }

    fn emit_shape_key_switch(method: &mut CodeWriter<W>, switch: &ShapeKeySwitch) -> IoResult<()> {
        method.write_empty()?;
        method.write(format_args!(
            r#"// Shape Key Switch "{}""#,
            switch.common.name
        ))?;
        method.with_block(|mut b| {
            RendererFetch::new(&switch.common.mesh).write_into(&mut b)?;
            ParameterDefinition::Bool(&switch.common.name).write_into(&mut b)?;
            b.write_empty()?;

            // States
            StateDefinition::new("disabled", "false: Disabled")
                .blend_shapes(&[(switch.shape.as_str(), switch.disabled_value.get())])
                .write_into(&mut b)?;
            StateDefinition::new("enabled", "true: Enabled")
                .blend_shapes(&[(switch.shape.as_str(), switch.enabled_value.get())])
                .write_into(&mut b)?;
            b.write_empty()?;

            // Transitions
            Transition::new("disabled", "enabled")
                .cond(Cond::Term(Expr::IsTrue(
                    ParameterDefinition::PARAMETER_VARNAME,
                )))
                .write_into(&mut b)?;
            Transition::new("enabled", "disabled")
                .cond(Cond::Term(Expr::IsFalse(
                    ParameterDefinition::PARAMETER_VARNAME,
                )))
                .write_into(&mut b)
        })
    }

    fn emit_shape_key_group(method: &mut CodeWriter<W>, group: &ShapeKeyGroup) -> IoResult<()> {
        const ALIGN_UNIT: usize = 8;

        let default_values: HashMap<_, _> = group
            .defaults
            .iter()
            .map(|d| (d.shape.as_str(), d.value.get()))
            .collect();
        let mut drive_names: Vec<_> = group
            .options
            .iter()
            .map(|o| o.shapes.iter())
            .flatten()
            .map(|d| d.shape.as_str())
            .collect();
        drive_names.sort();
        drive_names.dedup();
        let default_drives: Vec<_> = drive_names
            .into_iter()
            .map(|n| {
                if let Some(dv) = default_values.get(n) {
                    (n, *dv)
                } else {
                    (n, 0.0)
                }
            })
            .collect();

        method.write_empty()?;
        method.write(format_args!(
            r#"// Shape Key Switch "{}""#,
            group.common.name
        ))?;
        method.with_block(|mut b| {
            RendererFetch::new(&group.common.mesh).write_into(&mut b)?;
            ParameterDefinition::Integer(&group.common.name).write_into(&mut b)?;
            b.write_empty()?;

            StateDefinition::new("disabled", "0: Disabled")
                .blend_shapes(&default_drives)
                .indented()
                .write_into(&mut b)?;

            // TODO: Check id duplicate
            let mut right_of = "disabled".to_string();
            for (i, option) in group.options.iter().enumerate() {
                let index = option.index.map(|i| i.get()).unwrap_or(i + 1);

                let state_name = format!("enabled{index}");
                let state_label = format!("{index}: {}", option.label);
                let blend_shapes: Vec<_> = option
                    .shapes
                    .iter()
                    .map(|d| (d.shape.as_str(), d.value.get()))
                    .collect();

                b.write_empty()?;

                // State
                let mut statedef =
                    StateDefinition::new(&state_name, &state_label).blend_shapes(&blend_shapes);
                if i % ALIGN_UNIT == 0 {
                    statedef = statedef.right_of(&right_of);
                    right_of = state_name.clone();
                }
                statedef.write_into(&mut b)?;

                // Transitions
                Transition::new("disabled", &state_name)
                    .cond(Cond::Term(Expr::IntEqual(
                        ParameterDefinition::PARAMETER_VARNAME,
                        index,
                    )))
                    .write_into(&mut b)?;
                Transition::exits(&state_name)
                    .cond(Cond::Term(Expr::IntNotEqual(
                        ParameterDefinition::PARAMETER_VARNAME,
                        index,
                    )))
                    .write_into(&mut b)?;
            }
            Ok(())
        })
    }
}
