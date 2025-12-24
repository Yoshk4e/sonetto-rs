use crate::codegen::rust_ident::snake_to_pascal;

pub fn emit_root_module(table_names: &[String]) -> String {
    let mut lines = vec![
        "// Auto-generated module declarations".to_string(),
        "".to_string(),
    ];

    for table_name in table_names {
        lines.push(format!("pub mod {};", table_name));
    }

    lines.push("".to_string());
    lines.push("use std::sync::OnceLock;".to_string());
    lines.push("".to_string());
    lines.push("pub struct GameDB {".to_string());

    for table_name in table_names {
        let struct_name = snake_to_pascal(table_name);
        lines.push(format!(
            "    pub {}: {}::{}Table,",
            table_name, table_name, struct_name
        ));
    }

    lines.push("}".to_string());
    lines.push("".to_string());
    lines.push("impl GameDB {".to_string());
    lines.push("    pub fn load(data_dir: &str) -> anyhow::Result<Self> {".to_string());

    for table_name in table_names {
        let struct_name = snake_to_pascal(table_name);
        lines.push(format!(
            "        let {} = {}::{}Table::load(",
            table_name, table_name, struct_name
        ));
        lines.push(format!(
            "            &format!(\"{{}}/{}.json\", data_dir)",
            table_name
        ));
        lines.push(format!(
            "        ).map_err(|e| anyhow::anyhow!(\"Failed to load {}.json: {{}}\", e))?;",
            table_name
        ));
    }

    lines.push("".to_string());
    lines.push("        Ok(Self {".to_string());
    for table_name in table_names {
        lines.push(format!("            {},", table_name));
    }
    lines.push("        })".to_string());
    lines.push("    }".to_string());
    lines.push("".to_string());
    lines.push("    pub fn global() -> &'static GameDB {".to_string());
    lines.push("        static DB: OnceLock<GameDB> = OnceLock::new();".to_string());
    lines.push("        DB.get_or_init(|| {".to_string());
    lines.push(
        "            Self::load(\"data\").expect(\"Failed to load game database\")".to_string(),
    );
    lines.push("        })".to_string());
    lines.push("    }".to_string());
    lines.push("}".to_string());
    lines.push("".to_string());
    lines.push("static GAME_DATA: OnceLock<GameDB> = OnceLock::new();".to_string());
    lines.push("".to_string());
    lines.push("pub fn init(data_dir: &str) -> anyhow::Result<()> {".to_string());
    lines.push("    let db = GameDB::load(data_dir)?;".to_string());
    lines.push("    GAME_DATA.set(db)".to_string());
    lines.push(
        "        .map_err(|_| anyhow::anyhow!(\"Game data already initialized\"))".to_string(),
    );
    lines.push("}".to_string());
    lines.push("".to_string());
    lines.push("#[inline]".to_string());
    lines.push("pub fn get() -> &'static GameDB {".to_string());
    lines.push(
        "    GAME_DATA.get().expect(\"Game data not initialized. Call init() first.\")".to_string(),
    );
    lines.push("}".to_string());
    lines.push("".to_string());
    lines.push("#[inline]".to_string());
    lines.push("pub fn try_get() -> Option<&'static GameDB> {".to_string());
    lines.push("    GAME_DATA.get()".to_string());
    lines.push("}".to_string());

    lines.join("\n")
}
