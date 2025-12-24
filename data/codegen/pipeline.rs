use anyhow::{Result, anyhow};
use serde_json::Value;
use std::{collections::HashSet, fs, path::Path};
use walkdir::WalkDir;

use crate::codegen::{
    emit_file::emit_table_file, emit_root::emit_root_module, tables::FILTER_TABLES,
};

pub fn generate_rust_modules(json_dir: &str, output_dir: &str) -> Result<()> {
    let output = Path::new(output_dir);
    fs::create_dir_all(output)?;

    let filter: HashSet<&str> = FILTER_TABLES.iter().copied().collect();
    let mut tables = Vec::new();

    for entry in WalkDir::new(json_dir).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let raw = entry
            .path()
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid filename"))?;

        if !filter.contains(raw) {
            continue;
        }

        let snake = raw.to_string();
        tables.push(snake.clone());

        let json = fs::read_to_string(entry.path())?;
        let data: Value = serde_json::from_str(&json)?;

        let (table, records) = match data {
            Value::Array(ref a) if a.len() == 2 => (
                a[0].as_str().unwrap_or(raw).to_string(),
                a[1].as_array().cloned().unwrap_or_default(),
            ),
            Value::Array(a) => (raw.to_string(), a),
            _ => (raw.to_string(), vec![data]),
        };

        fs::write(
            output.join(format!("{}.rs", snake)),
            emit_table_file(&table, &records),
        )?;
    }

    tables.sort();
    fs::write(output.join("mod.rs"), emit_root_module(&tables))?;
    Ok(())
}
