use crate::errors::AppError;
use std::collections::{BTreeMap, HashSet};

const DEFAULT_TEMPLATE: &str = concat!(
    "class {class} < Formula\n",
    "  desc \"{desc}\"\n",
    "  homepage \"{homepage}\"\n",
    "{assets}",
    "  license \"{license}\"\n",
    "  version \"{version}\"\n",
    "\n",
    "  def install\n",
    "{install_block}",
    "  end\n",
    "end\n",
);

pub fn default_template() -> &'static str {
    DEFAULT_TEMPLATE
}

pub fn validate_template_string(template: &str) -> Result<(), AppError> {
    let spec = template_validation_spec();
    spec.render_with_template(template).map(|_| ())
}

fn template_validation_spec() -> FormulaSpec {
    let asset = FormulaAsset {
        url: "https://example.com/example-0.0.0.tar.gz".to_string(),
        sha256: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    };

    FormulaSpec {
        name: "example".to_string(),
        desc: "Example formula".to_string(),
        homepage: "https://example.com".to_string(),
        license: "MIT".to_string(),
        version: "0.0.0".to_string(),
        bins: vec!["example".to_string()],
        assets: AssetMatrix::Universal(asset),
        install_block: None,
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaAsset {
    pub url: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Os {
    Darwin,
    Linux,
}

impl Os {
    fn ruby_label(self) -> &'static str {
        match self {
            Os::Darwin => "macos",
            Os::Linux => "linux",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Arch {
    Arm64,
    Amd64,
}

impl Arch {
    fn predicate(self) -> &'static str {
        match self {
            Arch::Arm64 => "Hardware::CPU.arm?",
            Arch::Amd64 => "Hardware::CPU.intel?",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetAsset {
    pub os: Os,
    pub arch: Arch,
    pub asset: FormulaAsset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetMatrix {
    Universal(FormulaAsset),
    PerOs {
        macos: Option<FormulaAsset>,
        linux: Option<FormulaAsset>,
    },
    PerTarget(Vec<TargetAsset>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaSpec {
    pub name: String,
    pub desc: String,
    pub homepage: String,
    pub license: String,
    pub version: String,
    pub bins: Vec<String>,
    pub assets: AssetMatrix,
    pub install_block: Option<String>,
}

impl FormulaSpec {
    pub fn render(&self) -> Result<String, AppError> {
        self.validate()?;

        let class_name = formula_class_name(&self.name)?;
        let mut output = String::new();
        output.push_str(&format!("class {} < Formula\n", class_name));
        push_line(
            &mut output,
            2,
            &format!("desc \"{}\"", escape_ruby(&self.desc)),
        );
        push_line(
            &mut output,
            2,
            &format!("homepage \"{}\"", escape_ruby(&self.homepage)),
        );
        render_assets(&mut output, &self.assets)?;
        push_line(
            &mut output,
            2,
            &format!("license \"{}\"", escape_ruby(&self.license)),
        );
        push_line(
            &mut output,
            2,
            &format!("version \"{}\"", escape_ruby(&self.version)),
        );
        output.push('\n');
        output.push_str("  def install\n");
        output.push_str(&render_install_block_string(
            self.install_block.as_deref(),
            &self.bins,
        )?);
        output.push_str("  end\n");
        output.push_str("end\n");
        Ok(output)
    }

    pub fn render_with_template(&self, template: &str) -> Result<String, AppError> {
        self.validate()?;

        let class_name = formula_class_name(&self.name)?;
        let assets_block = render_assets_block(&self.assets)?;
        let install_block = render_install_block_string(self.install_block.as_deref(), &self.bins)?;

        let mut output = template.to_string();
        replace_required(&mut output, "{class}", &class_name)?;
        replace_required(&mut output, "{desc}", &escape_ruby(&self.desc))?;
        replace_required(&mut output, "{homepage}", &escape_ruby(&self.homepage))?;
        replace_required(&mut output, "{license}", &escape_ruby(&self.license))?;
        replace_required(&mut output, "{version}", &escape_ruby(&self.version))?;
        replace_required(&mut output, "{assets}", &assets_block)?;
        replace_required(&mut output, "{install_block}", &install_block)?;

        if output.contains("{name}") {
            output = output.replace("{name}", &escape_ruby(&self.name));
        }

        Ok(output)
    }

    pub fn validate(&self) -> Result<(), AppError> {
        if self.name.trim().is_empty() {
            return Err(AppError::InvalidInput(
                "formula name cannot be empty".to_string(),
            ));
        }
        if self.desc.trim().is_empty() {
            return Err(AppError::InvalidInput(
                "formula description cannot be empty".to_string(),
            ));
        }
        if self.homepage.trim().is_empty() {
            return Err(AppError::InvalidInput(
                "formula homepage cannot be empty".to_string(),
            ));
        }
        if self.license.trim().is_empty() {
            return Err(AppError::InvalidInput(
                "formula license cannot be empty".to_string(),
            ));
        }
        if self.version.trim().is_empty() {
            return Err(AppError::InvalidInput(
                "formula version cannot be empty".to_string(),
            ));
        }
        if self.bins.is_empty() {
            return Err(AppError::InvalidInput(
                "formula must define at least one binary".to_string(),
            ));
        }
        for bin in &self.bins {
            if bin.trim().is_empty() {
                return Err(AppError::InvalidInput(
                    "formula bin entries cannot be empty".to_string(),
                ));
            }
        }
        if let Some(install_block) = self.install_block.as_deref() {
            if install_block.trim().is_empty() {
                return Err(AppError::InvalidInput(
                    "template.install_block cannot be empty".to_string(),
                ));
            }
        }

        validate_assets(&self.assets)?;
        Ok(())
    }
}

fn render_install_block(output: &mut String, install_block: &str) -> Result<(), AppError> {
    if install_block.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "template.install_block cannot be empty".to_string(),
        ));
    }

    for line in install_block.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            output.push_str("    \n");
        } else {
            output.push_str("    ");
            output.push_str(trimmed.trim_start());
            output.push('\n');
        }
    }

    Ok(())
}

fn render_install_block_string(
    install_block: Option<&str>,
    bins: &[String],
) -> Result<String, AppError> {
    let mut output = String::new();
    if let Some(install_block) = install_block {
        render_install_block(&mut output, install_block)?;
        return Ok(output);
    }

    let bins = normalized_bins(bins)?;
    let install = if bins.len() == 1 {
        format!("    bin.install \"{}\"\n", escape_ruby(&bins[0]))
    } else {
        let joined = bins
            .iter()
            .map(|bin| format!("\"{}\"", escape_ruby(bin)))
            .collect::<Vec<_>>()
            .join(", ");
        format!("    bin.install {joined}\n")
    };
    output.push_str(&install);
    Ok(output)
}

fn render_assets(output: &mut String, assets: &AssetMatrix) -> Result<(), AppError> {
    match assets {
        AssetMatrix::Universal(asset) => {
            render_asset_lines(output, 2, asset)?;
        }
        AssetMatrix::PerOs { macos, linux } => {
            if macos.is_none() && linux.is_none() {
                return Err(AppError::InvalidInput(
                    "per-OS asset matrix must include at least one target".to_string(),
                ));
            }
            if let Some(asset) = macos {
                render_os_block(output, Os::Darwin, asset)?;
            }
            if let Some(asset) = linux {
                render_os_block(output, Os::Linux, asset)?;
            }
        }
        AssetMatrix::PerTarget(targets) => {
            let mut grouped: BTreeMap<Os, Vec<&TargetAsset>> = BTreeMap::new();
            for target in targets {
                grouped.entry(target.os).or_default().push(target);
            }
            for (os, mut entries) in grouped {
                entries.sort_by_key(|entry| entry.arch);
                render_os_arch_block(output, os, &entries)?;
            }
        }
    }
    Ok(())
}

fn render_assets_block(assets: &AssetMatrix) -> Result<String, AppError> {
    let mut output = String::new();
    render_assets(&mut output, assets)?;
    Ok(output)
}

fn render_os_block(output: &mut String, os: Os, asset: &FormulaAsset) -> Result<(), AppError> {
    push_line(output, 2, &format!("on_{} do", os.ruby_label()));
    render_asset_lines(output, 4, asset)?;
    push_line(output, 2, "end");
    Ok(())
}

fn render_os_arch_block(
    output: &mut String,
    os: Os,
    targets: &[&TargetAsset],
) -> Result<(), AppError> {
    push_line(output, 2, &format!("on_{} do", os.ruby_label()));

    if targets.is_empty() {
        push_line(output, 2, "end");
        return Ok(());
    }

    if targets.len() == 1 {
        let target = targets[0];
        push_line(output, 4, &format!("if {}", target.arch.predicate()));
        render_asset_lines(output, 6, &target.asset)?;
        push_line(output, 4, "end");
        push_line(output, 2, "end");
        return Ok(());
    }

    let arm = targets.iter().find(|entry| entry.arch == Arch::Arm64);
    let intel = targets.iter().find(|entry| entry.arch == Arch::Amd64);

    if let (Some(arm), Some(intel)) = (arm, intel) {
        push_line(output, 4, "if Hardware::CPU.arm?");
        render_asset_lines(output, 6, &arm.asset)?;
        push_line(output, 4, "else");
        render_asset_lines(output, 6, &intel.asset)?;
        push_line(output, 4, "end");
    } else {
        for (idx, target) in targets.iter().enumerate() {
            let keyword = if idx == 0 { "if" } else { "elsif" };
            push_line(output, 4, &format!("{keyword} {}", target.arch.predicate()));
            render_asset_lines(output, 6, &target.asset)?;
        }
        push_line(output, 4, "end");
    }

    push_line(output, 2, "end");
    Ok(())
}

fn render_asset_lines(
    output: &mut String,
    indent: usize,
    asset: &FormulaAsset,
) -> Result<(), AppError> {
    if asset.url.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "formula asset url cannot be empty".to_string(),
        ));
    }
    if asset.sha256.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "formula asset sha256 cannot be empty".to_string(),
        ));
    }
    push_line(
        output,
        indent,
        &format!("url \"{}\"", escape_ruby(&asset.url)),
    );
    push_line(
        output,
        indent,
        &format!("sha256 \"{}\"", escape_ruby(&asset.sha256)),
    );
    Ok(())
}

fn validate_assets(assets: &AssetMatrix) -> Result<(), AppError> {
    match assets {
        AssetMatrix::Universal(asset) => {
            validate_asset(asset)?;
        }
        AssetMatrix::PerOs { macos, linux } => {
            if macos.is_none() && linux.is_none() {
                return Err(AppError::InvalidInput(
                    "per-OS asset matrix must include at least one target".to_string(),
                ));
            }
            if let Some(asset) = macos {
                validate_asset(asset)?;
            }
            if let Some(asset) = linux {
                validate_asset(asset)?;
            }
        }
        AssetMatrix::PerTarget(targets) => {
            if targets.is_empty() {
                return Err(AppError::InvalidInput(
                    "per-target asset matrix must include at least one target".to_string(),
                ));
            }
            let mut seen = HashSet::new();
            for target in targets {
                validate_asset(&target.asset)?;
                let key = (target.os, target.arch);
                if !seen.insert(key) {
                    return Err(AppError::InvalidInput(format!(
                        "duplicate asset target for {:?} {:?}",
                        target.os, target.arch
                    )));
                }
            }
        }
    }
    Ok(())
}

fn validate_asset(asset: &FormulaAsset) -> Result<(), AppError> {
    if asset.url.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "formula asset url cannot be empty".to_string(),
        ));
    }
    if asset.sha256.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "formula asset sha256 cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn push_line(output: &mut String, indent: usize, line: &str) {
    for _ in 0..indent {
        output.push(' ');
    }
    output.push_str(line);
    output.push('\n');
}

fn escape_ruby(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn replace_required(output: &mut String, key: &str, value: &str) -> Result<(), AppError> {
    if !output.contains(key) {
        return Err(AppError::InvalidInput(format!(
            "template is missing required placeholder {key}"
        )));
    }
    *output = output.replace(key, value);
    Ok(())
}

fn normalized_bins(bins: &[String]) -> Result<Vec<String>, AppError> {
    let mut output = Vec::new();
    let mut seen = HashSet::new();
    for bin in bins {
        let trimmed = bin.trim();
        if trimmed.is_empty() {
            return Err(AppError::InvalidInput(
                "formula bin entries cannot be empty".to_string(),
            ));
        }
        if seen.insert(trimmed.to_string()) {
            output.push(trimmed.to_string());
        }
    }
    if output.is_empty() {
        return Err(AppError::InvalidInput(
            "formula must define at least one binary".to_string(),
        ));
    }
    Ok(output)
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

    #[test]
    fn renders_universal_formula() {
        let spec = FormulaSpec {
            name: "brewtool".to_string(),
            desc: "Brew tool".to_string(),
            homepage: "https://example.com".to_string(),
            license: "MIT".to_string(),
            version: "1.2.3".to_string(),
            bins: vec!["brewtool".to_string()],
            assets: AssetMatrix::Universal(FormulaAsset {
                url: "https://example.com/brewtool.tar.gz".to_string(),
                sha256: "deadbeef".to_string(),
            }),
            install_block: None,
        };

        let rendered = spec.render().unwrap();
        let expected = concat!(
            "class Brewtool < Formula\n",
            "  desc \"Brew tool\"\n",
            "  homepage \"https://example.com\"\n",
            "  url \"https://example.com/brewtool.tar.gz\"\n",
            "  sha256 \"deadbeef\"\n",
            "  license \"MIT\"\n",
            "  version \"1.2.3\"\n",
            "\n",
            "  def install\n",
            "    bin.install \"brewtool\"\n",
            "  end\n",
            "end\n"
        );
        assert_eq!(rendered, expected);
    }

    #[test]
    fn renders_multi_bin_install() {
        let spec = FormulaSpec {
            name: "brewtool".to_string(),
            desc: "Brew tool".to_string(),
            homepage: "https://example.com".to_string(),
            license: "MIT".to_string(),
            version: "1.2.3".to_string(),
            bins: vec!["brewctl".to_string(), "brewtool".to_string()],
            assets: AssetMatrix::Universal(FormulaAsset {
                url: "https://example.com/brewtool.tar.gz".to_string(),
                sha256: "deadbeef".to_string(),
            }),
            install_block: None,
        };

        let rendered = spec.render().unwrap();
        assert!(rendered.contains("bin.install \"brewctl\", \"brewtool\""));
    }

    #[test]
    fn renders_install_block_override() {
        let spec = FormulaSpec {
            name: "brewtool".to_string(),
            desc: "Brew tool".to_string(),
            homepage: "https://example.com".to_string(),
            license: "MIT".to_string(),
            version: "1.2.3".to_string(),
            bins: vec!["brewtool".to_string()],
            assets: AssetMatrix::Universal(FormulaAsset {
                url: "https://example.com/brewtool.tar.gz".to_string(),
                sha256: "deadbeef".to_string(),
            }),
            install_block: Some("bin.install \"brewtool\"\nlibexec.install Dir[\"*\"]".to_string()),
        };

        let rendered = spec.render().unwrap();
        assert!(rendered.contains(
            "  def install\n    bin.install \"brewtool\"\n    libexec.install Dir[\"*\"]\n  end"
        ));
    }

    #[test]
    fn renders_per_os_formula() {
        let spec = FormulaSpec {
            name: "brewtool".to_string(),
            desc: "Brew tool".to_string(),
            homepage: "https://example.com".to_string(),
            license: "MIT".to_string(),
            version: "1.2.3".to_string(),
            bins: vec!["brewtool".to_string()],
            assets: AssetMatrix::PerOs {
                macos: Some(FormulaAsset {
                    url: "https://example.com/brewtool-darwin.tar.gz".to_string(),
                    sha256: "macsha".to_string(),
                }),
                linux: Some(FormulaAsset {
                    url: "https://example.com/brewtool-linux.tar.gz".to_string(),
                    sha256: "linuxsha".to_string(),
                }),
            },
            install_block: None,
        };

        let rendered = spec.render().unwrap();
        assert!(rendered.contains("on_macos do"));
        assert!(rendered.contains("https://example.com/brewtool-darwin.tar.gz"));
        assert!(rendered.contains("on_linux do"));
        assert!(rendered.contains("https://example.com/brewtool-linux.tar.gz"));
    }

    #[test]
    fn renders_per_target_formula() {
        let spec = FormulaSpec {
            name: "brewtool".to_string(),
            desc: "Brew tool".to_string(),
            homepage: "https://example.com".to_string(),
            license: "MIT".to_string(),
            version: "1.2.3".to_string(),
            bins: vec!["brewtool".to_string()],
            assets: AssetMatrix::PerTarget(vec![
                TargetAsset {
                    os: Os::Darwin,
                    arch: Arch::Arm64,
                    asset: FormulaAsset {
                        url: "https://example.com/brewtool-darwin-arm64.tar.gz".to_string(),
                        sha256: "armsha".to_string(),
                    },
                },
                TargetAsset {
                    os: Os::Darwin,
                    arch: Arch::Amd64,
                    asset: FormulaAsset {
                        url: "https://example.com/brewtool-darwin-amd64.tar.gz".to_string(),
                        sha256: "amdsha".to_string(),
                    },
                },
                TargetAsset {
                    os: Os::Linux,
                    arch: Arch::Arm64,
                    asset: FormulaAsset {
                        url: "https://example.com/brewtool-linux-arm64.tar.gz".to_string(),
                        sha256: "linuxarm".to_string(),
                    },
                },
                TargetAsset {
                    os: Os::Linux,
                    arch: Arch::Amd64,
                    asset: FormulaAsset {
                        url: "https://example.com/brewtool-linux-amd64.tar.gz".to_string(),
                        sha256: "linuxamd".to_string(),
                    },
                },
            ]),
            install_block: None,
        };

        let rendered = spec.render().unwrap();
        let expected = concat!(
            "class Brewtool < Formula\n",
            "  desc \"Brew tool\"\n",
            "  homepage \"https://example.com\"\n",
            "  on_macos do\n",
            "    if Hardware::CPU.arm?\n",
            "      url \"https://example.com/brewtool-darwin-arm64.tar.gz\"\n",
            "      sha256 \"armsha\"\n",
            "    else\n",
            "      url \"https://example.com/brewtool-darwin-amd64.tar.gz\"\n",
            "      sha256 \"amdsha\"\n",
            "    end\n",
            "  end\n",
            "  on_linux do\n",
            "    if Hardware::CPU.arm?\n",
            "      url \"https://example.com/brewtool-linux-arm64.tar.gz\"\n",
            "      sha256 \"linuxarm\"\n",
            "    else\n",
            "      url \"https://example.com/brewtool-linux-amd64.tar.gz\"\n",
            "      sha256 \"linuxamd\"\n",
            "    end\n",
            "  end\n",
            "  license \"MIT\"\n",
            "  version \"1.2.3\"\n",
            "\n",
            "  def install\n",
            "    bin.install \"brewtool\"\n",
            "  end\n",
            "end\n"
        );
        assert_eq!(rendered, expected);
    }

    #[test]
    fn rejects_empty_target_matrix() {
        let spec = FormulaSpec {
            name: "brewtool".to_string(),
            desc: "Brew tool".to_string(),
            homepage: "https://example.com".to_string(),
            license: "MIT".to_string(),
            version: "1.2.3".to_string(),
            bins: vec!["brewtool".to_string()],
            assets: AssetMatrix::PerTarget(Vec::new()),
            install_block: None,
        };

        let err = spec.render().unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
    }

    #[test]
    fn renders_custom_template() {
        let spec = FormulaSpec {
            name: "brewtool".to_string(),
            desc: "Brew tool".to_string(),
            homepage: "https://example.com".to_string(),
            license: "MIT".to_string(),
            version: "1.2.3".to_string(),
            bins: vec!["brewtool".to_string()],
            assets: AssetMatrix::Universal(FormulaAsset {
                url: "https://example.com/brewtool.tar.gz".to_string(),
                sha256: "deadbeef".to_string(),
            }),
            install_block: None,
        };

        let template = concat!(
            "class {class} < Formula\n",
            "  desc \"{desc}\"\n",
            "  homepage \"{homepage}\"\n",
            "{assets}",
            "  license \"{license}\"\n",
            "  version \"{version}\"\n",
            "\n",
            "  def install\n",
            "{install_block}",
            "  end\n",
            "end\n"
        );

        let rendered = spec.render_with_template(template).unwrap();
        assert!(rendered.contains("class Brewtool < Formula"));
        assert!(rendered.contains("url \"https://example.com/brewtool.tar.gz\""));
        assert!(rendered.contains("bin.install \"brewtool\""));
    }

    #[test]
    fn default_template_matches_render_output() {
        let spec = FormulaSpec {
            name: "brewtool".to_string(),
            desc: "Brew tool".to_string(),
            homepage: "https://example.com".to_string(),
            license: "MIT".to_string(),
            version: "1.2.3".to_string(),
            bins: vec!["brewtool".to_string()],
            assets: AssetMatrix::Universal(FormulaAsset {
                url: "https://example.com/brewtool.tar.gz".to_string(),
                sha256: "deadbeef".to_string(),
            }),
            install_block: None,
        };

        let rendered = spec.render().unwrap();
        let templated = spec.render_with_template(default_template()).unwrap();
        assert_eq!(templated, rendered);
    }

    #[test]
    fn custom_template_requires_placeholders() {
        let spec = FormulaSpec {
            name: "brewtool".to_string(),
            desc: "Brew tool".to_string(),
            homepage: "https://example.com".to_string(),
            license: "MIT".to_string(),
            version: "1.2.3".to_string(),
            bins: vec!["brewtool".to_string()],
            assets: AssetMatrix::Universal(FormulaAsset {
                url: "https://example.com/brewtool.tar.gz".to_string(),
                sha256: "deadbeef".to_string(),
            }),
            install_block: None,
        };

        let err = spec
            .render_with_template("class {class} < Formula\n")
            .unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
    }
}
