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
    BEVY_VERSION, EVENT_SCHEMA_V3, HISTORY_SCHEMA_V1, HistoryConfigV1, HistoryManifestV1,
    LOCAL_HISTORY_SCHEMA_V1, LocalHistoryConfigV1, LocalHistoryManifestV1, LocalHistoryPlaybackV1,
    LocalHistoryReportV1, MANIFEST_SCHEMA_V1, RUST_TOOLCHAIN_VERSION, RegionalHistoryV1,
    RunManifestV1, ScenarioV1, SimDuration, SourceVersionV1, WORLD_GENESIS_SCHEMA_V1,
    WorldGenesisConfigV1, WorldGenesisManifestV1,
};
use merra_sim::{
    HistoricalReport, HistorySimulationError, LocalHistoryError, Simulation, SimulationError,
    SimulationReport, regional_history, run_history, run_local_history,
};
use merra_worldgen::{
    AtlasLayer, GenerationError, generate_world, generator_version, render_snapshot, render_svg,
    summarize_world, world_hash,
};
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
    /// Generate a deterministic continent, island, atlas, and place graph.
    Worldgen {
        /// RON world template.
        #[arg(long)]
        template: PathBuf,
        /// Root deterministic seed.
        #[arg(long)]
        seed: u64,
        /// New directory in which world evidence will be created.
        #[arg(long)]
        output: PathBuf,
    },
    /// Advance aggregate cultures and populations across a generated world.
    History {
        /// World-generation output directory or `world.json`.
        #[arg(long)]
        world: PathBuf,
        /// RON historical-age configuration.
        #[arg(long)]
        scenario: PathBuf,
        /// Root deterministic history seed.
        #[arg(long)]
        seed: u64,
        /// Override the scenario's complete historical years.
        #[arg(long)]
        years: Option<NonZeroU32>,
        /// New directory in which historical evidence will be created.
        #[arg(long)]
        output: PathBuf,
    },
    /// Project a completed regional history into five detailed villages.
    Villages {
        /// Aggregate-history output directory containing `world.json` and `regional-history.json`.
        #[arg(long)]
        history: PathBuf,
        /// RON detailed local-history configuration.
        #[arg(long)]
        scenario: PathBuf,
        /// Root deterministic local-history seed.
        #[arg(long)]
        seed: u64,
        /// New directory in which five-village evidence will be created.
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
        Commands::Worldgen {
            template,
            seed,
            output,
        } => run_worldgen(&template, seed, &output),
        Commands::History {
            world,
            scenario,
            seed,
            years,
            output,
        } => run_world_history(&world, &scenario, seed, years, &output),
        Commands::Villages {
            history,
            scenario,
            seed,
            output,
        } => run_villages(&history, &scenario, seed, &output),
    }
}

fn run_worldgen(template_path: &Path, seed: u64, output: &Path) -> Result<(), CliError> {
    ensure_new_output(output)?;
    let template_bytes = fs::read(template_path)?;
    let template: WorldGenesisConfigV1 = ron::de::from_bytes(&template_bytes)?;
    let world = generate_world(&template, seed)?;
    let hash = world_hash(&world)?;
    let manifest = WorldGenesisManifestV1 {
        schema_version: WORLD_GENESIS_SCHEMA_V1,
        template_id: template.id,
        template_hash: blake3::hash(&template_bytes).to_hex().to_string(),
        world_hash: hash,
        seed,
        generator_version: generator_version().to_owned(),
    };
    let summary = summarize_world(&world);

    fs::create_dir_all(output)?;
    write_json(output.join("world.json"), &world)?;
    write_json(output.join("manifest.json"), &manifest)?;
    write_json(output.join("features.json"), &world.features)?;
    write_json(output.join("places.json"), &world.places)?;
    write_json(output.join("summary.json"), &summary)?;
    write_json(output.join("passes.json"), &world.passes)?;
    fs::write(output.join("atlas.svg"), render_svg(&world))?;
    fs::write(
        output.join("atlas.txt"),
        render_snapshot(&world, AtlasLayer::Terrain, 120, 42),
    )?;
    println!(
        "generated `{}` seed {}: {} regions, {} places; atlas: {}",
        world.title,
        seed,
        summary.regions,
        summary.location_count,
        output.join("atlas.svg").display()
    );
    Ok(())
}

fn run_world_history(
    world_path: &Path,
    scenario_path: &Path,
    seed: u64,
    years: Option<NonZeroU32>,
    output: &Path,
) -> Result<(), CliError> {
    ensure_new_output(output)?;
    let resolved_world = if world_path.is_dir() {
        world_path.join("world.json")
    } else {
        world_path.to_path_buf()
    };
    let world_bytes = fs::read(&resolved_world)?;
    let world: merra_core::SurfaceWorldV1 = serde_json::from_slice(&world_bytes)?;
    let scenario_bytes = fs::read(scenario_path)?;
    let mut scenario: HistoryConfigV1 = ron::de::from_bytes(&scenario_bytes)?;
    if let Some(years) = years {
        scenario.years = years.get();
    }
    scenario.validate()?;
    let report = run_history(&world, scenario.clone(), seed)?;
    let manifest = HistoryManifestV1 {
        schema_version: HISTORY_SCHEMA_V1,
        history_id: scenario.id,
        history_hash: blake3::hash(&scenario_bytes).to_hex().to_string(),
        world_hash: blake3::hash(&world_bytes).to_hex().to_string(),
        seed,
        years: report.years,
    };

    fs::create_dir_all(output)?;
    write_json(output.join("world.json"), &world)?;
    write_json(output.join("manifest.json"), &manifest)?;
    write_history_events(output.join("events.jsonl"), &report)?;
    write_json(output.join("summary.json"), &report.summary)?;
    write_json(output.join("populations.json"), &report.populations)?;
    write_json(output.join("settlements.json"), &report.settlements)?;
    write_json(output.join("cultures.json"), &report.cultures)?;
    write_json(output.join("faiths.json"), &report.faiths)?;
    write_json(output.join("institutions.json"), &report.institutions)?;
    write_json(output.join("polities.json"), &report.polities)?;
    write_json(output.join("lore.json"), &report.lore)?;
    write_json(
        output.join("important-places.json"),
        &report.important_places,
    )?;
    write_json(output.join("starting-region.json"), &report.starting_region)?;
    write_json(
        output.join("regional-history.json"),
        &regional_history(&report),
    )?;
    fs::write(output.join("chronicle.md"), &report.chronicle)?;
    fs::write(
        output.join("history-atlas.svg"),
        render_history_svg(&world, &report),
    )?;
    fs::write(
        output.join("atlas.txt"),
        render_history_snapshot(&world, &report),
    )?;
    println!(
        "advanced `{}` through {} years with {} people, {} settlements, and {} events; evidence: {}",
        report.title,
        report.years,
        report.summary.total_population,
        report.summary.settlements,
        report.summary.event_count,
        output.display()
    );
    Ok(())
}

fn run_villages(
    history: &Path,
    scenario_path: &Path,
    seed: u64,
    output: &Path,
) -> Result<(), CliError> {
    ensure_new_output(output)?;
    if !history.is_dir() {
        return Err(CliError::HistoryDirectoryRequired(history.to_path_buf()));
    }
    let world_bytes = fs::read(history.join("world.json"))?;
    let world: merra_core::SurfaceWorldV1 = serde_json::from_slice(&world_bytes)?;
    let regional_bytes = fs::read(history.join("regional-history.json"))?;
    let regional: RegionalHistoryV1 = serde_json::from_slice(&regional_bytes)?;
    let scenario_bytes = fs::read(scenario_path)?;
    let config: LocalHistoryConfigV1 = ron::de::from_bytes(&scenario_bytes)?;
    config.validate()?;
    let report = run_local_history(&world, &regional, config.clone(), seed)?;
    let manifest = LocalHistoryManifestV1 {
        schema_version: LOCAL_HISTORY_SCHEMA_V1,
        event_schema_version: EVENT_SCHEMA_V3,
        merra_version: env!("CARGO_PKG_VERSION").to_owned(),
        bevy_version: BEVY_VERSION.to_owned(),
        rust_version: RUST_TOOLCHAIN_VERSION.to_owned(),
        source: source_version(),
        local_history_id: config.id,
        local_history_hash: blake3::hash(&scenario_bytes).to_hex().to_string(),
        world_hash: blake3::hash(&world_bytes).to_hex().to_string(),
        regional_history_hash: blake3::hash(&regional_bytes).to_hex().to_string(),
        seed,
        years: config.years,
    };

    fs::create_dir_all(output)?;
    write_json(output.join("manifest.json"), &manifest)?;
    write_json(output.join("local-history.json"), &report)?;
    write_json(
        output.join("playback.json"),
        &LocalHistoryPlaybackV1::from_report(&report),
    )?;
    write_local_events(output.join("events.jsonl"), &report)?;
    write_json(output.join("summary.json"), &report.summary)?;
    write_json(output.join("population.json"), &report.people)?;
    write_json(output.join("households.json"), &report.households)?;
    write_json(
        output.join("household-contexts.json"),
        &report.household_contexts,
    )?;
    write_json(
        output.join("residence-decisions.json"),
        &report.residence_decisions,
    )?;
    write_json(output.join("settlements.json"), &report.settlements)?;
    write_json(output.join("connections.json"), &report.connections)?;
    write_json(output.join("institutions.json"), &report.institutions)?;
    write_json(output.join("lore.json"), &report.lore)?;
    fs::write(output.join("chronicle.md"), &report.chronicle)?;
    println!(
        "projected {} aggregate people into {} detailed settlements for {} years; evidence: {}",
        report.summary.represented_population,
        report.summary.settlements,
        report.summary.elapsed_years,
        output.display()
    );
    Ok(())
}

fn run_simulation(
    scenario_path: &Path,
    seed: u64,
    years: NonZeroU32,
    output: &Path,
) -> Result<(), CliError> {
    ensure_new_output(output)?;

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

fn ensure_new_output(output: &Path) -> Result<(), CliError> {
    if output.exists() {
        return Err(CliError::OutputExists(output.to_path_buf()));
    }
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

fn write_history_events(path: PathBuf, report: &HistoricalReport) -> Result<(), CliError> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    for event in &report.events {
        serde_json::to_writer(&mut writer, event)?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}

fn write_local_events(path: PathBuf, report: &LocalHistoryReportV1) -> Result<(), CliError> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    for event in &report.events {
        serde_json::to_writer(&mut writer, event)?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}

fn render_history_svg(world: &merra_core::SurfaceWorldV1, report: &HistoricalReport) -> String {
    use std::collections::BTreeMap;

    let scale = 6_u32;
    let map_width = u32::from(world.width) * scale;
    let map_height = u32::from(world.height) * scale;
    let region_lookup: BTreeMap<_, _> = world
        .cells
        .iter()
        .map(|cell| (cell.id, cell.coordinate))
        .collect();
    let location_lookup: BTreeMap<_, _> = world
        .places
        .locations
        .iter()
        .filter_map(|location| location.region.map(|region| (location.id, region)))
        .collect();
    let population_lookup = report.populations.iter().fold(
        BTreeMap::<_, (u64, BTreeMap<merra_core::LineageId, u64>)>::new(),
        |mut totals, population| {
            let entry = totals.entry(population.location_id).or_default();
            entry.0 = entry.0.saturating_add(u64::from(population.people));
            for share in &population.lineage {
                let weighted = u64::from(population.people) * u64::from(share.parts_per_10_000);
                let lineage_total = entry.1.entry(share.id).or_default();
                *lineage_total = lineage_total.saturating_add(weighted);
            }
            totals
        },
    );
    let mut svg = render_svg(world);
    let closing = svg.rfind("</svg>").unwrap_or(svg.len());
    let overlay = {
        let mut value = String::from(
            "<g aria-label=\"Historical populations\" stroke=\"#f7ead2\" stroke-width=\"1.2\">",
        );
        for (location_id, (people, lineages)) in population_lookup {
            let Some(coordinate) = location_lookup
                .get(&location_id)
                .and_then(|region| region_lookup.get(region))
            else {
                continue;
            };
            let radius = people.ilog10().max(1) + 2;
            let color = if lineages.len() > 1 {
                "#d29a49"
            } else {
                match lineages.keys().next().map_or(0, |lineage| lineage.0) {
                    1 => "#d7d2c5",
                    2 => "#8eb06b",
                    3 => "#77a9c9",
                    _ => "#c7a4d8",
                }
            };
            value.push_str(&format!(
                "<circle cx=\"{}\" cy=\"{}\" r=\"{radius}\" fill=\"{color}\"/>",
                u32::from(coordinate.x) * scale + scale / 2,
                u32::from(coordinate.y) * scale + scale / 2
            ));
        }
        value.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" fill=\"#f4ead4\" stroke=\"none\" font-family=\"ui-monospace,monospace\" font-size=\"12\">Year {} · lineage 1 □ · lineage 2 ● · mixed ◆</text></g>",
            map_width + 28,
            map_height.saturating_sub(42),
            report.years
        ));
        value
    };
    svg.insert_str(closing, &overlay);
    svg
}

fn render_history_snapshot(
    world: &merra_core::SurfaceWorldV1,
    report: &HistoricalReport,
) -> String {
    let mut output = render_snapshot(world, AtlasLayer::Terrain, 120, 34);
    output.push_str(&format!(
        "\nHISTORY / YEAR {}\n{} people · {} settlements · {} cultures · {} faiths\n",
        report.years,
        report.summary.total_population,
        report.summary.settlements,
        report.summary.cultures,
        report.summary.faiths
    ));
    if let Some(year) = report.summary.first_contact_year {
        output.push_str(&format!(
            "First cross-homeland contact: Year {year} · {} mixed population(s)\n",
            report.summary.mixed_lineage_populations
        ));
    } else {
        output.push_str("The route remained closed; the founding histories stayed separate.\n");
    }
    output.push_str(&format!("{}\n", report.starting_region.summary));
    output
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
    #[error("aggregate history input must be a directory: {0}")]
    HistoryDirectoryRequired(PathBuf),
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid RON scenario: {0}")]
    Ron(#[from] ron::error::SpannedError),
    #[error(transparent)]
    Scenario(#[from] merra_core::ScenarioError),
    #[error(transparent)]
    Simulation(#[from] SimulationError),
    #[error(transparent)]
    Worldgen(#[from] GenerationError),
    #[error(transparent)]
    HistoryConfig(#[from] merra_core::HistoryError),
    #[error(transparent)]
    History(#[from] HistorySimulationError),
    #[error(transparent)]
    LocalHistoryConfig(#[from] merra_core::LocalHistoryConfigError),
    #[error(transparent)]
    LocalHistory(#[from] LocalHistoryError),
    #[error("could not encode JSON report: {0}")]
    Json(#[from] serde_json::Error),
}
