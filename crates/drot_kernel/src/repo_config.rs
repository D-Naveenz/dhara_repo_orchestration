use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use semver::Version;
use serde::{Deserialize, Serialize};
use toml_edit::{Array, DocumentMut, value};
use xmltree::{Element, XMLNode};

pub const CONFIG_PATH: &str = "dhara.config.toml";
pub const ENV_EXAMPLE_PATH: &str = ".env.example";
pub const ENV_LOCAL_PATH: &str = ".env.local";
pub const ROOT_CARGO_TOML_PATH: &str = "Cargo.toml";

/// Environment-variable name expected to hold the NuGet publish API key.
pub const NUGET_API_KEY_ENV: &str = "NUGET_API_KEY";
/// Environment-variable name expected to hold the crates.io publish token.
pub const CARGO_REGISTRY_TOKEN_ENV: &str = "CARGO_REGISTRY_TOKEN";
/// Scaffolded content written for a missing `.env.example` / `.env.local`.
pub const DEFAULT_ENV_EXAMPLE_CONTENT: &str = "CARGO_REGISTRY_TOKEN=\nNUGET_API_KEY=\n";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DharaRepoConfig {
    pub versions: VersionConfig,
    pub product: ProductConfig,
    pub nuget: NuGetConfig,
    pub ci: CiConfig,
    pub targets: TargetsConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionConfig {
    #[serde(alias = "rust_workspace", alias = "nuget_package")]
    pub workspace: String,
}

/// Shared product metadata synced into Cargo.toml and package project manifests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductConfig {
    pub authors: Vec<String>,
    pub repository_url: String,
    pub project_url: String,
    #[serde(default)]
    pub license: Option<String>,
}

/// NuGet feed configuration; package-specific metadata lives in the package project files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NuGetConfig {
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CiConfig {
    pub smoke_project: String,
    pub package_project: String,
    /// Additional package projects synced alongside `package_project` (shared fields only).
    #[serde(default)]
    pub managed_package_projects: Vec<String>,
    pub tests_project: String,
    pub native_runtimes: Vec<String>,
    pub host_runtime_smoke: String,
    pub aot_runtime_smoke: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetsConfig {
    pub rust_targets: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShowOutput {
    pub config: DharaRepoConfig,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionPart {
    Major,
    Minor,
    Patch,
}

/// Returns `package_project` followed by any distinct `managed_package_projects` entries.
pub fn package_projects(config: &DharaRepoConfig) -> Vec<&str> {
    let mut projects = vec![config.ci.package_project.as_str()];
    for project in &config.ci.managed_package_projects {
        let project = project.as_str();
        if !projects.contains(&project) {
            projects.push(project);
        }
    }
    projects
}

pub fn load_config(repo_root: &Path) -> Result<DharaRepoConfig> {
    let config_path = repo_root.join(CONFIG_PATH);
    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;
    toml::from_str(&content).with_context(|| format!("failed to parse {}", config_path.display()))
}

pub fn load_env(repo_root: &Path) -> Result<BTreeMap<String, String>> {
    let env_path = repo_root.join(ENV_LOCAL_PATH);
    if !env_path.exists() {
        return Ok(BTreeMap::new());
    }

    let content = fs::read_to_string(&env_path)
        .with_context(|| format!("failed to read {}", env_path.display()))?;
    parse_env_content(&content)
}

pub fn show(repo_root: &Path) -> Result<String> {
    let output = ShowOutput {
        config: load_config(repo_root)?,
        env: masked_env(load_env(repo_root)?),
    };
    toml::to_string_pretty(&output).context("failed to serialize configuration")
}

/// Creates `dhara.config.toml`, `.env.example`, and `.env.local` when missing (config is truth).
///
/// Used when binding a directory that does not yet look like a Dhara repository, so that
/// subsequent [`crate::paths::is_repo_root`] checks succeed once scaffolding completes.
pub fn ensure_repo_scaffolding(repo_root: &Path) -> Result<()> {
    fs::create_dir_all(repo_root).with_context(|| {
        format!(
            "failed to create repository directory '{}'",
            repo_root.display()
        )
    })?;

    let config_path = repo_root.join(CONFIG_PATH);
    if !config_path.exists() {
        let content = toml::to_string_pretty(&skeleton_config())
            .context("failed to serialize skeleton configuration")?;
        fs::write(&config_path, content)
            .with_context(|| format!("failed to write {}", config_path.display()))?;
    }

    let example_path = repo_root.join(ENV_EXAMPLE_PATH);
    if !example_path.exists() {
        fs::write(&example_path, DEFAULT_ENV_EXAMPLE_CONTENT)
            .with_context(|| format!("failed to write {}", example_path.display()))?;
    }

    let local_path = repo_root.join(ENV_LOCAL_PATH);
    if !local_path.exists() {
        let content = fs::read_to_string(&example_path)
            .unwrap_or_else(|_| DEFAULT_ENV_EXAMPLE_CONTENT.to_owned());
        fs::write(&local_path, content)
            .with_context(|| format!("failed to write {}", local_path.display()))?;
    }

    Ok(())
}

fn skeleton_config() -> DharaRepoConfig {
    DharaRepoConfig {
        versions: VersionConfig {
            workspace: "0.1.0".to_owned(),
        },
        product: ProductConfig {
            authors: Vec::new(),
            repository_url: String::new(),
            project_url: String::new(),
            license: None,
        },
        nuget: NuGetConfig {
            source: "https://api.nuget.org/v3/index.json".to_owned(),
        },
        ci: CiConfig {
            smoke_project: String::new(),
            package_project: String::new(),
            managed_package_projects: Vec::new(),
            tests_project: String::new(),
            native_runtimes: Vec::new(),
            host_runtime_smoke: String::new(),
            aot_runtime_smoke: String::new(),
        },
        targets: TargetsConfig {
            rust_targets: BTreeMap::new(),
        },
    }
}

pub fn init_env(repo_root: &Path) -> Result<bool> {
    let local_path = repo_root.join(ENV_LOCAL_PATH);
    if local_path.exists() {
        return Ok(false);
    }

    let example_path = repo_root.join(ENV_EXAMPLE_PATH);
    let content = if example_path.exists() {
        fs::read_to_string(&example_path)
            .with_context(|| format!("failed to read {}", example_path.display()))?
    } else {
        DEFAULT_ENV_EXAMPLE_CONTENT.to_owned()
    };
    fs::write(&local_path, content)
        .with_context(|| format!("failed to write {}", local_path.display()))?;
    Ok(true)
}

pub fn verify_release(repo_root: &Path) -> Result<()> {
    let config = load_config(repo_root)?;
    validate_config(repo_root, &config)
}

/// Kind of manifest drift relative to `dhara.config.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigDriftKind {
    WorkspaceCargoToml,
    PackageCsproj,
}

/// One detected drift item shown in activation prompts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigDriftItem {
    pub kind: ConfigDriftKind,
    pub summary: String,
}

/// Returns manifest fields that differ from `dhara.config.toml` (config is truth).
pub fn detect_config_drift(repo_root: &Path) -> Result<Vec<ConfigDriftItem>> {
    let config = load_config(repo_root)?;
    let mut drifts = Vec::new();

    let cargo_path = repo_root.join(ROOT_CARGO_TOML_PATH);
    let cargo_content = fs::read_to_string(&cargo_path)
        .with_context(|| format!("failed to read {}", cargo_path.display()))?;
    if cargo_toml_needs_sync(&cargo_content, &config)? {
        drifts.push(ConfigDriftItem {
            kind: ConfigDriftKind::WorkspaceCargoToml,
            summary: format!(
                "{ROOT_CARGO_TOML_PATH} workspace metadata -> {}",
                config.versions.workspace
            ),
        });
    }

    for path in package_projects(&config) {
        let csproj_path = repo_root.join(path);
        let csproj_content = fs::read_to_string(&csproj_path)
            .with_context(|| format!("failed to read {}", csproj_path.display()))?;
        if csproj_needs_sync(&csproj_content, &config)? {
            drifts.push(ConfigDriftItem {
                kind: ConfigDriftKind::PackageCsproj,
                summary: format!("shared product/version -> {path}"),
            });
        }
    }

    Ok(drifts)
}

/// Writes manifest updates for the given drift items (config is truth).
pub fn apply_config_drift(repo_root: &Path, items: &[ConfigDriftItem]) -> Result<()> {
    if items.is_empty() {
        return Ok(());
    }

    let config = load_config(repo_root)?;
    let kinds: std::collections::HashSet<_> = items.iter().map(|item| item.kind).collect();

    if kinds.contains(&ConfigDriftKind::WorkspaceCargoToml) {
        let path = repo_root.join(ROOT_CARGO_TOML_PATH);
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let updated = sync_cargo_toml(&content, &config)?;
        if updated != content {
            fs::write(&path, updated)
                .with_context(|| format!("failed to write {}", path.display()))?;
        }
    }

    if kinds.contains(&ConfigDriftKind::PackageCsproj) {
        for relative_path in package_projects(&config) {
            let path = repo_root.join(relative_path);
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let updated = sync_csproj(&content, &config)?;
            if updated != content {
                fs::write(&path, updated)
                    .with_context(|| format!("failed to write {}", path.display()))?;
            }
        }
    }

    Ok(())
}

pub fn set_version(repo_root: &Path, version: &str) -> Result<()> {
    let parsed = Version::parse(version).with_context(|| format!("invalid semver: {version}"))?;
    let mut config = load_config(repo_root)?;
    config.versions.workspace = parsed.to_string();
    write_config(repo_root, &config)
}

pub fn bump_version(repo_root: &Path, part: VersionPart) -> Result<String> {
    let mut config = load_config(repo_root)?;
    let current = &config.versions.workspace;
    let mut version =
        Version::parse(current).with_context(|| format!("invalid configured semver: {current}"))?;
    match part {
        VersionPart::Major => {
            version.major += 1;
            version.minor = 0;
            version.patch = 0;
        }
        VersionPart::Minor => {
            version.minor += 1;
            version.patch = 0;
        }
        VersionPart::Patch => {
            version.patch += 1;
        }
    }

    let next = version.to_string();
    config.versions.workspace = next.clone();
    write_config(repo_root, &config)?;
    Ok(next)
}

/// Workspace metadata fields owned by `dhara.config.toml` (semantic compare; not full-file text).
#[derive(Debug, Clone, PartialEq, Eq)]
struct ManagedCargoSnapshot {
    workspace_version: String,
    dependency_versions: Vec<String>,
    authors: Vec<String>,
    repository: String,
    homepage: String,
    license: Option<String>,
}

pub fn sync_cargo_toml(content: &str, config: &DharaRepoConfig) -> Result<String> {
    Version::parse(&config.versions.workspace).with_context(|| {
        format!(
            "invalid rust workspace version: {}",
            config.versions.workspace
        )
    })?;
    if !cargo_toml_needs_sync(content, config)? {
        return Ok(content.to_owned());
    }
    let mut document = content
        .parse::<DocumentMut>()
        .context("failed to parse Cargo.toml")?;
    document["workspace"]["package"]["version"] = value(config.versions.workspace.as_str());
    for dep in cargo_workspace_deps() {
        document["workspace"]["dependencies"][*dep]["version"] =
            value(config.versions.workspace.as_str());
    }

    let authors: Array = config.product.authors.iter().map(String::as_str).collect();
    document["workspace"]["package"]["authors"] = value(authors);
    document["workspace"]["package"]["repository"] = value(config.product.repository_url.as_str());
    document["workspace"]["package"]["homepage"] = value(config.product.project_url.as_str());
    match &config.product.license {
        Some(license) => {
            document["workspace"]["package"]["license"] = value(license.as_str());
        }
        None => {
            if let Some(package) = document
                .get_mut("workspace")
                .and_then(|workspace| workspace.get_mut("package"))
                .and_then(|package| package.as_table_like_mut())
            {
                package.remove("license");
            }
        }
    }

    Ok(document.to_string())
}

fn default_cargo_workspace_deps() -> &'static [&'static str] {
    &["dhara_storage_core", "dhara_storage"]
}

fn cargo_workspace_deps() -> &'static [&'static str] {
    crate::product::product_hooks()
        .map(|hooks| hooks.cargo_workspace_deps())
        .unwrap_or_else(default_cargo_workspace_deps)
}

fn managed_cargo_snapshot_from_content(content: &str) -> Result<ManagedCargoSnapshot> {
    let document = content
        .parse::<DocumentMut>()
        .context("failed to parse Cargo.toml")?;
    let workspace_version = workspace_package_version(&document)
        .ok_or_else(|| anyhow::anyhow!("Cargo.toml is missing workspace.package.version"))?;
    let mut dependency_versions = Vec::new();
    for name in cargo_workspace_deps() {
        let version = workspace_dependency_version(&document, name)
            .ok_or_else(|| anyhow::anyhow!("Cargo.toml is missing {name} version"))?;
        dependency_versions.push(version);
    }
    Ok(ManagedCargoSnapshot {
        workspace_version,
        dependency_versions,
        authors: workspace_package_authors(&document),
        repository: workspace_package_string_field(&document, "repository").unwrap_or_default(),
        homepage: workspace_package_string_field(&document, "homepage").unwrap_or_default(),
        license: workspace_package_string_field(&document, "license"),
    })
}

fn managed_cargo_snapshot_for_config(config: &DharaRepoConfig) -> ManagedCargoSnapshot {
    let deps = cargo_workspace_deps();
    ManagedCargoSnapshot {
        workspace_version: config.versions.workspace.clone(),
        dependency_versions: deps
            .iter()
            .map(|_| config.versions.workspace.clone())
            .collect(),
        authors: config.product.authors.clone(),
        repository: config.product.repository_url.clone(),
        homepage: config.product.project_url.clone(),
        license: config.product.license.clone(),
    }
}

pub fn cargo_toml_needs_sync(content: &str, config: &DharaRepoConfig) -> Result<bool> {
    Version::parse(&config.versions.workspace).with_context(|| {
        format!(
            "invalid rust workspace version: {}",
            config.versions.workspace
        )
    })?;
    let expected = managed_cargo_snapshot_for_config(config);
    let current = match managed_cargo_snapshot_from_content(content) {
        Ok(current) => current,
        Err(_) => return Ok(true),
    };
    Ok(current != expected)
}

fn workspace_package_version(document: &DocumentMut) -> Option<String> {
    workspace_package_string_field(document, "version")
}

fn workspace_package_string_field(document: &DocumentMut, name: &str) -> Option<String> {
    document
        .get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .and_then(|package| package.get(name))
        .and_then(|value| value.as_str())
        .map(str::to_owned)
}

fn workspace_package_authors(document: &DocumentMut) -> Vec<String> {
    document
        .get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .and_then(|package| package.get("authors"))
        .and_then(|authors| authors.as_array())
        .map(|array| {
            array
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn workspace_dependency_version(document: &DocumentMut, name: &str) -> Option<String> {
    document
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(|dependencies| dependencies.get(name))
        .and_then(|dependency| dependency.get("version"))
        .and_then(|version| version.as_str())
        .map(str::to_owned)
}

/// Package project fields owned by `dhara.config.toml` (semantic compare; not full-file text).
#[derive(Debug, Clone, PartialEq, Eq)]
struct ManagedCsprojSnapshot {
    version: String,
    authors: String,
    repository_url: String,
    package_project_url: String,
    package_license_expression: Option<String>,
}

pub fn csproj_needs_sync(content: &str, config: &DharaRepoConfig) -> Result<bool> {
    Ok(managed_csproj_snapshot_from_content(content)?
        != managed_csproj_snapshot_from_config(config))
}

pub fn sync_csproj(content: &str, config: &DharaRepoConfig) -> Result<String> {
    let expected = managed_csproj_snapshot_from_config(config);
    let current = managed_csproj_snapshot_from_content(content)?;
    if current == expected {
        return Ok(content.to_owned());
    }

    let mut updated = content.to_owned();
    upsert_property_element(&mut updated, "Version", &expected.version)?;
    upsert_property_element(&mut updated, "Authors", &expected.authors)?;
    upsert_property_element(&mut updated, "RepositoryUrl", &expected.repository_url)?;
    upsert_property_element(
        &mut updated,
        "PackageProjectUrl",
        &expected.package_project_url,
    )?;

    match (
        &expected.package_license_expression,
        current.package_license_expression.as_ref(),
    ) {
        (Some(license), _) => {
            upsert_property_element(&mut updated, "PackageLicenseExpression", license)?
        }
        (None, Some(_)) => remove_property_element(&mut updated, "PackageLicenseExpression"),
        (None, None) => {}
    }

    Ok(updated)
}

fn managed_csproj_snapshot_from_config(config: &DharaRepoConfig) -> ManagedCsprojSnapshot {
    ManagedCsprojSnapshot {
        version: config.versions.workspace.clone(),
        authors: config.product.authors.join(";"),
        repository_url: config.product.repository_url.clone(),
        package_project_url: config.product.project_url.clone(),
        package_license_expression: config.product.license.clone(),
    }
}

fn managed_csproj_snapshot_from_content(content: &str) -> Result<ManagedCsprojSnapshot> {
    let project = Element::parse(content.as_bytes()).context("failed to parse package csproj")?;

    Ok(ManagedCsprojSnapshot {
        version: find_property_text(&project, "Version").unwrap_or_default(),
        authors: find_property_text(&project, "Authors").unwrap_or_default(),
        repository_url: find_property_text(&project, "RepositoryUrl").unwrap_or_default(),
        package_project_url: find_property_text(&project, "PackageProjectUrl").unwrap_or_default(),
        package_license_expression: find_property_text(&project, "PackageLicenseExpression"),
    })
}

/// Reads the `PackageId` MSBuild property from a package project (config no longer owns it).
pub fn read_csproj_package_id(repo_root: &Path, relative_csproj: &str) -> Result<String> {
    let path = repo_root.join(relative_csproj);
    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let project = Element::parse(content.as_bytes())
        .with_context(|| format!("failed to parse {}", path.display()))?;
    find_property_text(&project, "PackageId")
        .with_context(|| format!("PackageId property missing from {}", path.display()))
}

fn find_property_text(project: &Element, name: &str) -> Option<String> {
    for child in &project.children {
        let XMLNode::Element(group) = child else {
            continue;
        };
        if group.name != "PropertyGroup" {
            continue;
        }

        for item in &group.children {
            let XMLNode::Element(property) = item else {
                continue;
            };
            if property.name == name {
                return property
                    .get_text()
                    .map(|value| value.trim().to_owned())
                    .filter(|value| !value.is_empty());
            }
        }
    }

    None
}

fn upsert_property_element(content: &mut String, name: &str, value: &str) -> Result<()> {
    let open = format!("<{name}>");
    let close = format!("</{name}>");
    if let Some(start) = content.find(&open) {
        let value_start = start + open.len();
        let rel_end = content[value_start..]
            .find(&close)
            .with_context(|| format!("malformed csproj element <{name}>"))?;
        let value_end = value_start + rel_end;
        if &content[value_start..value_end] == value {
            return Ok(());
        }
        content.replace_range(value_start..value_end, value);
        return Ok(());
    }

    let updated = insert_property_in_first_group(content, name, value)?;
    *content = updated;
    Ok(())
}

fn remove_property_element(content: &mut String, name: &str) {
    let open = format!("<{name}>");
    let close = format!("</{name}>");
    let Some(start) = content.find(&open) else {
        return;
    };
    let Some(rel_end) = content[start..].find(&close) else {
        return;
    };
    let end = start + rel_end + close.len();
    content.replace_range(start..end, "");
}

fn insert_property_in_first_group(content: &str, name: &str, value: &str) -> Result<String> {
    if let Some(index) = content.find("<PropertyGroup>") {
        let insert_at = index + "<PropertyGroup>".len();
        let insertion = format!("\n    <{name}>{value}</{name}>");
        let mut updated = String::with_capacity(content.len() + insertion.len());
        updated.push_str(&content[..insert_at]);
        updated.push_str(&insertion);
        updated.push_str(&content[insert_at..]);
        return Ok(updated);
    }

    let project_open = content
        .find("<Project")
        .context("csproj is missing a Project root element")?;
    let rel_close = content[project_open..]
        .find('>')
        .context("csproj has a malformed Project opening tag")?;
    let insert_at = project_open + rel_close + 1;
    let insertion =
        format!("\n  <PropertyGroup>\n    <{name}>{value}</{name}>\n  </PropertyGroup>");
    let mut updated = String::with_capacity(content.len() + insertion.len());
    updated.push_str(&content[..insert_at]);
    updated.push_str(&insertion);
    updated.push_str(&content[insert_at..]);
    Ok(updated)
}

pub fn parse_env_content(content: &str) -> Result<BTreeMap<String, String>> {
    let mut values = BTreeMap::new();
    for (line_number, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (key, value) = line.split_once('=').with_context(|| {
            format!(
                "invalid env entry on line {}: expected KEY=VALUE",
                line_number + 1
            )
        })?;
        values.insert(key.trim().to_owned(), value.trim().to_owned());
    }
    Ok(values)
}

fn masked_env(values: BTreeMap<String, String>) -> BTreeMap<String, String> {
    values
        .into_iter()
        .map(|(key, value)| {
            let upper_key = key.to_ascii_uppercase();
            if upper_key.contains("KEY")
                || upper_key.contains("TOKEN")
                || upper_key.contains("SECRET")
                || upper_key.contains("PASSWORD")
            {
                (key, "<redacted>".to_owned())
            } else {
                (key, value)
            }
        })
        .collect()
}

pub fn validate_config(repo_root: &Path, config: &DharaRepoConfig) -> Result<()> {
    Version::parse(&config.versions.workspace)
        .with_context(|| format!("invalid workspace version: {}", config.versions.workspace))?;

    if config.product.authors.is_empty() {
        bail!("product.authors must not be empty");
    }
    if config.product.repository_url.trim().is_empty() {
        bail!("product.repository_url must not be empty");
    }
    if config.product.project_url.trim().is_empty() {
        bail!("product.project_url must not be empty");
    }
    if config.nuget.source.trim().is_empty() {
        bail!("nuget.source must not be empty");
    }
    if config.ci.native_runtimes.is_empty() {
        bail!("ci.native_runtimes must not be empty");
    }
    for runtime in &config.ci.native_runtimes {
        if !config.targets.rust_targets.contains_key(runtime) {
            bail!("targets.rust_targets is missing an entry for runtime '{runtime}'");
        }
    }
    if !config
        .ci
        .native_runtimes
        .contains(&config.ci.host_runtime_smoke)
    {
        bail!(
            "ci.host_runtime_smoke '{}' must be present in ci.native_runtimes",
            config.ci.host_runtime_smoke
        );
    }
    if !config
        .ci
        .native_runtimes
        .contains(&config.ci.aot_runtime_smoke)
    {
        bail!(
            "ci.aot_runtime_smoke '{}' must be present in ci.native_runtimes",
            config.ci.aot_runtime_smoke
        );
    }

    require_exists(repo_root, CONFIG_PATH)?;
    require_exists(repo_root, ROOT_CARGO_TOML_PATH)?;
    for path in package_projects(config) {
        require_exists(repo_root, path)?;
    }
    require_exists(repo_root, &config.ci.tests_project)?;
    require_exists(repo_root, &config.ci.smoke_project)?;
    require_exists(repo_root, ENV_EXAMPLE_PATH)?;
    require_exists(repo_root, crate::paths::runtime_defs_relative())?;

    Ok(())
}

fn write_config(repo_root: &Path, config: &DharaRepoConfig) -> Result<()> {
    validate_config(repo_root, config)?;
    let content = toml::to_string_pretty(config).context("failed to serialize config")?;
    let config_path = repo_root.join(CONFIG_PATH);
    fs::write(&config_path, content)
        .with_context(|| format!("failed to write {}", config_path.display()))
}

fn require_exists(repo_root: &Path, relative_path: &str) -> Result<PathBuf> {
    let path = repo_root.join(relative_path);
    if !path.exists() {
        bail!("required path does not exist: {}", path.display());
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    fn sample_config() -> DharaRepoConfig {
        let mut targets = BTreeMap::new();
        targets.insert("win-x64".to_owned(), "x86_64-pc-windows-msvc".to_owned());
        targets.insert("win-arm64".to_owned(), "aarch64-pc-windows-msvc".to_owned());
        targets.insert(
            "linux-x64".to_owned(),
            "x86_64-unknown-linux-gnu".to_owned(),
        );
        targets.insert(
            "linux-arm64".to_owned(),
            "aarch64-unknown-linux-gnu".to_owned(),
        );
        targets.insert("osx-arm64".to_owned(), "aarch64-apple-darwin".to_owned());

        DharaRepoConfig {
            versions: VersionConfig {
                workspace: "0.2.0".to_owned(),
            },
            product: ProductConfig {
                authors: vec!["Naveen Dharmathunga".to_owned()],
                repository_url: "https://github.com/D-Naveenz/rheo_storage".to_owned(),
                project_url: "https://github.com/D-Naveenz/rheo_storage".to_owned(),
                license: None,
            },
            nuget: NuGetConfig {
                source: "https://api.nuget.org/v3/index.json".to_owned(),
            },
            ci: CiConfig {
                smoke_project:
                    "src/bindings/csharp/Dhara.Storage.ConsumerSmoke/Dhara.Storage.ConsumerSmoke.csproj"
                        .to_owned(),
                package_project: "src/bindings/csharp/Dhara.Storage/Dhara.Storage.csproj".to_owned(),
                managed_package_projects: Vec::new(),
                tests_project: "src/bindings/csharp/Dhara.Storage.Tests/Dhara.Storage.Tests.csproj"
                    .to_owned(),
                native_runtimes: vec![
                    "win-x64".to_owned(),
                    "win-arm64".to_owned(),
                    "linux-x64".to_owned(),
                    "linux-arm64".to_owned(),
                    "osx-arm64".to_owned(),
                ],
                host_runtime_smoke: "linux-x64".to_owned(),
                aot_runtime_smoke: "linux-x64".to_owned(),
            },
            targets: TargetsConfig {
                rust_targets: targets,
            },
        }
    }

    fn write_required_files(repo_root: &Path) {
        fs::create_dir_all(repo_root.join("src/bindings/csharp/Dhara.Storage")).unwrap();
        fs::create_dir_all(repo_root.join("src/bindings/csharp/Dhara.Storage.Tests")).unwrap();
        fs::create_dir_all(repo_root.join("src/bindings/csharp/Dhara.Storage.ConsumerSmoke"))
            .unwrap();
        fs::write(repo_root.join(CONFIG_PATH), "placeholder").unwrap();
        fs::write(
            repo_root.join(ROOT_CARGO_TOML_PATH),
            "[workspace]\n[workspace.package]\nversion = \"0.2.0\"\nauthors = [\"Naveen Dharmathunga\"]\nrepository = \"https://github.com/D-Naveenz/rheo_storage\"\nhomepage = \"https://github.com/D-Naveenz/rheo_storage\"\n[workspace.dependencies]\ndhara_storage_core = { version = \"0.2.0\", path = \"src/core/dhara_storage_core\" }\ndhara_storage = { version = \"0.2.0\", path = \"src/core/dhara_storage\" }\n",
        )
        .unwrap();
        fs::write(
            repo_root.join(ENV_EXAMPLE_PATH),
            DEFAULT_ENV_EXAMPLE_CONTENT,
        )
        .unwrap();
        fs::write(
            repo_root.join("src/bindings/csharp/Dhara.Storage/Dhara.Storage.csproj"),
            r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><Version>0.2.0</Version><Authors>Naveen Dharmathunga</Authors><RepositoryUrl>https://github.com/D-Naveenz/rheo_storage</RepositoryUrl><PackageProjectUrl>https://github.com/D-Naveenz/rheo_storage</PackageProjectUrl></PropertyGroup></Project>"#,
        )
        .unwrap();
        fs::write(
            repo_root.join("src/bindings/csharp/Dhara.Storage.Tests/Dhara.Storage.Tests.csproj"),
            "<Project />",
        )
        .unwrap();
        fs::write(
            repo_root.join(
                "src/bindings/csharp/Dhara.Storage.ConsumerSmoke/Dhara.Storage.ConsumerSmoke.csproj",
            ),
            "<Project />",
        )
        .unwrap();
        fs::create_dir_all(repo_root.join(crate::paths::embedded_defs_dir_relative())).unwrap();
        fs::write(
            repo_root.join(crate::paths::runtime_defs_relative()),
            "placeholder",
        )
        .unwrap();
    }

    #[test]
    fn parse_env_content_ignores_comments_and_blank_lines() {
        let parsed = parse_env_content(
            r#"
            # comment
            NUGET_API_KEY=test-key

            NUGET_SOURCE=https://api.nuget.org/v3/index.json
            "#,
        )
        .unwrap();

        assert_eq!(parsed.get("NUGET_API_KEY"), Some(&"test-key".to_owned()));
        assert_eq!(
            parsed.get("NUGET_SOURCE"),
            Some(&"https://api.nuget.org/v3/index.json".to_owned())
        );
    }

    #[test]
    fn masked_env_redacts_secret_like_keys() {
        let mut values = BTreeMap::new();
        values.insert("NUGET_API_KEY".to_owned(), "secret".to_owned());
        values.insert(
            "NUGET_SOURCE".to_owned(),
            "https://api.nuget.org/v3/index.json".to_owned(),
        );

        let masked = masked_env(values);

        assert_eq!(masked.get("NUGET_API_KEY"), Some(&"<redacted>".to_owned()));
        assert_eq!(
            masked.get("NUGET_SOURCE"),
            Some(&"https://api.nuget.org/v3/index.json".to_owned())
        );
    }

    #[test]
    fn sync_cargo_toml_updates_workspace_metadata() {
        let config = sample_config();
        let updated = sync_cargo_toml(
            "[workspace]\n[workspace.package]\nversion = \"0.1.0\"\n[workspace.dependencies]\ndhara_storage_core = { version = \"0.1.0\", path = \"src/core/dhara_storage_core\" }\ndhara_storage = { version = \"0.1.0\", path = \"src/core/dhara_storage\" }\n",
            &config,
        )
        .unwrap();

        assert!(updated.contains("version = \"0.2.0\""));
        assert!(updated.contains(
            "dhara_storage_core = { version = \"0.2.0\", path = \"src/core/dhara_storage_core\" }"
        ));
        assert!(updated.contains(
            "dhara_storage = { version = \"0.2.0\", path = \"src/core/dhara_storage\" }"
        ));
        assert!(updated.contains("authors = [\"Naveen Dharmathunga\"]"));
        assert!(updated.contains("repository = \"https://github.com/D-Naveenz/rheo_storage\""));
        assert!(updated.contains("homepage = \"https://github.com/D-Naveenz/rheo_storage\""));
    }

    #[test]
    fn cargo_toml_needs_sync_ignores_line_endings_when_metadata_matches() {
        let config = sample_config();
        let formatted = "[workspace]\r\n[workspace.package]\r\nversion = \"0.2.0\"\r\nauthors = [\"Naveen Dharmathunga\"]\r\nrepository = \"https://github.com/D-Naveenz/rheo_storage\"\r\nhomepage = \"https://github.com/D-Naveenz/rheo_storage\"\r\n[workspace.dependencies]\r\ndhara_storage_core = { version = \"0.2.0\", path = \"src/core/dhara_storage_core\" }\r\ndhara_storage = { version = \"0.2.0\", path = \"src/core/dhara_storage\" }\r\n";

        assert!(!cargo_toml_needs_sync(formatted, &config).unwrap());
        assert_eq!(sync_cargo_toml(formatted, &config).unwrap(), formatted);
    }

    #[test]
    fn detect_config_drift_ignores_cargo_toml_formatting() {
        let temp = tempdir().unwrap();
        write_required_files(temp.path());
        let config = sample_config();
        fs::write(
            temp.path().join(CONFIG_PATH),
            toml::to_string_pretty(&config).unwrap(),
        )
        .unwrap();
        fs::write(
            temp.path().join(ROOT_CARGO_TOML_PATH),
            "[workspace]\r\n[workspace.package]\r\nversion = \"0.2.0\"\r\nauthors = [\"Naveen Dharmathunga\"]\r\nrepository = \"https://github.com/D-Naveenz/rheo_storage\"\r\nhomepage = \"https://github.com/D-Naveenz/rheo_storage\"\r\n[workspace.dependencies]\r\ndhara_storage_core = { version = \"0.2.0\", path = \"src/core/dhara_storage_core\" }\r\ndhara_storage = { version = \"0.2.0\", path = \"src/core/dhara_storage\" }\r\n",
        )
        .unwrap();

        let drifts = detect_config_drift(temp.path()).unwrap();
        assert!(
            !drifts
                .iter()
                .any(|item| item.kind == ConfigDriftKind::WorkspaceCargoToml)
        );
    }

    #[test]
    fn sync_csproj_updates_shared_metadata() {
        let config = sample_config();
        let updated = sync_csproj(
            r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><Version>1.0.0</Version></PropertyGroup></Project>"#,
            &config,
        )
        .unwrap();

        assert!(updated.contains("<Version>0.2.0</Version>"));
        assert!(updated.contains("<Authors>Naveen Dharmathunga</Authors>"));
        assert!(
            updated.contains(
                "<RepositoryUrl>https://github.com/D-Naveenz/rheo_storage</RepositoryUrl>"
            )
        );
        assert!(updated.contains(
            "<PackageProjectUrl>https://github.com/D-Naveenz/rheo_storage</PackageProjectUrl>"
        ));
        assert!(!updated.contains("PackageId"));
        assert!(!updated.contains("PackageTags"));
    }

    #[test]
    fn sync_csproj_syncs_license_when_configured() {
        let mut config = sample_config();
        config.product.license = Some("MIT".to_owned());
        let updated = sync_csproj(
            r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><Version>1.0.0</Version></PropertyGroup></Project>"#,
            &config,
        )
        .unwrap();

        assert!(updated.contains("<PackageLicenseExpression>MIT</PackageLicenseExpression>"));
    }

    #[test]
    fn csproj_needs_sync_ignores_msbuild_xml_formatting() {
        let config = sample_config();
        let formatted = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <StagedNativeRoot Condition="&apos;$(StagedNativeRoot)&apos; == &apos;&apos;" />
    <Version>0.2.0</Version>
    <RepositoryUrl>https://github.com/D-Naveenz/rheo_storage</RepositoryUrl>
    <PackageProjectUrl>https://github.com/D-Naveenz/rheo_storage</PackageProjectUrl>
    <Authors>Naveen Dharmathunga</Authors>
  </PropertyGroup>
</Project>"#;

        assert!(!csproj_needs_sync(formatted, &config).unwrap());
        assert_eq!(sync_csproj(formatted, &config).unwrap(), formatted);
    }

    #[test]
    fn package_projects_deduplicates_and_leads_with_primary() {
        let mut config = sample_config();
        config.ci.managed_package_projects = vec![
            "src/bindings/csharp/Dhara.Storage.Extensions.Hosting/Dhara.Storage.Extensions.Hosting.csproj".to_owned(),
            config.ci.package_project.clone(),
        ];

        let projects = package_projects(&config);
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0], config.ci.package_project.as_str());
    }

    #[test]
    fn read_csproj_package_id_reads_property() {
        let temp = tempdir().unwrap();
        fs::create_dir_all(temp.path().join("pkg")).unwrap();
        fs::write(
            temp.path().join("pkg/Package.csproj"),
            r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><PackageId>Dhara.Storage</PackageId></PropertyGroup></Project>"#,
        )
        .unwrap();

        let package_id = read_csproj_package_id(temp.path(), "pkg/Package.csproj").unwrap();
        assert_eq!(package_id, "Dhara.Storage");
    }

    #[test]
    fn validate_config_accepts_complete_repo_layout() {
        let temp = tempdir().unwrap();
        write_required_files(temp.path());
        let config = sample_config();

        validate_config(temp.path(), &config).unwrap();
    }

    #[test]
    fn validate_config_rejects_empty_authors() {
        let temp = tempdir().unwrap();
        write_required_files(temp.path());
        let mut config = sample_config();
        config.product.authors.clear();

        let error = validate_config(temp.path(), &config).unwrap_err();
        assert!(error.to_string().contains("product.authors"));
    }

    #[test]
    fn ensure_repo_scaffolding_creates_missing_files() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("repo");
        fs::create_dir_all(&root).unwrap();

        ensure_repo_scaffolding(&root).unwrap();

        assert!(root.join(CONFIG_PATH).is_file());
        assert!(root.join(ENV_EXAMPLE_PATH).is_file());
        assert!(root.join(ENV_LOCAL_PATH).is_file());
        let config = load_config(&root).unwrap();
        assert_eq!(config.nuget.source, "https://api.nuget.org/v3/index.json");
    }

    #[test]
    fn ensure_repo_scaffolding_preserves_existing_config() {
        let temp = tempdir().unwrap();
        write_required_files(temp.path());
        let config = sample_config();
        fs::write(
            temp.path().join(CONFIG_PATH),
            toml::to_string_pretty(&config).unwrap(),
        )
        .unwrap();

        ensure_repo_scaffolding(temp.path()).unwrap();

        let reloaded = load_config(temp.path()).unwrap();
        assert_eq!(reloaded, config);
    }

    #[test]
    fn detect_config_drift_reports_workspace_cargo_mismatch() {
        let temp = tempdir().unwrap();
        write_required_files(temp.path());
        let config = sample_config();
        fs::write(
            temp.path().join(CONFIG_PATH),
            toml::to_string_pretty(&config).unwrap(),
        )
        .unwrap();
        fs::write(
            temp.path().join(ROOT_CARGO_TOML_PATH),
            "[workspace]\n[workspace.package]\nversion = \"0.1.0\"\n[workspace.dependencies]\ndhara_storage_core = { version = \"0.1.0\", path = \"src/core/dhara_storage_core\" }\ndhara_storage = { version = \"0.1.0\", path = \"src/core/dhara_storage\" }\n",
        )
        .unwrap();

        let drifts = detect_config_drift(temp.path()).unwrap();
        assert!(drifts.iter().any(|item| {
            item.kind == ConfigDriftKind::WorkspaceCargoToml && item.summary.contains("0.2.0")
        }));
    }

    #[test]
    fn apply_config_drift_writes_workspace_cargo_from_config() {
        let temp = tempdir().unwrap();
        write_required_files(temp.path());
        let config = sample_config();
        fs::write(
            temp.path().join(CONFIG_PATH),
            toml::to_string_pretty(&config).unwrap(),
        )
        .unwrap();
        fs::write(
            temp.path().join(ROOT_CARGO_TOML_PATH),
            "[workspace]\n[workspace.package]\nversion = \"0.1.0\"\n[workspace.dependencies]\ndhara_storage_core = { version = \"0.1.0\", path = \"src/core/dhara_storage_core\" }\ndhara_storage = { version = \"0.1.0\", path = \"src/core/dhara_storage\" }\n",
        )
        .unwrap();

        let drifts = detect_config_drift(temp.path()).unwrap();
        apply_config_drift(temp.path(), &drifts).unwrap();
        let cargo = fs::read_to_string(temp.path().join(ROOT_CARGO_TOML_PATH)).unwrap();
        assert!(cargo.contains("version = \"0.2.0\""));
        assert!(
            !detect_config_drift(temp.path())
                .unwrap()
                .iter()
                .any(|item| item.kind == ConfigDriftKind::WorkspaceCargoToml)
        );
    }

    #[test]
    fn apply_config_drift_syncs_all_package_projects() {
        let temp = tempdir().unwrap();
        write_required_files(temp.path());
        let mut config = sample_config();
        let managed_relative = "src/bindings/csharp/Dhara.Storage.Extensions.Hosting/Dhara.Storage.Extensions.Hosting.csproj";
        config.ci.managed_package_projects = vec![managed_relative.to_owned()];
        fs::write(
            temp.path().join(CONFIG_PATH),
            toml::to_string_pretty(&config).unwrap(),
        )
        .unwrap();
        fs::create_dir_all(
            temp.path()
                .join("src/bindings/csharp/Dhara.Storage.Extensions.Hosting"),
        )
        .unwrap();
        fs::write(
            temp.path().join(managed_relative),
            r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><Version>0.0.1</Version></PropertyGroup></Project>"#,
        )
        .unwrap();

        let drifts = detect_config_drift(temp.path()).unwrap();
        assert!(
            drifts
                .iter()
                .any(|item| item.summary.contains(managed_relative))
        );
        apply_config_drift(temp.path(), &drifts).unwrap();

        let managed_content = fs::read_to_string(temp.path().join(managed_relative)).unwrap();
        assert!(managed_content.contains("<Version>0.2.0</Version>"));
        assert!(
            !detect_config_drift(temp.path())
                .unwrap()
                .iter()
                .any(|item| item.kind == ConfigDriftKind::PackageCsproj)
        );
    }

    #[test]
    fn bump_version_updates_workspace_version() {
        let temp = tempdir().unwrap();
        write_required_files(temp.path());
        let config = sample_config();
        fs::write(
            temp.path().join(CONFIG_PATH),
            toml::to_string_pretty(&config).unwrap(),
        )
        .unwrap();

        let bumped = bump_version(temp.path(), VersionPart::Major).unwrap();
        let reloaded = load_config(temp.path()).unwrap();

        assert_eq!(bumped, "1.0.0");
        assert_eq!(reloaded.versions.workspace, "1.0.0");
    }
}
