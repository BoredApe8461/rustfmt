#[cfg(test)]
#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::env;
use std::fmt;
use std::io::{self, stdin, stdout, Error as IoError, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{format_err, Result};
use structopt::StructOpt;
use thiserror::Error;

use rustfmt_lib::{
    load_config, CliOptions, Config, Edition, EmitMode, FileLines, FileName,
    FormatReportFormatterBuilder, Input, Session, Verbosity,
};

fn main() {
    env_logger::init();
    let opt: Opt = Opt::from_args();

    let exit_code = match execute(opt) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{}", e.to_string());
            1
        }
    };
    // Make sure standard output is flushed before we exit.
    std::io::stdout().flush().unwrap();

    // Exit with given exit code.
    //
    // NOTE: this immediately terminates the process without doing any cleanup,
    // so make sure to finish all necessary cleanup before this is called.
    std::process::exit(exit_code);
}

/// Format Rust code
#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "rustfmt", version = include_str!(concat!(env!("OUT_DIR"),"/version-info.txt")))]
struct Opt {
    /// Run in 'check' mode.
    ///
    /// Exits with 0 if input is formatted correctly.
    /// Exits with 1 and prints a diff if formatting is required.
    #[structopt(short, long)]
    check: bool,
    /// Specify the format of rustfmt's output.
    #[cfg_attr(nightly, structopt(long, name = "files|stdout|checkstyle|json"))]
    #[cfg_attr(not(nightly), structopt(long, name = "files|stdout"))]
    emit: Option<Emit>,
    /// A path to the configuration file.
    #[structopt(long = "config-path", parse(from_os_str))]
    config_path: Option<PathBuf>,
    /// Rust compiler edition
    ///
    /// Specify which edition of the compiler to use when formatting code.
    #[structopt(long, name = "2015|2018")]
    edition: Option<Edition>,
    /// Print configuration options.
    ///
    /// `default` will print the default configuration options. `current` will print the
    /// current configuration options. `minimal` will print the minimal subset of the
    /// current configuration options that have non-default values.
    #[structopt(long = "print-config", name = "default|current|minimal")]
    print_config: Option<PrintConfig>,
    /// Prints the names of files with diff.
    #[structopt(short = "l", long = "files-with-diff")]
    files_with_diff: bool,
    /// Set options from command line.
    ///
    /// Set configuration options via command line by specifying a list of key-value pairs
    /// separated by commas (e.g., rustfmt --config=max_width=100,merge_imports=true).
    /// These settings precedes any other settings specified in configuration files.
    #[structopt(long = "config")]
    inline_config: Option<Vec<InlineConfig>>,
    /// Recursively format submodules.
    ///
    /// Format all encountered modules recursively regardless of whether the modules
    /// are defined inline or in another file.
    #[structopt(short, long)]
    recursive: bool,
    /// Print no output.
    #[structopt(short, long)]
    quiet: bool,
    /// Print verbose output.
    #[structopt(short, long)]
    verbose: bool,

    // Nightly-only options.
    /// Limit formatting to specified ranges.
    ///
    /// If you want to restrict reformatting to specific sets of lines, you can
    /// use the `--file-lines` option. Its argument is a JSON array of objects
    /// with `file` and `range` properties, where `file` is a file name, and
    /// `range` is an array representing a range of lines like `[7,13]`. Ranges
    /// are 1-based and inclusive of both end points. Specifying an empty array
    /// will result in no files being formatted. For example,
    ///
    /// ```
    /// rustfmt --file-lines '[
    ///    {{\"file\":\"src/lib.rs\",\"range\":[7,13]}},
    ///    {{\"file\":\"src/lib.rs\",\"range\":[21,29]}},
    ///    {{\"file\":\"src/foo.rs\",\"range\":[10,11]}},
    ///    {{\"file\":\"src/foo.rs\",\"range\":[15,15]}}]'
    /// ```
    ///
    /// would format lines `7-13` and `21-29` of `src/lib.rs`, and lines `10-11`,
    /// and `15` of `src/foo.rs`. No other files would be formatted, even if they
    /// are included as out of line modules from `src/lib.rs`.
    #[cfg_attr(nightly, structopt(long = "file-lines", default_value = "null"))]
    #[cfg_attr(not(nightly), structopt(skip))]
    file_lines: FileLines,

    /// Error if unable to get comments or string literals within max_width,
    /// or they are left with trailing whitespaces (unstable).
    #[cfg_attr(nightly, structopt(long = "error-on-unformatted"))]
    #[cfg_attr(not(nightly), structopt(skip))]
    error_on_unformatted: bool,

    // Positional arguments.
    #[structopt(parse(from_os_str))]
    files: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
struct InlineConfig(HashMap<String, String>, bool /* is help */);

impl InlineConfig {
    fn is_help(&self) -> bool {
        self.1
    }
}

impl FromStr for InlineConfig {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim() == "help" {
            return Ok(InlineConfig(HashMap::default(), true));
        }

        s.split(',')
            .map(
                |key_val| match key_val.char_indices().find(|(_, ch)| *ch == '=') {
                    Some((middle, _)) => {
                        let (key, val) = (&key_val[..middle], &key_val[middle + 1..]);
                        if !Config::is_valid_key_val(key, val) {
                            Err(format_err!("invalid key=val pair: `{}`", key_val))
                        } else {
                            Ok((key.to_string(), val.to_string()))
                        }
                    }

                    None => Err(format_err!(
                        "--config expects comma-separated list of key=val pairs, found `{}`",
                        key_val
                    )),
                },
            )
            .collect::<Result<HashMap<_, _>, _>>()
            .map(|map| InlineConfig(map, false))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum PrintConfig {
    Default,
    Minimal,
    Current,
}

impl FromStr for PrintConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(PrintConfig::Default),
            "minimal" => Ok(PrintConfig::Minimal),
            "current" => Ok(PrintConfig::Current),
            _ => Err(format!(
                "expected one of [current,default,minimal], found `{}`",
                s
            )),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Emit {
    Files,
    Stdout,
    Checkstyle,
    Json,
}

impl Emit {
    fn to_emit_mode(self) -> EmitMode {
        match self {
            Emit::Files => EmitMode::Files,
            Emit::Json => EmitMode::Json,
            Emit::Checkstyle => EmitMode::Checkstyle,
            Emit::Stdout => EmitMode::Stdout,
        }
    }
}

impl fmt::Display for Emit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Emit::Files => f.write_str("files"),
            Emit::Stdout => f.write_str("stdout"),
            Emit::Checkstyle => f.write_str("checkstyle"),
            Emit::Json => f.write_str("json"),
        }
    }
}

impl FromStr for Emit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "files" => Ok(Emit::Files),
            "stdout" => Ok(Emit::Stdout),
            "checkstyle" => Ok(Emit::Checkstyle),
            "json" => Ok(Emit::Json),
            _ => Err(format!("unknown --emit mode: {}", s)),
        }
    }
}

/// Rustfmt command line option errors.
#[derive(Error, Debug)]
enum OptError {
    /// Attempt to use --quiet and --verbose at once.
    #[error("--quiet and --verbose cannot be used at once.")]
    QuietAndVerbose,
    /// Attempt to use --emit and --check at once.
    #[error("--emit and --check cannot be used at once.")]
    EmitAndCheck,
    /// Attempt to use --emit with a mode which is not currently
    /// supported with standard input.
    #[error("Emit mode {0} not supported with standard output.")]
    StdinBadEmit(Emit),
}

impl Opt {
    fn canonicalize(&mut self) {
        for f in &mut self.files {
            if let Ok(canonical_path) = f.canonicalize() {
                *f = canonical_path;
            }
        }
    }

    fn verify(&self) -> Result<(), OptError> {
        if self.quiet && self.verbose {
            return Err(OptError::QuietAndVerbose);
        }

        if self.check && self.emit.is_some() {
            return Err(OptError::EmitAndCheck);
        }

        if self.files.is_empty() {
            match self.emit {
                // Emit modes which work with standard input
                // None means default, which is Stdout.
                None | Some(Emit::Stdout) | Some(Emit::Checkstyle) | Some(Emit::Json) => {}
                Some(emit_mode) => {
                    return Err(OptError::StdinBadEmit(emit_mode));
                }
            }
        }

        Ok(())
    }
}

/// Rustfmt operations errors.
#[derive(Error, Debug)]
pub enum OperationError {
    /// An unknown help topic was requested.
    #[error("Unknown help topic: `{0}`.")]
    UnknownHelpTopic(String),
    /// An unknown print-config option was requested.
    #[error("Unknown print-config option: `{0}`.")]
    UnknownPrintConfigTopic(String),
    /// Attempt to generate a minimal config from standard input.
    #[error("The `--print-config=minimal` option doesn't work with standard input.")]
    MinimalPathWithStdin,
    /// An io error during reading or writing.
    #[error("io error: {0}")]
    IoError(IoError),
}

impl From<IoError> for OperationError {
    fn from(e: IoError) -> OperationError {
        OperationError::IoError(e)
    }
}

impl CliOptions for Opt {
    fn apply_to(&self, config: &mut Config) {
        if self.verbose {
            config.set().verbose(Verbosity::Verbose);
        } else if self.quiet {
            config.set().verbose(Verbosity::Quiet);
        }
        config.set().file_lines(self.file_lines.clone());
        if self.recursive {
            config.set().recursive(true);
        }
        if self.error_on_unformatted {
            config.set().error_on_unformatted(true);
        }
        if let Some(ref edition) = self.edition {
            config.set().edition((*edition).clone());
        }
        if self.check {
            config.set().emit_mode(EmitMode::Diff);
        } else if let Some(emit) = self.emit {
            config.set().emit_mode(emit.to_emit_mode());
        }
        if self.files_with_diff {
            config.set().print_misformatted_file_names(true);
        }
        if let Some(ref inline_configs) = self.inline_config {
            for inline_config in inline_configs {
                for (k, v) in &inline_config.0 {
                    config.override_value(k, v);
                }
            }
        }
    }

    fn config_path(&self) -> Option<&Path> {
        self.config_path.as_ref().map(PathBuf::as_path)
    }
}

// Returned i32 is an exit code
fn execute(mut opt: Opt) -> Result<i32> {
    opt.verify()?;

    if opt.inline_config.as_ref().map_or(false, |inline_configs| {
        inline_configs.iter().any(InlineConfig::is_help)
    }) {
        Config::print_docs(&mut stdout(), cfg!(nightly));
        return Ok(0);
    }

    opt.canonicalize();

    match opt.print_config {
        Some(PrintConfig::Default) => print_default_config(),
        Some(PrintConfig::Minimal) => print_config(&opt, PrintConfig::Minimal),
        Some(PrintConfig::Current) => print_config(&opt, PrintConfig::Current),
        None => format(opt),
    }
}

fn print_default_config() -> Result<i32> {
    let toml = Config::default().all_options().to_toml()?;
    io::stdout().write_all(toml.as_bytes())?;
    Ok(0)
}

fn print_config(opt: &Opt, print_config: PrintConfig) -> Result<i32> {
    let (config, config_path) = load_config(
        env::current_dir().ok().as_ref().map(PathBuf::as_path),
        Some(opt),
    )?;
    let actual_config =
        FileConfigPairIter::new(&opt, config_path.is_some()).find_map(|pair| match pair.config {
            FileConfig::Local(config, Some(_)) => Some(config),
            _ => None,
        });
    let used_config = actual_config.unwrap_or(config);
    let toml = if print_config == PrintConfig::Minimal {
        used_config.used_options().to_toml()?
    } else {
        used_config.all_options().to_toml()?
    };
    io::stdout().write_all(toml.as_bytes())?;
    Ok(0)
}

fn format_string(input: String, opt: Opt) -> Result<i32> {
    // try to read config from local directory
    let (mut config, _) = load_config(Some(Path::new(".")), Some(&opt))?;

    if opt.check {
        config.set().emit_mode(EmitMode::Diff);
    } else {
        config
            .set()
            .emit_mode(opt.emit.map_or(EmitMode::Stdout, Emit::to_emit_mode));
    }
    config.set().verbose(Verbosity::Quiet);

    // parse file_lines
    config.set().file_lines(opt.file_lines);
    for f in config.file_lines().files() {
        match *f {
            FileName::Stdin => {}
            _ => eprintln!("Warning: Extra file listed in file_lines option '{}'", f),
        }
    }

    let out = &mut stdout();
    let mut session = Session::new(config, Some(out));
    format_and_emit_report(&mut session, Input::Text(input));

    let exit_code = if session.has_operational_errors() || session.has_parsing_errors() {
        1
    } else {
        0
    };
    Ok(exit_code)
}

enum FileConfig {
    Default,
    Local(Config, Option<PathBuf>),
}

struct FileConfigPair<'a> {
    file: &'a Path,
    config: FileConfig,
}

struct FileConfigPairIter<'a> {
    has_config_from_commandline: bool,
    files: std::slice::Iter<'a, PathBuf>,
    opt: &'a Opt,
}

impl<'a> FileConfigPairIter<'a> {
    fn new(opt: &'a Opt, has_config_from_commandline: bool) -> Self {
        FileConfigPairIter {
            has_config_from_commandline,
            files: opt.files.iter(),
            opt,
        }
    }
}

impl<'a> Iterator for FileConfigPairIter<'a> {
    type Item = FileConfigPair<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let file = self.files.next()?;
        let config = if self.has_config_from_commandline {
            FileConfig::Default
        } else {
            let (local_config, config_path) =
                load_config(Some(file.parent()?), Some(self.opt)).ok()?;
            FileConfig::Local(local_config, config_path)
        };

        Some(FileConfigPair { file, config })
    }
}

fn format(opt: Opt) -> Result<i32> {
    if opt.files.is_empty() {
        let mut buf = String::new();
        stdin().read_to_string(&mut buf)?;
        return format_string(buf, opt);
    }

    let (config, config_path) = load_config(None, Some(&opt))?;

    if config.verbose() == Verbosity::Verbose {
        if let Some(path) = config_path.as_ref() {
            println!("Using rustfmt config file {}", path.display());
        }
    }

    let out = &mut stdout();
    let mut session = Session::new(config, Some(out));

    for pair in FileConfigPairIter::new(&opt, config_path.is_some()) {
        let file = pair.file;

        if !file.exists() {
            eprintln!("Error: file `{}` does not exist", file.display());
            session.add_operational_error();
        } else if file.is_dir() {
            eprintln!("Error: `{}` is a directory", file.display());
            session.add_operational_error();
        } else {
            if let FileConfig::Local(local_config, config_path) = pair.config {
                if let Some(path) = config_path {
                    if local_config.verbose() == Verbosity::Verbose {
                        println!(
                            "Using rustfmt config file {} for {}",
                            path.display(),
                            file.display()
                        );
                    }
                }

                session.override_config(local_config, |sess| {
                    format_and_emit_report(sess, Input::File(file.to_path_buf()))
                });
            } else {
                format_and_emit_report(&mut session, Input::File(file.to_path_buf()));
            }
        }
    }

    let exit_code = if session.has_operational_errors()
        || session.has_parsing_errors()
        || ((session.has_diff() || session.has_check_errors()) && opt.check)
    {
        1
    } else {
        0
    };
    Ok(exit_code)
}

fn format_and_emit_report<T: Write>(session: &mut Session<'_, T>, input: Input) {
    match session.format(input) {
        Ok(report) => {
            if report.has_warnings() {
                eprintln!(
                    "{}",
                    FormatReportFormatterBuilder::new(&report)
                        .enable_colors(should_print_with_colors(session))
                        .build()
                );
            }
        }
        Err(msg) => {
            eprintln!("Error writing files: {}", msg);
            session.add_operational_error();
        }
    }
}

fn should_print_with_colors<T: Write>(session: &mut Session<'_, T>) -> bool {
    match term::stderr() {
        Some(ref t)
            if session.config.color().use_colored_tty()
                && t.supports_color()
                && t.supports_attr(term::Attr::Bold) =>
        {
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::process::{Command, Stdio};

    fn init_log() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn stdin_disable_all_formatting_test() {
        init_log();
        match option_env!("CFG_RELEASE_CHANNEL") {
            None | Some("nightly") => {}
            // These tests require nightly.
            _ => return,
        }
        let input = "fn main() { println!(\"This should not be formatted.\"); }";
        let mut child = Command::new(rustfmt())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .arg("--config-path=./tests/config/disable_all_formatting.toml")
            .spawn()
            .expect("failed to execute child");

        {
            let stdin = child.stdin.as_mut().expect("failed to get stdin");
            stdin
                .write_all(input.as_bytes())
                .expect("failed to write stdin");
        }

        let output = child.wait_with_output().expect("failed to wait on child");
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert_eq!(input, String::from_utf8(output.stdout).unwrap());
    }

    #[test]
    fn format_lines_errors_are_reported() {
        init_log();
        let long_identifier = String::from_utf8(vec![b'a'; 239]).unwrap();
        let input = Input::Text(format!("fn {}() {{}}", long_identifier));
        let mut config = Config::default();
        config.set().error_on_line_overflow(true);
        let mut session = Session::<io::Stdout>::new(config, None);
        session.format(input).unwrap();
        assert!(session.has_formatting_errors());
    }

    #[test]
    fn format_lines_errors_are_reported_with_tabs() {
        init_log();
        let long_identifier = String::from_utf8(vec![b'a'; 97]).unwrap();
        let input = Input::Text(format!("fn a() {{\n\t{}\n}}", long_identifier));
        let mut config = Config::default();
        config.set().error_on_line_overflow(true);
        config.set().hard_tabs(true);
        let mut session = Session::<io::Stdout>::new(config, None);
        session.format(input).unwrap();
        assert!(session.has_formatting_errors());
    }

    struct TempFile {
        path: PathBuf,
    }

    fn make_temp_file(file_name: &'static str) -> TempFile {
        use std::env::var;
        use std::fs::File;

        // Used in the Rust build system.
        let target_dir = var("RUSTFMT_TEST_DIR").unwrap_or_else(|_| ".".to_owned());
        let path = Path::new(&target_dir).join(file_name);

        let mut file = File::create(&path).expect("couldn't create temp file");
        let content = b"fn main() {}\n";
        file.write_all(content).expect("couldn't write temp file");
        TempFile { path }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            use std::fs::remove_file;
            remove_file(&self.path).expect("couldn't delete temp file");
        }
    }

    fn rustfmt() -> &'static Path {
        lazy_static! {
            static ref RUSTFMT_PATH: PathBuf = {
                let mut me = env::current_exe().expect("failed to get current executable");
                // Chop of the test name.
                me.pop();
                // Chop off `deps`.
                me.pop();

                // If we run `cargo test --release`, we might only have a release build.
                if cfg!(release) {
                    // `../release/`
                    me.pop();
                    me.push("release");
                }
                me.push("rustfmt");
                assert!(
                    me.is_file() || me.with_extension("exe").is_file(),
                    if cfg!(release) {
                        "no rustfmt bin, try running `cargo build --release` before testing"
                    } else {
                        "no rustfmt bin, try running `cargo build` before testing"
                    }
                );
                me
            };
        }
        &RUSTFMT_PATH
    }

    #[test]
    fn verify_check_works() {
        init_log();
        let temp_file = make_temp_file("temp_check.rs");

        Command::new(rustfmt())
            .arg("--check")
            .arg(&temp_file.path)
            .status()
            .expect("run with check option failed");
    }

    #[test]
    fn verify_check_works_with_stdin() {
        init_log();

        let mut child = Command::new(rustfmt())
            .arg("--check")
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run with check option failed");

        {
            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            stdin
                .write_all(b"fn main() {}\n")
                .expect("Failed to write to rustfmt --check");
        }
        let output = child
            .wait_with_output()
            .expect("Failed to wait on rustfmt child");
        assert!(output.status.success());
    }

    #[test]
    fn verify_check_l_works_with_stdin() {
        init_log();

        let mut child = Command::new(rustfmt())
            .arg("--check")
            .arg("-l")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run with check option failed");

        {
            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            stdin
                .write_all(b"fn main()\n{}\n")
                .expect("Failed to write to rustfmt --check");
        }
        let output = child
            .wait_with_output()
            .expect("Failed to wait on rustfmt child");
        assert!(output.status.success());
        assert_eq!(std::str::from_utf8(&output.stdout).unwrap(), "stdin\n");
    }

    #[cfg(nightly)]
    #[test]
    fn verify_error_on_unformatted() {
        init_log();

        let mut child = Command::new(rustfmt())
            .arg("--error-on-unformatted")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run with check option failed");

        {
            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            stdin
                .write_all(b"fn main()\n{}\n")
                .expect("Failed to write to rustfmt --check");
        }

        let output = child
            .wait_with_output()
            .expect("Failed to wait on rustfmt child");
        assert!(output.status.success());
    }
}
