//! Cross-platform repository automation.

use std::{
    ffi::OsString,
    fs::{self, OpenOptions},
    io::Write,
    num::NonZeroU32,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use clap::{Parser, Subcommand};
use merra_core::ScenarioV1;
use merra_sim::run_years;
use serde::Serialize;
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
    /// Run a deterministic seed cohort and write aggregate evidence.
    SeedLab {
        /// RON scenario evaluated for every seed.
        #[arg(long, default_value = "scenarios/era-01/century.ron")]
        scenario: PathBuf,
        /// First inclusive root seed.
        #[arg(long, default_value_t = 1)]
        first_seed: u64,
        /// Number of consecutive seeds.
        #[arg(long, default_value = "100")]
        count: NonZeroU32,
        /// Complete scenario years per run.
        #[arg(long, default_value = "100")]
        years: NonZeroU32,
        /// New directory for Markdown, JSON, and CSV evidence.
        #[arg(long, default_value = "runs/seed-lab")]
        output: PathBuf,
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
        Commands::SeedLab {
            scenario,
            first_seed,
            count,
            years,
            output,
        } => seed_lab(&scenario, first_seed, count, years, &output),
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
        "docs/architecture/automation-and-evidence.md",
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

#[derive(Debug, Serialize)]
struct SeedLabRun {
    seed: u64,
    living_population: u32,
    deaths: u32,
    event_count: usize,
    last_death_year: Option<u64>,
    mean_age_at_death_times_100: u64,
    maximum_age_at_death: u64,
}

#[derive(Debug, Serialize)]
struct SeedLabReport {
    schema_version: u32,
    scenario_id: String,
    first_seed: u64,
    count: u32,
    years: u32,
    minimum_living: u32,
    maximum_living: u32,
    mean_living: f64,
    extinct_runs: usize,
    earliest_extinction_year: Option<u64>,
    latest_extinction_year: Option<u64>,
    minimum_mean_age_at_death_times_100: u64,
    maximum_mean_age_at_death_times_100: u64,
    runs: Vec<SeedLabRun>,
}

fn seed_lab(
    scenario_path: &Path,
    first_seed: u64,
    count: NonZeroU32,
    years: NonZeroU32,
    output: &Path,
) -> Result<(), XtaskError> {
    if output.exists() {
        return Err(XtaskError::OutputExists(output.to_path_buf()));
    }
    let scenario_bytes = fs::read(scenario_path)?;
    let scenario: ScenarioV1 = ron::de::from_bytes(&scenario_bytes)?;
    scenario.validate()?;

    let mut runs = Vec::with_capacity(count.get() as usize);
    for offset in 0..u64::from(count.get()) {
        let seed = first_seed.saturating_add(offset);
        let simulation = run_years(scenario.clone(), seed, years.get())?;
        let dead: Vec<_> = simulation
            .people
            .iter()
            .filter(|person| !person.alive)
            .collect();
        let total_age: u64 = dead.iter().map(|person| person.final_age_years).sum();
        let mean_age_at_death_times_100 = if dead.is_empty() {
            0
        } else {
            total_age.saturating_mul(100) / dead.len() as u64
        };
        runs.push(SeedLabRun {
            seed,
            living_population: simulation.summary.living_population,
            deaths: simulation.summary.deaths,
            event_count: simulation.summary.event_count,
            last_death_year: dead
                .iter()
                .filter_map(|person| person.death_day)
                .max()
                .map(|day| day / u64::from(simulation.summary.days_per_year)),
            mean_age_at_death_times_100,
            maximum_age_at_death: dead
                .iter()
                .map(|person| person.final_age_years)
                .max()
                .unwrap_or(0),
        });
    }

    let minimum = runs
        .iter()
        .min_by_key(|run| (run.living_population, run.seed))
        .ok_or(XtaskError::EmptySeedLab)?;
    let maximum = runs
        .iter()
        .max_by_key(|run| (run.living_population, std::cmp::Reverse(run.seed)))
        .ok_or(XtaskError::EmptySeedLab)?;
    let total_living: u64 = runs
        .iter()
        .map(|run| u64::from(run.living_population))
        .sum();
    let mean_living = total_living as f64 / f64::from(count.get());
    let extinct_runs = runs.iter().filter(|run| run.living_population == 0).count();
    let minimum_living = minimum.living_population;
    let minimum_seed = minimum.seed;
    let maximum_living = maximum.living_population;
    let maximum_seed = maximum.seed;
    let earliest_extinction_year = runs
        .iter()
        .filter(|run| run.living_population == 0)
        .filter_map(|run| run.last_death_year)
        .min();
    let latest_extinction_year = runs
        .iter()
        .filter(|run| run.living_population == 0)
        .filter_map(|run| run.last_death_year)
        .max();
    let minimum_mean_age_at_death_times_100 = runs
        .iter()
        .map(|run| run.mean_age_at_death_times_100)
        .min()
        .ok_or(XtaskError::EmptySeedLab)?;
    let maximum_mean_age_at_death_times_100 = runs
        .iter()
        .map(|run| run.mean_age_at_death_times_100)
        .max()
        .ok_or(XtaskError::EmptySeedLab)?;
    let report = SeedLabReport {
        schema_version: 1,
        scenario_id: scenario.id,
        first_seed,
        count: count.get(),
        years: years.get(),
        minimum_living,
        maximum_living,
        mean_living,
        extinct_runs,
        earliest_extinction_year,
        latest_extinction_year,
        minimum_mean_age_at_death_times_100,
        maximum_mean_age_at_death_times_100,
        runs,
    };

    fs::create_dir_all(output)?;
    fs::write(
        output.join("summary.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    fs::write(output.join("results.csv"), seed_lab_csv(&report))?;
    fs::write(
        output.join("summary.md"),
        format!(
            "# Merra Seed Laboratory\n\n\
             ## Population Result\n\n\
             - Scenario: `{}`\n\
             - Seeds: {} through {}\n\
             - Duration: {} years each\n\
             - Living population range: {} (seed {}) to {} (seed {})\n\
             - Mean living population: {:.2}\n\
             - Extinct runs: {} of {}\n\n\
             ## Lifespan Variation\n\n\
             - Extinction year range: {} to {}\n\
             - Mean age-at-death range: {:.2} to {:.2}\n\n\
             These results are deterministic for the tagged source, lockfile, scenario, and seed cohort.\n",
            report.scenario_id,
            report.first_seed,
            report
                .first_seed
                .saturating_add(u64::from(report.count).saturating_sub(1)),
            report.years,
            minimum_living,
            minimum_seed,
            maximum_living,
            maximum_seed,
            report.mean_living,
            report.extinct_runs,
            report.count,
            report
                .earliest_extinction_year
                .map_or_else(|| String::from("n/a"), |year| year.to_string()),
            report
                .latest_extinction_year
                .map_or_else(|| String::from("n/a"), |year| year.to_string()),
            report.minimum_mean_age_at_death_times_100 as f64 / 100.0,
            report.maximum_mean_age_at_death_times_100 as f64 / 100.0,
        ),
    )?;
    println!("seed laboratory evidence: {}", output.display());
    Ok(())
}

fn seed_lab_csv(report: &SeedLabReport) -> String {
    let mut csv = String::from(
        "seed,living_population,deaths,event_count,last_death_year,mean_age_at_death_times_100,maximum_age_at_death\n",
    );
    for run in &report.runs {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            run.seed,
            run.living_population,
            run.deaths,
            run.event_count,
            run.last_death_year
                .map_or_else(String::new, |year| year.to_string()),
            run.mean_age_at_death_times_100,
            run.maximum_age_at_death,
        ));
    }
    csv
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
    #[error("output directory already exists: {0}")]
    OutputExists(PathBuf),
    #[error("seed laboratory unexpectedly had no runs")]
    EmptySeedLab,
    #[error("invalid RON scenario: {0}")]
    Ron(#[from] ron::error::SpannedError),
    #[error(transparent)]
    Scenario(#[from] merra_core::ScenarioError),
    #[error(transparent)]
    Simulation(#[from] merra_sim::SimulationError),
    #[error("could not encode seed laboratory JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("required tool `{program}` is unavailable: {source}")]
    ToolUnavailable {
        program: String,
        source: std::io::Error,
    },
    #[error("command failed: {command}")]
    CommandFailed { command: String },
}
