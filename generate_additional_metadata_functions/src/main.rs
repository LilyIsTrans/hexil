#![feature(never_type)]
#![feature(unwrap_infallible)]

use itertools::Itertools;
use quote::quote;
use vulkan_registry::*;

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum NumericFormat {
    UNorm,
    SNorm,
    UScaled,
    SScaled,
    UInt,
    SInt,
    SRGB,
    SFloat,
    UFloat,
    Multiple,
    Bool,
    Unknown,
}

impl std::str::FromStr for NumericFormat {
    type Err = !;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "UNORM" => Ok(Self::UNorm),
            "SNORM" => Ok(Self::SNorm),
            "USCALED" => Ok(Self::UScaled),
            "SSCALED" => Ok(Self::SScaled),
            "UINT" => Ok(Self::UInt),
            "SINT" => Ok(Self::SInt),
            "SRGB" => Ok(Self::SRGB),
            "SFLOAT" => Ok(Self::SFloat),
            "UFLOAT" => Ok(Self::UFloat),
            "MULTIPLE" => Ok(Self::Multiple),
            "BOOL" => Ok(Self::Bool),
            _ => Ok(Self::Unknown),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FormatMetadata {
    pub r_bits: u8,
    pub r_fmt: NumericFormat,
    pub g_bits: u8,
    pub g_fmt: NumericFormat,
    pub b_bits: u8,
    pub b_fmt: NumericFormat,
    pub a_bits: u8,
    pub a_fmt: NumericFormat,

    pub d_bits: u8,
    pub d_fmt: NumericFormat,
    pub s_bits: u8,
    pub s_fmt: NumericFormat,

    pub pack_bits: u8,

    pub unused_bits: u8,
}

fn quote_format_content_as_metadata(component: Vec<Component>) -> proc_macro2::TokenStream {
    let r_bits: u8 = component
        .iter()
        .filter_map(|component| {
            if component.name == Some("R".to_string()) {
                Some(
                    component
                        .bits
                        .clone()
                        .expect("A component should have bits"),
                )
            } else {
                None
            }
        })
        .map(|bits| u8::from_str_radix(&bits, 10))
        .fold_ok(0u8, std::ops::Add::add)
        .expect("Those definitely should all be numbers");
    let r_fmt: NumericFormat = component
        .iter()
        .filter_map(|component| {
            if component.name == Some("R".to_string()) {
                Some(
                    component
                        .numeric_format
                        .clone()
                        .expect("A component should have a format"),
                )
            } else {
                None
            }
        })
        .unique()
        .all_equal_value()
        .ok()
        .unwrap_or("Multiple".to_string())
        .parse()
        .into_ok();
    let g_bits: u8 = component
        .iter()
        .filter_map(|component| {
            if component.name == Some("G".to_string()) {
                Some(
                    component
                        .bits
                        .clone()
                        .expect("A component should have bits"),
                )
            } else {
                None
            }
        })
        .map(|bits| u8::from_str_radix(&bits, 10))
        .fold_ok(0u8, std::ops::Add::add)
        .expect("Those definitely should all be numbers");
    let g_fmt: NumericFormat = component
        .iter()
        .filter_map(|component| {
            if component.name == Some("G".to_string()) {
                Some(
                    component
                        .numeric_format
                        .clone()
                        .expect("A component should have a format"),
                )
            } else {
                None
            }
        })
        .unique()
        .all_equal_value()
        .ok()
        .unwrap_or("Multiple".to_string())
        .parse()
        .into_ok();
    let b_bits: u8 = component
        .iter()
        .filter_map(|component| {
            if component.name == Some("B".to_string()) {
                Some(
                    component
                        .bits
                        .clone()
                        .expect("A component should have bits"),
                )
            } else {
                None
            }
        })
        .map(|bits| u8::from_str_radix(&bits, 10))
        .fold_ok(0u8, std::ops::Add::add)
        .expect("Those definitely should all be numbers");
    let b_fmt: NumericFormat = component
        .iter()
        .filter_map(|component| {
            if component.name == Some("B".to_string()) {
                Some(
                    component
                        .numeric_format
                        .clone()
                        .expect("A component should have a format"),
                )
            } else {
                None
            }
        })
        .unique()
        .all_equal_value()
        .ok()
        .unwrap_or("Multiple".to_string())
        .parse()
        .into_ok();
    let a_bits: u8 = component
        .iter()
        .filter_map(|component| {
            if component.name == Some("A".to_string()) {
                Some(
                    component
                        .bits
                        .clone()
                        .expect("A component should have bits"),
                )
            } else {
                None
            }
        })
        .map(|bits| u8::from_str_radix(&bits, 10))
        .fold_ok(0u8, std::ops::Add::add)
        .expect("Those definitely should all be numbers");
    let a_fmt: NumericFormat = component
        .iter()
        .filter_map(|component| {
            if component.name == Some("A".to_string()) {
                Some(
                    component
                        .numeric_format
                        .clone()
                        .expect("A component should have a format"),
                )
            } else {
                None
            }
        })
        .unique()
        .all_equal_value()
        .ok()
        .unwrap_or("Multiple".to_string())
        .parse()
        .into_ok();
    let d_bits: u8 = component
        .iter()
        .filter_map(|component| {
            if component.name == Some("D".to_string()) {
                Some(
                    component
                        .bits
                        .clone()
                        .expect("A component should have bits"),
                )
            } else {
                None
            }
        })
        .map(|bits| u8::from_str_radix(&bits, 10))
        .fold_ok(0u8, std::ops::Add::add)
        .expect("Those definitely should all be numbers");
    let d_fmt: NumericFormat = component
        .iter()
        .filter_map(|component| {
            if component.name == Some("D".to_string()) {
                Some(
                    component
                        .numeric_format
                        .clone()
                        .expect("A component should have a format"),
                )
            } else {
                None
            }
        })
        .unique()
        .all_equal_value()
        .ok()
        .unwrap_or("Multiple".to_string())
        .parse()
        .into_ok();
    let s_bits: u8 = component
        .iter()
        .filter_map(|component| {
            if component.name == Some("S".to_string()) {
                Some(
                    component
                        .bits
                        .clone()
                        .expect("A component should have bits"),
                )
            } else {
                None
            }
        })
        .map(|bits| u8::from_str_radix(&bits, 10))
        .fold_ok(0u8, std::ops::Add::add)
        .expect("Those definitely should all be numbers");
    let s_fmt: NumericFormat = component
        .iter()
        .filter_map(|component| {
            if component.name == Some("S".to_string()) {
                Some(
                    component
                        .numeric_format
                        .clone()
                        .expect("A component should have a format"),
                )
            } else {
                None
            }
        })
        .unique()
        .all_equal_value()
        .ok()
        .unwrap_or("Multiple".to_string())
        .parse()
        .into_ok();
    // let pack_bits = ;
    let unused_bits = component
        .iter()
        .filter_map(|component| {
            if component.name == Some("X".to_string()) {
                Some(
                    component
                        .bits
                        .clone()
                        .expect("A component should have bits"),
                )
            } else {
                None
            }
        })
        .map(|bits| u8::from_str_radix(&bits, 10))
        .fold_ok(0u8, std::ops::Add::add)
        .expect("Those definitely should all be numbers");

    quote! {
        FormatMetadata {

        }
    }
}

fn main() {
    let registry = Registry::vk();

    for content in registry.contents {
        if let RegistryContent::Formats(formats) = content {
            let formats: Vec<Vec<_>> = formats
                .contents
                .into_iter()
                .map(|content| {
                    let FormatsContent::Format(format) = content;
                    format.contents
                })
                .map(|content| {
                    content
                        .into_iter()
                        .filter_map(|content| {
                            if let FormatContent::Component(component) = content {
                                Some(component)
                            } else {
                                None
                            }
                        })
                        .collect()
                })
                .collect();
            println!("{:#?}", formats)
        }
    }
}
