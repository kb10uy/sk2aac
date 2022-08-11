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

    Preamble.write_into(&mut writer)?;
    writer.write_empty()?;
    CustomEditorClass::new(class_name.clone()).write_into(&mut writer)?;
    writer.write_empty()?;
    BehaviourClass::new(class_name.clone(), descriptor).write_into(&mut writer)?;

    Ok(class_name)
}

/// Emits piece of AAC code.
trait AacObject {
    fn write_into<W: Write>(self, w: &mut CodeWriter<W>) -> IoResult<()>;
}

/// Tracking layer.
#[derive(Debug, Clone, Copy)]
enum AnimationTarget {
    Eyelids,
    JawAndMouth,
}

impl AnimationTarget {
    fn tracking_element(&self) -> &str {
        match self {
            AnimationTarget::Eyelids => "TrackingElement.Eyes",
            AnimationTarget::JawAndMouth => "TrackingElement.Mouth",
        }
    }

    fn displayed_name(&self) -> &str {
        match self {
            AnimationTarget::Eyelids => "Eyes",
            AnimationTarget::JawAndMouth => "Mouth",
        }
    }
}

#[derive(Debug, Clone)]
struct Preamble;

impl AacObject for Preamble {
    fn write_into<W: Write>(self, w: &mut CodeWriter<W>) -> IoResult<()> {
        w.write(r#"// This file is generated by sk2aac"#)?;
        w.write(r#"using UnityEngine;"#)?;
        w.with_ifdef("UNITY_EDITOR", |mut cw| {
            cw.write(r#"using UnityEditor;"#)?;
            cw.write(r#"using UnityEditor.Animations;"#)?;
            cw.write(r#"using VRC.SDK3.Avatars.Components;"#)?;
            cw.write(r#"using static AnimatorAsCode.V0.AacFlState;"#)?;
            cw.write(r#"using AnimatorAsCodeFramework.Examples;"#)
        })
    }
}

/// `public class <AvatarName>_Editor...`
#[derive(Debug, Clone)]
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
}

/// `public class <AvatarName>`
#[derive(Debug, Clone)]
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
                cw.write(r#"var aac = AacExample.AnimatorAsCode("SK2AAC", avatarDescriptor, TargetContainer, AssetKey, AacExample.Options().WriteDefaultsOff());"#)?;
                cw.write(r#"// var fxDefault = aac.CreateMainFxLayer();"#)?;

                let eyelids_preventions = descriptor
                    .shape_groups
                    .iter()
                    .filter_map(|g| {
                        if g.common.prevent_eyelids {
                            Some(ParameterType::Integer(g.common.name.clone()))
                        } else {
                            None
                        }
                    })
                    .chain(descriptor.shape_switches.iter().filter_map(|g| {
                        if g.common.prevent_eyelids {
                            Some(ParameterType::Bool(g.common.name.clone()))
                        } else {
                            None
                        }
                    }));
                cw.write_empty()?;
                PreventionLayer::new(AnimationTarget::Eyelids, eyelids_preventions)
                    .write_into(&mut cw)?;

                let mouth_preventions = descriptor
                    .shape_groups
                    .iter()
                    .filter_map(|g| {
                        if g.common.prevent_mouth {
                            Some(ParameterType::Integer(g.common.name.clone()))
                        } else {
                            None
                        }
                    })
                    .chain(descriptor.shape_switches.iter().filter_map(|g| {
                        if g.common.prevent_mouth {
                            Some(ParameterType::Bool(g.common.name.clone()))
                        } else {
                            None
                        }
                    }));
                cw.write_empty()?;
                PreventionLayer::new(AnimationTarget::JawAndMouth, mouth_preventions)
                    .write_into(&mut cw)?;

                for switch in descriptor.shape_switches {
                    cw.write_empty()?;
                    ShapeKeySwitchLayer::new(switch).write_into(&mut cw)?;
                }
                for group in descriptor.shape_groups {
                    cw.write_empty()?;
                    ShapeKeyGroupLayer::new(group).write_into(&mut cw)?;
                }
                Ok(())
            })
        })
    }
}

/// `Blocks default animation...`
#[derive(Debug, Clone)]
struct PreventionLayer {
    target: AnimationTarget,
    params: Vec<ParameterType>,
}

impl PreventionLayer {
    fn new(
        target: AnimationTarget,
        params: impl IntoIterator<Item = ParameterType>,
    ) -> PreventionLayer {
        PreventionLayer {
            target,
            params: params.into_iter().collect(),
        }
    }
}

impl AacObject for PreventionLayer {
    fn write_into<W: Write>(self, w: &mut CodeWriter<W>) -> IoResult<()> {
        if self.params.is_empty() {
            return Ok(());
        }

        let animated_condition = Cond::Or(
            self.params
                .iter()
                .filter_map(|p| match p {
                    ParameterType::Bool(p) => Some(Cond::Term(Expr::IsTrue(format!("param{p}")))),
                    ParameterType::Integer(p) => {
                        Some(Cond::Term(Expr::IntNotEqual(format!("param{p}"), 0)))
                    }
                    _ => None,
                })
                .collect(),
        );
        let tracking_condition = Cond::And(
            self.params
                .iter()
                .filter_map(|p| match p {
                    ParameterType::Bool(p) => Some(Cond::Term(Expr::IsFalse(format!("param{p}")))),
                    ParameterType::Integer(p) => {
                        Some(Cond::Term(Expr::IntEqual(format!("param{p}"), 0)))
                    }
                    _ => None,
                })
                .collect(),
        );

        w.write(format_args!(r#"// Prevents Animation"#))?;
        w.with_block(|mut b| {
            LayerDefinition::new(format!("{}_TrackingControl", self.target.displayed_name()))
                .write_into(&mut b)?;

            for param in self.params {
                let var_name = match &param {
                    ParameterType::Bool(p) => format!("param{p}"),
                    ParameterType::Integer(p) => format!("param{p}"),
                    _ => continue,
                };
                ParameterDefinition::new(param)
                    .var_name(var_name)
                    .write_into(&mut b)?;
            }
            b.write_empty()?;

            // States
            StateDefinition::new("tracking", "Tracking").write_into(&mut b)?;
            StateOptions::new("tracking")
                .tracks(self.target)
                .write_into(&mut b)?;
            StateDefinition::new("animated", "Animated").write_into(&mut b)?;
            StateOptions::new("animated")
                .animates(self.target)
                .write_into(&mut b)?;
            b.write_empty()?;

            // Transitions
            Transition::new("tracking", "animated")
                .cond(animated_condition)
                .write_into(&mut b)?;
            Transition::new("animated", "tracking")
                .cond(tracking_condition)
                .write_into(&mut b)
        })
    }
}

/// `// Shape Key Switch ...`
#[derive(Debug, Clone)]
struct ShapeKeySwitchLayer(ShapeKeySwitch);

impl ShapeKeySwitchLayer {
    fn new(switch: ShapeKeySwitch) -> Self {
        ShapeKeySwitchLayer(switch)
    }
}

impl AacObject for ShapeKeySwitchLayer {
    fn write_into<W: Write>(self, w: &mut CodeWriter<W>) -> IoResult<()> {
        let switch = self.0;

        w.write(format_args!(
            r#"// Shape Key Switch "{}""#,
            switch.common.name
        ))?;
        w.with_block(|mut b| {
            LayerDefinition::new(format!("{}", switch.common.name)).write_into(&mut b)?;
            RendererFetch::new(switch.common.mesh).write_into(&mut b)?;
            ParameterDefinition::bool(switch.common.name).write_into(&mut b)?;
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
                    ParameterDefinition::DEFAULT_VARNAME.into(),
                )))
                .write_into(&mut b)?;
            Transition::new("enabled", "disabled")
                .cond(Cond::Term(Expr::IsFalse(
                    ParameterDefinition::DEFAULT_VARNAME.into(),
                )))
                .write_into(&mut b)
        })
    }
}

/// `// Shape Key Group ...`
#[derive(Debug, Clone)]
struct ShapeKeyGroupLayer(ShapeKeyGroup);

impl ShapeKeyGroupLayer {
    const ALIGN_UNIT: usize = 8;

    fn new(group: ShapeKeyGroup) -> Self {
        ShapeKeyGroupLayer(group)
    }
}

impl AacObject for ShapeKeyGroupLayer {
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

        w.write(format_args!(
            r#"// Shape Key Switch "{}""#,
            group.common.name
        ))?;
        w.with_block(|mut b| {
            LayerDefinition::new(format!("{}", group.common.name)).write_into(&mut b)?;
            RendererFetch::new(group.common.mesh).write_into(&mut b)?;
            ParameterDefinition::integer(group.common.name).write_into(&mut b)?;
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
                        ParameterDefinition::DEFAULT_VARNAME.into(),
                        index,
                    )))
                    .write_into(&mut b)?;
                Transition::exits(state_name.clone())
                    .cond(Cond::Term(Expr::IntNotEqual(
                        ParameterDefinition::DEFAULT_VARNAME.into(),
                        index,
                    )))
                    .write_into(&mut b)?;
            }
            Ok(())
        })
    }
}

/// `var layer = ...`
#[derive(Debug, Clone)]
struct LayerDefinition(String);

impl LayerDefinition {
    fn new(name: impl Into<String>) -> Self {
        LayerDefinition(name.into())
    }
}

impl AacObject for LayerDefinition {
    fn write_into<W: Write>(self, w: &mut CodeWriter<W>) -> IoResult<()> {
        let layer_name = self.0;

        w.write_yield(|w| {
            write!(
                w,
                r#"var layer = aac.CreateSupportingFxLayer("{layer_name}");"#
            )
        })
    }
}

/// `var renderer = ...`
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
struct ParameterDefinition {
    var_name: String,
    param_type: ParameterType,
}

impl ParameterDefinition {
    const DEFAULT_VARNAME: &'static str = "parameter";

    fn new(param_type: ParameterType) -> ParameterDefinition {
        ParameterDefinition {
            var_name: Self::DEFAULT_VARNAME.into(),
            param_type,
        }
    }

    fn var_name(mut self, var_name: impl Into<String>) -> ParameterDefinition {
        self.var_name = var_name.into();
        self
    }

    fn integer(name: impl Into<String>) -> ParameterDefinition {
        ParameterDefinition {
            var_name: Self::DEFAULT_VARNAME.into(),
            param_type: ParameterType::Integer(name.into()),
        }
    }

    fn bool(name: impl Into<String>) -> ParameterDefinition {
        ParameterDefinition {
            var_name: Self::DEFAULT_VARNAME.into(),
            param_type: ParameterType::Bool(name.into()),
        }
    }

    fn integer_group(names: impl IntoIterator<Item = String>) -> ParameterDefinition {
        ParameterDefinition {
            var_name: Self::DEFAULT_VARNAME.into(),
            param_type: ParameterType::IntegerGroup(names.into_iter().collect()),
        }
    }

    fn bool_group(names: impl IntoIterator<Item = String>) -> ParameterDefinition {
        ParameterDefinition {
            var_name: Self::DEFAULT_VARNAME.into(),
            param_type: ParameterType::BoolGroup(names.into_iter().collect()),
        }
    }
}

impl AacObject for ParameterDefinition {
    fn write_into<W: Write>(self, w: &mut CodeWriter<W>) -> IoResult<()> {
        let ParameterDefinition {
            var_name,
            param_type,
        } = self;

        match param_type {
            ParameterType::Bool(p) => w.write(format_args!(
                r#"var {var_name} = layer.BoolParameter("{p}");"#
            )),
            ParameterType::Integer(p) => w.write(format_args!(
                r#"var {var_name} = layer.IntParameter("{p}");"#
            )),
            ParameterType::BoolGroup(ps) => {
                let joined = ps.join(r#"", ""#);
                w.write(format_args!(
                    r#"var {var_name} = layer.BoolParameters("{joined}");"#
                ))
            }
            ParameterType::IntegerGroup(ps) => {
                let joined = ps.join(r#"", ""#);
                w.write(format_args!(
                    r#"var {var_name} = layer.IntParameters("{joined}");"#
                ))
            }
        }
    }
}

#[derive(Debug, Clone)]
enum ParameterType {
    Bool(String),
    Integer(String),
    BoolGroup(Vec<String>),
    IntegerGroup(Vec<String>),
}

/// `var state = ...`
#[derive(Debug, Clone)]
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

    fn indented(mut self) -> Self {
        self.indented = true;
        self
    }

    fn blend_shapes(mut self, items: impl IntoIterator<Item = (String, f64)>) -> Self {
        self.blend_shapes = Some(items.into_iter().collect());
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

/// `state.Tracks/Animates()...`
#[derive(Debug, Clone)]
struct StateOptions {
    state_var: String,
    options: Vec<StateOption>,
}

impl StateOptions {
    fn new(state_var: impl Into<String>) -> StateOptions {
        StateOptions {
            state_var: state_var.into(),
            options: vec![],
        }
    }

    fn tracks(mut self, target: AnimationTarget) -> Self {
        self.options.push(StateOption::Tracks(target));
        self
    }

    fn animates(mut self, target: AnimationTarget) -> Self {
        self.options.push(StateOption::Animates(target));
        self
    }
}

impl AacObject for StateOptions {
    fn write_into<W: Write>(self, w: &mut CodeWriter<W>) -> IoResult<()> {
        if self.options.is_empty() {
            return Ok(());
        }

        let StateOptions { state_var, options } = self;
        w.write_yield(|w| {
            write!(w, r#"{state_var}"#)?;
            for option in options {
                match option {
                    StateOption::Tracks(at) => {
                        write!(w, r#".TrackingTracks({})"#, at.tracking_element())?
                    }
                    StateOption::Animates(at) => {
                        write!(w, r#".TrackingAnimates({})"#, at.tracking_element())?
                    }
                }
            }
            write!(w, r#";"#)
        })
    }
}

#[derive(Debug, Clone)]
enum StateOption {
    Tracks(AnimationTarget),
    Animates(AnimationTarget),
}

/// `state.TransitionTo()...`
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
            Cond::Term(_) => true,
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
