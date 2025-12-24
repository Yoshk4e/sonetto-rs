use serde_json::Value;

use crate::codegen::{
    rust_ident::{camel_to_snake, snake_to_pascal},
    rust_types::resolve_field_type,
    schema_infer::analyze_field_types,
};

pub fn emit_struct(table: &str, records: &[Value]) -> String {
    let name = snake_to_pascal(table);
    let field_types = analyze_field_types(records);

    let mut fields: Vec<_> = field_types.keys().collect();
    fields.sort();

    let mut out = vec![
        "#[derive(Debug, Clone, Serialize, Deserialize)]".into(),
        format!("pub struct {} {{", name),
    ];

    for f in fields {
        out.push(format!("    #[serde(rename = \"{}\")]", f));
        out.push(format!(
            "    pub {}: {},",
            camel_to_snake(f),
            resolve_field_type(f, &field_types, records)
        ));
    }

    out.push("}".into());
    out.join("\n")
}
