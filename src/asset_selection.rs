use crate::errors::AppError;
use crate::formula::{Arch, Os};
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct AssetSelectionOptions {
    pub asset_name: Option<String>,
    pub asset_template: Option<String>,
    pub project_name: Option<String>,
    pub version: Option<String>,
    pub os: Option<Os>,
    pub arch: Option<Arch>,
    pub target_key: Option<String>,
}

pub fn select_asset_name(
    available: &[String],
    options: &AssetSelectionOptions,
) -> Result<String, AppError> {
    let assets = unique_assets(available);
    let target_label = format_target_label(options);

    if let Some(name) = options.asset_name.as_deref() {
        if assets.iter().any(|asset| asset == name) {
            return Ok(name.to_string());
        }
        return Err(AppError::InvalidInput(format!(
            "asset not found: {name}; available assets: {}",
            format_available(&assets)
        )));
    }

    if let Some(template) = options.asset_template.as_deref() {
        let rendered = render_template(template, options)?;
        if assets.iter().any(|asset| asset == &rendered) {
            return Ok(rendered);
        }
        return Err(AppError::InvalidInput(format!(
            "no release asset matches template '{template}' for target '{target_label}'"
        )));
    }

    let filtered = filter_non_checksum(&assets);
    if filtered.is_empty() {
        return Err(AppError::InvalidInput(
            "no usable release assets found (checksum/signature assets are ignored)".to_string(),
        ));
    }

    let matched = filter_by_target(&filtered, options.os, options.arch);
    if matched.is_empty() {
        return Err(AppError::InvalidInput(format!(
            "no release assets match target '{target_label}'; available assets: {}",
            format_available(&filtered)
        )));
    }

    pick_best_match(&matched, options.version.as_deref(), &target_label)
}

fn render_template(template: &str, options: &AssetSelectionOptions) -> Result<String, AppError> {
    let mut output = template.to_string();

    if output.contains("{name}") {
        let name = options
            .project_name
            .as_deref()
            .ok_or_else(|| AppError::InvalidInput("asset_template requires {name}".to_string()))?;
        output = output.replace("{name}", name);
    }

    if output.contains("{version}") {
        let version = options.version.as_deref().ok_or_else(|| {
            AppError::InvalidInput("asset_template requires {version}".to_string())
        })?;
        output = output.replace("{version}", version);
    }

    if output.contains("{os}") {
        let os = options
            .os
            .ok_or_else(|| AppError::InvalidInput("asset_template requires {os}".to_string()))?;
        output = output.replace("{os}", normalized_os(os));
    }

    if output.contains("{arch}") {
        let arch = options
            .arch
            .ok_or_else(|| AppError::InvalidInput("asset_template requires {arch}".to_string()))?;
        output = output.replace("{arch}", normalized_arch(arch));
    }

    Ok(output)
}

fn filter_non_checksum(assets: &[String]) -> Vec<String> {
    assets
        .iter()
        .filter(|asset| !is_checksum_asset(asset))
        .cloned()
        .collect()
}

fn is_checksum_asset(asset: &str) -> bool {
    let lower = asset.to_ascii_lowercase();
    lower.ends_with(".sha256")
        || lower.ends_with(".sha256sum")
        || lower.ends_with(".sig")
        || lower.ends_with(".asc")
}

fn filter_by_target(assets: &[String], os: Option<Os>, arch: Option<Arch>) -> Vec<String> {
    let os_tokens = os.map(os_tokens);
    let arch_tokens = arch.map(arch_tokens);

    assets
        .iter()
        .filter(|asset| {
            let lower = asset.to_ascii_lowercase();
            let os_match = os_tokens
                .map(|tokens| tokens.iter().any(|token| lower.contains(token)))
                .unwrap_or(true);
            let arch_match = arch_tokens
                .map(|tokens| tokens.iter().any(|token| lower.contains(token)))
                .unwrap_or(true);
            os_match && arch_match
        })
        .cloned()
        .collect()
}

fn pick_best_match(
    assets: &[String],
    version: Option<&str>,
    target_label: &str,
) -> Result<String, AppError> {
    if assets.is_empty() {
        return Err(AppError::InvalidInput(
            "no release assets available for selection".to_string(),
        ));
    }

    let mut candidates: Vec<Candidate> = assets
        .iter()
        .map(|asset| Candidate::new(asset, version))
        .collect();

    candidates.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));

    let best_key = candidates[0].sort_key();
    let ties: Vec<&Candidate> = candidates
        .iter()
        .filter(|candidate| candidate.sort_key() == best_key)
        .collect();

    if ties.len() > 1 {
        return Err(AppError::InvalidInput(format!(
            "multiple release assets match target '{target_label}'; specify --asset-name or --asset-template"
        )));
    }

    Ok(candidates[0].name.clone())
}

#[derive(Debug, Clone)]
struct Candidate {
    name: String,
    version_match: bool,
    extension_rank: u8,
    name_len: usize,
}

impl Candidate {
    fn new(name: &str, version: Option<&str>) -> Self {
        let version_match = version
            .map(|version| name.contains(version))
            .unwrap_or(false);
        let extension_rank = extension_rank(name);
        let name_len = name.len();
        Self {
            name: name.to_string(),
            version_match,
            extension_rank,
            name_len,
        }
    }

    fn sort_key(&self) -> (u8, u8, usize) {
        let version_rank = if self.version_match { 0 } else { 1 };
        (version_rank, self.extension_rank, self.name_len)
    }
}

fn extension_rank(name: &str) -> u8 {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".tar.gz") {
        0
    } else if lower.ends_with(".zip") {
        1
    } else {
        2
    }
}

fn normalized_os(os: Os) -> &'static str {
    match os {
        Os::Darwin => "darwin",
        Os::Linux => "linux",
    }
}

fn normalized_arch(arch: Arch) -> &'static str {
    match arch {
        Arch::Arm64 => "arm64",
        Arch::Amd64 => "amd64",
    }
}

fn os_tokens(os: Os) -> &'static [&'static str] {
    match os {
        Os::Darwin => &["darwin", "macos", "osx"],
        Os::Linux => &["linux"],
    }
}

fn arch_tokens(arch: Arch) -> &'static [&'static str] {
    match arch {
        Arch::Arm64 => &["arm64", "aarch64"],
        Arch::Amd64 => &["amd64", "x86_64", "x64", "x86-64"],
    }
}

fn unique_assets(assets: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut output = Vec::new();
    for asset in assets {
        if seen.insert(asset.clone()) {
            output.push(asset.clone());
        }
    }
    output
}

fn format_available(assets: &[String]) -> String {
    if assets.is_empty() {
        return "(none)".to_string();
    }
    assets.join(", ")
}

fn format_target_label(options: &AssetSelectionOptions) -> String {
    if let Some(key) = options.target_key.as_deref() {
        let trimmed = key.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    match (options.os, options.arch) {
        (Some(os), Some(arch)) => format!("{}-{}", normalized_os(os), normalized_arch(arch)),
        (Some(os), None) => normalized_os(os).to_string(),
        (None, Some(arch)) => normalized_arch(arch).to_string(),
        (None, None) => "universal".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_assets() -> Vec<String> {
        vec![
            "brewtool-1.2.3-darwin-arm64.tar.gz".to_string(),
            "brewtool-1.2.3-darwin-amd64.tar.gz".to_string(),
            "brewtool-1.2.3-linux-arm64.tar.gz".to_string(),
            "brewtool-1.2.3-linux-amd64.tar.gz".to_string(),
        ]
    }

    #[test]
    fn selects_exact_asset_name() {
        let options = AssetSelectionOptions {
            asset_name: Some("brewtool-1.2.3-linux-amd64.tar.gz".to_string()),
            ..AssetSelectionOptions::default()
        };

        let selected = select_asset_name(&base_assets(), &options).unwrap();
        assert_eq!(selected, "brewtool-1.2.3-linux-amd64.tar.gz");
    }

    #[test]
    fn renders_template_requires_context() {
        let options = AssetSelectionOptions {
            asset_template: Some("{name}-{version}-{os}-{arch}.tar.gz".to_string()),
            ..AssetSelectionOptions::default()
        };

        let err = select_asset_name(&base_assets(), &options).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
    }

    #[test]
    fn selects_template_asset() {
        let options = AssetSelectionOptions {
            asset_template: Some("{name}-{version}-{os}-{arch}.tar.gz".to_string()),
            project_name: Some("brewtool".to_string()),
            version: Some("1.2.3".to_string()),
            os: Some(Os::Darwin),
            arch: Some(Arch::Arm64),
            ..AssetSelectionOptions::default()
        };

        let selected = select_asset_name(&base_assets(), &options).unwrap();
        assert_eq!(selected, "brewtool-1.2.3-darwin-arm64.tar.gz");
    }

    #[test]
    fn errors_when_asset_name_is_missing() {
        let assets = base_assets();
        let options = AssetSelectionOptions {
            asset_name: Some("missing.tar.gz".to_string()),
            ..AssetSelectionOptions::default()
        };

        let err = select_asset_name(&assets, &options).unwrap_err();
        let expected = format!(
            "asset not found: missing.tar.gz; available assets: {}",
            format_available(&assets)
        );
        assert_eq!(err.to_string(), expected);
    }

    #[test]
    fn errors_when_template_does_not_match_target() {
        let options = AssetSelectionOptions {
            asset_template: Some("{name}-{version}-{os}-{arch}.tar.gz".to_string()),
            project_name: Some("brewtool".to_string()),
            version: Some("9.9.9".to_string()),
            os: Some(Os::Darwin),
            arch: Some(Arch::Arm64),
            target_key: Some("darwin-arm64".to_string()),
            ..AssetSelectionOptions::default()
        };

        let err = select_asset_name(&base_assets(), &options).unwrap_err();
        assert_eq!(
            err.to_string(),
            "no release asset matches template '{name}-{version}-{os}-{arch}.tar.gz' for target 'darwin-arm64'"
        );
    }

    #[test]
    fn ignores_checksum_assets_in_fallback() {
        let assets = vec![
            "brewtool-1.2.3-darwin-arm64.tar.gz.sha256".to_string(),
            "brewtool-1.2.3-darwin-arm64.tar.gz.asc".to_string(),
        ];

        let options = AssetSelectionOptions {
            os: Some(Os::Darwin),
            arch: Some(Arch::Arm64),
            ..AssetSelectionOptions::default()
        };

        let err = select_asset_name(&assets, &options).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
    }

    #[test]
    fn matches_os_arch_tokens() {
        let assets = vec![
            "brewtool-1.2.3-macos-aarch64.tar.gz".to_string(),
            "brewtool-1.2.3-linux-x86_64.tar.gz".to_string(),
        ];

        let options = AssetSelectionOptions {
            os: Some(Os::Darwin),
            arch: Some(Arch::Arm64),
            ..AssetSelectionOptions::default()
        };

        let selected = select_asset_name(&assets, &options).unwrap();
        assert_eq!(selected, "brewtool-1.2.3-macos-aarch64.tar.gz");
    }

    #[test]
    fn prefers_version_and_tarball() {
        let assets = vec![
            "brewtool-latest-darwin-arm64.zip".to_string(),
            "brewtool-1.2.3-darwin-arm64.zip".to_string(),
            "brewtool-1.2.3-darwin-arm64.tar.gz".to_string(),
        ];

        let options = AssetSelectionOptions {
            version: Some("1.2.3".to_string()),
            os: Some(Os::Darwin),
            arch: Some(Arch::Arm64),
            ..AssetSelectionOptions::default()
        };

        let selected = select_asset_name(&assets, &options).unwrap();
        assert_eq!(selected, "brewtool-1.2.3-darwin-arm64.tar.gz");
    }

    #[test]
    fn errors_on_ambiguous_matches() {
        let assets = vec![
            "brewtool-1.2.3-darwin-arm64-a.tar.gz".to_string(),
            "brewtool-1.2.3-darwin-arm64-b.tar.gz".to_string(),
        ];

        let options = AssetSelectionOptions {
            version: Some("1.2.3".to_string()),
            os: Some(Os::Darwin),
            arch: Some(Arch::Arm64),
            target_key: Some("darwin-arm64".to_string()),
            ..AssetSelectionOptions::default()
        };

        let err = select_asset_name(&assets, &options).unwrap_err();
        assert_eq!(
            err.to_string(),
            "multiple release assets match target 'darwin-arm64'; specify --asset-name or --asset-template"
        );
    }
}
