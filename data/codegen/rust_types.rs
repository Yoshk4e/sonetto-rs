use serde_json::Value;
use std::collections::{HashMap, HashSet};

use crate::codegen::schema_inspect::check_if_optional;

pub fn resolve_field_type(
    name: &str,
    types: &HashMap<String, HashSet<String>>,
    records: &[Value],
) -> String {
    let mut ty = if let Some(set) = types.get(name) {
        if set.len() == 1 {
            set.iter().next().unwrap().clone()
        } else {
            "Option<serde_json::Value>".to_string()
        }
    } else {
        "serde_json::Value".to_string()
    };

    if check_if_optional(records, name) && !ty.starts_with("Option<") {
        ty = format!("Option<{}>", ty);
    }

    ty
}
