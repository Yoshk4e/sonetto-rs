pub mod emit_file;
pub mod emit_root;
pub mod emit_struct;
pub mod emit_table;
pub mod pipeline;
pub mod rust_ident;
pub mod rust_types;
pub mod schema_infer;
pub mod schema_inspect;
pub mod tables;

pub use pipeline::generate_rust_modules;
