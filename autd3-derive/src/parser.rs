use proc_macro::TokenStream;

#[derive(Debug)]
pub struct DeriveInput {
    pub ident: String,
    pub generics: Generics,
}

#[derive(Debug, Default)]
pub struct Generics {
    pub lifetimes: Vec<String>,
    pub type_params: Vec<String>,
    pub type_params_with_bounds: Vec<String>,
    pub where_clause: Option<String>,
}

impl Generics {
    pub fn type_generics(&self) -> String {
        if self.lifetimes.is_empty() && self.type_params.is_empty() {
            String::new()
        } else {
            let mut parts = Vec::new();
            parts.extend(self.lifetimes.iter().map(|l| format!("'{}", l)));
            parts.extend(self.type_params.iter().cloned());
            format!("<{}>", parts.join(", "))
        }
    }
}

pub fn parse_derive_input(input: TokenStream) -> DeriveInput {
    let input_str = input.to_string();

    // Remove doc comments and attributes to avoid parsing issues
    let cleaned = remove_doc_comments(&input_str);

    let mut ident = String::new();
    let mut generics = Generics::default();

    // Find struct/enum declaration - look for the keyword followed by identifier
    let struct_re = regex_lite_find(&cleaned, &["struct ", "enum "]);
    if let Some((keyword_pos, keyword)) = struct_re {
        let after_keyword = &cleaned[keyword_pos + keyword.len()..];

        // Skip whitespace
        let trimmed = after_keyword.trim_start();

        // Extract identifier (alphanumeric and underscore)
        ident = trimmed
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();

        // Now look for generics immediately after the identifier
        if !ident.is_empty() {
            let after_ident = trimmed[ident.len()..].trim_start();
            if after_ident.starts_with('<')
                && let Some(gen_end) = find_matching_angle_bracket(after_ident)
            {
                let generics_content = &after_ident[1..gen_end];
                parse_generics_string(generics_content, &mut generics);
            }
        }
    }

    // Parse where clause
    if let Some(where_pos) = cleaned.find(" where ") {
        let after_where = &cleaned[where_pos + 7..]; // " where ".len()
        if let Some(body_pos) = after_where.find('{') {
            let where_content = after_where[..body_pos].trim();
            if !where_content.is_empty() {
                generics.where_clause = Some(format!("where {}", where_content));
            }
        }
    }

    DeriveInput { ident, generics }
}

fn remove_doc_comments(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '/' {
            if chars.peek() == Some(&'/') {
                // Skip until end of line
                chars.next(); // consume second /
                for c in chars.by_ref() {
                    if c == '\n' {
                        result.push('\n');
                        break;
                    }
                }
            } else if chars.peek() == Some(&'*') {
                // Skip until */
                chars.next(); // consume *
                let mut prev = ' ';
                for c in chars.by_ref() {
                    if prev == '*' && c == '/' {
                        break;
                    }
                    prev = c;
                }
                result.push(' ');
            } else {
                result.push(ch);
            }
        } else if ch == '#' {
            // Could be an attribute
            if chars.peek() == Some(&'[') {
                // Skip attribute
                let mut depth = 0;
                result.push(ch);
                for c in chars.by_ref() {
                    result.push(c);
                    if c == '[' {
                        depth += 1;
                    } else if c == ']' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

fn regex_lite_find<'a>(haystack: &str, needles: &[&'a str]) -> Option<(usize, &'a str)> {
    for needle in needles {
        if let Some(pos) = haystack.find(needle) {
            return Some((pos, needle));
        }
    }
    None
}

fn find_matching_angle_bracket(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, ch) in s.chars().enumerate() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_generics_string(content: &str, generics: &mut Generics) {
    let mut current_param = String::new();
    let mut depth = 0;

    for ch in content.chars() {
        match ch {
            '<' | '(' => {
                depth += 1;
                current_param.push(ch);
            }
            '>' | ')' => {
                depth -= 1;
                current_param.push(ch);
            }
            ',' if depth == 0 => {
                process_param(current_param.trim(), generics);
                current_param.clear();
            }
            _ => {
                current_param.push(ch);
            }
        }
    }

    if !current_param.trim().is_empty() {
        process_param(current_param.trim(), generics);
    }
}

fn process_param(param: &str, generics: &mut Generics) {
    if param.is_empty() {
        return;
    }

    if param.starts_with('\'') {
        // It's a lifetime
        let lifetime = param
            .split(':')
            .next()
            .unwrap_or("")
            .trim()
            .trim_start_matches('\'');
        // Don't filter 'geo - we need it in the output
        if !lifetime.is_empty() {
            generics.lifetimes.push(lifetime.to_string());
        }
    } else if !param.is_empty() {
        // It's a type parameter
        // Store full param with bounds
        generics.type_params_with_bounds.push(param.to_string());
        // Also store just the name
        let param_name = param.split(':').next().unwrap_or("").trim();
        generics.type_params.push(param_name.to_string());
    }
}
