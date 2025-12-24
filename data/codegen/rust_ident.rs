use heck::ToPascalCase;

pub fn snake_to_pascal(name: &str) -> String {
    name.to_pascal_case()
}

pub fn camel_to_snake(name: &str) -> String {
    let mut result = String::new();
    let mut chars = name.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_uppercase() && !result.is_empty() && !result.ends_with('_') {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());

        if c.is_uppercase() && chars.peek().map_or(false, |n| n.is_uppercase()) {
            while let Some(&n) = chars.peek() {
                if n.is_uppercase() {
                    result.push(chars.next().unwrap().to_ascii_lowercase());
                } else {
                    break;
                }
            }
        }
    }

    let keywords = [
        "type", "mod", "fn", "let", "mut", "const", "static", "if", "else", "match", "loop",
        "while", "for", "in", "return", "break", "continue", "as", "use", "pub", "crate", "super",
        "self", "struct", "enum", "trait", "impl", "where", "async", "await", "dyn", "move", "ref",
        "box", "yield", "try",
    ];

    if keywords.contains(&result.as_str()) {
        format!("r#{}", result)
    } else {
        result
    }
}
