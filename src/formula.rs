use crate::errors::AppError;

pub fn normalize_formula_name(input: &str) -> Result<String, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput(
            "formula name cannot be empty".to_string(),
        ));
    }

    let mut normalized = String::with_capacity(trimmed.len());
    let mut last_dash = false;

    for ch in trimmed.chars() {
        let mapped = match ch {
            ' ' | '_' | '-' => '-',
            _ => ch.to_ascii_lowercase(),
        };

        if mapped == '-' {
            if last_dash {
                continue;
            }
            last_dash = true;
            normalized.push('-');
            continue;
        }

        last_dash = false;
        normalized.push(mapped);
    }

    let normalized = normalized.trim_matches('-').to_string();

    if normalized.is_empty() {
        return Err(AppError::InvalidInput(
            "formula name cannot be empty".to_string(),
        ));
    }

    if normalized.starts_with("homebrew-") {
        return Err(AppError::InvalidInput(
            "formula name must not start with 'homebrew-'".to_string(),
        ));
    }

    if normalized
        .chars()
        .any(|ch| !matches!(ch, 'a'..='z' | '0'..='9' | '-'))
    {
        return Err(AppError::InvalidInput(
            "formula name may only contain lowercase letters, numbers, and dashes".to_string(),
        ));
    }

    Ok(normalized)
}

pub fn formula_class_name(input: &str) -> Result<String, AppError> {
    let normalized = normalize_formula_name(input)?;
    let mut class_name = String::with_capacity(normalized.len());

    for segment in normalized.split(['-', '_']) {
        if segment.is_empty() {
            continue;
        }

        let mut chars = segment.chars();
        if let Some(first) = chars.next() {
            class_name.push(first.to_ascii_uppercase());
        }
        for ch in chars {
            class_name.push(ch.to_ascii_lowercase());
        }
    }

    if class_name.is_empty() {
        return Err(AppError::InvalidInput(
            "formula class name cannot be empty".to_string(),
        ));
    }

    Ok(class_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_formula_name() {
        let name = normalize_formula_name("My Tool").unwrap();
        assert_eq!(name, "my-tool");

        let name = normalize_formula_name("  my__tool  ").unwrap();
        assert_eq!(name, "my-tool");

        let name = normalize_formula_name("foo--bar").unwrap();
        assert_eq!(name, "foo-bar");
    }

    #[test]
    fn rejects_invalid_formula_name() {
        let err = normalize_formula_name("homebrew-foo").unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));

        let err = normalize_formula_name("foo@bar").unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
    }

    #[test]
    fn builds_class_name() {
        let class_name = formula_class_name("foo2-bar").unwrap();
        assert_eq!(class_name, "Foo2Bar");

        let class_name = formula_class_name("http-server").unwrap();
        assert_eq!(class_name, "HttpServer");
    }
}
