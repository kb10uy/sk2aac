use crate::{
    codegen::CodeWriter,
    data::{AnimationDescriptor, AnimationGroup, AnimationGroupShapes, AnimationObject},
};

use std::io::prelude::*;

use anyhow::{ensure, Result};

const ALIGN_UNIT: usize = 8;

pub fn emit_descriptor<W: Write>(
    writer: &mut W,
    descriptor: &AnimationDescriptor,
) -> Result<String> {
    let avatar_name = &descriptor.name;
    let asset_key = format!("SK2AAC_{avatar_name}");
    let class_name = format!("SK2AACGenerator_{avatar_name}");

    let mut root = CodeWriter::new(writer, 4);

    root.write(r#"using UnityEngine;"#)?;
    {
        let mut using = root.wrap_ifdef("UNITY_EDITOR")?;
        using.write(r#"using UnityEditor;"#)?;
        using.write(r#"using UnityEditor.Animations;;"#)?;
        using.write(r#"using VRC.SDK3.Avatars.Components;"#)?;
        using.write(r#"using AnimatorAsCodeFramework.Examples;"#)?;
    }
    root.write_empty()?;

    {
        let mut ce = root.wrap_ifdef("UNITY_EDITOR")?;
        ce.write("public class {class_name}_Editor : Editor")?;
        ce.write(r#"public class {class_name}_Editor : Editor"#)?;
        ce.write(r#"{{"#)?;
        ce.write(r#"    public override void OnInspectorGUI()"#)?;
        ce.write(r#"    {{"#)?;
        ce.write(r#"        base.OnInspectorGUI();"#)?;
        ce.write(r#"        var executor = target as {class_name};"#)?;
        ce.write(r#"        if (GUILayout.Button("Generate"))"#)?;
        ce.write(r#"        {{"#)?;
        ce.write(r#"            executor.GenerateAnimator();"#)?;
        ce.write(r#"        }}"#)?;
        ce.write(r#"    }}"#)?;
        ce.write(r#"}}"#)?;
    }
    root.write_empty()?;

    {
        root.write(format_args!(r#"public class {class_name} : MonoBehaviour"#));
        let mut class = root.indent_with_block()?;
        let mut class = class.wrap_ifdef("UNITY_EDITOR")?;

        class.write(r#"public AnimatorController TargetContainer;"#)?;
        class.write(format_args!(r#"public string AssetKey = "{asset_key}";"#))?;
        class.write_empty()?;

        class.write(r#" public void GenerateAnimator()"#)?;
        let mut method = class.indent_with_block()?;
        method.write(r#"var avatarDescriptor = GetComponent<VRCAvatarDescriptor>();"#)?;
        method.write(r#"var aac = AacExample.AnimatorAsCode("SK2AAC {avatar_name}", avatarDescriptor, TargetContainer, AssetKey, AacExample.Options().WriteDefaultsOff());"#)?;
        method.write(r#"var clipEmpty = aac.NewClip();"#)?;
        method.write(r#"// var fxDefault = aac.CreateMainFxLayer();"#)?;

        for object in &descriptor.animation_objects {
            emit_object(&mut method, object)?;
        }
    }

    root.flush()?;
    Ok(class_name)
}

pub fn emit_object<W: Write>(method: &mut CodeWriter<W>, object: &AnimationObject) -> Result<()> {
    let object_name = &object.name;

    method.write_empty()?;
    method.write(format_args!(r#"// Object {object_name}"#))?;

    let mut obj_block = method.indent_with_block()?;
    obj_block.write(format_args!(r#"var renderer = (SkinnedMeshRenderer) gameObject.transform.Find("{object_name}").GetComponent<SkinnedMeshRenderer>();"#))?;
    for group in &object.groups {
        emit_group(&mut obj_block, &object.name, group)?;
    }
    Ok(())
}

pub fn emit_group<W: Write>(
    block: &mut CodeWriter<W>,
    object_name: &str,
    group: &AnimationGroup,
) -> Result<()> {
    if !group.emit {
        return Ok(());
    }

    let group_name = &group.name;
    let layer_name = format!("{object_name}_{group_name}");

    block.write_empty()?;
    block.write(format_args!(r#"// Group {group_name}"#))?;

    let mut gblock = block.indent_with_block()?;

    // gblock.write(format_args!(r#""#));
    gblock.write(format_args!(
        r#"var layer = aac.CreateSupportingFxLayer("{layer_name}");"#
    ))?;

    match &group.group_keys {
        AnimationGroupShapes::Select { shapes } => {
            ensure!(
                shapes.iter().all(|s| s.index.is_some()),
                "All shapes in select group must have indices"
            );

            gblock.write(format_args!(
                r#"var parameter = layer.IntParameter("{group_name}");"#
            ))?;
            gblock.write_empty()?;

            // Emit disabled states.
            gblock.write(format_args!(
                r#"var stateDisabled = layer.NewState("Disabled").WithAnimation("#
            ))?;
            {
                let mut diss = gblock.indent();
                diss.write(format_args!(r#"aac.NewClip("{layer_name}_Disabled")"#))?;
                {
                    let mut diss = diss.indent();
                    for shape in shapes {
                        let shape_name = &shape.shape_name;
                        diss.write(format_args!(
                            r#".BlendShape(renderer, "{shape_name}", 0.0f)"#
                        ))?;
                    }
                }
            }
            gblock.write(r#");"#)?;

            // Emit enabled states.
            let mut align_target = "stateDisabled".into();
            for (i, shape) in shapes.iter().enumerate() {
                // Object-Shape
                let animation_name = &shape.animation_name;
                let shape_name = &shape.shape_name;
                let shape_index = shape.index.expect("Already checked");
                let shape_value = format!("{:.1}f", shape.value.unwrap_or(1.0) * 100.0);
                let state_var = format!("stateEnabled{shape_index}");

                gblock.write_empty()?;
                gblock.write(format_args!(
                    r#"var {state_var} = layer.NewState("{animation_name}").WithAnimation(aac.NewClip("{layer_name}_{animation_name}").BlendShape(renderer, "{shape_name}", {shape_value})){};"#,
                    if i % ALIGN_UNIT == 0 {
                        let next = format!( r#".RightOf({align_target})"#);
                        align_target = state_var.clone();
                        next
                    } else {
                        format!(r#""#)
                    }
                ))?;

                gblock.write(format_args!(r#"stateDisabled.TransitionsTo({state_var}).When(parameter.IsEqualTo({shape_index}));"#))?;
                gblock.write(format_args!(
                    r#"{state_var}.Exits().When(parameter.IsNotEqualTo({shape_index}));"#
                ))?;
            }
        }
        AnimationGroupShapes::Switch { shape } => {
            // Object-Shape-Enabled
            ensure!(shape.index.is_none(), "Switch group must not have index");

            let shape_name = &shape.shape_name;
            let shape_value = format!("{:.1}f", shape.value.unwrap_or(1.0) * 100.0);

            // Emit disabled and enabled.
            gblock.write(format_args!(
                r#"var parameter = layer.BoolParameter("{group_name}");"#
            ))?;
            gblock.write(format_args!(r#"var stateDisabled = layer.NewState("Disabled").WithAnimation(aac.NewClip("{layer_name}_Disabled").BlendShape(renderer, "{shape_name}", 0.0f));"#))?;
            gblock.write(format_args!(r#"var stateEnabled = layer.NewState("Enabled").WithAnimation(aac.NewClip("{layer_name}_Enabled").BlendShape(renderer, "{shape_name}", {shape_value}));"#))?;
            gblock
                .write(r#"stateDisabled.TransitionsTo(stateEnabled).When(parameter.IsTrue());"#)?;
            gblock
                .write(r#"stateEnabled.TransitionsTo(stateDisabled).When(parameter.IsFalse());"#)?;
        }
    }
    Ok(())
}
