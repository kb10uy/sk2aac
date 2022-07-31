use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationDescriptor {
    pub name: String,
    pub animation_objects: Vec<AnimationObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationObject {
    pub name: String,
    pub groups: Vec<AnimationGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationGroup {
    pub name: String,
    pub emit: bool,

    #[serde(flatten)]
    pub group_keys: AnimationGroupShapes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "animation_type")]
pub enum AnimationGroupShapes {
    Select { shapes: Vec<AnimationShape> },
    Switch { shape: AnimationShape },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationShape {
    pub animation_name: String,
    pub shape_name: String,
    pub index: Option<usize>,
}
