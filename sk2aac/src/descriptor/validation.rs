use crate::descriptor::{Descriptor, Drive, Driver, ShapeKeyCommon, ShapeKeyGroup, ShapeKeySwitch};

use thiserror::Error as ThisError;

#[non_exhaustive]
#[derive(Debug, Clone, ThisError)]
pub enum ValidationError {
    /// Name is invalid for identifier or Expression Parameter name.
    #[error("invalid name for an identifier: \"{0}\"")]
    InvalidName(String),

    /// Name not found.
    #[error("No group or switch found: \"{0}\"")]
    NameNotExist(String),
}

/// Shorthand for `Result<(), ValidationError>`.
pub type ValidationResult = Result<(), ValidationError>;

pub fn validate_descriptor(descriptor: &Descriptor) -> ValidationResult {
    if descriptor.name.chars().any(|c| !c.is_ascii_alphanumeric()) {
        return Err(ValidationError::InvalidName(descriptor.name.clone()));
    }
    for switch in &descriptor.shape_switches {
        validate_shape_key_switch(switch)?;
    }
    for group in &descriptor.shape_groups {
        validate_shape_key_group(group)?;
    }
    for driver in &descriptor.drivers {
        validate_driver(driver, descriptor)?;
    }

    Ok(())
}

fn validate_shape_key_switch(switch: &ShapeKeySwitch) -> ValidationResult {
    validate_shape_key_common(&switch.common)?;

    Ok(())
}

fn validate_shape_key_group(group: &ShapeKeyGroup) -> ValidationResult {
    validate_shape_key_common(&group.common)?;

    Ok(())
}

fn validate_shape_key_common(common: &ShapeKeyCommon) -> ValidationResult {
    if common.name.chars().any(|c| !c.is_ascii_alphanumeric()) {
        return Err(ValidationError::InvalidName(common.name.clone()));
    }

    Ok(())
}

fn validate_driver(driver: &Driver, descriptor: &Descriptor) -> ValidationResult {
    if driver.name.chars().any(|c| !c.is_ascii_alphanumeric()) {
        return Err(ValidationError::InvalidName(driver.name.clone()));
    }
    for option in &driver.options {
        for drive in &option.drives {
            match drive {
                Drive::Switch { name, .. } => {
                    let exists_shape_switch = descriptor
                        .shape_switches
                        .iter()
                        .any(|s| name == &s.common.name);
                    if !exists_shape_switch {
                        return Err(ValidationError::NameNotExist(name.clone()));
                    }
                }
                Drive::Group { name, label } => {
                    let exists_shape_group = descriptor.shape_groups.iter().any(|g| {
                        name == &g.common.name && g.options.iter().any(|o| &o.label == label)
                    });
                    if !exists_shape_group {
                        return Err(ValidationError::NameNotExist(name.clone()));
                    }
                }
            }
        }
    }

    Ok(())
}
