use anyhow::Result;

use drot_kernel::{CommandResult, ToolContext};

use crate::ops::build::{BuildRunOptions, run};

use super::{BuildRunArgs, parse_args};

pub(crate) fn build_run_command(context: &ToolContext, args: &[String]) -> Result<CommandResult> {
    let Some(args) = parse_args::<BuildRunArgs>("build run", args)? else {
        return Ok(CommandResult::success());
    };

    run(
        context,
        &BuildRunOptions {
            skip_config: args.skip_config,
            skip_defs: args.skip_defs,
            skip_quality: args.skip_quality,
            skip_docs: args.skip_docs,
            skip_dotnet: args.skip_dotnet,
            skip_native: args.skip_native,
            skip_verify: args.skip_verify,
            cross_native: args.cross_native,
            configuration: args.configuration,
        },
    )?;

    Ok(CommandResult::with_message(
        "Repository build workflow completed.",
    ))
}
