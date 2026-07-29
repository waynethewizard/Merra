use std::convert::Infallible;

use merra_core::{
    EventKindV1, EventPayloadV1, HouseholdId, PersonId, PersonRecordV1, WorldEventV1,
};
use merra_sim::SimulationReport;
use ratatui::{
    Frame, Terminal,
    backend::TestBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap},
};

use crate::model::{
    Focus, HouseholdMoment, Inspector, PartnershipRecord, View, children_of, event_household_id,
    generation_stats, household_historical_members, household_moments, partnership_history,
    population_by_year, resolve_household, resolve_person, surname_stats,
};

/// Draws the complete terminal inspector.
pub fn render(frame: &mut Frame<'_>, inspector: &Inspector) {
    let area = frame.area();
    if area.height < 16 || area.width < 60 {
        render_too_small(frame, area, inspector);
        return;
    }
    let [header, tabs, body, footer] = Layout::vertical([
        Constraint::Length(5),
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(area);
    render_header(frame, header, inspector);
    render_tabs(frame, tabs, inspector);
    match inspector.view {
        View::Overview => render_overview(frame, body, inspector),
        View::History => render_history(frame, body, inspector),
        View::People => render_people(frame, body, inspector),
        View::Lineage => render_lineage(frame, body, inspector),
        View::Households => render_households(frame, body, inspector),
    }
    render_footer(frame, footer, inspector);
}

fn render_too_small(frame: &mut Frame<'_>, area: Rect, inspector: &Inspector) {
    let title = format!(
        "MERRA // {}",
        inspector.report.scenario_title.to_uppercase()
    );
    frame.render_widget(
        Paragraph::new(format!(
            "{title}\n\nTerminal is {}×{}; use at least 60×16.\nSeed {} · Year {}",
            area.width,
            area.height,
            inspector.report.summary.seed,
            inspector.report.summary.elapsed_years
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Chronicle Lab "),
        )
        .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_header(frame: &mut Frame<'_>, area: Rect, inspector: &Inspector) {
    let report = &inspector.report;
    let summary = &report.summary;
    let births = report
        .people
        .iter()
        .filter(|person| person.birth_day.is_some())
        .count();
    let population = if births == 0 {
        format!(
            "Population {} → {} living · {} deaths",
            summary.initial_population, summary.living_population, summary.deaths
        )
    } else {
        format!(
            "Population {} + {births} born → {} living · {} deaths",
            summary.initial_population, summary.living_population, summary.deaths
        )
    };
    let household = if report.households.is_empty() || area.width < 92 {
        String::new()
    } else {
        let active = report
            .households
            .iter()
            .filter(|household| household.dissolved_day.is_none())
            .count();
        format!(
            " · Households {} formed / {active} active",
            report.households.len()
        )
    };
    let text = Text::from(vec![
        Line::from(vec![
            Span::styled(
                format!("MERRA // {}", report.scenario_title.to_uppercase()),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("   seed {}", summary.seed)),
        ]),
        Line::from(format!(
            "Scenario {} · Year {} · {} authoritative events",
            summary.scenario_id, summary.elapsed_years, summary.event_count
        )),
        Line::from(format!("{population}{household}")),
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
        View::Overview => 0,
        View::History => 1,
        View::People => 2,
        View::Lineage => 3,
        View::Households => 4,
    };
    let titles = ["Overview", "History", "People", "Lineage", "Households"]
        .into_iter()
        .enumerate()
        .map(|(index, title)| {
            if index == selected {
                format!("[{title}]")
            } else {
                title.to_owned()
            }
        });
    frame.render_widget(
        Tabs::new(titles)
            .select(selected)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .divider(" │ "),
        area,
    );
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, inspector: &Inspector) {
    let footer = if let Some(input) = &inspector.search_input {
        format!("SEARCH / {input}_   Enter apply · Esc cancel · Backspace edit")
    } else if area.width < 92 {
        match inspector.view {
            View::Overview => String::from("q quit · Tab/1–5 views · Enter featured lineage"),
            View::History => format!(
                "q quit · ↑↓ move · f filter ({}) · / search",
                inspector.event_filter.label()
            ),
            View::People => String::from("q quit · ↑↓ move · Enter lineage · / search"),
            View::Lineage => String::from("q quit · ↑↓ person · h household · e events"),
            View::Households => String::from("q quit · ↑↓ household · Enter lineage · e events"),
        }
    } else {
        match inspector.view {
            View::Overview => String::from("q quit · Tab/1–5 views · Enter featured lineage"),
            View::History => format!(
                "q quit · Tab/1–5 views · ↑↓ move · Enter inspect · f filter ({}) · / search",
                inspector.event_filter.label()
            ),
            View::People => format!(
                "q quit · ↑↓ move · Enter lineage · h household · e events · s sort ({}) · / search",
                inspector.person_sort.label()
            ),
            View::Lineage => format!(
                "q quit · ↑↓ person · h household · e events · s sort ({}) · / search",
                inspector.person_sort.label()
            ),
            View::Households => format!(
                "q quit · ↑↓ move · Enter member lineage · e events · s sort ({}) · / search",
                inspector.household_sort.label()
            ),
        }
    };
    frame.render_widget(
        Paragraph::new(footer).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}

fn render_overview(frame: &mut Frame<'_>, area: Rect, inspector: &Inspector) {
    if area.width < 92 || area.height < 22 {
        render_compact_overview(frame, area, inspector);
        return;
    }
    let [world, middle, featured] = Layout::vertical([
        Constraint::Length(5),
        Constraint::Min(8),
        Constraint::Length(8),
    ])
    .areas(area);
    render_world_result(frame, world, inspector);
    let [generations, surnames] =
        Layout::horizontal([Constraint::Percentage(46), Constraint::Percentage(54)]).areas(middle);
    render_generations(frame, generations, inspector);
    render_surnames(frame, surnames, inspector);
    render_featured_life(frame, featured, inspector);
}

fn render_world_result(frame: &mut Frame<'_>, area: Rect, inspector: &Inspector) {
    let report = &inspector.report;
    let births = report
        .people
        .iter()
        .filter(|person| person.birth_day.is_some())
        .count();
    let active = report
        .households
        .iter()
        .filter(|household| household.dissolved_day.is_none())
        .count();
    let ended = report.households.len().saturating_sub(active);
    let population = population_by_year(report);
    let peak = population.iter().copied().max().unwrap_or(0);
    let peak_year = population
        .iter()
        .position(|value| *value == peak)
        .unwrap_or(0);
    let text = Text::from(vec![
        Line::from(format!(
            "{} people recorded · {} founders · {births} births · {} living · {} dead",
            report.people.len(),
            report.summary.initial_population,
            report.summary.living_population,
            report.summary.deaths,
        )),
        Line::from(format!(
            "{} households formed · {active} active · {ended} ended · peak population {peak} in Year {peak_year}",
            report.households.len()
        )),
        Line::from(format!(
            "Population: {}  {}",
            sparkline(&population, 52),
            population.last().map_or_else(
                || String::from("no observations"),
                |value| format!("{value} at Year {}", report.summary.elapsed_years)
            )
        )),
    ]);
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" World at Year {} ", report.summary.elapsed_years)),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_generations(frame: &mut Frame<'_>, area: Rect, inspector: &Inspector) {
    let stats = generation_stats(&inspector.report);
    let maximum = stats.iter().map(|stat| stat.total).max().unwrap_or(1);
    let mut lines = Vec::new();
    for stat in stats {
        lines.push(Line::from(format!(
            "G{}  {}  {:>2}/{:<2} living · {} total",
            stat.generation,
            bar(stat.living, maximum, 18),
            stat.living,
            stat.total,
            stat.total
        )));
    }
    let historical = inspector
        .report
        .events
        .iter()
        .filter(|event| {
            !matches!(
                event.kind,
                EventKindV1::SimulationStarted
                    | EventKindV1::TimeAdvanced
                    | EventKindV1::SeasonBegan
                    | EventKindV1::SimulationCompleted
            )
        })
        .count();
    lines.push(Line::from(""));
    lines.push(Line::from(format!(
        "{historical} historical events · {} clock/season/debug events",
        inspector.report.events.len().saturating_sub(historical)
    )));
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Generations "),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_surnames(frame: &mut Frame<'_>, area: Rect, inspector: &Inspector) {
    let available = usize::from(area.height.saturating_sub(2));
    let mut lines = Vec::new();
    for stat in surname_stats(&inspector.report).into_iter().take(available) {
        let fate = if stat.living == 0 {
            String::from("EXTINCT")
        } else {
            format!("{} living", stat.living)
        };
        let generations = if stat.minimum_generation == stat.maximum_generation {
            format!("G{} only", stat.minimum_generation)
        } else {
            format!("G{}–{}", stat.minimum_generation, stat.maximum_generation)
        };
        lines.push(Line::from(format!(
            "{:<10} {:>2} people · {:<10} · {generations}",
            stat.surname, stat.total, fate
        )));
    }
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Surname Survival "),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_featured_life(frame: &mut Frame<'_>, area: Rect, inspector: &Inspector) {
    let Some(person) = inspector.report.people.get(inspector.selected_person) else {
        frame.render_widget(
            Paragraph::new("No person exists in this history.").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Featured Life "),
            ),
            area,
        );
        return;
    };
    let partnerships = partnership_history(&inspector.report, person.id);
    let children = children_of(&inspector.report, person.id);
    let household_count = partnerships
        .iter()
        .map(|partnership| partnership.household_id)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let mut lines = vec![Line::from(vec![
        Span::styled(
            format!("{}  #{}", person.name, person.id.0),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            " · G{} · age {} · {} · {} partnerships · {} children · {household_count} households",
            person.generation,
            person.final_age_years,
            if person.alive { "living" } else { "dead" },
            partnerships.len(),
            children.len()
        )),
    ])];
    if partnerships.is_empty() {
        lines.push(Line::from("No partnerships were recorded."));
    } else {
        for partnership in partnerships.iter().take(3) {
            lines.push(Line::from(partnership_summary(
                &inspector.report,
                partnership,
            )));
        }
    }
    lines.push(Line::from(
        "Enter opens the union-aware lineage; parentage remains attached to each partnership.",
    ));
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Featured Life "),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_compact_overview(frame: &mut Frame<'_>, area: Rect, inspector: &Inspector) {
    let report = &inspector.report;
    let births = report
        .people
        .iter()
        .filter(|person| person.birth_day.is_some())
        .count();
    let active = report
        .households
        .iter()
        .filter(|household| household.dissolved_day.is_none())
        .count();
    let generations = generation_stats(report)
        .iter()
        .map(|stat| format!("G{} {}/{}", stat.generation, stat.living, stat.total))
        .collect::<Vec<_>>()
        .join(" · ");
    let surnames = surname_stats(report);
    let surname_line = surnames
        .iter()
        .filter(|stat| stat.surname == "Fen" || stat.surname == "Thorn" || stat.surname == "Gorse")
        .map(|stat| {
            format!(
                "{} {} ({})",
                stat.surname,
                stat.total,
                if stat.living == 0 {
                    String::from("extinct")
                } else {
                    format!("{} living", stat.living)
                }
            )
        })
        .collect::<Vec<_>>()
        .join(" · ");
    let featured = report.people.get(inspector.selected_person);
    let mut lines = vec![
        Line::from(format!(
            "{} founders + {births} births = {} people · {} living · {} dead",
            report.summary.initial_population,
            report.people.len(),
            report.summary.living_population,
            report.summary.deaths
        )),
        Line::from(format!(
            "{} households · {active} active · {} ended",
            report.households.len(),
            report.households.len().saturating_sub(active)
        )),
        Line::from(generations),
    ];
    if !surname_line.is_empty() {
        lines.push(Line::from(surname_line));
    }
    if let Some(person) = featured {
        lines.push(Line::from(format!(
            "Featured: {} · {} partnerships · {} children",
            person.name,
            partnership_history(report, person.id).len(),
            children_of(report, person.id).len()
        )));
        for partnership in partnership_history(report, person.id).iter().take(3) {
            lines.push(Line::from(partnership_summary(report, partnership)));
        }
    }
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" World at Year {} ", report.summary.elapsed_years)),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_history(frame: &mut Frame<'_>, area: Rect, inspector: &Inspector) {
    let (list_area, detail_area) = collection_areas(area);
    let indices = inspector.visible_event_indices();
    let items: Vec<_> = indices
        .iter()
        .filter_map(|index| inspector.report.events.get(*index))
        .map(|event| {
            ListItem::new(format!(
                "{:>4}  Y{:>3}  {}",
                event.id.0,
                year(&inspector.report, event.time.day()),
                event_list_label(&inspector.report, event)
            ))
        })
        .collect();
    let selected = indices
        .iter()
        .position(|index| *index == inspector.selected_event);
    let mut state = ListState::default().with_selected(selected);
    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title(format!(
                " History · {} · {} shown{} ",
                inspector.event_filter.label(),
                indices.len(),
                query_suffix(inspector)
            )))
            .highlight_symbol("▶ ")
            .highlight_style(Style::default().fg(Color::Yellow)),
        list_area,
        &mut state,
    );
    let detail = inspector
        .report
        .events
        .get(inspector.selected_event)
        .map_or_else(
            || String::from("No event matches the current filter."),
            |event| event_detail(&inspector.report, event),
        );
    frame.render_widget(
        Paragraph::new(detail)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Resolved Event Evidence "),
            )
            .wrap(Wrap { trim: false }),
        detail_area,
    );
}

fn render_people(frame: &mut Frame<'_>, area: Rect, inspector: &Inspector) {
    let (list_area, detail_area) = collection_areas(area);
    let indices = inspector.visible_person_indices();
    let items: Vec<_> = indices
        .iter()
        .filter_map(|index| inspector.report.people.get(*index))
        .map(|person| {
            ListItem::new(format!(
                "{:>3}  G{}  {:<20} age {:>3}  {}",
                person.id.0,
                person.generation,
                person.name,
                person.final_age_years,
                if person.alive { "living" } else { "dead" }
            ))
        })
        .collect();
    let selected = indices
        .iter()
        .position(|index| *index == inspector.selected_person);
    let mut state = ListState::default().with_selected(selected);
    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title(format!(
                " People · sort {} · {} shown{} ",
                inspector.person_sort.label(),
                indices.len(),
                query_suffix(inspector)
            )))
            .highlight_symbol("▶ ")
            .highlight_style(Style::default().fg(Color::Yellow)),
        list_area,
        &mut state,
    );
    let detail = inspector
        .report
        .people
        .get(inspector.selected_person)
        .map_or_else(
            || String::from("No person matches the current search."),
            |person| person_detail(&inspector.report, person),
        );
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

fn render_lineage(frame: &mut Frame<'_>, area: Rect, inspector: &Inspector) {
    let (list_area, detail_area) = collection_areas(area);
    let indices = inspector.visible_person_indices();
    let items: Vec<_> = indices
        .iter()
        .filter_map(|index| inspector.report.people.get(*index))
        .map(|person| {
            ListItem::new(format!(
                "G{}  {:>3}  {:<17} {:>2} unions · {:>2} kids",
                person.generation,
                person.id.0,
                person.name,
                partnership_history(&inspector.report, person.id).len(),
                children_of(&inspector.report, person.id).len()
            ))
        })
        .collect();
    let selected = indices
        .iter()
        .position(|index| *index == inspector.selected_person);
    let mut state = ListState::default().with_selected(selected);
    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title(format!(
                " Lineages · sort {} · {} shown{} ",
                inspector.person_sort.label(),
                indices.len(),
                query_suffix(inspector)
            )))
            .highlight_symbol("▶ ")
            .highlight_style(Style::default().fg(Color::Yellow)),
        list_area,
        &mut state,
    );
    let detail = inspector
        .report
        .people
        .get(inspector.selected_person)
        .map_or_else(
            || String::from("No person matches the current search."),
            |person| lineage_detail(&inspector.report, person),
        );
    frame.render_widget(
        Paragraph::new(detail)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Union-Aware Lineage "),
            )
            .wrap(Wrap { trim: false }),
        detail_area,
    );
}

fn render_households(frame: &mut Frame<'_>, area: Rect, inspector: &Inspector) {
    let (list_area, detail_area) = collection_areas(area);
    let indices = inspector.visible_household_indices();
    let items: Vec<_> = indices
        .iter()
        .filter_map(|index| inspector.report.households.get(*index))
        .map(|household| {
            ListItem::new(format!(
                "{:>3}  {:<16} {:>2} now · {:>2} born · {}",
                household.id.0,
                household.name,
                household.member_ids.len(),
                household.children_born,
                if household.dissolved_day.is_some() {
                    "ended"
                } else {
                    "active"
                }
            ))
        })
        .collect();
    let selected = indices
        .iter()
        .position(|index| *index == inspector.selected_household);
    let mut state = ListState::default().with_selected(selected);
    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title(format!(
                " Households · sort {} · {} shown{} ",
                inspector.household_sort.label(),
                indices.len(),
                query_suffix(inspector)
            )))
            .highlight_symbol("▶ ")
            .highlight_style(Style::default().fg(Color::Yellow)),
        list_area,
        &mut state,
    );
    let detail = inspector
        .report
        .households
        .get(inspector.selected_household)
        .map_or_else(
            || String::from("No household matches the current search."),
            |household| household_detail(&inspector.report, household.id),
        );
    frame.render_widget(
        Paragraph::new(detail)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Household History "),
            )
            .wrap(Wrap { trim: false }),
        detail_area,
    );
}

fn person_detail(report: &SimulationReport, person: &PersonRecordV1) -> String {
    let born = person.birth_day.map_or_else(
        || String::from("Scenario founder"),
        |day| format!("Born Year {} · Day {day}", year(report, day)),
    );
    let ending = person.death_day.map_or_else(
        || format!("Living at Year {}", report.summary.elapsed_years),
        |day| format!("Died Year {} · Day {day}", year(report, day)),
    );
    let parents = if person.parent_ids.is_empty() {
        String::from("scenario founder")
    } else {
        person
            .parent_ids
            .iter()
            .map(|parent| with_id(report, *parent))
            .collect::<Vec<_>>()
            .join(" + ")
    };
    let current_partner = person.partner_id.map_or_else(
        || String::from("none"),
        |partner| format!("{} (current)", with_id(report, partner)),
    );
    let household = person.household_id.map_or_else(
        || String::from("none"),
        |household| {
            format!(
                "{} (#{} current)",
                resolve_household(report, household),
                household.0
            )
        },
    );
    let partnerships = partnership_history(report, person.id);
    let children = children_of(report, person.id);
    let mut lines = vec![
        person.name.clone(),
        String::new(),
        format!(
            "Person #{} · Generation {} · surname {}",
            person.id.0, person.generation, person.surname
        ),
        format!("{born} · {ending} · final age {}", person.final_age_years),
        format!("Parents: {parents}"),
        format!("Current partner: {current_partner}"),
        format!("Current household: {household}"),
        format!(
            "Recorded family: {} partnerships · {} children",
            partnerships.len(),
            children.len()
        ),
        String::new(),
        String::from("Life events"),
    ];
    for event in report
        .events
        .iter()
        .filter(|event| event.actors.contains(&person.id))
        .filter(|event| {
            matches!(
                event.kind,
                EventKindV1::PersonBorn
                    | EventKindV1::PersonDied
                    | EventKindV1::HouseholdFormed
                    | EventKindV1::PartnershipFormed
                    | EventKindV1::PartnershipEnded
            )
        })
        .take(10)
    {
        lines.push(format!(
            "Y{:>2}  {}",
            year(report, event.time.day()),
            event_short_label(report, event)
        ));
    }
    lines.join("\n")
}

fn lineage_detail(report: &SimulationReport, person: &PersonRecordV1) -> String {
    let status = if person.alive {
        format!("living at age {}", person.final_age_years)
    } else {
        format!("died at age {}", person.final_age_years)
    };
    let parents = if person.parent_ids.is_empty() {
        String::from("scenario founder")
    } else {
        person
            .parent_ids
            .iter()
            .map(|parent| with_id(report, *parent))
            .collect::<Vec<_>>()
            .join(" + ")
    };
    let partnerships = partnership_history(report, person.id);
    let mut lines = vec![
        format!("{}  #{}  [{status}]", person.name, person.id.0),
        format!("Generation {} · Parents: {parents}", person.generation),
        String::new(),
        String::from("PARTNERSHIPS AND THEIR CHILDREN"),
    ];
    if partnerships.is_empty() {
        lines.push(String::from("No partnership was recorded."));
    } else {
        for partnership in &partnerships {
            lines.push(partnership_line(report, partnership));
            lines.push(format!(
                "  Household: {} (#{}).",
                resolve_household(report, partnership.household_id),
                partnership.household_id.0
            ));
            lines.push(format!(
                "  Children: {}",
                child_names(report, &partnership.children)
            ));
        }
    }
    let matched_children: std::collections::BTreeSet<_> = partnerships
        .iter()
        .flat_map(|partnership| partnership.children.iter().copied())
        .collect();
    let unmatched: Vec<_> = children_of(report, person.id)
        .into_iter()
        .filter(|child| !matched_children.contains(&child.id))
        .collect();
    if !unmatched.is_empty() {
        lines.push(String::from("Other parentage evidence:"));
        for child in unmatched {
            let other = child
                .parent_ids
                .iter()
                .find(|parent| **parent != person.id)
                .map_or_else(
                    || String::from("unknown"),
                    |parent| with_id(report, *parent),
                );
            lines.push(format!("  {} #{} with {other}", child.name, child.id.0));
        }
    }
    lines.push(String::new());
    lines.push(String::from("DESCENDANT BRANCHES"));
    let children = children_of(report, person.id);
    if children.is_empty() {
        lines.push(String::from("No children were recorded."));
    } else {
        for child in children {
            lines.push(format!(
                "├─ {} #{} · G{} · {} · {} children",
                child.name,
                child.id.0,
                child.generation,
                if child.alive { "living" } else { "dead" },
                children_of(report, child.id).len()
            ));
        }
    }
    lines.join("\n")
}

fn household_detail(report: &SimulationReport, household_id: HouseholdId) -> String {
    let Some(household) = report
        .households
        .iter()
        .find(|household| household.id == household_id)
    else {
        return format!("Household #{} is missing.", household_id.0);
    };
    let status = household.dissolved_day.map_or_else(
        || format!("active at Year {}", report.summary.elapsed_years),
        |day| format!("dissolved Year {} · Day {day}", year(report, day)),
    );
    let current = if household.member_ids.is_empty() {
        String::from("none")
    } else {
        household
            .member_ids
            .iter()
            .map(|person| with_id(report, *person))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let historical = household_historical_members(report, household_id)
        .into_iter()
        .map(|person| with_id(report, person))
        .collect::<Vec<_>>()
        .join(", ");
    let mut lines = vec![
        format!("{}  #{}  [{status}]", household.name, household.id.0),
        format!(
            "Surname {} · founded Year {} · {} children born",
            household.surname,
            year(report, household.founded_day),
            household.children_born
        ),
        format!("Current members: {current}"),
        format!("All recorded members: {historical}"),
        String::new(),
        String::from("HOUSEHOLD HISTORY"),
    ];
    for moment in household_moments(report, household_id).into_iter().take(14) {
        lines.push(household_moment_line(report, moment));
    }
    lines.join("\n")
}

fn household_moment_line(report: &SimulationReport, moment: HouseholdMoment) -> String {
    match moment {
        HouseholdMoment::Formed { day, member_ids } => format!(
            "Y{:>2}  Formed with {}",
            year(report, day),
            member_ids
                .iter()
                .map(|person| with_id(report, *person))
                .collect::<Vec<_>>()
                .join(" + ")
        ),
        HouseholdMoment::Born { day, person_id } => format!(
            "Y{:>2}  {} was born into the household",
            year(report, day),
            with_id(report, person_id)
        ),
        HouseholdMoment::Departed {
            day,
            person_id,
            destination_id,
        } => format!(
            "Y{:>2}  {} left for {} #{}",
            year(report, day),
            with_id(report, person_id),
            resolve_household(report, destination_id),
            destination_id.0
        ),
        HouseholdMoment::Died { day, person_id } => format!(
            "Y{:>2}  {} died while resident",
            year(report, day),
            with_id(report, person_id)
        ),
        HouseholdMoment::Dissolved { day } => {
            format!("Y{:>2}  Household dissolved", year(report, day))
        }
    }
}

pub(crate) fn event_short_label(report: &SimulationReport, event: &WorldEventV1) -> String {
    match &event.payload {
        EventPayloadV1::SimulationStarted { .. } => String::from("simulation started"),
        EventPayloadV1::PopulationInitialized { people } => {
            format!("{people} people entered the history")
        }
        EventPayloadV1::TimeAdvanced { to_day, .. } => {
            format!("clock advanced to Day {to_day}")
        }
        EventPayloadV1::SeasonBegan {
            season_name, year, ..
        } => format!("{season_name} began in Year {year}"),
        EventPayloadV1::HouseholdFormed {
            household_id,
            name,
            member_ids,
            ..
        } => format!(
            "{name} #{} formed: {}",
            household_id.0,
            member_ids
                .iter()
                .map(|person| resolve_person(report, *person))
                .collect::<Vec<_>>()
                .join(" + ")
        ),
        EventPayloadV1::PartnershipFormed {
            household_id,
            partners,
        } => format!(
            "{} + {} partnered in {} #{}",
            resolve_person(report, partners[0]),
            resolve_person(report, partners[1]),
            resolve_household(report, *household_id),
            household_id.0
        ),
        EventPayloadV1::PartnershipEnded {
            partners,
            deceased_id,
        } => format!(
            "{} + {} ended; {} died",
            resolve_person(report, partners[0]),
            resolve_person(report, partners[1]),
            resolve_person(report, *deceased_id)
        ),
        EventPayloadV1::PersonBorn {
            name,
            parent_ids,
            generation,
            ..
        } => format!(
            "{name} born G{generation} to {} + {}",
            resolve_person(report, parent_ids[0]),
            resolve_person(report, parent_ids[1])
        ),
        EventPayloadV1::HouseholdDissolved { household_id, name } => {
            format!("{name} #{} dissolved", household_id.0)
        }
        EventPayloadV1::HouseholdSettled {
            household_id,
            destination_location_id,
            living_kin_support,
            ..
        } => format!(
            "household #{} settled at place #{} · {} living kin",
            household_id.0, destination_location_id.0, living_kin_support
        ),
        EventPayloadV1::PersonDied {
            name, age_years, ..
        } => format!("{name} died at age {age_years}"),
        EventPayloadV1::SimulationCompleted { .. } => String::from("simulation completed"),
    }
}

fn event_list_label(report: &SimulationReport, event: &WorldEventV1) -> String {
    match &event.payload {
        EventPayloadV1::HouseholdFormed {
            household_id, name, ..
        } => {
            format!("{name} #{} formed", household_id.0)
        }
        EventPayloadV1::PartnershipFormed { partners, .. } => format!(
            "{} + {} partnered",
            resolve_person(report, partners[0]),
            resolve_person(report, partners[1])
        ),
        EventPayloadV1::PartnershipEnded {
            partners,
            deceased_id,
        } => format!(
            "{} + {} ended ({})",
            resolve_person(report, partners[0]),
            resolve_person(report, partners[1]),
            resolve_person(report, *deceased_id)
        ),
        EventPayloadV1::PersonBorn {
            name, generation, ..
        } => format!("{name} born · G{generation}"),
        EventPayloadV1::PersonDied {
            name, age_years, ..
        } => format!("{name} died · age {age_years}"),
        _ => event_short_label(report, event),
    }
}

fn event_detail(report: &SimulationReport, event: &WorldEventV1) -> String {
    let actors = if event.actors.is_empty() {
        String::from("none")
    } else {
        event
            .actors
            .iter()
            .map(|actor| with_id(report, *actor))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let causes = if event.causes.is_empty() {
        String::from("none")
    } else {
        event
            .causes
            .iter()
            .map(|cause| {
                report
                    .events
                    .iter()
                    .find(|candidate| candidate.id == *cause)
                    .map_or_else(
                        || format!("#{} missing", cause.0),
                        |candidate| {
                            format!("#{} {}", cause.0, event_short_label(report, candidate))
                        },
                    )
            })
            .collect::<Vec<_>>()
            .join("\n  ")
    };
    let mut lines = vec![
        event_short_label(report, event),
        String::new(),
        format!(
            "Event #{} · {} · Year {} · Day {}",
            event.id.0,
            event_kind_label(event.kind),
            year(report, event.time.day()),
            event.time.day()
        ),
        format!("People: {actors}"),
    ];
    if let Some(household_id) = event_household_id(event) {
        lines.push(format!(
            "Household: {} #{}",
            resolve_household(report, household_id),
            household_id.0
        ));
    }
    lines.extend(payload_evidence(report, event));
    lines.push(String::new());
    lines.push(format!("Caused by:\n  {causes}"));
    lines.push(format!("Tags: {}", event.tags.join(", ")));
    lines.push(String::new());
    lines.push(String::from(
        "Authoritative world evidence; later records and memories may reinterpret it.",
    ));
    lines.join("\n")
}

fn payload_evidence(report: &SimulationReport, event: &WorldEventV1) -> Vec<String> {
    match &event.payload {
        EventPayloadV1::SimulationStarted { scenario_id, seed } => {
            vec![format!("Scenario {scenario_id} · root seed {seed}")]
        }
        EventPayloadV1::PopulationInitialized { people } => {
            vec![format!("Initialized people: {people}")]
        }
        EventPayloadV1::TimeAdvanced {
            from_day,
            to_day,
            elapsed_days,
        } => vec![format!(
            "Clock: Day {from_day} → Day {to_day} · {elapsed_days} elapsed"
        )],
        EventPayloadV1::SeasonBegan {
            season_id,
            season_name,
            year,
        } => vec![format!("Season: {season_name} ({season_id}) · Year {year}")],
        EventPayloadV1::HouseholdFormed {
            surname,
            member_ids,
            ..
        } => vec![
            format!("Inherited child surname: {surname}"),
            format!(
                "Founding members: {}",
                member_ids
                    .iter()
                    .map(|person| with_id(report, *person))
                    .collect::<Vec<_>>()
                    .join(" + ")
            ),
        ],
        EventPayloadV1::PartnershipFormed { partners, .. } => vec![format!(
            "Partners: {} + {}",
            with_id(report, partners[0]),
            with_id(report, partners[1])
        )],
        EventPayloadV1::PartnershipEnded {
            partners,
            deceased_id,
        } => vec![
            format!(
                "Former partners: {} + {}",
                with_id(report, partners[0]),
                with_id(report, partners[1])
            ),
            format!(
                "Death ending partnership: {}",
                with_id(report, *deceased_id)
            ),
        ],
        EventPayloadV1::PersonBorn {
            person_id,
            parent_ids,
            generation,
            ..
        } => vec![
            format!("Child: {}", with_id(report, *person_id)),
            format!(
                "Actual parents: {} + {}",
                with_id(report, parent_ids[0]),
                with_id(report, parent_ids[1])
            ),
            format!("Generation: {generation}"),
        ],
        EventPayloadV1::HouseholdDissolved { .. } => {
            vec![String::from("No living members remained.")]
        }
        EventPayloadV1::HouseholdSettled {
            origin_location_ids,
            destination_location_id,
            route_ids,
            travel_cost,
            travel_days,
            living_kin_support,
            reason,
            ..
        } => vec![
            format!(
                "Residence: {:?} → #{}",
                origin_location_ids, destination_location_id.0
            ),
            format!(
                "Journey: {travel_cost} cost · {travel_days} days · {} route(s)",
                route_ids.len()
            ),
            format!("Support: {living_kin_support} living kin · {reason:?}"),
        ],
        EventPayloadV1::PersonDied {
            person_id,
            age_years,
            annual_deaths_per_10_000,
            ..
        } => vec![
            format!("Person: {}", with_id(report, *person_id)),
            format!("Complete age: {age_years}"),
            format!("Annual mortality threshold: {annual_deaths_per_10_000} / 10,000"),
        ],
        EventPayloadV1::SimulationCompleted {
            final_day,
            elapsed_years,
        } => vec![format!(
            "Final day {final_day} · {elapsed_years} complete years"
        )],
    }
}

fn partnership_summary(report: &SimulationReport, partnership: &PartnershipRecord) -> String {
    let end = partnership.ended_day.map_or_else(
        || String::from("present"),
        |day| format!("Y{}", year(report, day)),
    );
    let current = if partnership.current {
        " · CURRENT"
    } else {
        ""
    };
    format!(
        "Y{}–{end}  {} · {} · {} children{current}",
        year(report, partnership.started_day),
        resolve_person(report, partnership.partner_id),
        resolve_household(report, partnership.household_id),
        partnership.children.len()
    )
}

fn partnership_line(report: &SimulationReport, partnership: &PartnershipRecord) -> String {
    let end = partnership.ended_day.map_or_else(
        || String::from("present"),
        |day| format!("Y{}", year(report, day)),
    );
    let ending = partnership.deceased_id.map_or_else(
        || {
            if partnership.current {
                String::from("CURRENT PARTNER")
            } else {
                String::from("open")
            }
        },
        |deceased| format!("ended when {} died", resolve_person(report, deceased)),
    );
    format!(
        "Y{}–{end}  {} #{}  [{ending}]",
        year(report, partnership.started_day),
        resolve_person(report, partnership.partner_id),
        partnership.partner_id.0
    )
}

fn child_names(report: &SimulationReport, children: &[PersonId]) -> String {
    if children.is_empty() {
        String::from("none")
    } else {
        children
            .iter()
            .map(|child| with_id(report, *child))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn with_id(report: &SimulationReport, person_id: PersonId) -> String {
    format!("{} #{}", resolve_person(report, person_id), person_id.0)
}

fn event_kind_label(kind: EventKindV1) -> &'static str {
    match kind {
        EventKindV1::SimulationStarted => "simulation started",
        EventKindV1::PopulationInitialized => "population initialized",
        EventKindV1::TimeAdvanced => "time advanced",
        EventKindV1::SeasonBegan => "season began",
        EventKindV1::HouseholdFormed => "household formed",
        EventKindV1::PartnershipFormed => "partnership formed",
        EventKindV1::PartnershipEnded => "partnership ended",
        EventKindV1::PersonBorn => "person born",
        EventKindV1::HouseholdDissolved => "household dissolved",
        EventKindV1::HouseholdSettled => "household settled",
        EventKindV1::PersonDied => "person died",
        EventKindV1::SimulationCompleted => "simulation completed",
    }
}

fn collection_areas(area: Rect) -> (Rect, Rect) {
    if area.width >= 92 {
        let [list, detail] =
            Layout::horizontal([Constraint::Percentage(43), Constraint::Percentage(57)])
                .areas(area);
        (list, detail)
    } else {
        let [list, detail] =
            Layout::vertical([Constraint::Percentage(30), Constraint::Percentage(70)]).areas(area);
        (list, detail)
    }
}

fn query_suffix(inspector: &Inspector) -> String {
    if inspector.query.is_empty() {
        String::new()
    } else {
        format!(" · search “{}”", inspector.query)
    }
}

fn year(report: &SimulationReport, day: u64) -> u64 {
    day / u64::from(report.summary.days_per_year)
}

fn bar(value: usize, maximum: usize, width: usize) -> String {
    if maximum == 0 || width == 0 {
        return String::new();
    }
    let filled = value.saturating_mul(width).div_ceil(maximum).min(width);
    format!("{}{}", "█".repeat(filled), "·".repeat(width - filled))
}

fn sparkline(values: &[usize], width: usize) -> String {
    const GLYPHS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if values.is_empty() || width == 0 {
        return String::new();
    }
    let maximum = values.iter().copied().max().unwrap_or(1).max(1);
    let sample_count = width.min(values.len());
    let mut output = String::with_capacity(sample_count);
    for sample in 0..sample_count {
        let index = if sample_count == 1 {
            0
        } else {
            sample.saturating_mul(values.len().saturating_sub(1)) / sample_count.saturating_sub(1)
        };
        let value = values.get(index).copied().unwrap_or(0);
        let glyph = value
            .saturating_mul(GLYPHS.len().saturating_sub(1))
            .div_ceil(maximum)
            .min(GLYPHS.len().saturating_sub(1));
        output.push(GLYPHS[glyph]);
    }
    output
}

/// Renders a portable, ANSI-free Overview screen.
#[must_use]
pub fn snapshot(report: SimulationReport, width: u16, height: u16) -> String {
    snapshot_view(report, width, height, View::Overview)
}

/// Renders a selected view with its derived meaningful default focus.
#[must_use]
pub fn snapshot_view(report: SimulationReport, width: u16, height: u16, view: View) -> String {
    snapshot_view_with_focus(report, width, height, view, None)
}

/// Renders a selected view focused on an optional stable identity.
#[must_use]
pub fn snapshot_view_with_focus(
    report: SimulationReport,
    width: u16,
    height: u16,
    view: View,
    focus: Option<Focus>,
) -> String {
    let mut inspector = Inspector::new(report);
    inspector.set_view(view);
    if let Some(focus) = focus {
        inspector.focus(focus);
    }
    render_snapshot(&inspector, width, height)
}

/// Renders the exact current inspector state as portable text.
#[must_use]
pub fn render_snapshot(inspector: &Inspector, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = infallible(Terminal::new(backend));
    infallible(terminal.draw(|frame| render(frame, inspector)));
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
