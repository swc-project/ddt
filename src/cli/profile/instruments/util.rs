//! Copied from https://github.com/cmyr/cargo-instruments/blob/952f1bbee1adbfe302bf8c6bfa2d05e74a00f955/src/instruments.rs
//!
//! interfacing with the `instruments` command line tool

use std::{
    mem,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use anyhow::{anyhow, Context, Result};
use semver::Version;
use tracing::info;

use crate::cli::profile::util::profiler::run_profiler;

#[derive(Debug)]
pub(super) struct CmdArgs {
    pub template_name: String,

    pub time_limit: Option<usize>,

    /// Arguments to pass to the target binary
    pub args: Vec<String>,

    pub output_path: Option<PathBuf>,

    pub envs: Vec<(String, String)>,
}

/// Holds available templates.
pub struct TemplateCatalog {
    standard_templates: Vec<String>,
    custom_templates: Vec<String>,
}

/// Represents the Xcode Instrument version detected.
#[derive(Debug, Clone, Copy)]
pub enum XcodeInstruments {
    XcTrace,
    InstrumentsBinary,
}

impl XcodeInstruments {
    /// Detects which version of Xcode Instruments is installed and if it can be
    /// launched.
    pub(crate) fn detect() -> Result<XcodeInstruments> {
        let cur_version = get_macos_version()?;
        let macos_xctrace_version = Version::parse("10.15.0").unwrap();

        if cur_version >= macos_xctrace_version {
            // This is the check used by Homebrew,see
            // https://github.com/Homebrew/install/blob/a1d820fc8950312c35073700d0ea88a531bc5950/install.sh#L216
            let clt_git_filepath = Path::new("/Library/Developer/CommandLineTools/usr/bin/git");
            if clt_git_filepath.exists() {
                return Ok(XcodeInstruments::XcTrace);
            }
        } else {
            let instruments_app_filepath = Path::new("/usr/bin/instruments");
            if instruments_app_filepath.exists() {
                return Ok(XcodeInstruments::InstrumentsBinary);
            }
        }
        Err(anyhow!(
            "Xcode Instruments is not installed. Please install the Xcode Command Line Tools."
        ))
    }

    /// Return a catalog of available Instruments Templates.
    ///
    /// The custom templates only appears if you have custom templates.
    pub(crate) fn available_templates(&self) -> Result<TemplateCatalog> {
        match self {
            XcodeInstruments::XcTrace => parse_xctrace_template_list(),
            XcodeInstruments::InstrumentsBinary => parse_instruments_template_list(),
        }
    }

    /// Prepare the Xcode Instruments profiling command
    ///
    /// If the `xctrace` tool is used, the prepared command looks like
    ///
    /// ```sh
    /// xcrun xctrace record --template MyTemplate \
    ///                      --time-limit 5000ms \
    ///                      --output path/to/tracefile \
    ///                      --launch \
    ///                      --
    /// ```
    ///
    /// If the older `instruments` tool is used, the prepared command looks
    /// like
    ///
    /// ```sh
    /// instruments -t MyTemplate \
    ///             -D /path/to/tracefile \
    ///             -l 5000ms
    /// ```
    fn profiling_command(
        &self,
        template_name: &str,
        trace_file_path: &Path,
        time_limit: Option<usize>,
    ) -> Result<Command> {
        match self {
            XcodeInstruments::XcTrace => {
                let mut command = Command::new("xcrun");
                command.args(&["xctrace", "record"]);

                command.args(&["--template", template_name]);

                if let Some(limit_millis) = time_limit {
                    let limit_millis_str = format!("{}ms", limit_millis);
                    command.args(&["--time-limit", &limit_millis_str]);
                }

                command.args(&["--output", trace_file_path.to_str().unwrap()]);
                // redirect stdin & err to the user's terminal
                if let Some(tty) = get_tty()? {
                    command.args(&["--target-stdin", &tty, "--target-stdout", &tty]);
                }

                command.args(&["--launch", "--"]);
                Ok(command)
            }
            XcodeInstruments::InstrumentsBinary => {
                let mut command = Command::new("instruments");
                command.args(&["-t", template_name]);

                command.arg("-D").arg(&trace_file_path);

                if let Some(limit) = time_limit {
                    command.args(&["-l", &limit.to_string()]);
                }
                Ok(command)
            }
        }
    }
}

/// Return the macOS version.
///
/// This function parses the output of `sw_vers -productVersion` (a string like
/// '11.2.3`) and returns the corresponding semver struct `Version{major: 11,
/// minor: 2, patch: 3}`.
fn get_macos_version() -> Result<Version> {
    let Output { status, stdout, .. } = Command::new("sw_vers")
        .args(&["-productVersion"])
        .output()?;

    if !status.success() {
        return Err(anyhow!("macOS version cannot be determined"));
    }

    semver_from_utf8(&stdout)
}

/// Returns a semver given a slice of bytes
///
/// This function tries to construct a semver struct given a raw utf8 byte array
/// that may not contain a patch number, `"11.1"` is parsed as `"11.1.0"`.
fn semver_from_utf8(version: &[u8]) -> Result<Version> {
    let to_semver = |version_string: &str| {
        Version::parse(version_string).map_err(|error| {
            anyhow!(
                "cannot parse version: `{}`, because of {}",
                version_string,
                error
            )
        })
    };

    let version_string = std::str::from_utf8(version)?;
    match version_string.split('.').count() {
        1 => to_semver(&format!("{}.0.0", version_string.trim())),
        2 => to_semver(&format!("{}.0", version_string.trim())),
        3 => to_semver(version_string.trim()),
        _ => Err(anyhow!("invalid version: {}", version_string)),
    }
}

/// Parse xctrace template listing.
///
/// Xctrace prints the list on either stderr (older versions) or stdout
/// (recent). In either case, the expected output is:
///
/// ```
/// == Standard Templates ==
/// Activity Monitor
/// Allocations
/// Animation Hitches
/// App Launch
/// Core Data
/// Counters
/// Energy Log
/// File Activity
/// Game Performance
/// Leaks
/// Logging
/// Metal System Trace
/// Network
/// SceneKit
/// SwiftUI
/// System Trace
/// Time Profiler
/// Zombies
///
/// == Custom Templates ==
/// MyTemplate
/// ```
fn parse_xctrace_template_list() -> Result<TemplateCatalog> {
    let Output {
        status,
        stdout,
        stderr,
    } = Command::new("xcrun")
        .args(&["xctrace", "list", "templates"])
        .output()?;

    if !status.success() {
        return Err(anyhow!(
            "Could not list templates. Please check your Xcode Instruments installation."
        ));
    }

    // Some older versions of xctrace print results on stderr,
    // newer version print results on stdout.
    let output = if stdout.is_empty() { stderr } else { stdout };

    let templates_str = std::str::from_utf8(&output)?;
    let mut templates_iter = templates_str.lines();

    let standard_templates = templates_iter
        .by_ref()
        .skip(1)
        .map(|line| line.trim())
        .take_while(|line| !line.starts_with('=') && !line.is_empty())
        .map(|line| line.into())
        .collect::<Vec<_>>();

    if standard_templates.is_empty() {
        return Err(anyhow!(
            "No available templates. Please check your Xcode Instruments installation."
        ));
    }

    let custom_templates = templates_iter
        .map(|line| line.trim())
        .skip_while(|line| line.starts_with('=') || line.is_empty())
        .map(|line| line.into())
        .collect::<Vec<_>>();

    Ok(TemplateCatalog {
        standard_templates,
        custom_templates,
    })
}

/// Parse /usr/bin/instruments template list.
///
/// The expected output on stdout is:
///
/// ```
/// Known Templates:
/// "Activity Monitor"
/// "Allocations"
/// "Animation Hitches"
/// "App Launch"
/// "Blank"
/// "Core Data"
/// "Counters"
/// "Energy Log"
/// "File Activity"
/// "Game Performance"
/// "Leaks"
/// "Logging"
/// "Metal System Trace"
/// "Network"
/// "SceneKit"
/// "SwiftUI"
/// "System Trace"
/// "Time Profiler"
/// "Zombies"
/// "~/Library/Application Support/Instruments/Templates/MyTemplate.tracetemplate"
/// ```
fn parse_instruments_template_list() -> Result<TemplateCatalog> {
    let Output { status, stdout, .. } = Command::new("instruments")
        .args(&["-s", "templates"])
        .output()?;

    if !status.success() {
        return Err(anyhow!(
            "Could not list templates. Please check your Xcode Instruments installation."
        ));
    }

    let templates_str = std::str::from_utf8(&stdout)?;

    let standard_templates = templates_str
        .lines()
        .skip(1)
        .map(|line| line.trim().trim_matches('"'))
        .take_while(|line| !line.starts_with("~/Library/"))
        .map(|line| line.into())
        .collect::<Vec<_>>();

    if standard_templates.is_empty() {
        return Err(anyhow!(
            "No available templates. Please check your Xcode Instruments installation."
        ));
    }

    let custom_templates = templates_str
        .lines()
        .map(|line| line.trim().trim_matches('"'))
        .skip_while(|line| !line.starts_with("~/Library/"))
        .take_while(|line| !line.is_empty())
        .map(|line| Path::new(line).file_stem().unwrap().to_string_lossy())
        .map(|line| line.into())
        .collect::<Vec<_>>();

    Ok(TemplateCatalog {
        standard_templates,
        custom_templates,
    })
}

/// Render the template catalog content as a string.
///
/// The returned string is similar to
///
/// ```text
/// Xcode Instruments templates:
///
/// built-in            abbrev
/// --------------------------
/// Activity Monitor
/// Allocations         (alloc)
/// Animation Hitches
/// App Launch
/// Core Data
/// Counters
/// Energy Log
/// File Activity       (io)
/// Game Performance
/// Leaks
/// Logging
/// Metal System Trace
/// Network
/// SceneKit
/// SwiftUI
/// System Trace        (sys)
/// Time Profiler       (time)
/// Zombies
///
/// custom
/// --------------------------
/// MyTemplate
/// ```
pub fn render_template_catalog(catalog: &TemplateCatalog) -> String {
    let mut output: String = "Xcode Instruments templates:\n".into();

    let max_width = catalog
        .standard_templates
        .iter()
        .chain(catalog.custom_templates.iter())
        .map(|name| name.len())
        .max()
        .unwrap();

    // column headers
    output.push_str(&format!(
        "\n{:width$}abbrev",
        "built-in",
        width = max_width + 2
    ));
    output.push_str(&format!("\n{:-<width$}", "", width = max_width + 8));

    for name in &catalog.standard_templates {
        output.push('\n');
        if let Some(abbrv) = abbrev_name(name.trim_matches('"')) {
            output.push_str(&format!(
                "{:width$}({abbrev})",
                name,
                width = max_width + 2,
                abbrev = abbrv
            ));
        } else {
            output.push_str(name);
        }
    }

    output.push('\n');

    // column headers
    output.push_str(&format!("\n{:width$}", "custom", width = max_width + 2));
    output.push_str(&format!("\n{:-<width$}", "", width = max_width + 8));

    for name in &catalog.custom_templates {
        output.push('\n');
        output.push_str(name);
    }

    output.push('\n');

    output
}

/// Compute the tracefile output path, creating the directory structure
/// in `target/instruments` if needed.
fn prepare_trace_filepath(
    target_filepath: &Path,
    template_name: &str,
    output_path: Option<&Path>,
) -> Result<(Option<tempfile::TempDir>, PathBuf)> {
    if let Some(output_path) = output_path {
        if let Some(parent_dir) = output_path.parent() {
            std::fs::create_dir_all(parent_dir).context("failed to prepare output path")?;
        }

        return Ok((None, output_path.to_path_buf()));
    }

    let trace_dir = tempfile::TempDir::new()?;

    let trace_filename = file_name_for_trace_file(target_filepath, template_name)?;

    let trace_filepath = trace_dir.path().join(trace_filename);

    Ok((Some(trace_dir), trace_filepath))
}

pub(super) fn file_name_for_trace_file(
    target_filepath: &Path,
    template_name: &str,
) -> Result<String> {
    let target_shortname = target_filepath
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("invalid target path {:?}", target_filepath))?;
    let template_name = template_name.replace(' ', "-");
    let now = chrono::Local::now();

    Ok(format!(
        "{}_{}_{}.trace",
        target_shortname,
        template_name,
        now.format("%F_%H%M%S-%3f")
    ))
}

/// Return the complete template name, replacing abbreviation if provided.
fn resolve_template_name(template_name: &str) -> &str {
    match template_name {
        "time" => "Time Profiler",
        "alloc" => "Allocations",
        "io" => "File Activity",
        "sys" => "System Trace",
        other => other,
    }
}

/// Return the template name abbreviation if available.
fn abbrev_name(template_name: &str) -> Option<&str> {
    match template_name {
        "Time Profiler" => Some("time"),
        "Allocations" => Some("alloc"),
        "File Activity" => Some("io"),
        "System Trace" => Some("sys"),
        _ => None,
    }
}

/// Profile the target binary at `binary_filepath`, write results at
/// `trace_filepath` and returns its path.
pub(super) fn profile_target(
    target_filepath: &Path,
    xctrace_tool: &XcodeInstruments,
    cmd: &CmdArgs,
) -> Result<PathBuf> {
    // 1. Get the template name from config
    // This borrows a ref to the String in Option<String>. The value can be
    // unwrapped because in this version the template was checked earlier to
    // be a `Some(x)`.
    let template_name = resolve_template_name(&cmd.template_name);

    // 2. Compute the trace filepath and create its parent directory
    let (trace_dir, trace_file_path) =
        prepare_trace_filepath(target_filepath, template_name, cmd.output_path.as_deref())?;

    // 3. Print current activity `Profiling target/debug/tries`
    info!(
        "Profiling {} with template '{}'",
        target_filepath.display(),
        template_name
    );

    let mut command =
        xctrace_tool.profiling_command(template_name, &trace_file_path, cmd.time_limit)?;

    command.arg(&target_filepath);

    if !cmd.args.is_empty() {
        command.args(&cmd.args);
    }

    for (k, v) in &cmd.envs {
        command.env(k, v);
    }

    info!("Running {:?}", command);

    run_profiler(command)?;

    info!("Trace file written to {:?}", trace_file_path);
    // Don't delete the trace file.
    mem::forget(trace_dir);

    Ok(trace_file_path)
}

/// get the tty of th current terminal session
fn get_tty() -> Result<Option<String>> {
    let mut command = Command::new("ps");
    command.arg("otty=").arg(std::process::id().to_string());
    Ok(String::from_utf8(command.output()?.stdout)?
        .split_whitespace()
        .next()
        .map(|tty| format!("/dev/{}", tty)))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn semvers_can_be_parsed() {
        assert_eq!(
            semver_from_utf8(b"2.3.4").unwrap(),
            Version::parse("2.3.4").unwrap()
        );
        assert_eq!(
            semver_from_utf8(b"11.1").unwrap(),
            Version::parse("11.1.0").unwrap()
        );
        assert_eq!(
            semver_from_utf8(b"11").unwrap(),
            Version::parse("11.0.0").unwrap()
        );
    }
}
