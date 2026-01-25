use crate::errors::AppError;

const SPDX_GUIDANCE: &str = "must be a valid SPDX license identifier (for example: MIT or Apache-2.0)";

pub fn canonicalize_spdx(input: &str, label: &str) -> Result<String, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput(format!("{label} {SPDX_GUIDANCE}")));
    }

    let canonical = match spdx::Expression::canonicalize(trimmed) {
        Ok(Some(value)) => value,
        Ok(None) => trimmed.to_string(),
        Err(_) => trimmed.to_string(),
    };

    if spdx::Expression::parse(&canonical).is_err() {
        return Err(AppError::InvalidInput(format!("{label} {SPDX_GUIDANCE}")));
    }

    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_lowercase_identifier() {
        let canonical = canonicalize_spdx("mit", "license").expect("canonicalize");
        assert_eq!(canonical, "MIT");
    }

    #[test]
    fn rejects_invalid_identifier() {
        let err = canonicalize_spdx("not-a-license", "license").expect_err("invalid");
        assert!(err.to_string().contains("valid SPDX"));
    }
}

