//! Terminal rendering and navigation for Merra simulation evidence.

use std::convert::Infallible;

use merra_core::{EventPayloadV1, PersonRecordV1, WorldEventV1};
use merra_sim::SimulationReport;
use ratatui::{
    Frame, Terminal,
    backend::TestBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap},
};

/// The two inspectable evidence collections.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum View {
    /// Ordered structured events.
    Events,
    /// Final person records.
    People,
}

/// Navigable state for the terminal inspector.
pub struct Inspector {
    report: SimulationReport,
    view: View,
    selected_event: usize,
    selected_person: usize,
}

impl Inspector {
    /// Creates an inspector focused on the first event.
    #[must_use]
    pub const fn new(report: SimulationReport) -> Self {
        Self {
            report,
            view: View::Events,
            selected_event: 0,
            selected_person: 0,
        }
    }

    /// Returns the current view.
    #[must_use]
    pub const fn view(&self) -> View {
        self.view
    }

    /// Switches between events and people.
    pub const fn toggle_view(&mut self) {
        self.view = match self.view {
            View::Events => View::People,
            View::People => View::Events,
        };
    }

    /// Selects a specific evidence view.
    pub const fn set_view(&mut self, view: View) {
        self.view = view;
    }

    /// Selects the preceding row.
    pub fn previous(&mut self) {
        let selected = self.selected_mut();
        *selected = selected.saturating_sub(1);
    }

    /// Selects the following row.
    pub fn next(&mut self) {
        let maximum = self.active_len().saturating_sub(1);
        let selected = self.selected_mut();
        *selected = selected.saturating_add(1).min(maximum);
    }

    /// Moves upward by a page.
    pub fn page_up(&mut self) {
        let selected = self.selected_mut();
        *selected = selected.saturating_sub(10);
    }

    /// Moves downward by a page.
    pub fn page_down(&mut self) {
        let maximum = self.active_len().saturating_sub(1);
        let selected = self.selected_mut();
        *selected = selected.saturating_add(10).min(maximum);
    }

    /// Selects the first row.
    pub fn first(&mut self) {
        *self.selected_mut() = 0;
    }

    /// Selects the last row.
    pub fn last(&mut self) {
        *self.selected_mut() = self.active_len().saturating_sub(1);
    }

    fn selected_mut(&mut self) -> &mut usize {
        match self.view {
            View::Events => &mut self.selected_event,
            View::People => &mut self.selected_person,
        }
    }

    fn active_len(&self) -> usize {
        match self.view {
            View::Events => self.report.events.len(),
            View::People => self.report.people.len(),
        }
    }
}

/// Draws the complete inspector.
pub fn render(frame: &mut Frame<'_>, inspector: &Inspector) {
    let [header, tabs, body, footer] = Layout::vertical([
        Constraint::Length(5),
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    render_header(frame, header, inspector);
    render_tabs(frame, tabs, inspector);
    let [list, detail] =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).areas(body);
    match inspector.view {
        View::Events => render_events(frame, list, detail, inspector),
        View::People => render_people(frame, list, detail, inspector),
    }
    frame.render_widget(
        Paragraph::new("q quit  Tab view  ↑/k ↓/j move  PgUp/PgDn page  Home/End jump")
            .style(Style::default().fg(Color::DarkGray)),
        footer,
    );
}

fn render_header(frame: &mut Frame<'_>, area: Rect, inspector: &Inspector) {
    let summary = &inspector.report.summary;
    let text = Text::from(vec![
        Line::from(vec![
            Span::styled(
                "MERRA // FIRST HUNDRED YEARS",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("   seed {}", summary.seed)),
        ]),
        Line::from(format!(
            "Scenario {}  |  Year {}  |  Events {}",
            summary.scenario_id, summary.elapsed_years, summary.event_count
        )),
        Line::from(format!(
            "Population {} → {} living  |  {} deaths",
            summary.initial_population, summary.living_population, summary.deaths
        )),
    ]);
    frame.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Chronicle Lab "),
        ),
        area,
    );
}

fn render_tabs(frame: &mut Frame<'_>, area: Rect, inspector: &Inspector) {
    let selected = match inspector.view {
        View::Events => 0,
        View::People => 1,
    };
    frame.render_widget(
        Tabs::new(["Events", "People"])
            .select(selected)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        area,
    );
}

fn render_events(frame: &mut Frame<'_>, list_area: Rect, detail_area: Rect, inspector: &Inspector) {
    let items: Vec<ListItem<'_>> = inspector
        .report
        .events
        .iter()
        .map(|event| {
            ListItem::new(format!(
                "{:>4}  Y{:>3}  {}",
                event.id.0,
                event.time.day() / u64::from(inspector.report.summary.days_per_year),
                event_short_label(event)
            ))
        })
        .collect();
    let mut state = ListState::default().with_selected(Some(inspector.selected_event));
    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Timeline "))
            .highlight_symbol("▶ ")
            .highlight_style(Style::default().fg(Color::Yellow)),
        list_area,
        &mut state,
    );

    let detail = inspector
        .report
        .events
        .get(inspector.selected_event)
        .map_or_else(|| String::from("No event selected."), event_detail);
    frame.render_widget(
        Paragraph::new(detail)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Event Evidence "),
            )
            .wrap(Wrap { trim: false }),
        detail_area,
    );
}

fn render_people(frame: &mut Frame<'_>, list_area: Rect, detail_area: Rect, inspector: &Inspector) {
    let items: Vec<ListItem<'_>> = inspector
        .report
        .people
        .iter()
        .map(|person| {
            let status = if person.alive { "living" } else { "dead" };
            ListItem::new(format!(
                "{:>3}  {:<22} age {:>3}  {status}",
                person.id.0, person.name, person.final_age_years
            ))
        })
        .collect();
    let mut state = ListState::default().with_selected(Some(inspector.selected_person));
    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" People "))
            .highlight_symbol("▶ ")
            .highlight_style(Style::default().fg(Color::Yellow)),
        list_area,
        &mut state,
    );

    let detail = inspector
        .report
        .people
        .get(inspector.selected_person)
        .map_or_else(|| String::from("No person selected."), person_detail);
    frame.render_widget(
        Paragraph::new(detail)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Life Record "),
            )
            .wrap(Wrap { trim: false }),
        detail_area,
    );
}

fn event_short_label(event: &WorldEventV1) -> String {
    match &event.payload {
        EventPayloadV1::SimulationStarted { .. } => String::from("simulation started"),
        EventPayloadV1::PopulationInitialized { people } => {
            format!("{people} people initialized")
        }
        EventPayloadV1::TimeAdvanced { to_day, .. } => {
            format!("time advanced to day {to_day}")
        }
        EventPayloadV1::SeasonBegan {
            season_name, year, ..
        } => format!("{season_name} began in Year {year}"),
        EventPayloadV1::PersonDied {
            name, age_years, ..
        } => format!("{name} died at {age_years}"),
        EventPayloadV1::SimulationCompleted { .. } => String::from("simulation completed"),
    }
}

fn event_detail(event: &WorldEventV1) -> String {
    let causes = if event.causes.is_empty() {
        String::from("none")
    } else {
        event
            .causes
            .iter()
            .map(|id| id.0.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "{}\n\nEvent ID: {}\nAbsolute day: {}\nCausal events: {}\nActors: {}\nTags: {}\n\nThis is an authoritative world event. Later memory and record systems may preserve or distort it.",
        event_short_label(event),
        event.id.0,
        event.time.day(),
        causes,
        event.actors.len(),
        event.tags.join(", "),
    )
}

fn person_detail(person: &PersonRecordV1) -> String {
    let ending = person.death_day.map_or_else(
        || String::from("Alive at end of run"),
        |day| format!("Died on absolute day {day}"),
    );
    format!(
        "{}\n\nPerson ID: {}\nStarting age: {}\nFinal age: {}\n{}\n\nThis record is stable simulation state, not a Bevy entity identifier.",
        person.name, person.id.0, person.starting_age_years, person.final_age_years, ending,
    )
}

/// Renders a portable, ANSI-free screen for tests, CI, and review.
pub fn snapshot(report: SimulationReport, width: u16, height: u16) -> String {
    snapshot_view(report, width, height, View::Events)
}

/// Renders a selected view as portable, ANSI-free text.
pub fn snapshot_view(report: SimulationReport, width: u16, height: u16, view: View) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = infallible(Terminal::new(backend));
    let mut inspector = Inspector::new(report);
    inspector.set_view(view);
    infallible(terminal.draw(|frame| render(frame, &inspector)));
    let buffer = terminal.backend().buffer();
    let mut output = String::new();
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            line.push_str(buffer[(x, y)].symbol());
        }
        output.push_str(line.trim_end());
        output.push('\n');
    }
    output
}

fn infallible<T>(result: Result<T, Infallible>) -> T {
    match result {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use merra_core::{
        CalendarConfig, PopulationConfigV1, SCENARIO_SCHEMA_V1, ScenarioV1, SeasonConfigV1,
    };
    use merra_sim::{SimulationReport, run_years};

    use super::{View, snapshot, snapshot_view};

    #[test]
    fn snapshot_is_plain_reviewable_text() -> Result<(), Box<dyn std::error::Error>> {
        let scenario = ScenarioV1 {
            schema_version: SCENARIO_SCHEMA_V1,
            id: String::from("tui-test"),
            title: String::from("TUI Test"),
            calendar: CalendarConfig {
                days_per_year: 360,
                seasons: vec![SeasonConfigV1 {
                    id: String::from("year"),
                    name: String::from("Year"),
                    days: 360,
                }],
            },
            population: PopulationConfigV1 {
                initial_people: 0,
                minimum_starting_age: 0,
                maximum_starting_age: 0,
                mortality_bands: Vec::new(),
            },
        };
        let screen = snapshot(run_years(scenario.clone(), 42, 1)?, 100, 30);

        assert!(screen.contains("MERRA // FIRST HUNDRED YEARS"));
        assert!(screen.contains("Population 0 → 0 living"));
        assert!(screen.contains("simulation started"));
        assert!(!screen.contains('\u{1b}'));
        let people_screen = snapshot_view(run_years(scenario, 42, 1)?, 100, 30, View::People);
        assert!(people_screen.contains("People"));
        assert!(people_screen.contains("No person selected."));
        Ok(())
    }

    #[test]
    fn canonical_century_views_match_golden_screens() -> Result<(), Box<dyn std::error::Error>> {
        let report = century_report()?;

        assert_eq!(
            snapshot_view(report.clone(), 120, 36, View::Events),
            include_str!("../../../golden/era-01/century-seed-42/tui-events.txt")
        );
        assert_eq!(
            snapshot_view(report, 120, 36, View::People),
            include_str!("../../../golden/era-01/century-seed-42/tui-people.txt")
        );
        Ok(())
    }

    fn century_report() -> Result<SimulationReport, Box<dyn std::error::Error>> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let bytes = std::fs::read(root.join("scenarios/era-01/century.ron"))?;
        let scenario: ScenarioV1 = ron::de::from_bytes(&bytes)?;
        Ok(run_years(scenario, 42, 100)?)
    }
}
