#[macro_use]
extern crate log;

use std::io::stdout;
use std::path::{Path, PathBuf};
use std::process::Command;

use structopt::StructOpt;

use rustfmt_lib::{
    emitter::{emit_format_report, EmitterConfig},
    format, load_config, CliOptions, FormatReportFormatterBuilder, Input, OperationSetting,
};

fn prune_files(files: Vec<&str>) -> Vec<&str> {
    let prefixes: Vec<_> = files
        .iter()
        .filter(|f| f.ends_with("mod.rs") || f.ends_with("lib.rs"))
        .map(|f| &f[..f.len() - 6])
        .collect();

    let mut pruned_prefixes = vec![];
    for p1 in prefixes {
        if p1.starts_with("src/bin/") || pruned_prefixes.iter().all(|p2| !p1.starts_with(p2)) {
            pruned_prefixes.push(p1);
        }
    }
    debug!("prefixes: {:?}", pruned_prefixes);

    files
        .into_iter()
        .filter(|f| {
            if f.ends_with("mod.rs") || f.ends_with("lib.rs") || f.starts_with("src/bin/") {
                return true;
            }
            pruned_prefixes.iter().all(|pp| !f.starts_with(pp))
        })
        .collect()
}

fn git_diff(commits: u64) -> String {
    let mut cmd = Command::new("git");
    cmd.arg("diff");
    if commits != 0 {
        cmd.arg(format!("HEAD~{}", commits));
    }
    let output = cmd.output().expect("Couldn't execute `git diff`");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn get_files(input: &str) -> Vec<&str> {
    input
        .lines()
        .filter(|line| line.starts_with("+++ b/") && line.ends_with(".rs"))
        .map(|line| &line[6..])
        .collect()
}

fn fmt_files(files: &[&str]) -> i32 {
    let (config, _) =
        load_config::<NullOptions>(Some(Path::new(".")), None).expect("couldn't load config");
    let setting = OperationSetting::default();

    let mut out = stdout();
    for file in files {
        let report = format(Input::File(PathBuf::from(file)), &config, setting).unwrap();
        if report.has_warnings() {
            eprintln!("{}", FormatReportFormatterBuilder::new(&report).build());
        }
        emit_format_report(report, &mut out, EmitterConfig::default()).unwrap();
    }

    todo!("Fix error handling")
}

struct NullOptions;

impl CliOptions for NullOptions {
    fn apply_to(&self, _: &mut rustfmt_lib::Config) {
        unreachable!();
    }
    fn config_path(&self) -> Option<&Path> {
        unreachable!();
    }
}

fn uncommitted_files() -> Vec<String> {
    let mut cmd = Command::new("git");
    cmd.arg("ls-files");
    cmd.arg("--others");
    cmd.arg("--modified");
    cmd.arg("--exclude-standard");
    let output = cmd.output().expect("Couldn't execute Git");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter(|s| s.ends_with(".rs"))
        .map(std::borrow::ToOwned::to_owned)
        .collect()
}

fn check_uncommitted() {
    let uncommitted = uncommitted_files();
    debug!("uncommitted files: {:?}", uncommitted);
    if !uncommitted.is_empty() {
        println!("Found untracked changes:");
        for f in &uncommitted {
            println!("  {}", f);
        }
        println!("Commit your work, or run with `-u`.");
        println!("Exiting.");
        std::process::exit(1);
    }
}

#[derive(Debug, StructOpt)]
struct Opt {
    /// Check only.
    #[structopt(short, long)]
    check: bool,
    /// Format uncommitted files.
    #[structopt(short, long)]
    uncommitted: bool,
    #[structopt(short, long)]
    commits: u64,
}

fn main() {
    env_logger::init();

    let opt: Opt = Opt::from_args();

    if !opt.uncommitted {
        check_uncommitted();
    }

    let stdout = git_diff(opt.commits);
    let files = get_files(&stdout);
    debug!("files: {:?}", files);
    let files = prune_files(files);
    debug!("pruned files: {:?}", files);
    let exit_code = fmt_files(&files);
    std::process::exit(exit_code);
}
