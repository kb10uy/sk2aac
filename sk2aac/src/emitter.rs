use crate::data::{AnimationDescriptor, AnimationGroup, AnimationGroupShapes, AnimationObject};

use std::io::prelude::*;

use anyhow::{ensure, Result};

pub fn emit_descriptor<W: Write>(writer: &mut W, descriptor: &AnimationDescriptor) -> Result<String> {
    let avatar_name = &descriptor.name;
    let asset_key = format!("SK2AAC_{avatar_name}");
    let class_name = format!("SK2AACGenerator_{avatar_name}");

    writeln!(writer, r#"using UnityEngine;"#)?;
    writeln!(writer, r#"using UnityEditor;"#)?;
    writeln!(writer, r#"using UnityEditor.Animations;"#)?;
    writeln!(writer, r#"using VRC.SDK3.Avatars.Components;"#)?;
    writeln!(writer, r#"using AnimatorAsCodeFramework.Examples;"#)?;
    writeln!(writer)?;
    writeln!(writer, r#"[CustomEditor(typeof({class_name}))]"#)?;
    writeln!(writer, r#"public class {class_name}_Editor : Editor"#)?;
    writeln!(writer, r#"{{"#)?;
    writeln!(writer, r#"    public override void OnInspectorGUI()"#)?;
    writeln!(writer, r#"    {{"#)?;
    writeln!(writer, r#"        base.OnInspectorGUI();"#)?;
    writeln!(writer, r#"        var executor = target as {class_name};"#)?;
    writeln!(writer, r#"        if (GUILayout.Button("Generate"))"#)?;
    writeln!(writer, r#"        {{"#)?;
    writeln!(writer, r#"            executor.GenerateAnimator();"#)?;
    writeln!(writer, r#"        }}"#)?;
    writeln!(writer, r#"    }}"#)?;
    writeln!(writer, r#"}}"#)?;
    writeln!(writer)?;
    writeln!(writer, r#"public class {class_name} : MonoBehaviour"#)?;
    writeln!(writer, r#"{{"#)?;
    writeln!(writer, r#"    public AnimatorController TargetContainer;"#)?;
    writeln!(writer, r#"    public string AssetKey = "{asset_key}";"#)?;
    writeln!(writer)?;
    writeln!(writer, r#"    public void GenerateAnimator()"#)?;
    writeln!(writer, r#"    {{"#)?;
    writeln!(writer, r#"        var avatarDescriptor = GetComponent<VRCAvatarDescriptor>();"#)?;
    writeln!(writer, r#"        var aac = AacExample.AnimatorAsCode("SK2AAC {avatar_name}", avatarDescriptor, TargetContainer, AssetKey, AacExample.Options().WriteDefaultsOff());"#)?;
    writeln!(writer, r#"        var clipEmpty = aac.NewClip();"#)?;
    writeln!(writer, r#"        // var fxDefault = aac.CreateMainFxLayer();"#)?;
    for object in &descriptor.animation_objects {
        emit_object(writer, object)?;
    }
    writeln!(writer, r#"    }}"#)?;
    writeln!(writer, r#"}}"#)?;

    Ok(class_name)
}

pub fn emit_object<W: Write>(writer: &mut W, object: &AnimationObject) -> Result<()> {
    let object_name = &object.name;

    writeln!(writer)?;
    writeln!(writer, r#"        // Object {object_name}"#)?;
    writeln!(writer, r#"        {{"#)?;
    writeln!(writer, r#"            var renderer = (SkinnedMeshRenderer) gameObject.transform.Find("{object_name}").GetComponent<SkinnedMeshRenderer>();"#)?;
    for group in &object.groups {
        emit_group(writer, &object.name, group)?;
    }
    writeln!(writer, r#"        }}"#)?;
    Ok(())
}

pub fn emit_group<W: Write>(writer: &mut W, object_name: &str, group: &AnimationGroup) -> Result<()> {
    if !group.emit {
        return Ok(());
    }

    let group_name = &group.name;
    let layer_name = format!("{object_name}_{group_name}");

    writeln!(writer)?;
    writeln!(writer, r#"            // Group {group_name}"#)?;
    writeln!(writer, r#"            {{"#)?;
    writeln!(writer, r#"                var layer = aac.CreateSupportingFxLayer("{layer_name}");"#)?;

    match &group.group_keys {
        AnimationGroupShapes::Select { shapes } => {
            ensure!(shapes.iter().all(|s| s.index.is_some()), "All shapes in select group must have indices");

            writeln!(writer, r#"                var parameter = layer.IntParameter("{group_name}");"#)?;
            writeln!(writer)?;

            // Emit disabled states.
            writeln!(writer, r#"                var stateDisabled = layer.NewState("Disabled").WithAnimation("#)?;
            writeln!(writer, r#"                    aac.NewClip("{layer_name}_Disabled")"#)?;
            for shape in shapes {
                let shape_name = &shape.shape_name;
                writeln!(writer, r#"                        .BlendShape(renderer, "{shape_name}", 0.0f)"#)?;
            }
            writeln!(writer, r#"                );"#)?;
            writeln!(writer)?;

            // Emit enabled states.
            const ALIGN_UNIT: usize = 8;
            let mut align_target = "stateDisabled".into();
            for (i, shape) in shapes.iter().enumerate() {
                // Object-Shape
                let animation_name = &shape.animation_name;
                let shape_name = &shape.shape_name;
                let shape_index = shape.index.expect("Already checked");
                let shape_value = format!("{:.1}f", shape.value.unwrap_or(1.0) * 100.0);
                let state_var = format!("stateEnabled{shape_index}");

                write!(writer, r#"                var {state_var} = layer.NewState("{animation_name}").WithAnimation(aac.NewClip("{layer_name}_{animation_name}").BlendShape(renderer, "{shape_name}", {shape_value}))"#)?;
                if i % ALIGN_UNIT == 0 {
                    write!(writer, r#".RightOf({align_target})"#)?;
                    align_target = state_var.clone();
                }
                writeln!(writer, r#";"#)?;

                writeln!(writer, r#"                stateDisabled.TransitionsTo({state_var}).When(parameter.IsEqualTo({shape_index}));"#)?;
                writeln!(writer, r#"                {state_var}.Exits().When(parameter.IsNotEqualTo({shape_index}));"#)?;
                writeln!(writer)?;
            }
        }
        AnimationGroupShapes::Switch { shape } => {
            // Object-Shape-Enabled
            ensure!(shape.index.is_none(), "Switch group must not have index");

            let shape_name = &shape.shape_name;
            let shape_value = format!("{:.1}f", shape.value.unwrap_or(1.0) * 100.0);

            // Emit disabled and enabled.
            writeln!(writer, r#"                var parameter = layer.BoolParameter("{group_name}");"#)?;
            writeln!(writer, r#"                var stateDisabled = layer.NewState("Disabled").WithAnimation(aac.NewClip("{layer_name}_Disabled").BlendShape(renderer, "{shape_name}", 0.0f));"#)?;
            writeln!(writer, r#"                var stateEnabled = layer.NewState("Enabled").WithAnimation(aac.NewClip("{layer_name}_Enabled").BlendShape(renderer, "{shape_name}", {shape_value}));"#)?;
            writeln!(writer, r#"                stateDisabled.TransitionsTo(stateEnabled).When(parameter.IsTrue());"#)?;
            writeln!(writer, r#"                stateEnabled.TransitionsTo(stateDisabled).When(parameter.IsFalse());"#)?;
        }
    }
    writeln!(writer, r#"            }}"#)?;

    Ok(())
}
