//! Interactive terminal entry point.

use std::{
    fs,
    io::{self, IsTerminal, stdout},
    num::NonZeroU32,
    path::PathBuf,
    process::ExitCode,
    time::{Duration, Instant},
};

use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};
use crossterm::{
    cursor::MoveTo,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use merra_core::{
    EventId, HistorySummaryV1, HouseholdId, ItemId, LocalHistoryReportV1, LocationId, PersonId,
    ScenarioV1, SurfaceWorldV1,
};
use merra_sim::run_years;
use merra_tui::{
    EntityRef, Focus, Inspector, LocalInspector, LocalView, MediaCatalog, MediaError, Observatory,
    ObservatoryData, ObservatoryError, ObservatoryTheme, ObservatoryView, PaneFocus, View, render,
    render_local_snapshot, render_observatory, render_observatory_snapshot, render_snapshot,
};
use merra_worldgen::{AtlasLayer, render_snapshot as render_world_snapshot};
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect, style::Color};
use tachyonfx::{EffectManager, Interpolation, fx};
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(
    name = "merra-tui",
    version,
    about = "Explore Merra from terrain and centuries to villages, lives, and artifacts"
)]
struct Args {
    /// Open a legacy single-scale inspector.
    #[command(subcommand)]
    command: Option<InspectorCommand>,
    /// World output directory or `world.json` to explore without history.
    #[arg(long, conflicts_with = "history")]
    world: Option<PathBuf>,
    /// Regional-history output directory containing the connected world and history.
    #[arg(long, conflicts_with = "world")]
    history: Option<PathBuf>,
    /// Local-history output directory to connect to `--history`.
    #[arg(long, requires = "history")]
    local: Option<PathBuf>,
    /// Print an ANSI-free observatory screen instead of entering interactive mode.
    #[arg(long)]
    snapshot: bool,
    /// Snapshot width in terminal cells.
    #[arg(long, default_value_t = 120)]
    width: u16,
    /// Snapshot height in terminal cells.
    #[arg(long, default_value_t = 40)]
    height: u16,
    /// Initial observatory workspace.
    #[arg(long, value_enum, default_value_t = InitialObservatoryView::Atlas)]
    workspace: InitialObservatoryView,
    /// Initial global year on the connected timeline.
    #[arg(long)]
    year: Option<u32>,
    /// Initial typed focus, for example `location:3`, `person:17`, or `item:4`.
    #[arg(long, value_name = "KIND:ID")]
    focus: Option<EntityRef>,
    /// Color treatment for interactive and snapshot rendering.
    #[arg(long, value_enum, default_value_t = InitialTheme::Archive)]
    theme: InitialTheme,
    /// Disable transition effects while retaining live playback.
    #[arg(long)]
    no_motion: bool,
    /// Custom observatory media manifest; assets resolve relative to this JSON file.
    #[arg(long, value_name = "MANIFEST.json")]
    media: Option<PathBuf>,
}

#[derive(Debug, ClapArgs)]
struct DynastyArgs {
    /// RON scenario to simulate before opening the legacy dynasty inspector.
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
    /// Open the original single-settlement dynasty inspector.
    Dynasty(DynastyArgs),
    /// Inspect continent generation and optional aggregate history.
    World(WorldArgs),
    /// Inspect the detailed five-settlement local history.
    Villages(VillageArgs),
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

#[derive(Debug, ClapArgs)]
struct VillageArgs {
    /// Local-history output directory or `local-history.json` file.
    #[arg(long)]
    input: PathBuf,
    /// Print an ANSI-free screen instead of entering interactive mode.
    #[arg(long)]
    snapshot: bool,
    /// Snapshot width in terminal cells.
    #[arg(long, default_value_t = 120)]
    width: u16,
    /// Snapshot height in terminal cells.
    #[arg(long, default_value_t = 36)]
    height: u16,
    /// Initial five-village collection.
    #[arg(long, value_enum, default_value_t = InitialLocalView::Overview)]
    view: InitialLocalView,
    /// Focus a stable settlement identity.
    #[arg(long, value_name = "ID", conflicts_with_all = ["focus_household", "focus_item"])]
    focus_settlement: Option<u64>,
    /// Focus a stable household identity.
    #[arg(long, value_name = "ID", conflicts_with_all = ["focus_settlement", "focus_item"])]
    focus_household: Option<u64>,
    /// Focus a stable durable item identity.
    #[arg(long, value_name = "ID", conflicts_with_all = ["focus_settlement", "focus_household"])]
    focus_item: Option<u64>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum InitialObservatoryView {
    Atlas,
    Chronicle,
    Relations,
    Catalog,
}

impl From<InitialObservatoryView> for ObservatoryView {
    fn from(value: InitialObservatoryView) -> Self {
        match value {
            InitialObservatoryView::Atlas => Self::Atlas,
            InitialObservatoryView::Chronicle => Self::Chronicle,
            InitialObservatoryView::Relations => Self::Relations,
            InitialObservatoryView::Catalog => Self::Catalog,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum InitialTheme {
    Archive,
    Monochrome,
}

impl From<InitialTheme> for ObservatoryTheme {
    fn from(value: InitialTheme) -> Self {
        match value {
            InitialTheme::Archive => Self::Archive,
            InitialTheme::Monochrome => Self::Monochrome,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum InitialWorldLayer {
    Terrain,
    Biome,
    Habitability,
    Resources,
    Mythic,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum InitialLocalView {
    Overview,
    Roads,
    Settlements,
    Migrations,
    Households,
    Items,
}

impl From<InitialLocalView> for LocalView {
    fn from(value: InitialLocalView) -> Self {
        match value {
            InitialLocalView::Overview => Self::Overview,
            InitialLocalView::Roads => Self::Roads,
            InitialLocalView::Settlements => Self::Settlements,
            InitialLocalView::Migrations => Self::Migrations,
            InitialLocalView::Households => Self::Households,
            InitialLocalView::Items => Self::Items,
        }
    }
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

fn run(mut args: Args) -> Result<(), TuiError> {
    match args.command.take() {
        Some(InspectorCommand::Dynasty(dynasty)) => run_dynasty(dynasty),
        Some(InspectorCommand::World(world)) => run_world(world),
        Some(InspectorCommand::Villages(villages)) => run_villages(villages),
        None => run_observatory(args),
    }
}

fn run_observatory(args: Args) -> Result<(), TuiError> {
    let data = ObservatoryData::load(
        args.world.as_deref(),
        args.history.as_deref(),
        args.local.as_deref(),
    )?;
    let mut observatory = Observatory::new(data);
    let media = args
        .media
        .as_deref()
        .map_or_else(MediaCatalog::canonical, MediaCatalog::load)?;
    observatory.set_media_catalog(media);
    observatory.set_view(args.workspace.into());
    if matches!(args.theme, InitialTheme::Monochrome) || std::env::var_os("NO_COLOR").is_some() {
        observatory.set_theme(ObservatoryTheme::Monochrome);
    }
    if let Some(year) = args.year {
        observatory.set_year(year);
    }
    if let Some(focus) = args.focus
        && !observatory.focus_entity(focus)
    {
        return Err(TuiError::ObservatoryFocusNotFound(focus));
    }
    if args.snapshot {
        print!(
            "{}",
            render_observatory_snapshot(&observatory, args.width, args.height)
        );
        return Ok(());
    }
    require_interactive_terminal()?;
    run_observatory_interactive(observatory, args.no_motion)
}

fn run_dynasty(args: DynastyArgs) -> Result<(), TuiError> {
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
    require_interactive_terminal()?;
    run_interactive(inspector)
}

type MerraTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

fn require_interactive_terminal() -> Result<(), TuiError> {
    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        Ok(())
    } else {
        Err(TuiError::InteractiveTerminalRequired)
    }
}

fn run_observatory_interactive(
    mut observatory: Observatory,
    no_motion: bool,
) -> Result<(), TuiError> {
    with_terminal(true, |terminal| {
        observatory_loop(terminal, &mut observatory, no_motion)
    })
}

fn observatory_loop(
    terminal: &mut MerraTerminal,
    app: &mut Observatory,
    no_motion: bool,
) -> Result<(), TuiError> {
    let effects_enabled = !no_motion && app.theme() != ObservatoryTheme::Monochrome;
    let mut effects = EffectManager::<u8>::default();
    if effects_enabled {
        add_transition(&mut effects);
    }
    let mut transition_epoch = app.transition_epoch();
    let mut last_frame = Instant::now();
    let mut last_playback = Instant::now();

    loop {
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(last_frame);
        last_frame = now;
        terminal.draw(|frame| {
            render_observatory(frame, app);
            if effects_enabled {
                let area = frame.area();
                effects.process_effects(elapsed, frame.buffer_mut(), area);
            }
        })?;

        let wait = if app.is_playing() || effects.is_running() {
            Duration::from_millis(33)
        } else {
            Duration::from_millis(250)
        };
        if event::poll(wait)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if handle_observatory_key(app, key) {
                        return Ok(());
                    }
                }
                Event::Mouse(mouse) => handle_observatory_mouse(app, mouse),
                Event::Resize(_, _) | Event::FocusGained | Event::FocusLost | Event::Paste(_) => {}
                Event::Key(_) => {}
            }
        }

        if app.is_playing() && last_playback.elapsed() >= Duration::from_millis(320) {
            app.playback_tick();
            last_playback = Instant::now();
        }
        if app.transition_epoch() != transition_epoch {
            transition_epoch = app.transition_epoch();
            if effects_enabled {
                add_transition(&mut effects);
            }
        }
    }
}

fn add_transition(effects: &mut EffectManager<u8>) {
    effects.add_unique_effect(
        0,
        fx::fade_from_fg(Color::Black, (180, Interpolation::QuadOut)),
    );
}

fn handle_observatory_key(app: &mut Observatory, key: KeyEvent) -> bool {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return true;
    }
    if app.is_searching() {
        match key.code {
            KeyCode::Enter => app.accept_search(),
            KeyCode::Esc => app.cancel_search(),
            KeyCode::Backspace => app.pop_search(),
            KeyCode::Up => app.move_selection(false),
            KeyCode::Down => app.move_selection(true),
            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                app.push_search(character);
            }
            _ => {}
        }
        return false;
    }
    if app.help_visible() {
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('?')) {
            app.toggle_help();
        }
        return false;
    }

    match key.code {
        KeyCode::Char('q') => return true,
        KeyCode::Esc | KeyCode::Char('b') => app.back(),
        KeyCode::Char('?') => app.toggle_help(),
        KeyCode::Char('/') => app.begin_search(),
        KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => app.cycle_pane(false),
        KeyCode::Tab => app.cycle_pane(true),
        KeyCode::BackTab => app.cycle_pane(false),
        KeyCode::Char('1') => app.set_view(ObservatoryView::Atlas),
        KeyCode::Char('2') => app.set_view(ObservatoryView::Chronicle),
        KeyCode::Char('3') => app.set_view(ObservatoryView::Relations),
        KeyCode::Char('4') => app.set_view(ObservatoryView::Catalog),
        KeyCode::Char('[') => app.step_event(false),
        KeyCode::Char(']') => app.step_event(true),
        KeyCode::Char(' ') => app.toggle_playback(),
        KeyCode::Char('r') => app.toggle_reverse_playback(),
        KeyCode::Char(',') | KeyCode::Char('<') => app.step_year(false),
        KeyCode::Char('.') | KeyCode::Char('>') => app.step_year(true),
        KeyCode::Home => app.first_year(),
        KeyCode::End => app.last_year(),
        KeyCode::Char('p') if app.view() == ObservatoryView::Atlas => {
            app.cycle_person_at_focus(true);
        }
        KeyCode::Char('P') if app.view() == ObservatoryView::Atlas => {
            app.cycle_person_at_focus(false);
        }
        KeyCode::Char('g') if app.view() == ObservatoryView::Relations => {
            app.toggle_family_tree();
        }
        KeyCode::Char('g') => app.show_family_tree(),
        KeyCode::Char('f') => app.toggle_debug(),
        KeyCode::Char('L') => app.next_layer(),
        KeyCode::Char('+') | KeyCode::Char('=') => app.zoom_map(true),
        KeyCode::Char('-') => app.zoom_map(false),
        KeyCode::PageUp if app.pane() == PaneFocus::Detail => app.page_detail(false),
        KeyCode::PageDown if app.pane() == PaneFocus::Detail => app.page_detail(true),
        KeyCode::Up | KeyCode::Char('k') if app.pane() == PaneFocus::Detail => {
            app.scroll_detail(false);
        }
        KeyCode::Down | KeyCode::Char('j') if app.pane() == PaneFocus::Detail => {
            app.scroll_detail(true);
        }
        KeyCode::Enter if app.pane() == PaneFocus::Detail => {
            app.set_view(ObservatoryView::Relations);
            app.set_pane(PaneFocus::Primary);
        }
        KeyCode::Up | KeyCode::Char('k') if app.pane() == PaneFocus::Timeline => {
            app.step_event(false);
        }
        KeyCode::Down | KeyCode::Char('j') if app.pane() == PaneFocus::Timeline => {
            app.step_event(true);
        }
        KeyCode::Left if app.pane() == PaneFocus::Timeline => app.step_year(false),
        KeyCode::Right if app.pane() == PaneFocus::Timeline => app.step_year(true),
        KeyCode::PageUp => app.page_selection(false),
        KeyCode::PageDown => app.page_selection(true),
        KeyCode::Enter => app.activate_selection(),
        KeyCode::Up | KeyCode::Char('k') if app.view() == ObservatoryView::Atlas => {
            app.move_map_cursor(0, -1);
        }
        KeyCode::Down | KeyCode::Char('j') if app.view() == ObservatoryView::Atlas => {
            app.move_map_cursor(0, 1);
        }
        KeyCode::Left | KeyCode::Char('h') if app.view() == ObservatoryView::Atlas => {
            app.move_map_cursor(-1, 0);
        }
        KeyCode::Right | KeyCode::Char('l') if app.view() == ObservatoryView::Atlas => {
            app.move_map_cursor(1, 0);
        }
        KeyCode::Left if app.view() == ObservatoryView::Catalog => app.cycle_catalog(false),
        KeyCode::Right if app.view() == ObservatoryView::Catalog => app.cycle_catalog(true),
        KeyCode::Left => app.step_year(false),
        KeyCode::Right => app.step_year(true),
        KeyCode::Up | KeyCode::Char('k') => app.move_selection(false),
        KeyCode::Down | KeyCode::Char('j') => app.move_selection(true),
        _ => {}
    }
    false
}

fn handle_observatory_mouse(app: &mut Observatory, mouse: MouseEvent) {
    let hits = app.hit_regions().clone();
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some((view, _)) = hits
                .tabs
                .iter()
                .find(|(_, area)| contains(*area, mouse.column, mouse.row))
            {
                app.set_view(*view);
                return;
            }
            if app.click_row(mouse.column, mouse.row) {
                app.set_pane(PaneFocus::Primary);
                return;
            }
            if contains(hits.timeline, mouse.column, mouse.row) {
                app.set_pane(PaneFocus::Timeline);
                app.select_timeline_position(mouse.column, hits.timeline);
                return;
            }
            if contains(hits.detail, mouse.column, mouse.row) {
                app.set_pane(PaneFocus::Detail);
                return;
            }
            if app.view() == ObservatoryView::Atlas
                && contains(hits.primary, mouse.column, mouse.row)
            {
                app.set_pane(PaneFocus::Primary);
                app.select_map_position(mouse.column, mouse.row, hits.primary);
            }
        }
        MouseEventKind::ScrollUp => {
            if contains(hits.detail, mouse.column, mouse.row) {
                app.set_pane(PaneFocus::Detail);
                app.scroll_detail(false);
            } else if app.view() == ObservatoryView::Atlas
                && contains(hits.primary, mouse.column, mouse.row)
            {
                app.set_pane(PaneFocus::Primary);
                app.zoom_map(true);
            } else if contains(hits.timeline, mouse.column, mouse.row) {
                app.set_pane(PaneFocus::Timeline);
                app.step_event(false);
            } else {
                app.set_pane(PaneFocus::Primary);
                app.move_selection(false);
            }
        }
        MouseEventKind::ScrollDown => {
            if contains(hits.detail, mouse.column, mouse.row) {
                app.set_pane(PaneFocus::Detail);
                app.scroll_detail(true);
            } else if app.view() == ObservatoryView::Atlas
                && contains(hits.primary, mouse.column, mouse.row)
            {
                app.set_pane(PaneFocus::Primary);
                app.zoom_map(false);
            } else if contains(hits.timeline, mouse.column, mouse.row) {
                app.set_pane(PaneFocus::Timeline);
                app.step_event(true);
            } else {
                app.set_pane(PaneFocus::Primary);
                app.move_selection(true);
            }
        }
        _ => {}
    }
}

const fn contains(area: Rect, column: u16, row: u16) -> bool {
    column >= area.x
        && column < area.x.saturating_add(area.width)
        && row >= area.y
        && row < area.y.saturating_add(area.height)
}

fn with_terminal<T>(
    mouse: bool,
    action: impl FnOnce(&mut MerraTerminal) -> Result<T, TuiError>,
) -> Result<T, TuiError> {
    enable_raw_mode()?;
    let mut output = stdout();
    let entered = if mouse {
        execute!(output, EnterAlternateScreen, EnableMouseCapture)
    } else {
        execute!(output, EnterAlternateScreen)
    };
    if let Err(error) = entered {
        let _ = disable_raw_mode();
        return Err(error.into());
    }
    let backend = CrosstermBackend::new(output);
    let mut terminal = match Terminal::new(backend) {
        Ok(terminal) => terminal,
        Err(error) => {
            let _ = disable_raw_mode();
            let _ = execute!(stdout(), LeaveAlternateScreen);
            return Err(error.into());
        }
    };
    let result = action(&mut terminal);
    let cleanup = restore_terminal(&mut terminal, mouse);
    match result {
        Ok(value) => cleanup.map(|()| value),
        Err(error) => {
            let _ = cleanup;
            Err(error)
        }
    }
}

fn run_villages(args: VillageArgs) -> Result<(), TuiError> {
    let report_path = if args.input.is_dir() {
        args.input.join("local-history.json")
    } else {
        args.input.clone()
    };
    let report: LocalHistoryReportV1 = serde_json::from_slice(&fs::read(report_path)?)?;
    let mut inspector = LocalInspector::new(report);
    inspector.set_view(LocalView::from(args.view));
    if let Some(id) = args.focus_settlement
        && !inspector.focus_location(LocationId(id))
    {
        return Err(TuiError::LocalFocusNotFound(format!("settlement #{id}")));
    }
    if let Some(id) = args.focus_household
        && !inspector.focus_household(HouseholdId(id))
    {
        return Err(TuiError::LocalFocusNotFound(format!("household #{id}")));
    }
    if let Some(id) = args.focus_item
        && !inspector.focus_item(ItemId(id))
    {
        return Err(TuiError::LocalFocusNotFound(format!("item #{id}")));
    }
    if args.snapshot {
        print!(
            "{}",
            render_local_snapshot(&inspector, args.width, args.height)
        );
        return Ok(());
    }
    require_interactive_terminal()?;
    run_villages_interactive(inspector, args.width, args.height)
}

fn run_villages_interactive(
    mut inspector: LocalInspector,
    width: u16,
    height: u16,
) -> Result<(), TuiError> {
    enable_raw_mode()?;
    let mut output = stdout();
    execute!(output, EnterAlternateScreen)?;
    let result = (|| {
        loop {
            execute!(output, MoveTo(0, 0), Clear(ClearType::All))?;
            print!("{}", render_local_snapshot(&inspector, width, height));
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
                KeyCode::Tab => inspector.toggle_view(),
                KeyCode::Char('1') => inspector.set_view(LocalView::Overview),
                KeyCode::Char('2') => inspector.set_view(LocalView::Roads),
                KeyCode::Char('3') => inspector.set_view(LocalView::Settlements),
                KeyCode::Char('4') => inspector.set_view(LocalView::Migrations),
                KeyCode::Char('5') => inspector.set_view(LocalView::Households),
                KeyCode::Char('6') => inspector.set_view(LocalView::Items),
                KeyCode::Up | KeyCode::Char('k') => inspector.previous(),
                KeyCode::Down | KeyCode::Char('j') => inspector.next(),
                KeyCode::Enter => inspector.activate(),
                KeyCode::Char('x') => inspector.clear_filter(),
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
    require_interactive_terminal()?;
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
    with_terminal(false, |terminal| interaction_loop(terminal, &mut inspector))
}

fn interaction_loop(
    terminal: &mut MerraTerminal,
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

fn restore_terminal(terminal: &mut MerraTerminal, mouse: bool) -> Result<(), TuiError> {
    disable_raw_mode()?;
    if mouse {
        execute!(
            terminal.backend_mut(),
            DisableMouseCapture,
            LeaveAlternateScreen
        )?;
    } else {
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    }
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
    #[error(transparent)]
    Observatory(#[from] ObservatoryError),
    #[error(transparent)]
    Media(#[from] MediaError),
    #[error("interactive mode requires a terminal; use --snapshot for redirected output")]
    InteractiveTerminalRequired,
    #[error("requested stable focus does not exist: {0:?}")]
    FocusNotFound(Focus),
    #[error("requested local focus does not exist: {0}")]
    LocalFocusNotFound(String),
    #[error("requested observatory focus does not exist: {0}")]
    ObservatoryFocusNotFound(EntityRef),
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Args, EntityRef, InitialObservatoryView, InspectorCommand};

    #[test]
    fn default_cli_accepts_observatory_workspace_year_and_typed_focus()
    -> Result<(), Box<dyn std::error::Error>> {
        let args = Args::try_parse_from([
            "merra-tui",
            "--snapshot",
            "--workspace",
            "relations",
            "--year",
            "610",
            "--focus",
            "person:17",
            "--media",
            "assets/observatory/media.json",
        ])?;

        assert!(args.command.is_none());
        assert!(args.snapshot);
        assert!(matches!(args.workspace, InitialObservatoryView::Relations));
        assert_eq!(args.year, Some(610));
        assert_eq!(args.focus, Some("person:17".parse::<EntityRef>()?));
        assert_eq!(
            args.media.as_deref(),
            Some(std::path::Path::new("assets/observatory/media.json"))
        );
        Ok(())
    }

    #[test]
    fn legacy_dynasty_is_explicit_and_local_requires_history()
    -> Result<(), Box<dyn std::error::Error>> {
        let args = Args::try_parse_from(["merra-tui", "dynasty", "--snapshot"])?;
        assert!(matches!(args.command, Some(InspectorCommand::Dynasty(_))));
        assert!(Args::try_parse_from(["merra-tui", "--local", "some-run", "--snapshot"]).is_err());
        Ok(())
    }
}
