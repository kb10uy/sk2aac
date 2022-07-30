use std::io::{prelude::*, Error as IoError};

use crate::data::{AnimationDescriptor, AnimationGroup, AnimationGroupShapes, AnimationObject};

pub fn emit_descriptor<W: Write>(writer: &mut W, descriptor: &AnimationDescriptor) -> Result<String, IoError> {
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
    writeln!(writer, r#"        var fxDefault = aac.CreateMainFxLayer();"#)?;
    writeln!(writer)?;
    for object in &descriptor.animation_objects {
        emit_object(writer, object)?;
    }
    writeln!(writer, r#"    }}"#)?;
    writeln!(writer, r#"}}"#)?;

    Ok(class_name)
}

pub fn emit_object<W: Write>(writer: &mut W, object: &AnimationObject) -> Result<(), IoError> {
    let object_name = &object.name;

    writeln!(writer, r#"        // Object {object_name}"#)?;
    writeln!(writer, r#"        {{"#)?;
    writeln!(writer, r#"            var renderer = (SkinnedMeshRenderer) gameObject.transform.Find("{object_name}").GetComponent<SkinnedMeshRenderer>();"#)?;
    writeln!(writer)?;

    for group in &object.groups {
        emit_group(writer, &object.name, group)?;
    }
    writeln!(writer, r#"        }}"#)?;
    Ok(())
}

pub fn emit_group<W: Write>(writer: &mut W, object_name: &str, group: &AnimationGroup) -> Result<(), IoError> {
    if !group.emit {
        return Ok(());
    }

    let group_name = &group.group_name;
    let layer_name = format!("{object_name} {group_name}");

    writeln!(writer, r#"            // Group {group_name}"#)?;
    writeln!(writer, r#"            {{"#)?;
    // ------------------------------------------------------------------------
    writeln!(writer, r#"                var layer = aac.CreateSupportingFxLayer("{layer_name}");"#)?;
    writeln!(writer, r#"                var stateDisabled = layer.NewState("Disabled").WithAnimation(clipEmpty);"#)?;
    writeln!(writer)?;

    match &group.group_keys {
        AnimationGroupShapes::Select { shapes } => {
            writeln!(writer, r#"                var parameter = fxDefault.IntParameter("{group_name}");"#)?;
            writeln!(writer)?;

            let mut aligned = false;
            for shape in shapes {
                let animation_name = format!("{object_name}-{}", shape.animation_name);
                let shape_name = &shape.shape_name;
                let shape_index = shape.index;
                let state_var = format!("stateEnabled{}", shape.index);
                if aligned {
                    writeln!(writer, r#"                var {state_var} = layer.NewState("{animation_name}").WithAnimation(aac.NewClip("{animation_name}").BlendShape(renderer, "{shape_name}", 100.0f));"#)?;
                } else {
                    writeln!(writer, r#"                var {state_var} = layer.NewState("{animation_name}").RightOf(stateDisabled).WithAnimation(aac.NewClip("{animation_name}").BlendShape(renderer, "{shape_name}", 100.0f));"#)?;
                    aligned = true;
                }
                writeln!(writer, r#"                stateDisabled.TransitionsTo({state_var}).When(parameter.IsEqualTo({shape_index}));"#)?;
                writeln!(writer, r#"                {state_var}.Exits().When(parameter.IsNotEqualTo({shape_index}));"#)?;
                writeln!(writer)?;
            }

            // Oh Sorry
        }
        AnimationGroupShapes::Switch { shape } => {
            // Object-Shape-Enabled
            let animation_name = format!("{object_name}-{}-Enabled", shape.animation_name);
            let shape_name = &shape.shape_name;
            writeln!(writer, r#"                var parameter = fxDefault.BoolParameter("{group_name}");"#)?;
            writeln!(writer, r#"                var stateEnabled = layer.NewState("Enabled").WithAnimation(aac.NewClip("{animation_name}").BlendShape(renderer, "{shape_name}", 100.0f));"#)?;
            writeln!(writer, r#"                stateDisabled.TransitionsTo(stateEnabled).When(parameter.IsTrue());"#)?;
            writeln!(writer, r#"                stateEnabled.TransitionsTo(stateDisabled).When(parameter.IsFalse());"#)?;
            writeln!(writer)?;
        }
    }
    // ------------------------------------------------------------------------
    writeln!(writer, r#"            }}"#)?;
    writeln!(writer)?;

    Ok(())
}
