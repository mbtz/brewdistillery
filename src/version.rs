use crate::errors::AppError;
use semver::Version;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedVersionTag {
    pub version: Option<String>,
    pub tag: Option<String>,
    pub normalized_tag: Option<String>,
}

pub fn resolve_version_tag(version: Option<&str>, tag: Option<&str>) -> Result<ResolvedVersionTag, AppError> {
    let version_trimmed = version.map(str::trim).filter(|value| !value.is_empty());
    let tag_trimmed = tag.map(str::trim).filter(|value| !value.is_empty());

    let parsed_version = match version_trimmed {
        Some(value) => Some(parse_semver(value, "--version")?),
        None => None,
    };

    let (tag_original, parsed_tag, normalized_tag) = match tag_trimmed {
        Some(value) => {
            let normalized = normalize_tag(value);
            let parsed = parse_semver(&normalized, "--tag")?;
            (Some(value.to_string()), Some(parsed), Some(normalized))
        }
        None => (None, None, None),
    };

    if let (Some(version), Some(tag_version)) = (parsed_version.as_ref(), parsed_tag.as_ref()) {
        if version != tag_version {
            return Err(AppError::InvalidInput(format!(
                "--version '{}' does not match --tag '{}'",
                version,
                tag_original.as_deref().unwrap_or("")
            )));
        }
    }

    let resolved_version = match (parsed_version, parsed_tag) {
        (Some(version), _) => Some(version.to_string()),
        (None, Some(tag_version)) => Some(tag_version.to_string()),
        (None, None) => None,
    };

    Ok(ResolvedVersionTag {
        version: resolved_version,
        tag: tag_original,
        normalized_tag,
    })
}

fn normalize_tag(tag: &str) -> String {
    let trimmed = tag.trim();
    if let Some(rest) = trimmed.strip_prefix('v').or_else(|| trimmed.strip_prefix('V')) {
        if rest.chars().next().map(|ch| ch.is_ascii_digit()).unwrap_or(false) {
            return rest.to_string();
        }
    }
    trimmed.to_string()
}

fn parse_semver(input: &str, label: &str) -> Result<Version, AppError> {
    Version::parse(input).map_err(|_| {
        AppError::InvalidInput(format!(
            "invalid {label} '{input}': expected semver (e.g. 1.2.3)",
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_version_only() {
        let resolved = resolve_version_tag(Some("1.2.3"), None).unwrap();
        assert_eq!(resolved.version.as_deref(), Some("1.2.3"));
        assert_eq!(resolved.tag, None);
        assert_eq!(resolved.normalized_tag, None);
    }

    #[test]
    fn resolves_tag_only_with_v_prefix() {
        let resolved = resolve_version_tag(None, Some("v1.2.3")).unwrap();
        assert_eq!(resolved.version.as_deref(), Some("1.2.3"));
        assert_eq!(resolved.tag.as_deref(), Some("v1.2.3"));
        assert_eq!(resolved.normalized_tag.as_deref(), Some("1.2.3"));
    }

    #[test]
    fn rejects_mismatched_version_and_tag() {
        let err = resolve_version_tag(Some("1.2.3"), Some("v1.2.4")).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
    }

    #[test]
    fn rejects_invalid_versions() {
        let err = resolve_version_tag(Some("1"), None).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));

        let err = resolve_version_tag(None, Some("v1")).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
    }

    #[test]
    fn resolves_none_inputs() {
        let resolved = resolve_version_tag(None, None).unwrap();
        assert_eq!(resolved.version, None);
        assert_eq!(resolved.tag, None);
        assert_eq!(resolved.normalized_tag, None);
    }
}
