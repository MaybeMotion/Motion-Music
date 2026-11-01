//! Configure libprojectM with its bundled evaluator so static MSVC builds use
//! the same release C runtime throughout. This wrapper changes build options,
//! never dependency source. Arguments go straight to Command, without a shell.

use std::ffi::OsString;
use std::process::{Command, ExitCode};

fn forwarded_args(mut args: Vec<OsString>) -> Vec<OsString> {
    let configure = !args.iter().filter_map(|arg| arg.to_str()).any(|arg| {
        matches!(
            arg,
            "--build"
                | "--install"
                | "--version"
                | "--open"
                | "--workflow"
                | "--find-package"
                | "--system-information"
                | "-E"
                | "-N"
        ) || arg.starts_with("--help")
            || arg.starts_with("--list-presets")
            || arg.starts_with("-P")
    });
    if configure {
        args.push("-DENABLE_SYSTEM_PROJECTM_EVAL=OFF".into());
    }
    args
}

fn run() -> Result<ExitCode, String> {
    let cmake = std::env::var_os("MOTION_MUSIC_CMAKE")
        .ok_or("MOTION_MUSIC_CMAKE must point to the real cmake executable.")?;
    let cmake = std::path::PathBuf::from(cmake);
    let actual = cmake
        .canonicalize()
        .map_err(|error| format!("CMake path: {error}"))?;
    let wrapper = std::env::current_exe().map_err(|error| error.to_string())?;
    if actual == wrapper.canonicalize().map_err(|error| error.to_string())? {
        return Err("MOTION_MUSIC_CMAKE points to this wrapper instead of real CMake.".into());
    }
    let status = Command::new(actual)
        .args(forwarded_args(std::env::args_os().skip(1).collect()))
        .status()
        .map_err(|error| format!("Could not run CMake: {error}"))?;
    Ok(ExitCode::from(
        status.code().unwrap_or(1).try_into().unwrap_or(1),
    ))
}

fn main() -> ExitCode {
    match run() {
        Ok(status) => status,
        Err(error) => {
            eprintln!("Motion Music MilkDrop build: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configure_preserves_every_argument_and_adds_the_upstream_option() {
        let args: Vec<OsString> = [
            r"C:\source with spaces\libprojectM",
            "-B",
            r"C:\build with spaces",
            "-DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded$<$<CONFIG:Debug>:Debug>",
            "-DQUOTED_VALUE=a;b<test>&value",
        ]
        .map(Into::into)
        .into();
        let forwarded = forwarded_args(args.clone());
        assert_eq!(&forwarded[..args.len()], &args);
        assert_eq!(
            forwarded.last().unwrap(),
            "-DENABLE_SYSTEM_PROJECTM_EVAL=OFF"
        );
    }

    #[test]
    fn build_install_version_and_other_command_modes_are_unchanged() {
        for mode in [
            "--build",
            "--install",
            "--version",
            "--open",
            "--workflow",
            "--find-package",
            "--system-information",
            "--help",
            "--help-command",
            "--list-presets=all",
            "-E",
            "-N",
            "-Pfile.cmake",
        ] {
            let args: Vec<OsString> = [mode, "a path with spaces"].map(Into::into).into();
            assert_eq!(forwarded_args(args.clone()), args, "{mode}");
        }
    }
}
