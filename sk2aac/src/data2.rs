use crate::data_raw::{
    RawDescriptor, RawShapeKeyCommon, RawShapeKeyDrive, RawShapeKeyGroup, RawShapeKeyOption,
    RawShapeKeySwitch,
};

use std::num::NonZeroUsize;

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

/// Represents a whole descriptor object.
#[derive(Debug, Clone, Serialize)]
pub struct Descriptor {
    /// Avatar name.
    pub name: String,

    /// Shape key switces.
    pub shape_switches: Vec<ShapeKeySwitch>,

    /// Shape key groups.
    pub shape_groups: Vec<ShapeKeyGroup>,
}

impl Descriptor {
    fn from_raw<'de, D>(raw: RawDescriptor) -> Result<Descriptor, D::Error>
    where
        D: Deserializer<'de>,
    {
        let shape_switches = raw
            .shape_switches
            .into_iter()
            .flatten()
            .map(|s| ShapeKeySwitch::from_raw::<'de, D>(s))
            .try_fold(vec![], |mut v, o| {
                v.push(o?);
                Ok(v)
            })?;
        let shape_groups = raw
            .shape_groups
            .into_iter()
            .flatten()
            .map(|s| ShapeKeyGroup::from_raw::<'de, D>(s))
            .try_fold(vec![], |mut v, o| {
                v.push(o?);
                Ok(v)
            })?;

        Ok(Descriptor {
            name: raw.name,
            shape_switches,
            shape_groups,
        })
    }
}

impl<'de> Deserialize<'de> for Descriptor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawDescriptor::deserialize(deserializer)?;
        let descriptor = Descriptor::from_raw::<'de, D>(raw)?;
        Ok(descriptor)
    }
}

/// Represents common part of shape key layers.
#[derive(Debug, Clone, Serialize)]
pub struct ShapeKeyCommon {
    /// Name used for both the layer and its Expression Parameter.
    pub name: String,

    /// Referencing SkinnedMeshRenderer name.
    pub mesh: String,

    /// Decides whether this layer prevents the eyelids animation.
    pub prevent_eyelids: bool,

    /// Decides whether this layer prevents the mouth animation.
    pub prevent_mouth: bool,
}

impl ShapeKeyCommon {
    fn from_raw<'de, D>(raw: RawShapeKeyCommon) -> Result<ShapeKeyCommon, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(ShapeKeyCommon {
            name: raw.name,
            mesh: raw.mesh,
            prevent_eyelids: raw.prevent_eyelids.unwrap_or(false),
            prevent_mouth: raw.prevent_mouth.unwrap_or(false),
        })
    }
}

impl<'de> Deserialize<'de> for ShapeKeyCommon {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawShapeKeyCommon::deserialize(deserializer)?;
        let skc = ShapeKeyCommon::from_raw::<'de, D>(raw)?;
        Ok(skc)
    }
}

/// Represents a shape key switch layer.
#[derive(Debug, Clone, Serialize)]
pub struct ShapeKeySwitch {
    /// Common part.
    #[serde(flatten)]
    pub common: ShapeKeyCommon,

    /// Target shape key.
    pub shape: String,

    /// The value on enabled.
    pub enabled_value: NormalizedF64,

    /// The value on disabled.
    pub disabled_value: NormalizedF64,
}

impl ShapeKeySwitch {
    fn from_raw<'de, D>(raw: RawShapeKeySwitch) -> Result<ShapeKeySwitch, D::Error>
    where
        D: Deserializer<'de>,
    {
        let common = ShapeKeyCommon {
            name: raw.common.name,
            mesh: raw.common.mesh,
            prevent_eyelids: raw.common.prevent_eyelids.unwrap_or(false),
            prevent_mouth: raw.common.prevent_mouth.unwrap_or(false),
        };
        let enabled_value = match NormalizedF64::new(raw.enabled_value.unwrap_or(1.0)) {
            Some(v) => v,
            None => return Err(D::Error::custom("enabled_value out of range")),
        };
        let disabled_value = match NormalizedF64::new(raw.disabled_value.unwrap_or(1.0)) {
            Some(v) => v,
            None => return Err(D::Error::custom("disabled_value out of range")),
        };

        Ok(ShapeKeySwitch {
            common,
            shape: raw.shape,
            enabled_value,
            disabled_value,
        })
    }
}

impl<'de> Deserialize<'de> for ShapeKeySwitch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawShapeKeySwitch::deserialize(deserializer)?;
        let sks = ShapeKeySwitch::from_raw::<'de, D>(raw)?;
        Ok(sks)
    }
}

/// Represents a group of shape key animations.
#[derive(Debug, Clone, Serialize)]
pub struct ShapeKeyGroup {
    /// Common part.
    #[serde(flatten)]
    pub common: ShapeKeyCommon,

    /// Default shape key values.
    pub defaults: Vec<ShapeKeyDrive>,

    /// Group options.
    pub options: Vec<ShapeKeyOption>,
}

impl ShapeKeyGroup {
    fn from_raw<'de, D>(raw: RawShapeKeyGroup) -> Result<ShapeKeyGroup, D::Error>
    where
        D: Deserializer<'de>,
    {
        let common = ShapeKeyCommon {
            name: raw.common.name,
            mesh: raw.common.mesh,
            prevent_eyelids: raw.common.prevent_eyelids.unwrap_or(false),
            prevent_mouth: raw.common.prevent_mouth.unwrap_or(false),
        };
        let defaults = raw
            .defaults
            .into_iter()
            .flatten()
            .map(|d| ShapeKeyDrive::from_raw::<'de, D>(d, 1.0))
            .try_fold(vec![], |mut v, o| {
                v.push(o?);
                Ok(v)
            })?;
        let options = raw
            .options
            .into_iter()
            .flatten()
            .map(|o| ShapeKeyOption::from_raw::<'de, D>(o))
            .try_fold(vec![], |mut v, o| {
                v.push(o?);
                Ok(v)
            })?;
        Ok(ShapeKeyGroup {
            common,
            defaults,
            options,
        })
    }
}

impl<'de> Deserialize<'de> for ShapeKeyGroup {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawShapeKeyGroup::deserialize(deserializer)?;
        let skg = ShapeKeyGroup::from_raw::<'de, D>(raw)?;
        Ok(skg)
    }
}

/// A option in `ShapeKeyGroup`.
#[derive(Debug, Clone, Serialize)]
pub struct ShapeKeyOption {
    /// Option label.
    label: String,

    /// Index value for Unity AnimatorController State.
    index: Option<NonZeroUsize>,

    /// Shape keys to move.
    shapes: Vec<ShapeKeyDrive>,
}

impl ShapeKeyOption {
    fn from_raw<'de, D>(raw: RawShapeKeyOption) -> Result<ShapeKeyOption, D::Error>
    where
        D: Deserializer<'de>,
    {
        let sko = match raw {
            RawShapeKeyOption::Simple(label) => {
                let shapes = vec![ShapeKeyDrive::new(&label)];
                ShapeKeyOption {
                    label,
                    index: None,
                    shapes,
                }
            }
            RawShapeKeyOption::Complex {
                label,
                value,
                index,
                shapes,
            } => {
                let default_value = value.unwrap_or(1.0);
                let index = match index {
                    Some(i) => {
                        let inner = NonZeroUsize::new(i)
                            .ok_or(D::Error::custom("Index must be non-zero"))?;
                        Some(inner)
                    }
                    None => None,
                };
                let shapes = shapes
                    .into_iter()
                    .flatten()
                    .map(|s| ShapeKeyDrive::from_raw::<'de, D>(s, default_value))
                    .try_fold(vec![], |mut v, o| {
                        v.push(o?);
                        Ok(v)
                    })?;

                ShapeKeyOption {
                    label,
                    index,
                    shapes,
                }
            }
        };
        Ok(sko)
    }
}

impl<'de> Deserialize<'de> for ShapeKeyOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawShapeKeyOption::deserialize(deserializer)?;
        let option = ShapeKeyOption::from_raw::<'de, D>(raw)?;
        Ok(option)
    }
}

/// Drive information of a shape key.
#[derive(Debug, Clone, Serialize)]
pub struct ShapeKeyDrive {
    /// Shape key name.
    pub shape: String,

    /// Shape key value.
    pub value: NormalizedF64,
}

impl ShapeKeyDrive {
    /// Creates new instance with default options.
    fn new(label: &str) -> ShapeKeyDrive {
        ShapeKeyDrive {
            shape: label.to_string(),
            value: NormalizedF64::new(1.0).expect("Should be valid"),
        }
    }

    fn with_default_value<'de, D>(
        shape: String,
        value: Option<f64>,
        default_value: f64,
    ) -> Result<ShapeKeyDrive, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = NormalizedF64::new(value.unwrap_or(default_value))
            .ok_or(D::Error::custom("Drive value out of range"))?;
        Ok(ShapeKeyDrive {
            shape: shape.to_string(),
            value,
        })
    }

    fn from_raw<'de, D>(
        raw: RawShapeKeyDrive,
        default_value: f64,
    ) -> Result<ShapeKeyDrive, D::Error>
    where
        D: Deserializer<'de>,
    {
        match raw {
            RawShapeKeyDrive::Simple(shape) => {
                let skd = ShapeKeyDrive::new(&shape);
                Ok(skd)
            }
            RawShapeKeyDrive::Complex { shape, value } => {
                let skd = ShapeKeyDrive::with_default_value::<'de, D>(shape, value, default_value)?;
                Ok(skd)
            }
        }
    }
}

impl<'de> Deserialize<'de> for ShapeKeyDrive {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawShapeKeyDrive::deserialize(deserializer)?;
        let drive = ShapeKeyDrive::from_raw::<'de, D>(raw, 1.0)?;
        Ok(drive)
    }
}

/// Normalized float value in [0, 1].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NormalizedF64(pub f64);

impl NormalizedF64 {
    pub fn new(v: f64) -> Option<NormalizedF64> {
        if v >= 0.0 && v <= 1.0 {
            Some(NormalizedF64(v as f64))
        } else {
            None
        }
    }

    pub const fn get(&self) -> f64 {
        self.0
    }
}

impl Serialize for NormalizedF64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.0)
    }
}
