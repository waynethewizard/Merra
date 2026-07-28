//! Interactive terminal entry point.

use std::{
    fs,
    io::{self, IsTerminal, stdout},
    num::NonZeroU32,
    path::PathBuf,
    process::ExitCode,
    time::Duration,
};

use clap::{Parser, ValueEnum};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use merra_core::ScenarioV1;
use merra_sim::run_years;
use merra_tui::{Inspector, View, render, snapshot_view};
use ratatui::{Terminal, backend::CrosstermBackend};
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(
    name = "merra-tui",
    version,
    about = "Inspect a deterministic Merra history"
)]
struct Args {
    /// RON scenario to simulate before opening the inspector.
    #[arg(long, default_value = "scenarios/era-01/century.ron")]
    scenario: PathBuf,
    /// Root deterministic seed.
    #[arg(long, default_value_t = 42)]
    seed: u64,
    /// Number of complete scenario years.
    #[arg(long, default_value = "100")]
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
    #[arg(long, value_enum, default_value_t = InitialView::Events)]
    view: InitialView,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum InitialView {
    Events,
    People,
}

impl From<InitialView> for View {
    fn from(value: InitialView) -> Self {
        match value {
            InitialView::Events => Self::Events,
            InitialView::People => Self::People,
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
    let scenario_bytes = fs::read(&args.scenario)?;
    let scenario: ScenarioV1 = ron::de::from_bytes(&scenario_bytes)?;
    scenario.validate()?;
    let report = run_years(scenario, args.seed, args.years.get())?;
    let initial_view = View::from(args.view);

    if args.snapshot {
        print!(
            "{}",
            snapshot_view(report, args.width, args.height, initial_view)
        );
        return Ok(());
    }
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(TuiError::InteractiveTerminalRequired);
    }
    let mut inspector = Inspector::new(report);
    inspector.set_view(initial_view);
    run_interactive(inspector)
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
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
            KeyCode::Tab => inspector.toggle_view(),
            KeyCode::Up | KeyCode::Char('k') => inspector.previous(),
            KeyCode::Down | KeyCode::Char('j') => inspector.next(),
            KeyCode::PageUp => inspector.page_up(),
            KeyCode::PageDown => inspector.page_down(),
            KeyCode::Home => inspector.first(),
            KeyCode::End => inspector.last(),
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
    #[error(transparent)]
    Scenario(#[from] merra_core::ScenarioError),
    #[error(transparent)]
    Simulation(#[from] merra_sim::SimulationError),
    #[error("interactive mode requires a terminal; use --snapshot for redirected output")]
    InteractiveTerminalRequired,
}
