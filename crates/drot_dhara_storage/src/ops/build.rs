use anyhow::Result;

use drot_kernel::{
    ToolContext,
    paths::embedded_defs_package_path,
    repo_config::{DharaRepoConfig, apply_config_drift, detect_config_drift, load_config},
};

use crate::filedefs::{DefsPaths, sync_embedded_package, trid_progress::log_build_progress};
use crate::ops::native_rids::staging_runtimes_on_host;
#[cfg(windows)]
use crate::ops::nuget::stage_native_under_msvc_env;
use crate::ops::nuget::{PackageOptions, stage_native_for_host, verify};
use crate::ops::quality;
use crate::ops::workflow_progress::{begin_workflow, plan_unit_step, run_planned_step};

/// Options for the composite local build workflow.
#[derive(Debug, Clone)]
pub struct BuildRunOptions {
    pub skip_config: bool,
    pub skip_defs: bool,
    pub skip_quality: bool,
    pub skip_docs: bool,
    pub skip_dotnet: bool,
    pub skip_native: bool,
    pub skip_verify: bool,
    pub cross_native: bool,
    pub configuration: String,
}

/// Runs the end-to-end repository build workflow in dependency order.
pub fn run(context: &ToolContext, options: &BuildRunOptions) -> Result<()> {
    let config = load_config(&context.repo_root)?;

    if let Some(session) = begin_workflow("Planning repository build…") {
        if !options.skip_config {
            plan_unit_step(&session, "config", "Applying configuration drift");
        }
        if !options.skip_defs {
            plan_unit_step(&session, "defs", "Syncing embedded definitions");
        }
        if !options.skip_quality {
            plan_unit_step(&session, "quality", "Running quality checks");
        }
        if !options.skip_native {
            plan_unit_step(&session, "native", "Staging host native assets");
        }
        if !options.skip_verify {
            plan_unit_step(&session, "verify", "Verifying NuGet package");
        }
        session.commit();
    }

    if !options.skip_config {
        run_planned_step(
            "config",
            "Applying configuration drift",
            "Synchronizing dhara.config.toml into manifests",
            || {
                let drifts = detect_config_drift(&context.repo_root)?;
                if !drifts.is_empty() {
                    apply_config_drift(&context.repo_root, &drifts)?;
                }
                Ok(())
            },
        )?;
    }

    if !options.skip_defs {
        run_planned_step(
            "defs",
            "Syncing embedded definitions",
            "Refreshing runtime filedefs.dat when TrID input is available",
            || run_defs_sync(context),
        )?;
    }

    if !options.skip_quality {
        run_planned_step(
            "quality",
            "Running quality checks",
            "Running fmt, clippy, doc, and tests",
            || {
                quality::run_all(
                    &context.repo_root,
                    &config,
                    options.skip_docs,
                    options.skip_dotnet,
                )
            },
        )?;
    }

    let staged_runtimes =
        staging_runtimes_on_host(&config.ci.native_runtimes, options.cross_native);

    let mut package_options = PackageOptions {
        configuration: options.configuration.clone(),
        version_override: None,
        source_override: None,
        output_dir: context.output_dir.clone(),
        execute_publish: false,
        native_stage_override: None,
        prepacked_nuget_override: None,
        include_cross_native: options.cross_native,
        expected_native_runtimes: None,
    };

    if !options.skip_native {
        run_planned_step(
            "native",
            "Staging host native assets",
            "Building dhara-sd for host runtimes",
            || {
                stage_native_for_build_run(context, &config, &package_options)?;
                Ok(())
            },
        )?;
        let artifacts_root = context.tool_root.join("artifacts");
        package_options.native_stage_override = Some(artifacts_root.join("native-stage"));
        package_options.expected_native_runtimes = Some(staged_runtimes);
    }

    if !options.skip_verify {
        run_planned_step(
            "verify",
            "Verifying NuGet package",
            "Packing and running consumer smoke checks",
            || {
                verify(
                    &context.repo_root,
                    &context.tool_root,
                    &config,
                    &package_options,
                )?;
                Ok(())
            },
        )?;
    }

    Ok(())
}

#[cfg_attr(windows, allow(unused_variables))]
fn stage_native_for_build_run(
    context: &ToolContext,
    config: &DharaRepoConfig,
    package_options: &PackageOptions,
) -> Result<()> {
    #[cfg(windows)]
    if package_options.include_cross_native {
        return stage_native_under_msvc_env(&context.repo_root, &package_options.configuration);
    }

    stage_native_for_host(
        &context.repo_root,
        &context.tool_root,
        config,
        package_options,
    )
    .map(|_| ())
}

fn run_defs_sync(context: &ToolContext) -> Result<()> {
    let paths = DefsPaths::from_context(context);
    let input = paths.default_trid_input_path();
    let output = embedded_defs_package_path(&context.repo_root);
    sync_embedded_package(&input, &output, false, |update| {
        log_build_progress(&update);
    })
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    Ok(())
}
