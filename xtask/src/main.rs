//! Cross-platform repository automation.

use std::{
    ffi::OsString,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use clap::{Parser, Subcommand};
use thiserror::Error;

const REQUIRED_CYCLE_HEADINGS: &[&str] = &[
    "## Question",
    "## Intended Evidence",
    "## Decisions and Rationale",
    "## Implementation Notes",
    "## Experiments and Results",
    "## Failures and Surprises",
    "## Tests and Invariants",
    "## Reproduction",
    "## Known Limitations and Debt",
    "## Newsletter Candidates",
    "## Next Questions",
];

#[derive(Debug, Parser)]
#[command(about = "Maintain and verify the Merra repository")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run all local checks required before publication.
    Preflight,
    /// Validate documentation structure and active cycle records.
    VerifyDocs,
    /// Create a cycle record from the standard template.
    NewCycle {
        /// Zero-padded or numeric era number.
        #[arg(long)]
        era: u16,
        /// Zero-padded or numeric cycle number.
        #[arg(long)]
        cycle: u16,
        /// Filesystem-safe short name.
        #[arg(long)]
        slug: String,
        /// Human-readable title.
        #[arg(long)]
        title: String,
    },
}

fn main() -> ExitCode {
    match execute(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn execute(cli: Cli) -> Result<(), XtaskError> {
    match cli.command {
        Commands::Preflight => preflight(),
        Commands::VerifyDocs => verify_docs(),
        Commands::NewCycle {
            era,
            cycle,
            slug,
            title,
        } => new_cycle(era, cycle, &slug, &title),
    }
}

fn preflight() -> Result<(), XtaskError> {
    verify_docs()?;
    run("cargo", &["fmt", "--all", "--check"])?;
    run(
        "cargo",
        &["check", "--workspace", "--all-targets", "--all-features"],
    )?;
    run("cargo", &["test", "--workspace", "--all-targets"])?;
    run(
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    run_with_env(
        "cargo",
        &["doc", "--workspace", "--no-deps"],
        "RUSTDOCFLAGS",
        "-D warnings",
    )?;
    run("cargo", &["deny", "check"])?;
    run(
        "gitleaks",
        &[
            "git",
            ".",
            "--staged",
            "--no-banner",
            "--redact",
            "--verbose",
        ],
    )?;
    run(
        "gitleaks",
        &["git", ".", "--no-banner", "--redact", "--verbose"],
    )?;
    println!("preflight passed");
    Ok(())
}

fn verify_docs() -> Result<(), XtaskError> {
    let required_paths = [
        "CHANGELOG.md",
        "docs/design-principles.md",
        "docs/roadmap.md",
        "docs/architecture/overview.md",
        "docs/architecture/determinism-and-replay.md",
        "docs/architecture/event-model.md",
        "docs/devlog/TEMPLATE.md",
        "docs/newsletter/TEMPLATE.md",
    ];
    for path in required_paths {
        if !Path::new(path).is_file() {
            return Err(XtaskError::MissingDocument(PathBuf::from(path)));
        }
    }

    let cycle_root = Path::new("docs/devlog");
    for entry in walk_markdown(cycle_root)? {
        if !entry
            .components()
            .any(|component| component.as_os_str() == "cycles")
        {
            continue;
        }
        let contents = fs::read_to_string(&entry)?;
        for heading in REQUIRED_CYCLE_HEADINGS {
            if !contents.contains(heading) {
                return Err(XtaskError::MissingHeading {
                    path: entry.clone(),
                    heading: heading.to_string(),
                });
            }
        }
    }
    println!("documentation structure is valid");
    Ok(())
}

fn walk_markdown(root: &Path) -> Result<Vec<PathBuf>, XtaskError> {
    let mut paths = Vec::new();
    if !root.exists() {
        return Ok(paths);
    }
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            paths.extend(walk_markdown(&path)?);
        } else if path.extension().is_some_and(|extension| extension == "md") {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn new_cycle(era: u16, cycle: u16, slug: &str, title: &str) -> Result<(), XtaskError> {
    if !valid_slug(slug) {
        return Err(XtaskError::InvalidSlug(slug.to_owned()));
    }

    let directory = PathBuf::from(format!("docs/devlog/era-{era:02}/cycles"));
    fs::create_dir_all(&directory)?;
    let path = directory.join(format!("{cycle:02}-{slug}.md"));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                XtaskError::CycleExists(path.clone())
            } else {
                XtaskError::Io(error)
            }
        })?;
    let template = fs::read_to_string("docs/devlog/TEMPLATE.md")?;
    let contents = template
        .replace("ERA_NUMBER", &era.to_string())
        .replace("CYCLE_NUMBER", &cycle.to_string())
        .replace("CYCLE_SLUG", slug)
        .replace("CYCLE_TITLE", title);
    file.write_all(contents.as_bytes())?;
    println!("created {}", path.display());
    Ok(())
}

fn valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn run(program: &str, args: &[&str]) -> Result<(), XtaskError> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|source| XtaskError::ToolUnavailable {
            program: program.to_owned(),
            source,
        })?;
    if !status.success() {
        return Err(XtaskError::CommandFailed {
            command: format_command(program, args),
        });
    }
    Ok(())
}

fn run_with_env(program: &str, args: &[&str], key: &str, value: &str) -> Result<(), XtaskError> {
    let status = Command::new(program)
        .args(args)
        .env(key, value)
        .status()
        .map_err(|source| XtaskError::ToolUnavailable {
            program: program.to_owned(),
            source,
        })?;
    if !status.success() {
        return Err(XtaskError::CommandFailed {
            command: format_command(program, args),
        });
    }
    Ok(())
}

fn format_command(program: &str, args: &[&str]) -> String {
    let mut command = OsString::from(program);
    for argument in args {
        command.push(" ");
        command.push(argument);
    }
    command.to_string_lossy().into_owned()
}

#[derive(Debug, Error)]
enum XtaskError {
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),
    #[error("required document is missing: {0}")]
    MissingDocument(PathBuf),
    #[error("{path} is missing required heading `{heading}`")]
    MissingHeading { path: PathBuf, heading: String },
    #[error("cycle slug must contain only lowercase ASCII letters, digits, and hyphens: {0}")]
    InvalidSlug(String),
    #[error("cycle record already exists: {0}")]
    CycleExists(PathBuf),
    #[error("required tool `{program}` is unavailable: {source}")]
    ToolUnavailable {
        program: String,
        source: std::io::Error,
    },
    #[error("command failed: {command}")]
    CommandFailed { command: String },
}
