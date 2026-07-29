//! Interactive terminal entry point.

use std::{
    fs,
    io::{self, IsTerminal, stdout},
    num::NonZeroU32,
    path::PathBuf,
    process::ExitCode,
    time::Duration,
};

use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use merra_core::{EventId, HistorySummaryV1, HouseholdId, PersonId, ScenarioV1, SurfaceWorldV1};
use merra_sim::run_years;
use merra_tui::{Focus, Inspector, View, render, render_snapshot};
use merra_worldgen::{AtlasLayer, render_snapshot as render_world_snapshot};
use ratatui::{Terminal, backend::CrosstermBackend};
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(
    name = "merra-tui",
    version,
    about = "Inspect a deterministic Merra history"
)]
struct Args {
    /// Inspect a generated world rather than a detailed local history.
    #[command(subcommand)]
    command: Option<InspectorCommand>,
    /// RON scenario to simulate before opening the inspector.
    #[arg(long, default_value = "scenarios/era-01/dynasty.ron")]
    scenario: PathBuf,
    /// Root deterministic seed.
    #[arg(long, default_value_t = 42)]
    seed: u64,
    /// Number of complete scenario years.
    #[arg(long, default_value = "60")]
    years: NonZeroU32,
    /// Print an ANSI-free screen instead of entering interactive mode.
    #[arg(long)]
    snapshot: bool,
    /// Snapshot width in terminal cells.
    #[arg(long, default_value_t = 120)]
    width: u16,
    /// Snapshot height in terminal cells.
    #[arg(long, default_value_t = 36)]
    height: u16,
    /// Initial collection displayed by interactive and snapshot modes.
    #[arg(long, value_enum, default_value_t = InitialView::Overview)]
    view: InitialView,
    /// Focus a stable person identity in snapshots or interactive mode.
    #[arg(
        long,
        value_name = "ID",
        conflicts_with_all = ["focus_household", "focus_event"]
    )]
    focus_person: Option<u64>,
    /// Focus a stable household identity in snapshots or interactive mode.
    #[arg(
        long,
        value_name = "ID",
        conflicts_with_all = ["focus_person", "focus_event"]
    )]
    focus_household: Option<u64>,
    /// Focus a stable event identity in snapshots or interactive mode.
    #[arg(
        long,
        value_name = "ID",
        conflicts_with_all = ["focus_person", "focus_household"]
    )]
    focus_event: Option<u64>,
}

#[derive(Debug, Subcommand)]
enum InspectorCommand {
    /// Inspect continent generation and optional aggregate history.
    World(WorldArgs),
}

#[derive(Debug, ClapArgs)]
struct WorldArgs {
    /// World-generation or history output directory, or a `world.json` file.
    #[arg(long)]
    input: PathBuf,
    /// Print an ANSI-free screen instead of entering interactive mode.
    #[arg(long)]
    snapshot: bool,
    /// Snapshot width in terminal cells.
    #[arg(long, default_value_t = 120)]
    width: u16,
    /// Snapshot height in terminal cells.
    #[arg(long, default_value_t = 42)]
    height: u16,
    /// Initial generated-world layer.
    #[arg(long, value_enum, default_value_t = InitialWorldLayer::Terrain)]
    layer: InitialWorldLayer,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum InitialWorldLayer {
    Terrain,
    Biome,
    Habitability,
    Resources,
    Mythic,
}

impl From<InitialWorldLayer> for AtlasLayer {
    fn from(value: InitialWorldLayer) -> Self {
        match value {
            InitialWorldLayer::Terrain => Self::Terrain,
            InitialWorldLayer::Biome => Self::Biome,
            InitialWorldLayer::Habitability => Self::Habitability,
            InitialWorldLayer::Resources => Self::Resources,
            InitialWorldLayer::Mythic => Self::Mythic,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum InitialView {
    Overview,
    #[value(alias = "events")]
    History,
    People,
    #[value(alias = "genealogy")]
    Lineage,
    Households,
}

impl From<InitialView> for View {
    fn from(value: InitialView) -> Self {
        match value {
            InitialView::Overview => Self::Overview,
            InitialView::History => Self::History,
            InitialView::People => Self::People,
            InitialView::Lineage => Self::Lineage,
            InitialView::Households => Self::Households,
        }
    }
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> Result<(), TuiError> {
    if let Some(InspectorCommand::World(world)) = args.command {
        return run_world(world);
    }
    let scenario_bytes = fs::read(&args.scenario)?;
    let scenario: ScenarioV1 = ron::de::from_bytes(&scenario_bytes)?;
    scenario.validate()?;
    let report = run_years(scenario, args.seed, args.years.get())?;
    let initial_view = View::from(args.view);
    let focus = args
        .focus_person
        .map(|id| Focus::Person(PersonId(id)))
        .or_else(|| {
            args.focus_household
                .map(|id| Focus::Household(HouseholdId(id)))
        })
        .or_else(|| args.focus_event.map(|id| Focus::Event(EventId(id))));

    let mut inspector = Inspector::new(report);
    inspector.set_view(initial_view);
    if let Some(focus) = focus
        && !inspector.focus(focus)
    {
        return Err(TuiError::FocusNotFound(focus));
    }
    if args.snapshot {
        print!("{}", render_snapshot(&inspector, args.width, args.height));
        return Ok(());
    }
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(TuiError::InteractiveTerminalRequired);
    }
    run_interactive(inspector)
}

fn run_world(args: WorldArgs) -> Result<(), TuiError> {
    let world_path = if args.input.is_dir() {
        args.input.join("world.json")
    } else {
        args.input.clone()
    };
    let world: SurfaceWorldV1 = serde_json::from_slice(&fs::read(world_path)?)?;
    let history_path = args.input.join("summary.json");
    let history = if args.input.is_dir() && args.input.join("events.jsonl").is_file() {
        Some(serde_json::from_slice::<HistorySummaryV1>(&fs::read(
            history_path,
        )?)?)
    } else {
        None
    };
    let layer = AtlasLayer::from(args.layer);
    if args.snapshot {
        print_world_snapshot(&world, history.as_ref(), layer, args.width, args.height);
        return Ok(());
    }
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(TuiError::InteractiveTerminalRequired);
    }
    run_world_interactive(&world, history.as_ref(), layer, args.width, args.height)
}

fn print_world_snapshot(
    world: &SurfaceWorldV1,
    history: Option<&HistorySummaryV1>,
    layer: AtlasLayer,
    width: u16,
    height: u16,
) {
    let mut screen = render_world_snapshot(world, layer, width, height);
    if let Some(history) = history {
        screen.push_str(&format!(
            "\nHISTORY / YEAR {}\n{} people · {} settlements · {} cultures · {} faiths\n",
            history.elapsed_years,
            history.total_population,
            history.settlements,
            history.cultures,
            history.faiths
        ));
        screen.push_str(&history.first_contact_year.map_or_else(
            || String::from("The capability-gated route remained closed.\n"),
            |year| {
                format!(
                    "First cross-homeland contact: Year {year} · {} mixed population(s)\n",
                    history.mixed_lineage_populations
                )
            },
        ));
    }
    print!("{screen}");
}

fn run_world_interactive(
    world: &SurfaceWorldV1,
    history: Option<&HistorySummaryV1>,
    initial_layer: AtlasLayer,
    width: u16,
    height: u16,
) -> Result<(), TuiError> {
    enable_raw_mode()?;
    let mut output = stdout();
    execute!(output, EnterAlternateScreen)?;
    let result = (|| {
        let mut layer = initial_layer;
        loop {
            execute!(output, MoveTo(0, 0), Clear(ClearType::All))?;
            let mut screen = render_world_snapshot(world, layer, width, height);
            if let Some(history) = history {
                screen.push_str(&format!(
                    "\nYear {} · {} people · first contact {}\n",
                    history.elapsed_years,
                    history.total_population,
                    history
                        .first_contact_year
                        .map_or_else(|| String::from("not reached"), |year| year.to_string())
                ));
            }
            screen.push_str("\nTab/l layer · q quit\n");
            print!("{screen}");
            use std::io::Write;
            output.flush()?;
            if !event::poll(Duration::from_millis(250))? {
                continue;
            }
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Tab | KeyCode::Char('l') => {
                    layer = next_world_layer(layer);
                }
                _ => {}
            }
        }
    })();
    let cleanup = (|| {
        disable_raw_mode()?;
        execute!(output, LeaveAlternateScreen)?;
        Ok(())
    })();
    result.and(cleanup)
}

const fn next_world_layer(layer: AtlasLayer) -> AtlasLayer {
    match layer {
        AtlasLayer::Terrain => AtlasLayer::Biome,
        AtlasLayer::Biome => AtlasLayer::Habitability,
        AtlasLayer::Habitability => AtlasLayer::Resources,
        AtlasLayer::Resources => AtlasLayer::Mythic,
        AtlasLayer::Mythic => AtlasLayer::Terrain,
    }
}

fn run_interactive(mut inspector: Inspector) -> Result<(), TuiError> {
    enable_raw_mode()?;
    let mut output = stdout();
    execute!(output, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(output);
    let mut terminal = Terminal::new(backend)?;

    let interaction_result = interaction_loop(&mut terminal, &mut inspector);
    let cleanup_result = restore_terminal(&mut terminal);
    interaction_result.and(cleanup_result)
}

fn interaction_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    inspector: &mut Inspector,
) -> Result<(), TuiError> {
    loop {
        terminal.draw(|frame| render(frame, inspector))?;
        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        if inspector.is_searching() {
            match key.code {
                KeyCode::Enter => inspector.accept_search(),
                KeyCode::Esc => inspector.cancel_search(),
                KeyCode::Backspace => inspector.pop_search_char(),
                KeyCode::Char(character) => inspector.push_search_char(character),
                _ => {}
            }
            continue;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
            KeyCode::Tab => inspector.toggle_view(),
            KeyCode::Char('1') => inspector.set_view(View::Overview),
            KeyCode::Char('2') => inspector.set_view(View::History),
            KeyCode::Char('3') => inspector.set_view(View::People),
            KeyCode::Char('4') => inspector.set_view(View::Lineage),
            KeyCode::Char('5') => inspector.set_view(View::Households),
            KeyCode::Up | KeyCode::Char('k') => inspector.previous(),
            KeyCode::Down | KeyCode::Char('j') => inspector.next(),
            KeyCode::PageUp => inspector.page_up(),
            KeyCode::PageDown => inspector.page_down(),
            KeyCode::Home => inspector.first(),
            KeyCode::End => inspector.last(),
            KeyCode::Enter => inspector.activate(),
            KeyCode::Char('/') => inspector.begin_search(),
            KeyCode::Char('x') => inspector.clear_search(),
            KeyCode::Char('f') => inspector.cycle_event_filter(),
            KeyCode::Char('s') => inspector.cycle_sort(),
            KeyCode::Char('e') => inspector.jump_to_related_event(),
            KeyCode::Char('h') => inspector.jump_to_household(),
            _ => {}
        }
    }
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<(), TuiError> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

#[derive(Debug, Error)]
enum TuiError {
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid RON scenario: {0}")]
    Ron(#[from] ron::error::SpannedError),
    #[error("invalid generated-world JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Scenario(#[from] merra_core::ScenarioError),
    #[error(transparent)]
    Simulation(#[from] merra_sim::SimulationError),
    #[error("interactive mode requires a terminal; use --snapshot for redirected output")]
    InteractiveTerminalRequired,
    #[error("requested stable focus does not exist: {0:?}")]
    FocusNotFound(Focus),
}
