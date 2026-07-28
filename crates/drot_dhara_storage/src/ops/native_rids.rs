use anyhow::{Result, bail};

/// Sidecar daemon file name placed under `runtimes/{rid}/native/`.
///
/// NuGet packs `dhara-sd` (not the FFI cdylib). Windows uses `.exe`; other RIDs use the
/// extensionless binary name until UDS transports ship.
pub fn native_lib_filename(rid: &str) -> Result<&'static str> {
    match rid {
        "win-x64" | "win-arm64" => Ok("dhara-sd.exe"),
        "linux-x64" | "linux-arm64" | "osx-arm64" => Ok("dhara-sd"),
        _ => bail!("unsupported runtime identifier for daemon sidecar name: {rid}"),
    }
}

/// Package-relative path for a daemon sidecar entry inside a `.nupkg`.
pub fn package_native_path(rid: &str) -> Result<String> {
    Ok(format!(
        "runtimes/{rid}/native/{}",
        native_lib_filename(rid)?
    ))
}

/// RIDs that can be built on the current host OS and CPU without cross-compilation.
pub fn buildable_runtimes_on_host(all_runtimes: &[String]) -> Vec<String> {
    let host_os = std::env::consts::OS;
    let host_arch = std::env::consts::ARCH;
    all_runtimes
        .iter()
        .filter(|rid| is_rid_buildable_on_host(rid, host_os, host_arch))
        .cloned()
        .collect()
}

fn is_rid_buildable_on_host(rid: &str, host_os: &str, host_arch: &str) -> bool {
    match host_os {
        "windows" => match host_arch {
            // MSVC can cross-compile ARM64 from an x64 host.
            "x86_64" => matches!(rid, "win-x64" | "win-arm64"),
            "aarch64" => rid == "win-arm64",
            _ => false,
        },
        "linux" => match host_arch {
            "x86_64" => rid == "linux-x64",
            "aarch64" => rid == "linux-arm64",
            _ => false,
        },
        "macos" => rid == "osx-arm64",
        _ => false,
    }
}

/// MSBuild `Platform` value when required for a RID.
pub fn platform(runtime: &str) -> Result<Option<&'static str>> {
    match runtime {
        "win-x64" => Ok(Some("x64")),
        "win-arm64" => Ok(Some("ARM64")),
        "win-x86" => Ok(Some("x86")),
        "linux-x64" | "linux-arm64" | "osx-arm64" => Ok(None),
        _ => bail!("unsupported runtime for Platform inference: {runtime}"),
    }
}

/// MSBuild `PlatformTarget` value when required for a RID.
pub fn platform_target(runtime: &str) -> Result<Option<&'static str>> {
    match runtime {
        "win-x64" => Ok(Some("x64")),
        "win-arm64" => Ok(Some("arm64")),
        "win-x86" => Ok(Some("x86")),
        "linux-x64" | "linux-arm64" | "osx-arm64" => Ok(None),
        _ => bail!("unsupported runtime for PlatformTarget inference: {runtime}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_native_paths_use_daemon_sidecar_names() {
        assert_eq!(
            package_native_path("win-x64").unwrap(),
            "runtimes/win-x64/native/dhara-sd.exe"
        );
        assert_eq!(
            package_native_path("linux-arm64").unwrap(),
            "runtimes/linux-arm64/native/dhara-sd"
        );
        assert_eq!(
            package_native_path("osx-arm64").unwrap(),
            "runtimes/osx-arm64/native/dhara-sd"
        );
    }

    #[test]
    fn buildable_runtimes_respect_host_arch_on_linux() {
        assert!(is_rid_buildable_on_host("linux-x64", "linux", "x86_64"));
        assert!(!is_rid_buildable_on_host("linux-arm64", "linux", "x86_64"));
        assert!(is_rid_buildable_on_host("linux-arm64", "linux", "aarch64"));
        assert!(!is_rid_buildable_on_host("linux-x64", "linux", "aarch64"));
        assert!(!is_rid_buildable_on_host("win-x64", "linux", "x86_64"));
    }
}
