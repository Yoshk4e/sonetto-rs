use serde_json::Value;

use crate::codegen::{
    emit_struct::emit_struct, emit_table::emit_table_store, schema_infer::analyze_field_types,
};

pub fn emit_table_file(table: &str, records: &[Value]) -> String {
    let field_types = analyze_field_types(records);

    [
        "// Auto-generated from JSON data",
        "// Do not edit manually",
        "",
        "use serde::{Deserialize, Serialize};",
        "",
        &emit_struct(table, records),
        &emit_table_store(table, &field_types, records),
    ]
    .join("\n")
}
