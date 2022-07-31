using UnityEngine;
using UnityEditor;
using UnityEditor.Animations;
using VRC.SDK3.Avatars.Components;
using AnimatorAsCodeFramework.Examples;

[CustomEditor(typeof(SK2AACGenerator_AvatarName))]
public class SK2AACGenerator_AvatarName_Editor : Editor
{
    public override void OnInspectorGUI()
    {
        base.OnInspectorGUI();
        var executor = target as SK2AACGenerator_AvatarName;
        if (GUILayout.Button("Generate"))
        {
            executor.GenerateAnimator();
        }
    }
}

public class SK2AACGenerator_AvatarName : MonoBehaviour
{
    public AnimatorController TargetContainer;
    public string AssetKey = "SK2AAC_AvatarName";

    public void GenerateAnimator()
    {
        var avatarDescriptor = GetComponent<VRCAvatarDescriptor>();
        var aac = AacExample.AnimatorAsCode("SK2AAC AvatarName", avatarDescriptor, TargetContainer, AssetKey, AacExample.Options().WriteDefaultsOff());
        var clipEmpty = aac.NewClip();
        // var fxDefault = aac.CreateMainFxLayer();

        // Object Face
        {
            var renderer = (SkinnedMeshRenderer) gameObject.transform.Find("Face").GetComponent<SkinnedMeshRenderer>();

            // Group Eyebrows
            {
                var layer = aac.CreateSupportingFxLayer("Face_Eyebrows");
                var parameter = layer.IntParameter("Eyebrows");

                var stateDisabled = layer.NewState("Disabled").WithAnimation(
                    aac.NewClip("Face_Eyebrows_Disabled")
                        .BlendShape(renderer, "眉_笑顔", 0.0f)
                        .BlendShape(renderer, "眉_怒", 0.0f)
                );

                var stateEnabled1 = layer.NewState("Smile").WithAnimation(aac.NewClip("Face_Eyebrows_Smile").BlendShape(renderer, "眉_笑顔", 100.0f)).RightOf(stateDisabled);
                stateDisabled.TransitionsTo(stateEnabled1).When(parameter.IsEqualTo(1));
                stateEnabled1.Exits().When(parameter.IsNotEqualTo(1));

                var stateEnabled2 = layer.NewState("Angry").WithAnimation(aac.NewClip("Face_Eyebrows_Angry").BlendShape(renderer, "眉_怒", 100.0f));
                stateDisabled.TransitionsTo(stateEnabled2).When(parameter.IsEqualTo(2));
                stateEnabled2.Exits().When(parameter.IsNotEqualTo(2));

            }

            // Group Eyelids
            {
                var layer = aac.CreateSupportingFxLayer("Face_Eyelids");
                var parameter = layer.IntParameter("Eyelids");

                var stateDisabled = layer.NewState("Disabled").WithAnimation(
                    aac.NewClip("Face_Eyelids_Disabled")
                        .BlendShape(renderer, "にっこり", 0.0f)
                        .BlendShape(renderer, "==", 0.0f)
                );

                var stateEnabled1 = layer.NewState("Smile").WithAnimation(aac.NewClip("Face_Eyelids_Smile").BlendShape(renderer, "にっこり", 100.0f)).RightOf(stateDisabled);
                stateDisabled.TransitionsTo(stateEnabled1).When(parameter.IsEqualTo(1));
                stateEnabled1.Exits().When(parameter.IsNotEqualTo(1));

                var stateEnabled2 = layer.NewState("Relaxed").WithAnimation(aac.NewClip("Face_Eyelids_Relaxed").BlendShape(renderer, "==", 100.0f));
                stateDisabled.TransitionsTo(stateEnabled2).When(parameter.IsEqualTo(2));
                stateEnabled2.Exits().When(parameter.IsNotEqualTo(2));

            }

            // Group Cheek
            {
                var layer = aac.CreateSupportingFxLayer("Face_Cheek");
                var parameter = layer.BoolParameter("Cheek");
                var stateDisabled = layer.NewState("Disabled").WithAnimation(aac.NewClip("Face_Cheek_Disabled").BlendShape(renderer, "頬染め", 0.0f));
                var stateEnabled = layer.NewState("Enabled").WithAnimation(aac.NewClip("Face_Cheek_Enabled").BlendShape(renderer, "頬染め", 100.0f));
                stateDisabled.TransitionsTo(stateEnabled).When(parameter.IsTrue());
                stateEnabled.TransitionsTo(stateDisabled).When(parameter.IsFalse());
            }
        }

        // Object Body
        {
            var renderer = (SkinnedMeshRenderer) gameObject.transform.Find("Body").GetComponent<SkinnedMeshRenderer>();

            // Group BodyShrink
            {
                var layer = aac.CreateSupportingFxLayer("Body_BodyShrink");
                var parameter = layer.BoolParameter("BodyShrink");
                var stateDisabled = layer.NewState("Disabled").WithAnimation(aac.NewClip("Body_BodyShrink_Disabled").BlendShape(renderer, "素体縮小", 0.0f));
                var stateEnabled = layer.NewState("Enabled").WithAnimation(aac.NewClip("Body_BodyShrink_Enabled").BlendShape(renderer, "素体縮小", 100.0f));
                stateDisabled.TransitionsTo(stateEnabled).When(parameter.IsTrue());
                stateEnabled.TransitionsTo(stateDisabled).When(parameter.IsFalse());
            }
        }
    }
}
