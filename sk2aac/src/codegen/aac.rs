use crate::{
    codegen::CodeWriter,
    descriptor::{Descriptor, ShapeKeyGroup, ShapeKeySwitch},
};

use std::{
    collections::HashMap,
    io::{prelude::*, Result as IoResult},
    iter::{once, repeat, zip},
};

/// Reads the descriptor and generates AAC code.
pub fn write_descriptor_code<W: Write>(writer: &mut W, descriptor: Descriptor) -> IoResult<String> {
    let mut writer = CodeWriter::new(writer, 4);
    let class_name = format!("SK2AACGenerator_{}", descriptor.name);
    let editor_class_name = format!("SK2AACGenerator_{}_Editor", descriptor.name);

    Preamble.write_into(&mut writer)?;
    writer.write_empty()?;
    CustomEditorClass::new(editor_class_name).write_into(&mut writer)?;
    writer.write_empty()?;
    BehaviourClass::new(class_name.clone(), descriptor).write_into(&mut writer)?;

    Ok(class_name)
}

/// Emits piece of AAC code.
trait AacObject {
    fn write_into<W: Write>(self, w: &mut CodeWriter<W>) -> IoResult<()>;
}

struct Preamble;

impl AacObject for Preamble {
    fn write_into<W: Write>(self, w: &mut CodeWriter<W>) -> IoResult<()> {
        w.write(r#"// This file is generated by sk2aac"#)?;
        w.write(r#"using UnityEngine;"#)?;
        w.with_ifdef("UNITY_EDITOR", |mut cw| {
            cw.write(r#"using UnityEditor;"#)?;
            cw.write(r#"using UnityEditor.Animations;"#)?;
            cw.write(r#"using VRC.SDK3.Avatars.Components;"#)?;
            cw.write(r#"using AnimatorAsCodeFramework.Examples;"#)
        })
    }
}

/// `public class <AvatarName>_Editor...`
struct CustomEditorClass(String);

impl CustomEditorClass {
    fn new(class_name: impl Into<String>) -> Self {
        CustomEditorClass(class_name.into())
    }
}

impl AacObject for CustomEditorClass {
    fn write_into<W: Write>(self, w: &mut CodeWriter<W>) -> IoResult<()> {
        let class_name = self.0;

        w.with_ifdef("UNITY_EDITOR", |mut ce| {
            ce.write(format_args!(r#"public class {class_name} : Editor"#))?;
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
}

/// `public class <AvatarName>`
struct BehaviourClass {
    class_name: String,
    descriptor: Descriptor,
}

impl BehaviourClass {
    fn new(class_name: impl Into<String>, descriptor: Descriptor) -> Self {
        BehaviourClass {
            class_name: class_name.into(),
            descriptor,
        }
    }
}

impl AacObject for BehaviourClass {
    fn write_into<W: Write>(self, w: &mut CodeWriter<W>) -> IoResult<()> {
        let BehaviourClass {
            class_name,
            descriptor,
        } = self;

        w.write(format_args!(r#"public class {class_name} : MonoBehaviour"#))?;
        w.with_block(|mut cw| {
            cw.write(r#"public void GenerateAnimator()"#)?;
            cw.with_block(|mut cw| {
                cw.write(r#"var avatarDescriptor = GetComponent<VRCAvatarDescriptor>();"#)?;
                cw.write(r#"var aac = AacExample.AnimatorAsCode("SK2AAC {avatar_name}", avatarDescriptor, TargetContainer, AssetKey, AacExample.Options().WriteDefaultsOff());"#)?;
                cw.write(r#"// var fxDefault = aac.CreateMainFxLayer();"#)?;

                for switch in descriptor.shape_switches {
                    ShapeKeySwitchBlock::new(switch).write_into(&mut cw)?;
                }
                for group in descriptor.shape_groups {
                    ShapeKeyGroupBlock::new(group).write_into(&mut cw)?;
                }
                Ok(())
            })
        })
    }
}

/// `// Shape Key Switch ...`
struct ShapeKeySwitchBlock(ShapeKeySwitch);

impl ShapeKeySwitchBlock {
    fn new(switch: ShapeKeySwitch) -> Self {
        ShapeKeySwitchBlock(switch)
    }
}

impl AacObject for ShapeKeySwitchBlock {
    fn write_into<W: Write>(self, w: &mut CodeWriter<W>) -> IoResult<()> {
        let switch = self.0;

        w.write_empty()?;
        w.write(format_args!(
            r#"// Shape Key Switch "{}""#,
            switch.common.name
        ))?;
        w.with_block(|mut b| {
            RendererFetch::new(switch.common.mesh).write_into(&mut b)?;
            ParameterDefinition::Bool(switch.common.name).write_into(&mut b)?;
            b.write_empty()?;

            // States
            StateDefinition::new("disabled", "false: Disabled")
                .blend_shapes([(switch.shape.clone(), switch.disabled_value.get())])
                .write_into(&mut b)?;
            StateDefinition::new("enabled", "true: Enabled")
                .blend_shapes([(switch.shape.clone(), switch.enabled_value.get())])
                .write_into(&mut b)?;
            b.write_empty()?;

            // Transitions
            Transition::new("disabled", "enabled")
                .cond(Cond::Term(Expr::IsTrue(
                    ParameterDefinition::PARAMETER_VARNAME.into(),
                )))
                .write_into(&mut b)?;
            Transition::new("enabled", "disabled")
                .cond(Cond::Term(Expr::IsFalse(
                    ParameterDefinition::PARAMETER_VARNAME.into(),
                )))
                .write_into(&mut b)
        })
    }
}

/// `// Shape Key Group ...`
struct ShapeKeyGroupBlock(ShapeKeyGroup);

impl ShapeKeyGroupBlock {
    const ALIGN_UNIT: usize = 8;

    fn new(group: ShapeKeyGroup) -> Self {
        ShapeKeyGroupBlock(group)
    }
}

impl AacObject for ShapeKeyGroupBlock {
    fn write_into<W: Write>(self, w: &mut CodeWriter<W>) -> IoResult<()> {
        let group = self.0;

        let default_values: HashMap<_, _> = group
            .defaults
            .into_iter()
            .map(|d| (d.shape, d.value.get()))
            .collect();
        let mut drive_names: Vec<_> = group
            .options
            .iter()
            .map(|o| o.shapes.iter())
            .flatten()
            .map(|d| d.shape.clone())
            .collect();
        drive_names.sort();
        drive_names.dedup();
        let default_drives = drive_names.into_iter().map(|n| {
            let value = default_values.get(&n).copied().unwrap_or(0.0);
            (n, value)
        });

        w.write_empty()?;
        w.write(format_args!(
            r#"// Shape Key Switch "{}""#,
            group.common.name
        ))?;
        w.with_block(|mut b| {
            RendererFetch::new(group.common.mesh).write_into(&mut b)?;
            ParameterDefinition::Integer(group.common.name).write_into(&mut b)?;
            b.write_empty()?;

            StateDefinition::new("disabled", "0: Disabled")
                .blend_shapes(default_drives)
                .indented()
                .write_into(&mut b)?;

            // TODO: Check id duplicate
            let mut right_of = "disabled".to_string();
            for (i, option) in group.options.into_iter().enumerate() {
                let index = option.index.map(|i| i.get()).unwrap_or(i + 1);

                let state_name = format!("enabled{index}");
                let state_label = format!("{index}: {}", option.label);
                let blend_shapes = option.shapes.into_iter().map(|d| (d.shape, d.value.get()));

                b.write_empty()?;

                // State
                let mut statedef = StateDefinition::new(state_name.clone(), state_label)
                    .blend_shapes(blend_shapes);
                if i % Self::ALIGN_UNIT == 0 {
                    statedef = statedef.right_of(right_of);
                    right_of = state_name.clone();
                }
                statedef.write_into(&mut b)?;

                // Transitions
                Transition::new("disabled", state_name.clone())
                    .cond(Cond::Term(Expr::IntEqual(
                        ParameterDefinition::PARAMETER_VARNAME.into(),
                        index,
                    )))
                    .write_into(&mut b)?;
                Transition::exits(state_name.clone())
                    .cond(Cond::Term(Expr::IntNotEqual(
                        ParameterDefinition::PARAMETER_VARNAME.into(),
                        index,
                    )))
                    .write_into(&mut b)?;
            }
            Ok(())
        })
    }
}

/// `var renderer = ...`
struct RendererFetch(String);

impl RendererFetch {
    fn new(name: impl Into<String>) -> Self {
        RendererFetch(name.into())
    }
}

impl AacObject for RendererFetch {
    fn write_into<W: Write>(self, w: &mut CodeWriter<W>) -> IoResult<()> {
        let object_name = self.0;

        w.write_yield(|w| {
            write!(
                w,
                r#"var renderer = (SkinnedMeshRenderer) gameObject.transform.Find("{object_name}").GetComponent<SkinnedMeshRenderer>();"#
            )
        })
    }
}

/// `var parameter = ...`
enum ParameterDefinition {
    Bool(String),
    Integer(String),
}

impl ParameterDefinition {
    const PARAMETER_VARNAME: &'static str = "parameter";
}

impl AacObject for ParameterDefinition {
    fn write_into<W: Write>(self, w: &mut CodeWriter<W>) -> IoResult<()> {
        let param_name = Self::PARAMETER_VARNAME;
        match self {
            ParameterDefinition::Bool(p) => w.write(format_args!(
                r#"var {param_name} = layer.BoolParameter("{p}");"#
            )),
            ParameterDefinition::Integer(p) => w.write(format_args!(
                r#"var {param_name} = layer.IntParameter("{p}");"#
            )),
        }
    }
}

/// `var state = ...`
struct StateDefinition {
    state_var: String,
    state_name: String,
    blend_shapes: Option<Vec<(String, f64)>>,
    renderer: String,
    right_of: Option<String>,
    indented: bool,
}

impl StateDefinition {
    fn new(state_var: impl Into<String>, state_name: impl Into<String>) -> Self {
        StateDefinition {
            state_var: state_var.into(),
            state_name: state_name.into(),
            blend_shapes: None,
            renderer: "renderer".into(),
            right_of: None,
            indented: false,
        }
    }

    fn right_of(mut self, state_name: impl Into<String>) -> Self {
        self.right_of = Some(state_name.into());
        self
    }

    fn blend_shapes(mut self, items: impl IntoIterator<Item = (String, f64)>) -> Self {
        self.blend_shapes = Some(items.into_iter().collect());
        self
    }

    fn indented(mut self) -> Self {
        self.indented = true;
        self
    }
}

impl AacObject for StateDefinition {
    fn write_into<W: Write>(self, w: &mut CodeWriter<W>) -> IoResult<()> {
        let StateDefinition {
            state_var,
            state_name,
            renderer,
            ..
        } = self;

        if self.indented {
            w.write_yield(|w| {
                write!(w, r#"var {state_var} = layer.NewState("{state_name}")"#)?;
                if let Some(ro) = self.right_of {
                    write!(w, r#".RightOf({ro})"#)?;
                }
                write!(w, r#".WithAnimation("#)
            })?;

            w.with_indent(|mut b| {
                b.write(r#"aac.NewClip()"#)?;
                if let Some(blend_shapes) = self.blend_shapes {
                    b.with_indent(|mut b| {
                        for (name, value) in blend_shapes {
                            b.write(format_args!(
                                r#".BlendShape({renderer}, "{name}", {value:.1}f)"#
                            ))?;
                        }
                        Ok(())
                    })?;
                }
                Ok(())
            })?;

            w.write(r#");"#)
        } else {
            w.write_yield(|w| {
                write!(w, r#"var {state_var} = layer.NewState("{state_name}")"#)?;
                if let Some(ro) = self.right_of {
                    write!(w, r#".RightOf({ro})"#)?;
                }
                write!(w, r#".WithAnimation(aac.NewClip()"#)?;
                if let Some(blend_shapes) = self.blend_shapes {
                    for (name, value) in blend_shapes {
                        write!(w, r#".BlendShape({renderer}, "{name}", {value:.1}f)"#)?;
                    }
                }
                write!(w, r#");"#)
            })
        }
    }
}

/// `state.TransitionTo()...`
struct Transition {
    from: Option<String>,
    to: Option<String>,
    condition: Option<Cond>,
}

impl Transition {
    fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Transition {
            from: Some(from.into()),
            to: Some(to.into()),
            condition: None,
        }
    }

    fn exits(from: impl Into<String>) -> Self {
        Transition {
            from: Some(from.into()),
            to: None,
            condition: None,
        }
    }

    fn cond(mut self, condition: Cond) -> Self {
        if condition.is_valid() {
            self.condition = Some(condition);
        }
        self
    }
}

impl AacObject for Transition {
    fn write_into<W: Write>(self, w: &mut CodeWriter<W>) -> IoResult<()> {
        let condition = match self.condition {
            Some(c) => c,
            None => return Ok(()),
        };

        w.write_yield(|w| {
            match (self.from, self.to) {
                (Some(f), Some(t)) => write!(w, r#"{f}.TransitionsTo({t})"#)?,
                (Some(f), None) => write!(w, r#"{f}.Exits()"#)?,
                _ => unreachable!("Invalid transition"),
            }
            condition.write(w)?;
            write!(w, r#";"#)
        })
    }
}

enum Expr {
    IntEqual(String, usize),
    IntNotEqual(String, usize),
    IsTrue(String),
    IsFalse(String),
}

impl Expr {
    fn write<W: Write>(&self, w: &mut W) -> IoResult<()> {
        match self {
            Expr::IntEqual(p, v) => write!(w, r#"{p}.IsEqualTo({v})"#),
            Expr::IntNotEqual(p, v) => write!(w, r#"{p}.IsNotEqualTo({v})"#),
            Expr::IsTrue(p) => write!(w, r#"{p}.IsTrue()"#),
            Expr::IsFalse(p) => write!(w, r#"{p}.IsFalse()"#),
        }
    }
}

enum Cond {
    Or(Vec<Cond>),
    And(Vec<Cond>),
    Term(Expr),
}

impl Cond {
    fn is_valid(&self) -> bool {
        match self {
            Cond::Or(_) => self.is_valid_or(),
            Cond::And(_) => self.is_valid_and(),
            Cond::Term(_) => true,
        }
    }

    fn is_valid_and(&self) -> bool {
        match self {
            Cond::And(terms) => terms.iter().all(|t| matches!(t, Cond::Term(_))),
            _ => false,
        }
    }

    fn is_valid_or(&self) -> bool {
        match self {
            Cond::Or(terms) => terms.iter().all(|t| t.is_valid_and()),
            _ => false,
        }
    }

    fn write<W: Write>(&self, w: &mut W) -> IoResult<()> {
        match self {
            Cond::Or(and_clauses) => {
                let or_splits = once("").chain(repeat(".Or()"));
                for (and_clause, or) in zip(and_clauses, or_splits) {
                    write!(w, r#"{or}"#)?;
                    and_clause.write(w)?;
                }
            }
            Cond::And(terms) => {
                let method_names = once("When").chain(repeat("And"));
                for (term, method) in zip(terms, method_names) {
                    let term = match term {
                        Cond::Term(t) => t,
                        _ => unreachable!("Should be validated"),
                    };
                    write!(w, r#".{method}("#)?;
                    term.write(w)?;
                    write!(w, r#")"#)?;
                }
            }
            Cond::Term(t) => {
                write!(w, r#".When("#)?;
                t.write(w)?;
                write!(w, r#")"#)?;
            }
        }
        Ok(())
    }
}
