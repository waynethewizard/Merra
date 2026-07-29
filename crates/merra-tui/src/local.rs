//! Story-first ANSI-free views for five-settlement local history.

use std::collections::BTreeMap;

use merra_core::{
    CultureId, EventId, FaithId, HouseholdHistoricalContextV1, HouseholdId, InstitutionId,
    LocalHistoryReportV1, LocalSettlementRecordV1, LocationId, PersonId, ResidenceDecisionV1,
    ResidenceReasonV1, RouteId,
};

/// Collections available in the local-history inspector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalView {
    /// Consequence-first summary of all five settlements.
    Overview,
    /// Honest shortest-path road diagram and matrix.
    Roads,
    /// Comparable vital and movement statistics by settlement.
    Settlements,
    /// Explainable household residence choices.
    Migrations,
    /// Household-level historical inheritance.
    Households,
}

impl LocalView {
    /// Advances to the next local-history collection.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Overview => Self::Roads,
            Self::Roads => Self::Settlements,
            Self::Settlements => Self::Migrations,
            Self::Migrations => Self::Households,
            Self::Households => Self::Overview,
        }
    }
}

/// Stateful navigation for interactive and snapshot local-history views.
#[derive(Clone, Debug)]
pub struct LocalInspector {
    report: LocalHistoryReportV1,
    view: LocalView,
    selection: usize,
    location_filter: Option<LocationId>,
}

impl LocalInspector {
    /// Creates an inspector on the overview.
    #[must_use]
    pub const fn new(report: LocalHistoryReportV1) -> Self {
        Self {
            report,
            view: LocalView::Overview,
            selection: 0,
            location_filter: None,
        }
    }

    /// Returns the active view.
    #[must_use]
    pub const fn view(&self) -> LocalView {
        self.view
    }

    /// Selects a view and restores a stable valid row.
    pub fn set_view(&mut self, view: LocalView) {
        self.view = view;
        self.clamp_selection();
    }

    /// Cycles the active view.
    pub fn toggle_view(&mut self) {
        self.set_view(self.view.next());
    }

    /// Moves down one row.
    pub fn next(&mut self) {
        let count = self.row_count();
        if count > 0 {
            self.selection = (self.selection + 1).min(count - 1);
        }
    }

    /// Moves up one row.
    pub fn previous(&mut self) {
        self.selection = self.selection.saturating_sub(1);
    }

    /// Focuses a stable settlement identity.
    pub fn focus_location(&mut self, location_id: LocationId) -> bool {
        let Some(index) = self
            .report
            .settlements
            .iter()
            .position(|settlement| settlement.location_id == location_id)
        else {
            return false;
        };
        self.view = LocalView::Settlements;
        self.selection = index;
        self.location_filter = Some(location_id);
        true
    }

    /// Focuses a stable household identity.
    pub fn focus_household(&mut self, household_id: HouseholdId) -> bool {
        let contexts = self.visible_contexts();
        let Some(index) = contexts
            .iter()
            .position(|context| context.household_id == household_id)
        else {
            self.location_filter = None;
            let all = self.report.household_contexts.iter().collect::<Vec<_>>();
            let Some(index) = all
                .iter()
                .position(|context| context.household_id == household_id)
            else {
                return false;
            };
            self.view = LocalView::Households;
            self.selection = index;
            return true;
        };
        self.view = LocalView::Households;
        self.selection = index;
        true
    }

    /// Opens the households resident in the selected settlement.
    pub fn activate(&mut self) {
        if self.view != LocalView::Settlements {
            return;
        }
        if let Some(settlement) = self.report.settlements.get(self.selection) {
            self.location_filter = Some(settlement.location_id);
            self.view = LocalView::Households;
            self.selection = 0;
        }
    }

    /// Removes a settlement filter from the household collection.
    pub fn clear_filter(&mut self) {
        self.location_filter = None;
        self.clamp_selection();
    }

    fn row_count(&self) -> usize {
        match self.view {
            LocalView::Overview | LocalView::Roads => 1,
            LocalView::Settlements => self.report.settlements.len(),
            LocalView::Migrations => self
                .report
                .residence_decisions
                .iter()
                .filter(|decision| decision_moved(decision))
                .count(),
            LocalView::Households => self.visible_contexts().len(),
        }
    }

    fn visible_contexts(&self) -> Vec<&HouseholdHistoricalContextV1> {
        self.report
            .household_contexts
            .iter()
            .filter(|context| {
                self.location_filter
                    .is_none_or(|location| context.residence_id == location)
            })
            .collect()
    }

    fn clamp_selection(&mut self) {
        self.selection = self.selection.min(self.row_count().saturating_sub(1));
    }
}

/// Renders the active local-history screen without ANSI control codes.
#[must_use]
pub fn render_local_snapshot(inspector: &LocalInspector, width: u16, height: u16) -> String {
    if width < 60 || height < 16 {
        return fit_screen(
            vec![
                String::from("MERRA // FIVE VILLAGES"),
                String::new(),
                format!("Terminal is {width}×{height}; use at least 60×16."),
            ],
            width,
            height,
        );
    }
    let mut lines = header(inspector);
    match inspector.view {
        LocalView::Overview => render_overview(inspector, &mut lines),
        LocalView::Roads => render_roads(inspector, &mut lines),
        LocalView::Settlements => render_settlements(inspector, &mut lines),
        LocalView::Migrations => render_migrations(inspector, &mut lines),
        LocalView::Households => render_households(inspector, &mut lines),
    }
    lines.push(String::new());
    lines.push(String::from(
        "Tab/1–5 view · ↑↓ select · Enter households · x clear filter · q quit",
    ));
    fit_screen(lines, width, height)
}

fn header(inspector: &LocalInspector) -> Vec<String> {
    let report = &inspector.report;
    let tabs = [
        (LocalView::Overview, "1 Overview"),
        (LocalView::Roads, "2 Roads"),
        (LocalView::Settlements, "3 Settlements"),
        (LocalView::Migrations, "4 Migrations"),
        (LocalView::Households, "5 Households"),
    ]
    .into_iter()
    .map(|(view, label)| {
        if view == inspector.view {
            format!("[{label}]")
        } else {
            label.to_owned()
        }
    })
    .collect::<Vec<_>>()
    .join("  ");
    vec![
        format!("MERRA // {}", report.title.to_uppercase()),
        format!(
            "YEAR {} + {} · seed {} · {} macro people represented exactly",
            report.summary.projection_year,
            report.summary.elapsed_years,
            report.seed,
            report.summary.represented_population,
        ),
        tabs,
        "─".repeat(120),
    ]
}

fn render_overview(inspector: &LocalInspector, lines: &mut Vec<String>) {
    let report = &inspector.report;
    let strongest = report.settlements.iter().max_by_key(|settlement| {
        i64::from(settlement.final_living_people) - i64::from(settlement.initial_sample_people)
    });
    let weakest = report.settlements.iter().min_by_key(|settlement| {
        (
            i64::from(settlement.final_living_people) - i64::from(settlement.initial_sample_people),
            settlement.final_living_people,
        )
    });
    lines.push(String::from("THE LOCAL CONSEQUENCE"));
    if let (Some(strongest), Some(weakest)) = (strongest, weakest) {
        lines.push(format!(
            "{} grew {}→{} while {} changed {}→{}{}.",
            strongest.name,
            strongest.initial_sample_people,
            strongest.final_living_people,
            weakest.name,
            weakest.initial_sample_people,
            weakest.final_living_people,
            if weakest.final_living_people == 0 {
                " and emptied"
            } else {
                ""
            }
        ));
    }
    lines.push(format!(
        "{} living sample · {} births · {} deaths · {} household migrations · {}/{} events located",
        report.summary.living_sample_people,
        report.summary.births,
        report.summary.deaths,
        report.summary.household_migrations,
        report.summary.located_events,
        report.events.len(),
    ));
    lines.push(String::new());
    lines.push(String::from(
        "SETTLEMENT       MACRO     SAMPLE  Δ    B/D    IN/OUT   LIVING POPULATION",
    ));
    let maximum = report
        .settlements
        .iter()
        .map(|settlement| settlement.final_living_people)
        .max()
        .unwrap_or(1);
    for settlement in &report.settlements {
        lines.push(settlement_row(settlement, maximum));
    }
    lines.push(String::new());
    let kin = count_reason(report, ResidenceReasonV1::LivingKin);
    let road = count_reason(report, ResidenceReasonV1::ShortestJourney);
    let tie = count_reason(report, ResidenceReasonV1::SeededTieBreak);
    lines.push(String::from("WHY HOUSEHOLDS MOVED"));
    lines.push(format!(
        "{kin} kin-led · {road} road-led · {tie} seeded ties · one residence per household"
    ));
    lines.push(format!(
        "{} claims · {} inherited institutions · {} shortest-path connections",
        report.lore.len(),
        report.institutions.len(),
        report.connections.len(),
    ));
    if let Some(claim) = report.lore.first() {
        lines.push(format!("Inherited claim: “{}”", claim.title));
    }
}

fn render_roads(inspector: &LocalInspector, lines: &mut Vec<String>) {
    let report = &inspector.report;
    let names = location_names(report);
    let Some(anchor) = report.settlements.first() else {
        return;
    };
    lines.push(format!(
        "ROADS // SHORTEST PATHS FROM {} #{}",
        anchor.name, anchor.location_id.0
    ));
    lines.push(String::from(
        "Costs come from the historically available world graph; paths may cross intermediate places.",
    ));
    for settlement in report.settlements.iter().skip(1) {
        if let Some(connection) = connection(report, anchor.location_id, settlement.location_id) {
            lines.push(format!(
                "├─ {:>3} cost / {:>3}d / {} segment(s) ─ {} #{}",
                connection.travel_cost,
                connection.travel_days,
                connection.route_ids.len(),
                settlement.name,
                settlement.location_id.0,
            ));
        }
    }
    lines.push(String::new());
    lines.push(String::from("PAIRWISE TRAVEL COST"));
    let labels = report
        .settlements
        .iter()
        .map(|settlement| short_name(&settlement.name))
        .collect::<Vec<_>>();
    lines.push(format!(
        "{:<11}{}",
        "",
        labels
            .iter()
            .map(|label| format!("{label:>9}"))
            .collect::<String>()
    ));
    for (row, from) in report.settlements.iter().enumerate() {
        let cells = report
            .settlements
            .iter()
            .map(|to| {
                if from.location_id == to.location_id {
                    String::from("—")
                } else {
                    connection(report, from.location_id, to.location_id)
                        .map_or_else(|| String::from("?"), |value| value.travel_cost.to_string())
                }
            })
            .map(|cell| format!("{cell:>9}"))
            .collect::<String>();
        lines.push(format!("{:<11}{cells}", labels[row]));
    }
    if let Some(longest) = report
        .connections
        .iter()
        .max_by_key(|connection| connection.travel_cost)
    {
        let path = longest
            .path
            .iter()
            .map(|location| {
                names
                    .get(location)
                    .cloned()
                    .unwrap_or_else(|| format!("#{}", location.0))
            })
            .collect::<Vec<_>>()
            .join(" → ");
        lines.push(String::new());
        lines.push(format!(
            "Longest: {path} · {} cost · {} days",
            longest.travel_cost, longest.travel_days
        ));
    }
}

fn render_settlements(inspector: &LocalInspector, lines: &mut Vec<String>) {
    let report = &inspector.report;
    lines.push(String::from(
        "SETTLEMENTS // MACRO PROVENANCE AND LOCAL CONSEQUENCES",
    ));
    lines.push(String::from(
        "   PLACE             MACRO=REP   START→LIVE   BIRTH DEATH   IN OUT   HOMES",
    ));
    for (index, settlement) in report.settlements.iter().enumerate() {
        lines.push(format!(
            "{}  {:<17} {:>6}={:<6} {:>3}→{:<3}      {:>3}   {:>3}  {:>3} {:>3}   {:>3}",
            if index == inspector.selection {
                ">"
            } else {
                " "
            },
            format!("{} #{}", settlement.name, settlement.location_id.0),
            settlement.macro_population,
            settlement.represented_population,
            settlement.initial_sample_people,
            settlement.final_living_people,
            settlement.births,
            settlement.deaths,
            settlement.arrivals,
            settlement.departures,
            settlement.active_households,
        ));
    }
    if let Some(selected) = report.settlements.get(inspector.selection) {
        lines.push(String::new());
        lines.push(format!(
            "SELECTED // {} #{}",
            selected.name, selected.location_id.0
        ));
        let context_count = report
            .household_contexts
            .iter()
            .filter(|context| context.residence_id == selected.location_id)
            .count();
        lines.push(format!(
            "{context_count} historical households · {} local institution(s) · macro events {}",
            selected.institution_ids.len(),
            id_list(&selected.historical_event_ids),
        ));
        lines.push(format!(
            "Population equation: {} represented = {} aggregate",
            selected.represented_population, selected.macro_population
        ));
        if selected.final_living_people == 0 {
            lines.push(String::from(
                "No sampled household remains: births and departures are still preserved as place history.",
            ));
        }
    }
}

fn render_migrations(inspector: &LocalInspector, lines: &mut Vec<String>) {
    let report = &inspector.report;
    let names = location_names(report);
    let decisions = report
        .residence_decisions
        .iter()
        .filter(|decision| decision_moved(decision))
        .collect::<Vec<_>>();
    lines.push(String::from(
        "MIGRATIONS // KIN FIRST, ROAD COST SECOND, SEEDED TIE LAST",
    ));
    lines.push(String::from(
        "   YEAR  HOUSEHOLD   ORIGIN                    → DESTINATION   WHY          KIN  COST/DAYS",
    ));
    let start = inspector.selection.saturating_sub(5);
    for (index, decision) in decisions.iter().enumerate().skip(start).take(14) {
        lines.push(migration_row(
            decision,
            index == inspector.selection,
            &names,
            report.simulation_summary.days_per_year,
        ));
    }
    if let Some(decision) = decisions.get(inspector.selection) {
        lines.push(String::new());
        lines.push(format!(
            "CAUSES {} · ROUTES {} · TRAVELERS {}",
            id_list(&decision.causes),
            id_list(&decision.route_ids),
            id_list(&decision.traveler_ids)
        ));
    }
}

fn render_households(inspector: &LocalInspector, lines: &mut Vec<String>) {
    let report = &inspector.report;
    let names = location_names(report);
    let contexts = inspector.visible_contexts();
    let filter = inspector.location_filter.map_or_else(
        || String::from("all places"),
        |location| {
            names
                .get(&location)
                .cloned()
                .unwrap_or_else(|| format!("place #{}", location.0))
        },
    );
    lines.push(format!(
        "HOUSEHOLDS // HISTORICAL INHERITANCE · {filter} · {} shown",
        contexts.len()
    ));
    lines.push(String::from(
        "   HOUSEHOLD   RESIDENCE          MEMBERS  REPRESENTS  CULT  FAITH  INST  LORE",
    ));
    let start = inspector.selection.saturating_sub(5);
    for (index, context) in contexts.iter().enumerate().skip(start).take(14) {
        let household = report
            .households
            .iter()
            .find(|household| household.id == context.household_id);
        let members = household.map_or(0, |household| household.member_ids.len());
        let represented = context
            .represented_populations
            .iter()
            .map(|allocation| allocation.people)
            .sum::<u32>();
        lines.push(format!(
            "{}  #{:<9} {:<18} {:>7}  {:>10}  {:>4}  {:>5}  {:>4}  {:>4}",
            if index == inspector.selection {
                ">"
            } else {
                " "
            },
            context.household_id.0,
            names
                .get(&context.residence_id)
                .map_or("unknown", String::as_str),
            members,
            represented,
            context.culture_ids.len(),
            context.faith_ids.len(),
            context.institution_ids.len(),
            context.lore_claim_ids.len(),
        ));
    }
    if let Some(context) = contexts.get(inspector.selection) {
        lines.push(String::new());
        let institution_names = context
            .institution_ids
            .iter()
            .filter_map(|id| {
                report
                    .institutions
                    .iter()
                    .find(|institution| institution.id == *id)
                    .map(|institution| (institution.name.clone(), institution.id))
            })
            .fold(
                BTreeMap::<String, Vec<InstitutionId>>::new(),
                |mut groups, (name, id)| {
                    groups.entry(name).or_default().push(id);
                    groups
                },
            )
            .into_iter()
            .map(|(name, ids)| format!("{name} {}", id_list(&ids)))
            .collect::<Vec<_>>()
            .join(" · ");
        let lore_titles = context
            .lore_claim_ids
            .iter()
            .filter_map(|id| {
                report
                    .lore
                    .iter()
                    .find(|claim| claim.id == *id)
                    .map(|claim| claim.title.as_str())
            })
            .collect::<Vec<_>>()
            .join(" · ");
        lines.push(format!(
            "Household #{} · cultures {} · faiths {}",
            context.household_id.0,
            id_list(&context.culture_ids),
            id_list(&context.faith_ids),
        ));
        lines.push(format!(
            "Institutions: {}",
            if institution_names.is_empty() {
                "none"
            } else {
                &institution_names
            }
        ));
        lines.push(format!(
            "Claims: {}",
            if lore_titles.is_empty() {
                "none"
            } else {
                &lore_titles
            }
        ));
    }
}

fn settlement_row(settlement: &LocalSettlementRecordV1, maximum: u32) -> String {
    let delta =
        i64::from(settlement.final_living_people) - i64::from(settlement.initial_sample_people);
    let blocks = if settlement.final_living_people == 0 {
        0
    } else {
        u32::from(14_u16)
            .saturating_mul(settlement.final_living_people)
            .div_ceil(maximum.max(1))
    } as usize;
    format!(
        "{:<16} {:>6}  {:>3}→{:<3} {delta:>+4}  {:>2}/{:<2}  {:>2}/{:<2}   {}{}",
        settlement.name,
        settlement.macro_population,
        settlement.initial_sample_people,
        settlement.final_living_people,
        settlement.births,
        settlement.deaths,
        settlement.arrivals,
        settlement.departures,
        "█".repeat(blocks),
        if settlement.final_living_people == 0 {
            " EMPTY"
        } else {
            ""
        }
    )
}

fn migration_row(
    decision: &ResidenceDecisionV1,
    selected: bool,
    names: &BTreeMap<LocationId, String>,
    days_per_year: u16,
) -> String {
    let origin = decision
        .origin_location_ids
        .iter()
        .map(|location| {
            names
                .get(location)
                .cloned()
                .unwrap_or_else(|| format!("#{}", location.0))
        })
        .collect::<Vec<_>>()
        .join("+");
    let destination = names
        .get(&decision.destination_location_id)
        .cloned()
        .unwrap_or_else(|| format!("#{}", decision.destination_location_id.0));
    let journey = format!("{}/{}", decision.travel_cost, decision.travel_days);
    format!(
        "{}  {:>3}   #{:<9} {:<23} → {:<13} {:<12} {:>3}  {journey:>7}",
        if selected { ">" } else { " " },
        decision.settled_day / u64::from(days_per_year),
        decision.household_id.0,
        origin,
        destination,
        reason_label(decision.reason),
        decision.living_kin_support,
    )
}

fn count_reason(report: &LocalHistoryReportV1, reason: ResidenceReasonV1) -> usize {
    report
        .residence_decisions
        .iter()
        .filter(|decision| decision_moved(decision) && decision.reason == reason)
        .count()
}

fn decision_moved(decision: &ResidenceDecisionV1) -> bool {
    !decision.origin_location_ids.is_empty()
        && decision
            .origin_location_ids
            .iter()
            .any(|origin| *origin != decision.destination_location_id)
}

const fn reason_label(reason: ResidenceReasonV1) -> &'static str {
    match reason {
        ResidenceReasonV1::MacroProjection => "projection",
        ResidenceReasonV1::LivingKin => "living kin",
        ResidenceReasonV1::ShortestJourney => "road cost",
        ResidenceReasonV1::SeededTieBreak => "seeded tie",
    }
}

fn connection(
    report: &LocalHistoryReportV1,
    first: LocationId,
    second: LocationId,
) -> Option<&merra_core::LocalConnectionV1> {
    report.connections.iter().find(|connection| {
        (connection.from == first && connection.to == second)
            || (connection.from == second && connection.to == first)
    })
}

fn location_names(report: &LocalHistoryReportV1) -> BTreeMap<LocationId, String> {
    report
        .settlements
        .iter()
        .map(|settlement| (settlement.location_id, settlement.name.clone()))
        .collect()
}

fn short_name(name: &str) -> String {
    name.chars().take(8).collect()
}

fn id_list<T: RawId>(ids: &[T]) -> String {
    if ids.is_empty() {
        return String::from("none");
    }
    ids.iter()
        .map(|id| format!("#{}", id.raw()))
        .collect::<Vec<_>>()
        .join(",")
}

trait RawId {
    fn raw(&self) -> u64;
}

macro_rules! raw_id {
    ($($id:ty),+ $(,)?) => {
        $(
            impl RawId for $id {
                fn raw(&self) -> u64 {
                    self.0
                }
            }
        )+
    };
}

raw_id!(
    CultureId,
    EventId,
    FaithId,
    InstitutionId,
    PersonId,
    RouteId
);

fn fit_screen(lines: Vec<String>, width: u16, height: u16) -> String {
    let width = usize::from(width);
    let height = usize::from(height);
    let mut output = lines
        .into_iter()
        .take(height)
        .map(|line| {
            line.chars()
                .take(width)
                .collect::<String>()
                .trim_end()
                .to_owned()
        })
        .collect::<Vec<_>>();
    let footer = output
        .last()
        .is_some_and(|line| line.starts_with("Tab/"))
        .then(|| output.pop())
        .flatten();
    output.resize(
        height.saturating_sub(usize::from(footer.is_some())),
        String::new(),
    );
    if let Some(footer) = footer {
        output.push(footer);
    }
    output.join("\n") + "\n"
}
