use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
};

use merra_core::{
    EventId, EventKindV1, EventPayloadV1, HouseholdId, PersonId, PersonRecordV1, WorldEventV1,
};
use merra_sim::SimulationReport;

/// Inspectable evidence collections.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum View {
    /// Derived world-level outcomes and a featured human story.
    Overview,
    /// Ordered historical events, with an optional debug stream.
    History,
    /// Searchable and sortable final person records.
    People,
    /// Focused ancestry, partnership, and descendant evidence.
    Lineage,
    /// Household membership, outcomes, and reconstructed history.
    Households,
}

impl View {
    pub(crate) const fn next(self) -> Self {
        match self {
            Self::Overview => Self::History,
            Self::History => Self::People,
            Self::People => Self::Lineage,
            Self::Lineage => Self::Households,
            Self::Households => Self::Overview,
        }
    }
}

/// Timeline filters from story-first evidence through the complete debug stream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventFilter {
    /// Population, family, household, birth, and death history.
    Historical,
    /// Births and deaths only.
    Lives,
    /// Household and partnership changes only.
    Households,
    /// Every event, including clock and season mechanics.
    All,
}

impl EventFilter {
    pub(crate) const fn next(self) -> Self {
        match self {
            Self::Historical => Self::Lives,
            Self::Lives => Self::Households,
            Self::Households => Self::All,
            Self::All => Self::Historical,
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Historical => "historical",
            Self::Lives => "lives",
            Self::Households => "households",
            Self::All => "all / debug",
        }
    }

    const fn includes(self, kind: EventKindV1) -> bool {
        match self {
            Self::Historical => !matches!(
                kind,
                EventKindV1::SimulationStarted
                    | EventKindV1::TimeAdvanced
                    | EventKindV1::SeasonBegan
                    | EventKindV1::SimulationCompleted
            ),
            Self::Lives => matches!(
                kind,
                EventKindV1::PopulationInitialized
                    | EventKindV1::PersonBorn
                    | EventKindV1::PersonDied
            ),
            Self::Households => matches!(
                kind,
                EventKindV1::HouseholdFormed
                    | EventKindV1::PartnershipFormed
                    | EventKindV1::PartnershipEnded
                    | EventKindV1::HouseholdDissolved
            ),
            Self::All => true,
        }
    }
}

/// Person ordering used by People and Lineage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonSort {
    /// People with the richest recorded family histories first.
    Story,
    /// Stable identity order.
    Identity,
    /// Display-name order.
    Name,
    /// Generation, then stable identity.
    Generation,
    /// Greatest final age first.
    Age,
}

impl PersonSort {
    pub(crate) const fn next(self) -> Self {
        match self {
            Self::Story => Self::Identity,
            Self::Identity => Self::Name,
            Self::Name => Self::Generation,
            Self::Generation => Self::Age,
            Self::Age => Self::Story,
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Story => "story",
            Self::Identity => "identity",
            Self::Name => "name",
            Self::Generation => "generation",
            Self::Age => "age",
        }
    }
}

/// Household ordering used by the Household view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HouseholdSort {
    /// Active and larger households first.
    Status,
    /// Stable identity order.
    Identity,
    /// Household-name order.
    Name,
    /// Greatest current membership first.
    Size,
}

impl HouseholdSort {
    pub(crate) const fn next(self) -> Self {
        match self {
            Self::Status => Self::Identity,
            Self::Identity => Self::Name,
            Self::Name => Self::Size,
            Self::Size => Self::Status,
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Identity => "identity",
            Self::Name => "name",
            Self::Size => "size",
        }
    }
}

/// Optional stable-identity target for portable snapshots.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Focus {
    /// Focus one historical event.
    Event(EventId),
    /// Focus one person.
    Person(PersonId),
    /// Focus one household.
    Household(HouseholdId),
}

/// Navigable terminal-inspector state.
pub struct Inspector {
    pub(crate) report: SimulationReport,
    pub(crate) view: View,
    pub(crate) selected_event: usize,
    pub(crate) selected_person: usize,
    pub(crate) selected_household: usize,
    pub(crate) event_filter: EventFilter,
    pub(crate) person_sort: PersonSort,
    pub(crate) household_sort: HouseholdSort,
    pub(crate) query: String,
    pub(crate) search_input: Option<String>,
}

impl Inspector {
    /// Creates a story-first inspector focused on a derived notable person.
    #[must_use]
    pub fn new(report: SimulationReport) -> Self {
        let selected_person = featured_person_index(&report).unwrap_or(0);
        let selected_event = curated_event_index(&report, selected_person).unwrap_or(0);
        let selected_household = report
            .people
            .get(selected_person)
            .and_then(|person| person.household_id)
            .and_then(|id| {
                report
                    .households
                    .iter()
                    .position(|household| household.id == id)
            })
            .or_else(|| {
                report
                    .households
                    .iter()
                    .position(|household| household.dissolved_day.is_none())
            })
            .unwrap_or(0);
        Self {
            report,
            view: View::Overview,
            selected_event,
            selected_person,
            selected_household,
            event_filter: EventFilter::Historical,
            person_sort: PersonSort::Story,
            household_sort: HouseholdSort::Status,
            query: String::new(),
            search_input: None,
        }
    }

    /// Returns the current evidence view.
    #[must_use]
    pub const fn view(&self) -> View {
        self.view
    }

    /// Selects a specific evidence view.
    pub fn set_view(&mut self, view: View) {
        self.view = view;
        self.normalize_selection();
    }

    /// Switches to the next evidence view.
    pub fn toggle_view(&mut self) {
        self.set_view(self.view.next());
    }

    /// Selects the preceding visible row.
    pub fn previous(&mut self) {
        self.move_selection(-1);
    }

    /// Selects the following visible row.
    pub fn next(&mut self) {
        self.move_selection(1);
    }

    /// Moves upward by ten visible rows.
    pub fn page_up(&mut self) {
        self.move_selection(-10);
    }

    /// Moves downward by ten visible rows.
    pub fn page_down(&mut self) {
        self.move_selection(10);
    }

    /// Selects the first visible row.
    pub fn first(&mut self) {
        self.select_boundary(false);
    }

    /// Selects the last visible row.
    pub fn last(&mut self) {
        self.select_boundary(true);
    }

    /// Cycles the story/debug filter in the History view.
    pub fn cycle_event_filter(&mut self) {
        self.event_filter = self.event_filter.next();
        self.normalize_selection();
    }

    /// Cycles the meaningful sort for the current collection.
    pub fn cycle_sort(&mut self) {
        match self.view {
            View::People | View::Lineage => self.person_sort = self.person_sort.next(),
            View::Households => self.household_sort = self.household_sort.next(),
            View::Overview | View::History => return,
        }
        self.normalize_selection();
    }

    /// Begins editing the active collection's text filter.
    pub fn begin_search(&mut self) {
        if self.view != View::Overview {
            self.search_input = Some(self.query.clone());
        }
    }

    /// Returns whether keystrokes currently edit a search query.
    #[must_use]
    pub const fn is_searching(&self) -> bool {
        self.search_input.is_some()
    }

    /// Appends a character to the in-progress query.
    pub fn push_search_char(&mut self, character: char) {
        if let Some(input) = &mut self.search_input
            && !character.is_control()
        {
            input.push(character);
        }
    }

    /// Removes one character from the in-progress query.
    pub fn pop_search_char(&mut self) {
        if let Some(input) = &mut self.search_input {
            input.pop();
        }
    }

    /// Applies the in-progress query.
    pub fn accept_search(&mut self) {
        if let Some(input) = self.search_input.take() {
            self.query = input.trim().to_owned();
            self.normalize_selection();
        }
    }

    /// Discards edits to the in-progress query.
    pub fn cancel_search(&mut self) {
        self.search_input = None;
    }

    /// Clears the active text filter.
    pub fn clear_search(&mut self) {
        self.search_input = None;
        self.query.clear();
        self.normalize_selection();
    }

    /// Follows the selected record into the most useful related view.
    pub fn activate(&mut self) {
        match self.view {
            View::History => {
                let Some(event) = self.report.events.get(self.selected_event) else {
                    return;
                };
                let person_id = event.actors.first().copied();
                let household_id = event_household_id(event);
                if let Some(person_id) = person_id
                    && self.focus(Focus::Person(person_id))
                {
                    self.view = View::Lineage;
                    self.query.clear();
                    return;
                }
                if let Some(household_id) = household_id
                    && self.focus(Focus::Household(household_id))
                {
                    self.view = View::Households;
                    self.query.clear();
                }
            }
            View::People => self.view = View::Lineage,
            View::Households => {
                let Some(person_id) = self
                    .report
                    .households
                    .get(self.selected_household)
                    .and_then(|household| household.member_ids.first())
                    .copied()
                    .or_else(|| {
                        self.report
                            .households
                            .get(self.selected_household)
                            .and_then(|household| {
                                household_historical_members(&self.report, household.id)
                                    .first()
                                    .copied()
                            })
                    })
                else {
                    return;
                };
                if self.focus(Focus::Person(person_id)) {
                    self.view = View::Lineage;
                    self.query.clear();
                }
            }
            View::Overview | View::Lineage => self.view = View::Lineage,
        }
        self.normalize_selection();
    }

    /// Jumps from a person or household to its first related historical event.
    pub fn jump_to_related_event(&mut self) {
        let event_index = match self.view {
            View::People | View::Lineage => {
                self.report
                    .people
                    .get(self.selected_person)
                    .and_then(|person| {
                        self.report.events.iter().position(|event| {
                            self.event_filter.includes(event.kind)
                                && event.actors.contains(&person.id)
                        })
                    })
            }
            View::Households => self
                .report
                .households
                .get(self.selected_household)
                .and_then(|household| {
                    self.report
                        .events
                        .iter()
                        .position(|event| event_household_id(event) == Some(household.id))
                }),
            View::Overview | View::History => None,
        };
        if let Some(index) = event_index {
            self.selected_event = index;
            self.event_filter = EventFilter::Historical;
            self.view = View::History;
            self.query.clear();
            self.normalize_selection();
        }
    }

    /// Jumps from the selected person to their current household.
    pub fn jump_to_household(&mut self) {
        let Some(household_id) = self
            .report
            .people
            .get(self.selected_person)
            .and_then(|person| {
                person.household_id.or_else(|| {
                    partnership_history(&self.report, person.id)
                        .last()
                        .map(|partnership| partnership.household_id)
                })
            })
        else {
            return;
        };
        if self.focus(Focus::Household(household_id)) {
            self.view = View::Households;
            self.query.clear();
            self.normalize_selection();
        }
    }

    /// Focuses a stable record identity when it exists.
    pub fn focus(&mut self, focus: Focus) -> bool {
        match focus {
            Focus::Event(id) => {
                let Some(index) = self.report.events.iter().position(|event| event.id == id) else {
                    return false;
                };
                self.selected_event = index;
                if !self.event_filter.includes(self.report.events[index].kind) {
                    self.event_filter = EventFilter::All;
                }
            }
            Focus::Person(id) => {
                let Some(index) = self.report.people.iter().position(|person| person.id == id)
                else {
                    return false;
                };
                self.selected_person = index;
            }
            Focus::Household(id) => {
                let Some(index) = self
                    .report
                    .households
                    .iter()
                    .position(|household| household.id == id)
                else {
                    return false;
                };
                self.selected_household = index;
            }
        }
        true
    }

    pub(crate) fn visible_event_indices(&self) -> Vec<usize> {
        let query = normalized_query(&self.query);
        self.report
            .events
            .iter()
            .enumerate()
            .filter(|(_, event)| self.event_filter.includes(event.kind))
            .filter(|(_, event)| {
                query.is_empty() || event_search_text(&self.report, event).contains(&query)
            })
            .map(|(index, _)| index)
            .collect()
    }

    pub(crate) fn visible_person_indices(&self) -> Vec<usize> {
        let query = normalized_query(&self.query);
        let mut indices: Vec<_> = self
            .report
            .people
            .iter()
            .enumerate()
            .filter(|(_, person)| query.is_empty() || person_search_text(person).contains(&query))
            .map(|(index, _)| index)
            .collect();
        match self.person_sort {
            PersonSort::Story => indices.sort_by_key(|index| {
                let person = &self.report.people[*index];
                (
                    Reverse(partnership_count(&self.report, person.id)),
                    Reverse(children_of(&self.report, person.id).len()),
                    Reverse(person.final_age_years),
                    person.id,
                )
            }),
            PersonSort::Identity => {
                indices.sort_by_key(|index| self.report.people[*index].id);
            }
            PersonSort::Name => indices.sort_by_key(|index| {
                (
                    self.report.people[*index].name.to_lowercase(),
                    self.report.people[*index].id,
                )
            }),
            PersonSort::Generation => indices.sort_by_key(|index| {
                (
                    self.report.people[*index].generation,
                    self.report.people[*index].id,
                )
            }),
            PersonSort::Age => indices.sort_by_key(|index| {
                (
                    Reverse(self.report.people[*index].final_age_years),
                    self.report.people[*index].id,
                )
            }),
        }
        indices
    }

    pub(crate) fn visible_household_indices(&self) -> Vec<usize> {
        let query = normalized_query(&self.query);
        let mut indices: Vec<_> = self
            .report
            .households
            .iter()
            .enumerate()
            .filter(|(_, household)| {
                query.is_empty()
                    || household_search_text(&self.report, household.id).contains(&query)
            })
            .map(|(index, _)| index)
            .collect();
        match self.household_sort {
            HouseholdSort::Status => indices.sort_by_key(|index| {
                let household = &self.report.households[*index];
                (
                    household.dissolved_day.is_some(),
                    Reverse(household.member_ids.len()),
                    household.id,
                )
            }),
            HouseholdSort::Identity => {
                indices.sort_by_key(|index| self.report.households[*index].id);
            }
            HouseholdSort::Name => indices.sort_by_key(|index| {
                (
                    self.report.households[*index].name.to_lowercase(),
                    self.report.households[*index].id,
                )
            }),
            HouseholdSort::Size => indices.sort_by_key(|index| {
                (
                    Reverse(self.report.households[*index].member_ids.len()),
                    self.report.households[*index].id,
                )
            }),
        }
        indices
    }

    fn move_selection(&mut self, offset: isize) {
        match self.view {
            View::Overview => {}
            View::History => {
                let indices = self.visible_event_indices();
                self.selected_event = moved_selection(&indices, self.selected_event, offset);
            }
            View::People | View::Lineage => {
                let indices = self.visible_person_indices();
                self.selected_person = moved_selection(&indices, self.selected_person, offset);
            }
            View::Households => {
                let indices = self.visible_household_indices();
                self.selected_household =
                    moved_selection(&indices, self.selected_household, offset);
            }
        }
    }

    fn select_boundary(&mut self, last: bool) {
        match self.view {
            View::Overview => {}
            View::History => {
                if let Some(index) = boundary(&self.visible_event_indices(), last) {
                    self.selected_event = index;
                }
            }
            View::People | View::Lineage => {
                if let Some(index) = boundary(&self.visible_person_indices(), last) {
                    self.selected_person = index;
                }
            }
            View::Households => {
                if let Some(index) = boundary(&self.visible_household_indices(), last) {
                    self.selected_household = index;
                }
            }
        }
    }

    fn normalize_selection(&mut self) {
        match self.view {
            View::Overview => {}
            View::History => {
                let indices = self.visible_event_indices();
                if !indices.contains(&self.selected_event)
                    && let Some(index) = indices.first()
                {
                    self.selected_event = *index;
                }
            }
            View::People | View::Lineage => {
                let indices = self.visible_person_indices();
                if !indices.contains(&self.selected_person)
                    && let Some(index) = indices.first()
                {
                    self.selected_person = *index;
                }
            }
            View::Households => {
                let indices = self.visible_household_indices();
                if !indices.contains(&self.selected_household)
                    && let Some(index) = indices.first()
                {
                    self.selected_household = *index;
                }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GenerationStat {
    pub(crate) generation: u16,
    pub(crate) total: usize,
    pub(crate) living: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SurnameStat {
    pub(crate) surname: String,
    pub(crate) total: usize,
    pub(crate) living: usize,
    pub(crate) minimum_generation: u16,
    pub(crate) maximum_generation: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PartnershipRecord {
    pub(crate) partner_id: PersonId,
    pub(crate) household_id: HouseholdId,
    pub(crate) started_day: u64,
    pub(crate) ended_day: Option<u64>,
    pub(crate) deceased_id: Option<PersonId>,
    pub(crate) children: Vec<PersonId>,
    pub(crate) current: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum HouseholdMoment {
    Formed {
        day: u64,
        member_ids: Vec<PersonId>,
    },
    Born {
        day: u64,
        person_id: PersonId,
    },
    Departed {
        day: u64,
        person_id: PersonId,
        destination_id: HouseholdId,
    },
    Died {
        day: u64,
        person_id: PersonId,
    },
    Dissolved {
        day: u64,
    },
}

pub(crate) fn generation_stats(report: &SimulationReport) -> Vec<GenerationStat> {
    let mut stats = BTreeMap::<u16, (usize, usize)>::new();
    for person in &report.people {
        let entry = stats.entry(person.generation).or_default();
        entry.0 += 1;
        if person.alive {
            entry.1 += 1;
        }
    }
    stats
        .into_iter()
        .map(|(generation, (total, living))| GenerationStat {
            generation,
            total,
            living,
        })
        .collect()
}

pub(crate) fn surname_stats(report: &SimulationReport) -> Vec<SurnameStat> {
    let mut stats = BTreeMap::<String, Vec<&PersonRecordV1>>::new();
    for person in &report.people {
        stats
            .entry(person.surname.clone())
            .or_default()
            .push(person);
    }
    let mut stats: Vec<_> = stats
        .into_iter()
        .map(|(surname, people)| {
            let minimum_generation = people
                .iter()
                .map(|person| person.generation)
                .min()
                .unwrap_or(0);
            let maximum_generation = people
                .iter()
                .map(|person| person.generation)
                .max()
                .unwrap_or(0);
            SurnameStat {
                surname,
                total: people.len(),
                living: people.iter().filter(|person| person.alive).count(),
                minimum_generation,
                maximum_generation,
            }
        })
        .collect();
    stats.sort_by_key(|stat| (Reverse(stat.total), stat.surname.clone()));
    stats
}

pub(crate) fn population_by_year(report: &SimulationReport) -> Vec<usize> {
    let years = report.summary.elapsed_years as usize;
    let mut deltas = vec![0_isize; years.saturating_add(1)];
    for event in &report.events {
        let year = (event.time.day() / u64::from(report.summary.days_per_year)) as usize;
        if year >= deltas.len() {
            continue;
        }
        match event.kind {
            EventKindV1::PersonBorn => deltas[year] += 1,
            EventKindV1::PersonDied => deltas[year] -= 1,
            _ => {}
        }
    }
    let mut population = report.summary.initial_population as isize;
    let mut series = Vec::with_capacity(deltas.len());
    for (year, delta) in deltas.into_iter().enumerate() {
        if year > 0 {
            population = population.saturating_add(delta);
        }
        series.push(population.max(0) as usize);
    }
    series
}

pub(crate) fn featured_person_index(report: &SimulationReport) -> Option<usize> {
    report
        .people
        .iter()
        .enumerate()
        .max_by_key(|(_, person)| {
            (
                partnership_count(report, person.id),
                Reverse(person.generation),
                children_of(report, person.id).len(),
                person.final_age_years,
                Reverse(person.id),
            )
        })
        .map(|(index, _)| index)
}

pub(crate) fn partnership_history(
    report: &SimulationReport,
    person_id: PersonId,
) -> Vec<PartnershipRecord> {
    let current_partner = report
        .people
        .iter()
        .find(|person| person.id == person_id)
        .and_then(|person| person.partner_id);
    let mut records = Vec::new();
    for (event_index, event) in report.events.iter().enumerate() {
        let EventPayloadV1::PartnershipFormed {
            household_id,
            partners,
        } = &event.payload
        else {
            continue;
        };
        if !partners.contains(&person_id) {
            continue;
        }
        let partner_id = if partners[0] == person_id {
            partners[1]
        } else {
            partners[0]
        };
        let ending = report
            .events
            .iter()
            .skip(event_index.saturating_add(1))
            .find_map(|candidate| {
                let EventPayloadV1::PartnershipEnded {
                    partners: ended,
                    deceased_id,
                } = &candidate.payload
                else {
                    return None;
                };
                (ended == partners).then_some((candidate.time.day(), *deceased_id))
            });
        let mut children: Vec<_> = children_of(report, person_id)
            .into_iter()
            .filter(|child| child.parent_ids.contains(&partner_id))
            .map(|child| child.id)
            .collect();
        children.sort_unstable();
        records.push(PartnershipRecord {
            partner_id,
            household_id: *household_id,
            started_day: event.time.day(),
            ended_day: ending.map(|(day, _)| day),
            deceased_id: ending.map(|(_, deceased)| deceased),
            children,
            current: ending.is_none() && current_partner == Some(partner_id),
        });
    }
    records.sort_by_key(|record| (record.started_day, record.partner_id));
    records
}

pub(crate) fn household_moments(
    report: &SimulationReport,
    household_id: HouseholdId,
) -> Vec<HouseholdMoment> {
    let mut current_household = BTreeMap::<PersonId, HouseholdId>::new();
    let mut moments = Vec::new();
    for event in &report.events {
        match &event.payload {
            EventPayloadV1::HouseholdFormed {
                household_id: formed_id,
                member_ids,
                ..
            } => {
                for person_id in member_ids {
                    if let Some(previous_id) = current_household.insert(*person_id, *formed_id)
                        && previous_id == household_id
                        && previous_id != *formed_id
                    {
                        moments.push(HouseholdMoment::Departed {
                            day: event.time.day(),
                            person_id: *person_id,
                            destination_id: *formed_id,
                        });
                    }
                }
                if *formed_id == household_id {
                    moments.push(HouseholdMoment::Formed {
                        day: event.time.day(),
                        member_ids: member_ids.clone(),
                    });
                }
            }
            EventPayloadV1::PersonBorn {
                person_id,
                household_id: birth_household,
                ..
            } => {
                current_household.insert(*person_id, *birth_household);
                if *birth_household == household_id {
                    moments.push(HouseholdMoment::Born {
                        day: event.time.day(),
                        person_id: *person_id,
                    });
                }
            }
            EventPayloadV1::PersonDied { person_id, .. } => {
                if current_household.get(person_id) == Some(&household_id) {
                    moments.push(HouseholdMoment::Died {
                        day: event.time.day(),
                        person_id: *person_id,
                    });
                }
                current_household.remove(person_id);
            }
            EventPayloadV1::HouseholdDissolved {
                household_id: dissolved_id,
                ..
            } if *dissolved_id == household_id => {
                moments.push(HouseholdMoment::Dissolved {
                    day: event.time.day(),
                });
            }
            _ => {}
        }
    }
    moments
}

pub(crate) fn household_historical_members(
    report: &SimulationReport,
    household_id: HouseholdId,
) -> Vec<PersonId> {
    let mut members = BTreeSet::new();
    for moment in household_moments(report, household_id) {
        match moment {
            HouseholdMoment::Formed { member_ids, .. } => members.extend(member_ids),
            HouseholdMoment::Born { person_id, .. }
            | HouseholdMoment::Departed { person_id, .. }
            | HouseholdMoment::Died { person_id, .. } => {
                members.insert(person_id);
            }
            HouseholdMoment::Dissolved { .. } => {}
        }
    }
    members.into_iter().collect()
}

pub(crate) fn resolve_person(report: &SimulationReport, id: PersonId) -> String {
    report
        .people
        .iter()
        .find(|person| person.id == id)
        .map_or_else(|| format!("Person #{}", id.0), |person| person.name.clone())
}

pub(crate) fn resolve_household(report: &SimulationReport, id: HouseholdId) -> String {
    report
        .households
        .iter()
        .find(|household| household.id == id)
        .map_or_else(
            || format!("Household #{}", id.0),
            |household| household.name.clone(),
        )
}

pub(crate) fn children_of(report: &SimulationReport, person_id: PersonId) -> Vec<&PersonRecordV1> {
    let mut children: Vec<_> = report
        .people
        .iter()
        .filter(|person| person.parent_ids.contains(&person_id))
        .collect();
    children.sort_by_key(|child| (child.birth_day, child.id));
    children
}

pub(crate) fn partnership_count(report: &SimulationReport, person_id: PersonId) -> usize {
    report
        .events
        .iter()
        .filter(|event| {
            matches!(
                &event.payload,
                EventPayloadV1::PartnershipFormed { partners, .. }
                    if partners.contains(&person_id)
            )
        })
        .count()
}

pub(crate) fn event_household_id(event: &WorldEventV1) -> Option<HouseholdId> {
    match &event.payload {
        EventPayloadV1::HouseholdFormed { household_id, .. }
        | EventPayloadV1::PartnershipFormed { household_id, .. }
        | EventPayloadV1::PersonBorn { household_id, .. }
        | EventPayloadV1::HouseholdDissolved { household_id, .. } => Some(*household_id),
        _ => None,
    }
}

fn curated_event_index(report: &SimulationReport, featured_person: usize) -> Option<usize> {
    let featured_id = report.people.get(featured_person).map(|person| person.id);
    featured_id
        .and_then(|id| {
            report.events.iter().position(|event| {
                matches!(
                    event.kind,
                    EventKindV1::PersonBorn
                        | EventKindV1::PersonDied
                        | EventKindV1::PartnershipEnded
                ) && event.actors.contains(&id)
            })
        })
        .or_else(|| {
            report
                .events
                .iter()
                .position(|event| event.kind == EventKindV1::PersonBorn)
        })
        .or_else(|| {
            report
                .events
                .iter()
                .position(|event| event.kind == EventKindV1::PersonDied)
        })
        .or_else(|| {
            report
                .events
                .iter()
                .position(|event| EventFilter::Historical.includes(event.kind))
        })
}

fn normalized_query(query: &str) -> String {
    query.trim().to_lowercase()
}

fn person_search_text(person: &PersonRecordV1) -> String {
    format!(
        "{} {} {} g{} {}",
        person.id.0,
        person.name,
        person.surname,
        person.generation,
        if person.alive { "living" } else { "dead" }
    )
    .to_lowercase()
}

fn event_search_text(report: &SimulationReport, event: &WorldEventV1) -> String {
    let actors = event
        .actors
        .iter()
        .map(|id| resolve_person(report, *id))
        .collect::<Vec<_>>()
        .join(" ");
    let household = event_household_id(event)
        .map(|id| resolve_household(report, id))
        .unwrap_or_default();
    format!(
        "{} {:?} {} {} {}",
        event.id.0,
        event.kind,
        actors,
        household,
        event.tags.join(" ")
    )
    .to_lowercase()
}

fn household_search_text(report: &SimulationReport, household_id: HouseholdId) -> String {
    let Some(household) = report
        .households
        .iter()
        .find(|household| household.id == household_id)
    else {
        return String::new();
    };
    let members = household_historical_members(report, household_id)
        .into_iter()
        .map(|id| resolve_person(report, id))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "{} {} {} {} {}",
        household.id.0,
        household.name,
        household.surname,
        if household.dissolved_day.is_some() {
            "dissolved"
        } else {
            "active"
        },
        members
    )
    .to_lowercase()
}

fn moved_selection(indices: &[usize], current: usize, offset: isize) -> usize {
    let Some(first) = indices.first().copied() else {
        return current;
    };
    let current_position = indices
        .iter()
        .position(|index| *index == current)
        .unwrap_or(0);
    let maximum = indices.len().saturating_sub(1);
    let next_position = if offset.is_negative() {
        current_position.saturating_sub(offset.unsigned_abs())
    } else {
        current_position
            .saturating_add(offset as usize)
            .min(maximum)
    };
    indices.get(next_position).copied().unwrap_or(first)
}

fn boundary(indices: &[usize], last: bool) -> Option<usize> {
    if last {
        indices.last().copied()
    } else {
        indices.first().copied()
    }
}
