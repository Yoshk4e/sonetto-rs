use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub fn infer_rust_type(value: &Value) -> String {
    match value {
        Value::Null => "Option<String>".to_string(),
        Value::Bool(_) => "bool".to_string(),
        Value::Number(n) => if n.is_i64() { "i32" } else { "f32" }.to_string(),
        Value::String(_) => "String".to_string(),
        Value::Array(arr) => {
            if arr.is_empty() {
                "Vec<serde_json::Value>".to_string()
            } else {
                format!("Vec<{}>", infer_rust_type(&arr[0]))
            }
        }
        Value::Object(_) => "serde_json::Value".to_string(),
    }
}

pub fn analyze_field_types(records: &[Value]) -> HashMap<String, HashSet<String>> {
    let mut map = HashMap::new();

    for record in records {
        if let Value::Object(obj) = record {
            for (k, v) in obj {
                map.entry(k.clone())
                    .or_insert_with(HashSet::new)
                    .insert(infer_rust_type(v));
            }
        }
    }

    map
}
