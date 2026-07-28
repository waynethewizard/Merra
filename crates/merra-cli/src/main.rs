//! Headless Merra command-line application.

use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{BufWriter, Write},
    num::NonZeroU32,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use clap::{Parser, Subcommand};
use merra_core::{
    BEVY_VERSION, MANIFEST_SCHEMA_V1, RUST_TOOLCHAIN_VERSION, RunManifestV1, ScenarioV1,
    SimDuration, SourceVersionV1,
};
use merra_sim::{Simulation, SimulationError, SimulationReport};
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(name = "merra", version, about = "Run reproducible Merra simulations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run a headless simulation and write deterministic reports.
    Run {
        /// RON scenario file.
        #[arg(long)]
        scenario: PathBuf,
        /// Root deterministic seed.
        #[arg(long)]
        seed: u64,
        /// Number of complete scenario years to simulate.
        #[arg(long)]
        years: NonZeroU32,
        /// New directory in which reports will be created.
        #[arg(long)]
        output: PathBuf,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Commands::Run {
            scenario,
            seed,
            years,
            output,
        } => run_simulation(&scenario, seed, years, &output),
    }
}

fn run_simulation(
    scenario_path: &Path,
    seed: u64,
    years: NonZeroU32,
    output: &Path,
) -> Result<(), CliError> {
    if output.exists() {
        return Err(CliError::OutputExists(output.to_path_buf()));
    }

    let scenario_bytes = fs::read(scenario_path)?;
    let scenario: ScenarioV1 = ron::de::from_bytes(&scenario_bytes)?;
    scenario.validate()?;
    let duration = SimDuration::from_years(years.get(), scenario.calendar.days_per_year);

    let mut simulation = Simulation::from_scenario(scenario.clone(), seed)?;
    simulation.advance(duration)?;
    simulation.finish()?;
    let report = simulation.report();
    let manifest = RunManifestV1 {
        schema_version: MANIFEST_SCHEMA_V1,
        event_schema_version: scenario.event_schema_version(),
        scenario_schema_version: scenario.schema_version,
        merra_version: env!("CARGO_PKG_VERSION").to_owned(),
        bevy_version: BEVY_VERSION.to_owned(),
        rust_version: RUST_TOOLCHAIN_VERSION.to_owned(),
        source: source_version(),
        scenario_id: scenario.id,
        scenario_hash: blake3::hash(&scenario_bytes).to_hex().to_string(),
        seed,
        years: years.get(),
        days: duration.days(),
    };

    fs::create_dir_all(output)?;
    write_json(output.join("manifest.json"), &manifest)?;
    write_events(output.join("events.jsonl"), &report)?;
    write_json(output.join("summary.json"), &report.summary)?;
    write_json(output.join("population.json"), &report.people)?;
    write_json(output.join("households.json"), &report.households)?;
    fs::write(output.join("chronicle.md"), &report.chronicle)?;

    println!(
        "completed scenario `{}` through day {} with {} events; reports: {}",
        manifest.scenario_id,
        manifest.days,
        report.events.len(),
        output.display()
    );
    Ok(())
}

fn write_json(path: PathBuf, value: &impl serde::Serialize) -> Result<(), CliError> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, value)?;
    writer.write_all(b"\n")?;
    Ok(())
}

fn write_events(path: PathBuf, report: &SimulationReport) -> Result<(), CliError> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    for event in &report.events {
        serde_json::to_writer(&mut writer, event)?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}

fn source_version() -> SourceVersionV1 {
    let git_commit = git_output(["rev-parse", "HEAD"]);
    let dirty = git_output(["status", "--porcelain"]).map(|status| !status.is_empty());

    SourceVersionV1 { git_commit, dirty }
}

fn git_output<const N: usize>(args: [&str; N]) -> Option<String> {
    let output = Command::new(OsStr::new("git")).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[derive(Debug, Error)]
enum CliError {
    #[error("output directory already exists: {0}")]
    OutputExists(PathBuf),
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid RON scenario: {0}")]
    Ron(#[from] ron::error::SpannedError),
    #[error(transparent)]
    Scenario(#[from] merra_core::ScenarioError),
    #[error(transparent)]
    Simulation(#[from] SimulationError),
    #[error("could not encode JSON report: {0}")]
    Json(#[from] serde_json::Error),
}
