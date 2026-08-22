use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use tracing::debug;

use xmltree::{Element, XMLNode};

use drot_kernel::CommandResult;
use drot_kernel::{
    ProgressSession, has_committed_progress_plan,
    logging::log_module_step_debug,
    paths::{default_artifacts_dir, resolve_output_dir},
    repo_config::{
        DharaRepoConfig, NUGET_API_KEY_ENV, load_env, read_csproj_package_id, verify_release,
    },
    subprocess::{
        inspect_package_entries, run_command, run_command_expect_failure,
        run_command_with_env_redacted, write_nuget_config,
    },
};

use crate::ops::native_rids::{
    native_lib_filename, package_native_path, platform, platform_target, staging_runtimes_on_host,
};
use crate::ops::workflow_progress::{begin_workflow, plan_unit_step, run_planned_step};

#[derive(Debug, Clone)]
pub struct PackageOptions {
    pub configuration: String,
    pub version_override: Option<String>,
    pub source_override: Option<String>,
    pub output_dir: Option<PathBuf>,
    pub execute_publish: bool,
    pub native_stage_override: Option<PathBuf>,
    pub prepacked_nuget_override: Option<PathBuf>,
    /// When staging on the current host, also build cross-native targets (for example `win-arm64`
    /// on Windows x64). Defaults to true for `package stage-native` / CI; local `build run` sets
    /// this to false unless `--cross-native` is passed.
    pub include_cross_native: bool,
    /// Native RIDs required in the stage directory and `.nupkg`. When unset, all
    /// `ci.native_runtimes` from config are required (merged CI layout).
    pub expected_native_runtimes: Option<Vec<String>>,
}

pub fn pack(
    repo_root: &Path,
    tool_root: &Path,
    config: &DharaRepoConfig,
    options: &PackageOptions,
) -> Result<CommandResult> {
    log_module_step_debug(&format!(
        "packing NuGet package (configuration={}, version={})",
        options.configuration,
        options.version_override.as_deref().unwrap_or("from config")
    ));
    verify_release(repo_root)?;

    let nested = has_committed_progress_plan();
    if !nested {
        setup_pack_plan(config, options)?;
    }

    let version = effective_version(config, &options.version_override);
    let artifacts_root = artifacts_root(tool_root)?;
    let output_root = output_root(tool_root, options.output_dir.as_ref())?;
    let native_stage_root = if let Some(stage) = options
        .native_stage_override
        .clone()
        .or_else(native_stage_from_env)
    {
        absolute_native_stage_root(repo_root, &stage)
    } else {
        let stage = artifacts_root.join("native-stage");
        reset_directory(&stage)?;
        stage_native_assets(repo_root, config, options, &stage, !nested)?;
        stage
    };
    let nuget_output = output_root.join("nuget");
    reset_directory(&nuget_output)?;

    validate_staged_native_assets(&native_stage_root, config, options)?;

    let pack_project = |project: &str| {
        run_command(
            "dotnet",
            &[
                "pack".to_owned(),
                project.to_owned(),
                "--configuration".to_owned(),
                options.configuration.clone(),
                "--include-symbols".to_owned(),
                "-p:ContinuousIntegrationBuild=true".to_owned(),
                "-p:Platform=AnyCPU".to_owned(),
                "-p:PlatformTarget=AnyCPU".to_owned(),
                format!("-p:Version={version}"),
                format!("-p:StagedNativeRoot={}", native_stage_root.display()),
                "--output".to_owned(),
                nuget_output.display().to_string(),
            ],
            repo_root,
        )
    };

    if nested {
        pack_project(&config.ci.package_project)?;
    } else {
        run_planned_step(
            "dotnet-pack",
            "Packing NuGet package",
            "Running dotnet pack",
            || pack_project(&config.ci.package_project),
        )?;
    }

    let package_id = read_csproj_package_id(repo_root, &config.ci.package_project)?;
    let package_path = nuget_output.join(format!("{package_id}.{version}.nupkg"));
    if nested {
        inspect_package_contents(repo_root, &package_path, config, options)?;
    } else {
        run_planned_step(
            "inspect",
            "Inspecting package",
            "Inspecting package contents",
            || inspect_package_contents(repo_root, &package_path, config, options),
        )?;
    }

    if !config.ci.managed_package_projects.is_empty() {
        let pack_managed = || {
            for project in &config.ci.managed_package_projects {
                pack_project(project)?;
            }
            Ok(())
        };
        if nested {
            pack_managed()?;
        } else {
            run_planned_step(
                "pack-managed",
                "Packing managed NuGet packages",
                "Packing managed NuGet packages",
                pack_managed,
            )?;
        }
    }

    log_module_step_debug(&format!(
        "packed NuGet package at {}",
        package_path.display()
    ));

    Ok(CommandResult::with_message(format!(
        "Packed {}",
        package_path.display()
    )))
}

fn setup_pack_plan(config: &DharaRepoConfig, options: &PackageOptions) -> Result<()> {
    let runtimes = staging_runtimes_on_host(
        &config.ci.native_runtimes,
        options.include_cross_native,
    );
    if runtimes.is_empty()
        && options.native_stage_override.is_none()
        && native_stage_from_env().is_none()
    {
        bail!("no native runtimes are buildable on the current host");
    }

    let Some(session) = begin_workflow("Planning package build…") else {
        return Ok(());
    };

    if options.native_stage_override.is_none() && native_stage_from_env().is_none() {
        let count = runtimes.len().max(1) as u64;
        session.plan("stage-native", "Staging native libraries", count);
        session.set_total("stage-native", count);
    }
    plan_unit_step(&session, "dotnet-pack", "Packing NuGet package");
    plan_unit_step(&session, "inspect", "Inspecting package");
    if !config.ci.managed_package_projects.is_empty() {
        plan_unit_step(&session, "pack-managed", "Packing managed NuGet packages");
    }
    session.commit();
    Ok(())
}

pub fn verify(
    repo_root: &Path,
    tool_root: &Path,
    config: &DharaRepoConfig,
    options: &PackageOptions,
) -> Result<CommandResult> {
    log_module_step_debug(&format!(
        "verifying NuGet package (configuration={})",
        options.configuration
    ));

    let nested = has_committed_progress_plan();
    if !nested && let Some(session) = begin_workflow("Planning package verification…") {
        plan_unit_step(&session, "pack", "Packing NuGet package");
        plan_unit_step(&session, "restore-smoke", "Restoring smoke consumer");
        plan_unit_step(&session, "run-smoke", "Running smoke consumer");
        plan_unit_step(
            &session,
            "reject-check",
            "Verifying unsupported runtime rejection",
        );
        plan_unit_step(&session, "aot-restore", "Restoring AOT smoke consumer");
        plan_unit_step(&session, "aot-publish", "Publishing AOT smoke consumer");
        session.commit();
    }

    let run_step = |id, label, detail, op: &dyn Fn() -> Result<()>| {
        if nested {
            op()
        } else {
            run_planned_step(id, label, detail, op)
        }
    };

    run_step(
        "pack",
        "Packing NuGet package",
        "Building and packing package",
        &|| pack(repo_root, tool_root, config, options).map(|_| ()),
    )?;

    let version = effective_version(config, &options.version_override);
    let artifacts_root = artifacts_root(tool_root)?;
    let output_root = output_root(tool_root, options.output_dir.as_ref())?;
    let package_id = read_csproj_package_id(repo_root, &config.ci.package_project)?;
    let package_path = output_root
        .join("nuget")
        .join(format!("{package_id}.{version}.nupkg"));
    let local_config = artifacts_root.join("local-package.nuget.config");
    let dependency_source = effective_source(repo_root, config, options)?;
    write_nuget_config(
        &local_config,
        &[
            package_path
                .parent()
                .context("package path should have a parent")?
                .to_path_buf(),
            PathBuf::from(&dependency_source),
        ],
    )?;

    run_step(
        "restore-smoke",
        "Restoring smoke consumer",
        "Restoring host smoke consumer",
        &|| {
            restore_smoke_consumer(
                repo_root,
                config,
                &version,
                &local_config,
                Some(&config.ci.host_runtime_smoke),
                false,
            )
        },
    )?;
    run_step(
        "run-smoke",
        "Running smoke consumer",
        "Running host smoke consumer",
        &|| run_smoke_consumer(repo_root, config, &version),
    )?;
    run_step(
        "reject-check",
        "Verifying unsupported runtime rejection",
        "Verifying unsupported runtime is rejected",
        &|| verify_unsupported_runtime_rejected(repo_root, config, &version, &local_config),
    )?;
    run_step(
        "aot-restore",
        "Restoring AOT smoke consumer",
        "Restoring AOT smoke consumer",
        &|| {
            restore_smoke_consumer(
                repo_root,
                config,
                &version,
                &local_config,
                Some(&config.ci.aot_runtime_smoke),
                true,
            )
        },
    )?;
    run_step(
        "aot-publish",
        "Publishing AOT smoke consumer",
        "Publishing AOT smoke consumer",
        &|| {
            publish_aot_smoke_consumer(
                repo_root,
                config,
                &version,
                &artifacts_root.join("smoke-aot"),
                &config.ci.aot_runtime_smoke,
            )
        },
    )?;
    log_module_step_debug(&format!(
        "completed NuGet verification at {}",
        package_path.display()
    ));
    Ok(CommandResult::with_message(
        "Package verified successfully.",
    ))
}

pub fn publish(
    repo_root: &Path,
    tool_root: &Path,
    config: &DharaRepoConfig,
    options: &PackageOptions,
) -> Result<CommandResult> {
    log_module_step_debug(&format!(
        "publishing NuGet package (execute={})",
        options.execute_publish
    ));

    let nested = has_committed_progress_plan();
    if !nested && let Some(session) = begin_workflow("Planning package publish…") {
        plan_unit_step(&session, "verify", "Verifying package");
        if options.execute_publish {
            plan_unit_step(&session, "push", "Publishing to NuGet feed");
        }
        session.commit();
    }

    if nested {
        verify(repo_root, tool_root, config, options)?;
    } else {
        run_planned_step(
            "verify",
            "Verifying package",
            "Running package verification",
            || verify(repo_root, tool_root, config, options).map(|_| ()),
        )?;
    }

    if !options.execute_publish {
        return Ok(CommandResult::with_message(
            "Dry run complete. Package was verified but not published.",
        ));
    }

    let source = effective_source(repo_root, config, options)?;
    let api_key = secret_from_env(repo_root, NUGET_API_KEY_ENV)?;

    let output_root = output_root(tool_root, options.output_dir.as_ref())?;
    let packages = collect_publishable_packages(&output_root.join("nuget"))?;

    let push = || push_packages(repo_root, &packages, &source, &api_key);

    if nested {
        push()?;
    } else {
        run_planned_step(
            "push",
            "Publishing to NuGet feed",
            "Running dotnet nuget push",
            push,
        )?;
    }

    log_module_step_debug(&format!(
        "published {} NuGet package(s) via {}",
        packages.len(),
        source
    ));

    Ok(CommandResult::with_message(
        "Published package successfully.",
    ))
}

pub fn publish_packed(
    repo_root: &Path,
    tool_root: &Path,
    config: &DharaRepoConfig,
    options: &PackageOptions,
) -> Result<CommandResult> {
    log_module_step_debug("publishing pre-packed NuGet package");

    let source = effective_source(repo_root, config, options)?;
    let api_key = secret_from_env(repo_root, NUGET_API_KEY_ENV)?;

    let output_root = output_root(tool_root, options.output_dir.as_ref())?;
    let packages = if let Some(path) = options.prepacked_nuget_override.as_ref() {
        if !path.exists() {
            bail!("NuGet package does not exist: {}", path.display());
        }
        vec![path.clone()]
    } else {
        collect_publishable_packages(&output_root.join("nuget"))?
    };

    push_packages(repo_root, &packages, &source, &api_key)?;

    log_module_step_debug(&format!(
        "published {} pre-packed NuGet package(s) via {}",
        packages.len(),
        source
    ));

    Ok(CommandResult::with_message(
        "Published package successfully.",
    ))
}

/// Non-symbols `*.nupkg` files under `nuget_dir`, sorted for deterministic publish order.
fn collect_publishable_packages(nuget_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut packages = Vec::new();
    let entries = fs::read_dir(nuget_dir)
        .with_context(|| format!("failed to read {}", nuget_dir.display()))?;
    for entry in entries {
        let entry =
            entry.with_context(|| format!("failed to read entry in {}", nuget_dir.display()))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("nupkg") {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if file_name.ends_with(".symbols.nupkg") {
            continue;
        }
        packages.push(path);
    }
    packages.sort();
    if packages.is_empty() {
        bail!("no NuGet packages found in {}", nuget_dir.display());
    }
    Ok(packages)
}

fn push_packages(
    repo_root: &Path,
    packages: &[PathBuf],
    source: &str,
    api_key: &str,
) -> Result<()> {
    for package_path in packages {
        run_command_with_env_redacted(
            "dotnet",
            &[
                "nuget".to_owned(),
                "push".to_owned(),
                package_path.display().to_string(),
                "--api-key".to_owned(),
                api_key.to_owned(),
                "--source".to_owned(),
                source.to_owned(),
                "--skip-duplicate".to_owned(),
            ],
            repo_root,
            &[],
            &[api_key],
        )?;
    }
    Ok(())
}

fn stage_native_assets(
    repo_root: &Path,
    config: &DharaRepoConfig,
    options: &PackageOptions,
    stage_root: &Path,
    report_progress: bool,
) -> Result<()> {
    let profile_flag = if options.configuration.eq_ignore_ascii_case("Release") {
        "--release"
    } else {
        bail!("only Release packaging is currently supported");
    };

    let runtimes = staging_runtimes_on_host(
        &config.ci.native_runtimes,
        options.include_cross_native,
    );
    if runtimes.is_empty() {
        bail!("no native runtimes are buildable on the current host");
    }

    for (index, rid) in runtimes.iter().enumerate() {
        if report_progress {
            let session = ProgressSession;
            session.tick(
                "stage-native",
                index as u64,
                format!("Staging native library for {rid}"),
            );
        }
        let target = config
            .targets
            .rust_targets
            .get(rid)
            .with_context(|| format!("missing rust target mapping for runtime '{rid}'"))?;
        let lib_name = native_lib_filename(rid)?;
        debug!(
            target: "drot::package_flow",
            runtime = %rid,
            rust_target = %target,
            stage_root = %stage_root.display(),
            "staging dhara-sd sidecar"
        );
        run_command(
            "cargo",
            &[
                "build".to_owned(),
                "-p".to_owned(),
                "dhara-sd".to_owned(),
                profile_flag.to_owned(),
                "--target".to_owned(),
                target.clone(),
            ],
            repo_root,
        )?;

        let source_path = repo_root
            .join("target")
            .join(target)
            .join("release")
            .join(lib_name);
        let destination_path = stage_root
            .join("runtimes")
            .join(rid)
            .join("native")
            .join(lib_name);
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::copy(&source_path, &destination_path).with_context(|| {
            format!(
                "failed to copy native asset from '{}' to '{}'",
                source_path.display(),
                destination_path.display()
            )
        })?;
    }

    if report_progress {
        let session = ProgressSession;
        session.finish_step("stage-native", "Staging native libraries");
    }

    Ok(())
}

/// Re-runs `package stage-native` inside the Visual Studio cross-compilation environment.
///
/// Loads `vcvarsall.bat x64_arm64` on Windows x64 hosts so `win-arm64` (`aarch64-pc-windows-msvc`)
/// links with the MSVC toolset. Used by `build run` and `package stage-native --msvc-env`.
#[cfg(windows)]
pub fn stage_native_under_msvc_env(repo_root: &Path, configuration: &str) -> Result<()> {
    use anyhow::Context;

    let exe = std::env::current_exe().context("failed to resolve drot executable path")?;
    let mut command = format!(
        "\"{}\" -r \"{}\" --yes package stage-native",
        exe.display(),
        repo_root.display()
    );
    if !configuration.eq_ignore_ascii_case("Release") {
        command.push_str(&format!(" --configuration {configuration}"));
    }
    drot_kernel::msvc::run_with_msvc_env(&command)
}

/// Stages native libraries buildable on the current host into `{tool_root}/artifacts/native-stage`.
pub fn stage_native_for_host(
    repo_root: &Path,
    tool_root: &Path,
    config: &DharaRepoConfig,
    options: &PackageOptions,
) -> Result<CommandResult> {
    let runtimes = staging_runtimes_on_host(
        &config.ci.native_runtimes,
        options.include_cross_native,
    );
    if runtimes.is_empty() {
        bail!("no native runtimes are buildable on the current host");
    }

    if let Some(session) = begin_workflow("Planning native staging…") {
        let count = runtimes.len() as u64;
        session.plan("stage-native", "Staging native libraries", count);
        session.set_total("stage-native", count);
        session.commit();
    }

    let artifacts_root = artifacts_root(tool_root)?;
    let stage_root = artifacts_root.join("native-stage");
    reset_directory(&stage_root)?;
    stage_native_assets(repo_root, config, options, &stage_root, true)?;
    Ok(CommandResult::with_message(format!(
        "Staged host native assets at {}",
        stage_root.display()
    )))
}

fn inspect_package_contents(
    repo_root: &Path,
    package_path: &Path,
    config: &DharaRepoConfig,
    options: &PackageOptions,
) -> Result<()> {
    let entries = inspect_package_entries(package_path)?;
    debug!(
        target: "drot::package_flow",
        package_path = %package_path.display(),
        entry_count = entries.len(),
        "inspecting package contents"
    );
    if !entries
        .iter()
        .any(|entry| entry == "lib/net10.0/Dhara.Storage.dll")
    {
        bail!("managed assembly missing from package");
    }

    for rid in expected_native_runtimes(config, options) {
        let expected = package_native_path(&rid)?;
        if !entries.iter().any(|entry| entry == &expected) {
            bail!("native asset missing from package: {expected}");
        }
    }
    if !entries.iter().any(|entry| entry == "README.md") {
        bail!("README.md missing from package");
    }
    if let Some(icon) = read_csproj_property(repo_root, &config.ci.package_project, "PackageIcon")?
    {
        let icon_name = Path::new(&icon)
            .file_name()
            .and_then(|value| value.to_str())
            .with_context(|| format!("PackageIcon path must end with a file name: {icon}"))?;
        if !entries.iter().any(|entry| entry == icon_name) {
            bail!("package icon missing from package: {icon_name}");
        }
    }
    if !entries
        .iter()
        .any(|entry| entry == "build/Dhara.Storage.targets")
    {
        bail!("build/Dhara.Storage.targets missing from package");
    }
    Ok(())
}

/// Reads an optional MSBuild `PropertyGroup` property from a package project.
fn read_csproj_property(
    repo_root: &Path,
    relative_csproj: &str,
    name: &str,
) -> Result<Option<String>> {
    let path = repo_root.join(relative_csproj);
    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let project = Element::parse(content.as_bytes())
        .with_context(|| format!("failed to parse {}", path.display()))?;
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
                return Ok(property
                    .get_text()
                    .map(|value| value.trim().to_owned())
                    .filter(|value| !value.is_empty()));
            }
        }
    }
    Ok(None)
}

fn restore_smoke_consumer(
    repo_root: &Path,
    config: &DharaRepoConfig,
    version: &str,
    nuget_config: &Path,
    runtime: Option<&str>,
    publish_aot: bool,
) -> Result<()> {
    log_module_step_debug(&format!(
        "restoring smoke consumer {} (runtime={}, aot={publish_aot})",
        config.ci.smoke_project,
        runtime.unwrap_or("default")
    ));
    let package_id = read_csproj_package_id(repo_root, &config.ci.package_project)?;
    remove_package_cache(repo_root, &package_id, version)?;
    reset_smoke_consumer_outputs(repo_root, config)?;
    let mut args = vec![
        "restore".to_owned(),
        config.ci.smoke_project.clone(),
        format!("-p:DharaStoragePackageVersion={version}"),
        format!("--configfile={}", nuget_config.display()),
        "--force-evaluate".to_owned(),
    ];
    if let Some(runtime) = runtime {
        args.push("--runtime".to_owned());
        args.push(runtime.to_owned());
        push_dotnet_platform_args(&mut args, runtime)?;
    }
    if publish_aot {
        args.push("-p:PublishAot=true".to_owned());
    }
    run_command("dotnet", &args, repo_root)
}

fn run_smoke_consumer(repo_root: &Path, config: &DharaRepoConfig, version: &str) -> Result<()> {
    log_module_step_debug(&format!(
        "running smoke consumer {} (version={version})",
        config.ci.smoke_project
    ));
    let mut run_args = vec![
        "run".to_owned(),
        "--project".to_owned(),
        config.ci.smoke_project.clone(),
        "--configuration".to_owned(),
        "Release".to_owned(),
        "--runtime".to_owned(),
        config.ci.host_runtime_smoke.clone(),
        "--no-restore".to_owned(),
        format!("-p:DharaStoragePackageVersion={version}"),
    ];
    push_dotnet_platform_args(&mut run_args, &config.ci.host_runtime_smoke)?;
    run_command("dotnet", &run_args, repo_root)
}

fn verify_unsupported_runtime_rejected(
    repo_root: &Path,
    config: &DharaRepoConfig,
    version: &str,
    nuget_config: &Path,
) -> Result<()> {
    log_module_step_debug(&format!(
        "verifying unsupported runtime rejection for {} (win-x86)",
        config.ci.smoke_project
    ));
    let package_id = read_csproj_package_id(repo_root, &config.ci.package_project)?;
    remove_package_cache(repo_root, &package_id, version)?;
    run_command_expect_failure(
        "dotnet",
        &[
            "build".to_owned(),
            config.ci.smoke_project.clone(),
            "--configuration".to_owned(),
            "Release".to_owned(),
            "--runtime".to_owned(),
            "win-x86".to_owned(),
            "-p:Platform=x86".to_owned(),
            "-p:PlatformTarget=x86".to_owned(),
            format!("--configfile={}", nuget_config.display()),
            format!("-p:DharaStoragePackageVersion={version}"),
        ],
        repo_root,
        "does not support 32-bit runtime identifier",
    )
}

fn publish_aot_smoke_consumer(
    repo_root: &Path,
    config: &DharaRepoConfig,
    version: &str,
    output_dir: &Path,
    runtime: &str,
) -> Result<()> {
    log_module_step_debug(&format!(
        "publishing AOT smoke consumer {} (runtime={runtime}, output={})",
        config.ci.smoke_project,
        output_dir.display()
    ));
    reset_directory(output_dir)?;
    let mut publish_args = vec![
        "publish".to_owned(),
        config.ci.smoke_project.clone(),
        "--configuration".to_owned(),
        "Release".to_owned(),
        "--runtime".to_owned(),
        runtime.to_owned(),
        "--self-contained".to_owned(),
        "true".to_owned(),
        "--no-restore".to_owned(),
        "-p:PublishAot=true".to_owned(),
        format!("-p:DharaStoragePackageVersion={version}"),
        "--output".to_owned(),
        output_dir.display().to_string(),
    ];
    push_dotnet_platform_args(&mut publish_args, runtime)?;
    run_command("dotnet", &publish_args, repo_root)?;

    let executable = smoke_consumer_executable(output_dir);
    run_command(
        executable
            .to_str()
            .context("published smoke consumer path was not valid utf-8")?,
        &[],
        repo_root,
    )
}

fn remove_package_cache(repo_root: &Path, package_id: &str, version: &str) -> Result<()> {
    let mut package_roots = Vec::new();
    if let Some(path) = std::env::var_os("NUGET_PACKAGES") {
        package_roots.push(PathBuf::from(path));
    }
    if let Some(path) = std::env::var_os("DOTNET_CLI_HOME") {
        package_roots.push(PathBuf::from(path).join(".nuget").join("packages"));
    }
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        package_roots.push(PathBuf::from(home).join(".nuget").join("packages"));
    }
    package_roots.push(repo_root.join(".dotnet").join(".nuget").join("packages"));

    package_roots.sort();
    package_roots.dedup();

    for package_root in package_roots {
        let package_path = package_root
            .join(package_id.to_ascii_lowercase())
            .join(version);
        if package_path.exists() {
            fs::remove_dir_all(&package_path).with_context(|| {
                format!(
                    "failed to remove stale package cache at {}",
                    package_path.display()
                )
            })?;
        }
    }
    Ok(())
}

fn reset_smoke_consumer_outputs(repo_root: &Path, config: &DharaRepoConfig) -> Result<()> {
    let project_path = repo_root.join(&config.ci.smoke_project);
    let project_dir = project_path.parent().with_context(|| {
        format!(
            "smoke project path must have a parent: {}",
            project_path.display()
        )
    })?;
    for directory in ["bin", "obj"] {
        let path = project_dir.join(directory);
        if path.exists() {
            fs::remove_dir_all(&path)
                .with_context(|| format!("failed to remove {}", path.display()))?;
        }
    }
    Ok(())
}

fn push_dotnet_platform_args(args: &mut Vec<String>, runtime: &str) -> Result<()> {
    if let Some(value) = platform(runtime)? {
        args.push(format!("-p:Platform={value}"));
    }
    if let Some(value) = platform_target(runtime)? {
        args.push(format!("-p:PlatformTarget={value}"));
    }
    Ok(())
}

fn smoke_consumer_executable(output_dir: &Path) -> PathBuf {
    if cfg!(windows) {
        output_dir.join("Dhara.Storage.ConsumerSmoke.exe")
    } else {
        output_dir.join("Dhara.Storage.ConsumerSmoke")
    }
}

fn effective_version(config: &DharaRepoConfig, override_value: &Option<String>) -> String {
    override_value
        .clone()
        .unwrap_or_else(|| config.versions.workspace.clone())
}

fn effective_source(
    repo_root: &Path,
    config: &DharaRepoConfig,
    options: &PackageOptions,
) -> Result<String> {
    if let Some(source) = options.source_override.clone().and_then(non_empty_option) {
        return Ok(source);
    }

    if let Some(source) = env_file_value(repo_root, "NUGET_SOURCE")? {
        return Ok(source);
    }

    if let Ok(source) = std::env::var("NUGET_SOURCE")
        && !source.trim().is_empty()
    {
        return Ok(source);
    }

    Ok(config.nuget.source.clone())
}

fn secret_from_env(repo_root: &Path, key: &str) -> Result<String> {
    if let Some(value) = env_file_value(repo_root, key)? {
        return Ok(value);
    }

    std::env::var(key)
        .ok()
        .and_then(non_empty_option)
        .with_context(|| format!("{key} is not set in the environment or .env.local"))
}

fn env_file_value(repo_root: &Path, key: &str) -> Result<Option<String>> {
    Ok(load_env(repo_root)?.remove(key).and_then(non_empty_option))
}

fn non_empty_option(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

fn validate_staged_native_assets(
    stage_root: &Path,
    config: &DharaRepoConfig,
    options: &PackageOptions,
) -> Result<()> {
    for rid in expected_native_runtimes(config, options) {
        let lib_name = native_lib_filename(&rid)?;
        let path = stage_root
            .join("runtimes")
            .join(&rid)
            .join("native")
            .join(lib_name);
        if !path.is_file() {
            bail!(
                "staged native asset missing before pack: {}",
                path.display()
            );
        }
    }
    Ok(())
}

fn expected_native_runtimes(
    config: &DharaRepoConfig,
    options: &PackageOptions,
) -> Vec<String> {
    options
        .expected_native_runtimes
        .clone()
        .unwrap_or_else(|| config.ci.native_runtimes.clone())
}

fn absolute_native_stage_root(repo_root: &Path, stage: &Path) -> PathBuf {
    if stage.is_absolute() {
        stage.to_path_buf()
    } else {
        repo_root.join(stage)
    }
}

fn artifacts_root(tool_root: &Path) -> Result<PathBuf> {
    let root = default_artifacts_dir(tool_root);
    fs::create_dir_all(&root).with_context(|| format!("failed to create {}", root.display()))?;
    Ok(root)
}

fn output_root(tool_root: &Path, override_value: Option<&PathBuf>) -> Result<PathBuf> {
    let root = resolve_output_dir(tool_root, override_value.map(PathBuf::as_path));
    fs::create_dir_all(&root).with_context(|| format!("failed to create {}", root.display()))?;
    Ok(root)
}

fn reset_directory(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path).with_context(|| format!("failed to remove {}", path.display()))?;
    }
    fs::create_dir_all(path).with_context(|| format!("failed to create {}", path.display()))?;
    Ok(())
}

fn native_stage_from_env() -> Option<PathBuf> {
    std::env::var_os("DHARA_NATIVE_STAGE_OVERRIDE").map(PathBuf::from)
}
