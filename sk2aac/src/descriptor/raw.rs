use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawDescriptor {
    pub name: String,
    pub shape_switches: Option<Vec<RawShapeKeySwitch>>,
    pub shape_groups: Option<Vec<RawShapeKeyGroup>>,
}

#[derive(Debug, Deserialize)]
pub struct RawShapeKeyCommon {
    pub name: String,
    pub mesh: String,
    pub prevent_eyelids: Option<bool>,
    pub prevent_mouth: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RawShapeKeySwitch {
    #[serde(flatten)]
    pub common: RawShapeKeyCommon,

    pub shape: String,
    pub enabled_value: Option<f64>,
    pub disabled_value: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct RawShapeKeyGroup {
    #[serde(flatten)]
    pub common: RawShapeKeyCommon,

    pub defaults: Option<Vec<RawShapeKeyDrive>>,
    pub options: Option<Vec<RawShapeKeyOption>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RawShapeKeyOption {
    Simple(String),
    Complex {
        label: String,
        value: Option<f64>,
        index: Option<usize>,
        shapes: Option<Vec<RawShapeKeyDrive>>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RawShapeKeyDrive {
    Simple(String),
    Complex { shape: String, value: Option<f64> },
}
