//! Shared fixtures and deterministic assertions for Merra tests.

use merra_core::{
    CalendarConfig, FamilyConfigV1, PopulationConfigV1, SCENARIO_SCHEMA_V1, ScenarioV1,
    SeasonConfigV1, SimDuration,
};
use merra_sim::{Simulation, SimulationError, SimulationReport};

/// Returns the canonical foundation smoke scenario.
#[must_use]
pub fn smoke_scenario() -> ScenarioV1 {
    ScenarioV1 {
        schema_version: SCENARIO_SCHEMA_V1,
        id: String::from("era-01-smoke"),
        title: String::from("The First Clock"),
        calendar: CalendarConfig {
            days_per_year: 360,
            seasons: vec![
                SeasonConfigV1 {
                    id: String::from("thaw"),
                    name: String::from("Thaw"),
                    days: 90,
                },
                SeasonConfigV1 {
                    id: String::from("bloom"),
                    name: String::from("Bloom"),
                    days: 90,
                },
                SeasonConfigV1 {
                    id: String::from("highsun"),
                    name: String::from("Highsun"),
                    days: 90,
                },
                SeasonConfigV1 {
                    id: String::from("emberfall"),
                    name: String::from("Emberfall"),
                    days: 90,
                },
            ],
        },
        population: PopulationConfigV1 {
            initial_people: 0,
            minimum_starting_age: 0,
            maximum_starting_age: 0,
            mortality_bands: Vec::new(),
        },
        family: FamilyConfigV1::default(),
    }
}

/// Runs the smoke scenario to completion without filesystem metadata.
pub fn run_smoke(seed: u64, years: u32) -> Result<SimulationReport, SimulationError> {
    let scenario = smoke_scenario();
    let duration = SimDuration::from_years(years, scenario.calendar.days_per_year);
    let mut simulation = Simulation::from_scenario(scenario, seed)?;
    simulation.advance(duration)?;
    simulation.finish()?;
    Ok(simulation.report())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet},
        fs,
        path::PathBuf,
    };

    use merra_core::{
        EventKindV1, EventPayloadV1, HouseholdId, PersonId, ScenarioV1, WorldEventV1,
    };
    use merra_sim::SimulationReport;

    use super::run_smoke;

    #[test]
    fn deterministic_serialization_is_byte_identical() -> Result<(), Box<dyn std::error::Error>> {
        let first = deterministic_bytes(&run_smoke(42, 1)?)?;
        let second = deterministic_bytes(&run_smoke(42, 1)?)?;

        assert_eq!(first, second);
        Ok(())
    }

    #[test]
    fn smoke_report_matches_golden_evidence() -> Result<(), Box<dyn std::error::Error>> {
        let report = run_smoke(42, 1)?;
        let root = golden_root();

        assert_eq!(events_jsonl(&report)?, fs::read(root.join("events.jsonl"))?);
        assert_eq!(
            serde_json::to_string_pretty(&report.summary)? + "\n",
            fs::read_to_string(root.join("summary.json"))?
        );
        assert_eq!(
            report.chronicle,
            fs::read_to_string(root.join("chronicle.md"))?
        );
        Ok(())
    }

    #[test]
    fn canonical_century_matches_golden_evidence() -> Result<(), Box<dyn std::error::Error>> {
        let scenario_bytes = fs::read(workspace_root().join("scenarios/era-01/century.ron"))?;
        let scenario: ScenarioV1 = ron::de::from_bytes(&scenario_bytes)?;
        let report = merra_sim::run_years(scenario, 42, 100)?;
        let golden = workspace_root().join("golden/era-01/century-seed-42");

        assert_eq!(report.people.len(), 100);
        assert_eq!(report.summary.deaths, 100);
        assert!(
            report
                .events
                .iter()
                .all(|event| { event.causes.iter().all(|cause| cause.0 < event.id.0) })
        );
        assert_eq!(
            serde_json::to_string_pretty(&report.summary)? + "\n",
            fs::read_to_string(golden.join("summary.json"))?
        );
        assert_eq!(
            report.chronicle,
            fs::read_to_string(golden.join("chronicle.md"))?
        );
        Ok(())
    }

    #[test]
    fn canonical_dynasty_matches_golden_evidence() -> Result<(), Box<dyn std::error::Error>> {
        let scenario_bytes = fs::read(workspace_root().join("scenarios/era-01/dynasty.ron"))?;
        let scenario: ScenarioV1 = ron::de::from_bytes(&scenario_bytes)?;
        let report = merra_sim::run_years(scenario.clone(), 42, 60)?;
        let repeated = merra_sim::run_years(scenario.clone(), 42, 60)?;
        let golden = workspace_root().join("golden/era-01/dynasty-seed-42");

        assert_eq!(report, repeated);
        assert_family_invariants(&report, &scenario);
        assert_eq!(report.people.len(), 65);
        assert_eq!(report.households.len(), 31);
        assert_eq!(
            report.people.iter().map(|person| person.generation).max(),
            Some(3)
        );
        assert!(
            report
                .households
                .iter()
                .all(|household| household.dissolved_day.is_some()
                    || !household.member_ids.is_empty())
        );
        assert!(
            report
                .events
                .iter()
                .all(|event| { event.causes.iter().all(|cause| cause.0 < event.id.0) })
        );
        for child in report
            .people
            .iter()
            .filter(|person| !person.parent_ids.is_empty())
        {
            let parent_generations: Vec<_> = child
                .parent_ids
                .iter()
                .filter_map(|parent_id| {
                    report
                        .people
                        .iter()
                        .find(|person| person.id == *parent_id)
                        .map(|parent| parent.generation)
                })
                .collect();
            assert_eq!(parent_generations.len(), 2);
            assert!(
                parent_generations
                    .iter()
                    .all(|generation| generation.saturating_add(1) == child.generation)
            );
        }
        for event in &report.events {
            if let EventPayloadV1::PersonBorn {
                person_id,
                household_id,
                ..
            } = &event.payload
            {
                assert!(event.actors.contains(person_id));
                assert!(event.causes.iter().any(|cause| {
                    report.events.iter().any(|candidate| {
                        candidate.id == *cause
                            && candidate.payload
                                == EventPayloadV1::SeasonBegan {
                                    season_id: String::from("thaw"),
                                    season_name: String::from("Thaw"),
                                    year: event.time.day() / 360,
                                }
                    })
                }));
                assert!(event.causes.iter().any(|cause| {
                    report.events.iter().any(|candidate| {
                        candidate.id == *cause
                            && matches!(
                                &candidate.payload,
                                EventPayloadV1::PartnershipFormed {
                                    household_id: formed_household,
                                    ..
                                } if *formed_household == *household_id
                            )
                    })
                }));
            }
            let EventPayloadV1::PartnershipFormed { partners, .. } = &event.payload else {
                continue;
            };
            let first = report.people.iter().find(|person| person.id == partners[0]);
            let second = report.people.iter().find(|person| person.id == partners[1]);
            assert!(first.zip(second).is_some_and(|(first, second)| {
                first.generation == second.generation
                    && !first
                        .parent_ids
                        .iter()
                        .any(|parent| second.parent_ids.contains(parent))
            }));
        }
        assert_eq!(
            serde_json::to_string_pretty(&report.summary)? + "\n",
            fs::read_to_string(golden.join("summary.json"))?
        );
        assert_eq!(
            report.chronicle,
            fs::read_to_string(golden.join("chronicle.md"))?
        );
        Ok(())
    }

    #[test]
    fn dynasty_cohort_preserves_family_invariants() -> Result<(), Box<dyn std::error::Error>> {
        let scenario_bytes = fs::read(workspace_root().join("scenarios/era-01/dynasty.ron"))?;
        let scenario: ScenarioV1 = ron::de::from_bytes(&scenario_bytes)?;
        let mut living = Vec::new();
        let mut births = Vec::new();
        let mut household_counts = Vec::new();
        let mut generation_counts = Vec::new();
        let mut surname_counts = Vec::new();

        for seed in 1..=100 {
            let report = merra_sim::run_years(scenario.clone(), seed, 60)?;
            assert_family_invariants(&report, &scenario);
            living.push(report.summary.living_population);
            births.push(
                report
                    .people
                    .iter()
                    .filter(|person| person.birth_day.is_some())
                    .count(),
            );
            household_counts.push(report.households.len());
            generation_counts.push(
                report
                    .people
                    .iter()
                    .map(|person| person.generation)
                    .max()
                    .map_or(0, |generation| generation.saturating_add(1)),
            );
            surname_counts.push(
                report
                    .people
                    .iter()
                    .map(|person| person.surname.as_str())
                    .collect::<BTreeSet<_>>()
                    .len(),
            );
        }

        assert_eq!(living.iter().min(), Some(&31));
        assert_eq!(living.iter().max(), Some(&51));
        assert_eq!(
            living.iter().map(|value| u64::from(*value)).sum::<u64>(),
            4_249
        );
        assert_eq!(births.iter().min(), Some(&34));
        assert_eq!(births.iter().max(), Some(&50));
        assert_eq!(household_counts.iter().min(), Some(&22));
        assert_eq!(household_counts.iter().max(), Some(&34));
        assert!(
            generation_counts
                .iter()
                .all(|generations| *generations == 4)
        );
        assert_eq!(surname_counts.iter().min(), Some(&8));
        assert_eq!(surname_counts.iter().max(), Some(&13));
        Ok(())
    }

    fn assert_family_invariants(report: &SimulationReport, scenario: &ScenarioV1) {
        assert!(scenario.family.enabled);
        assert!(
            report
                .people
                .windows(2)
                .all(|people| people[0].id < people[1].id)
        );
        assert!(
            report
                .households
                .windows(2)
                .all(|households| households[0].id < households[1].id)
        );
        assert_eq!(
            report.summary.living_population as usize,
            report.people.iter().filter(|person| person.alive).count()
        );
        assert_eq!(
            report.summary.deaths as usize,
            report.people.iter().filter(|person| !person.alive).count()
        );
        assert_eq!(report.summary.event_count, report.events.len());

        let people: BTreeMap<PersonId, _> = report
            .people
            .iter()
            .map(|person| (person.id, person))
            .collect();
        let households: BTreeMap<HouseholdId, _> = report
            .households
            .iter()
            .map(|household| (household.id, household))
            .collect();
        let event_ids: BTreeSet<_> = report.events.iter().map(|event| event.id).collect();

        for (index, event) in report.events.iter().enumerate() {
            assert_eq!(event.id.0, index as u64 + 1);
            assert!(
                event
                    .causes
                    .iter()
                    .all(|cause| { cause.0 < event.id.0 && event_ids.contains(cause) })
            );
            assert!(event.actors.iter().all(|actor| people.contains_key(actor)));
            if let Some(previous) = index
                .checked_sub(1)
                .and_then(|previous| report.events.get(previous))
            {
                assert!(previous.time <= event.time);
            }
            assert!(event_kind_matches_payload(event));
        }

        for person in &report.people {
            if person.alive {
                let household = person
                    .household_id
                    .and_then(|household_id| households.get(&household_id));
                assert!(household.is_some_and(|household| {
                    household.dissolved_day.is_none() && household.member_ids.contains(&person.id)
                }));
            } else {
                assert!(person.household_id.is_none());
                assert!(person.partner_id.is_none());
            }

            if person.parent_ids.is_empty() {
                assert_eq!(person.generation, 0);
            } else {
                assert_eq!(person.parent_ids.len(), 2);
                assert!(person.parent_ids[0] < person.parent_ids[1]);
                for parent_id in &person.parent_ids {
                    let parent = people.get(parent_id).copied();
                    assert!(parent.is_some_and(|parent| {
                        parent.id < person.id
                            && parent.generation.saturating_add(1) == person.generation
                    }));
                }
            }

            let Some(partner_id) = person.partner_id else {
                continue;
            };
            let partner = people.get(&partner_id).copied();
            assert!(partner.is_some_and(|partner| {
                partner.alive
                    && partner.partner_id == Some(person.id)
                    && partner.household_id == person.household_id
                    && partner.generation == person.generation
                    && !person.parent_ids.contains(&partner.id)
                    && !partner.parent_ids.contains(&person.id)
                    && !person
                        .parent_ids
                        .iter()
                        .any(|parent| partner.parent_ids.contains(parent))
            }));
        }

        for household in &report.households {
            assert!(
                household.member_ids.windows(2).all(|ids| ids[0] < ids[1]),
                "household member IDs must be unique and sorted"
            );
            if household.dissolved_day.is_some() {
                assert!(household.member_ids.is_empty());
            } else {
                assert!(!household.member_ids.is_empty());
            }
            for person_id in &household.member_ids {
                let person = people.get(person_id).copied();
                assert!(person.is_some_and(|person| {
                    person.alive && person.household_id == Some(household.id)
                }));
            }
            let recorded_births = report
                .events
                .iter()
                .filter(|event| {
                    matches!(
                        event.payload,
                        EventPayloadV1::PersonBorn { household_id, .. }
                            if household_id == household.id
                    )
                })
                .count();
            assert_eq!(usize::from(household.children_born), recorded_births);
            assert!(household.children_born <= scenario.family.maximum_children_per_household);
        }
    }

    fn event_kind_matches_payload(event: &WorldEventV1) -> bool {
        matches!(
            (&event.kind, &event.payload),
            (
                EventKindV1::SimulationStarted,
                EventPayloadV1::SimulationStarted { .. }
            ) | (
                EventKindV1::PopulationInitialized,
                EventPayloadV1::PopulationInitialized { .. }
            ) | (
                EventKindV1::TimeAdvanced,
                EventPayloadV1::TimeAdvanced { .. }
            ) | (EventKindV1::SeasonBegan, EventPayloadV1::SeasonBegan { .. })
                | (
                    EventKindV1::HouseholdFormed,
                    EventPayloadV1::HouseholdFormed { .. }
                )
                | (
                    EventKindV1::PartnershipFormed,
                    EventPayloadV1::PartnershipFormed { .. }
                )
                | (
                    EventKindV1::PartnershipEnded,
                    EventPayloadV1::PartnershipEnded { .. }
                )
                | (EventKindV1::PersonBorn, EventPayloadV1::PersonBorn { .. })
                | (
                    EventKindV1::HouseholdDissolved,
                    EventPayloadV1::HouseholdDissolved { .. }
                )
                | (EventKindV1::PersonDied, EventPayloadV1::PersonDied { .. })
                | (
                    EventKindV1::SimulationCompleted,
                    EventPayloadV1::SimulationCompleted { .. }
                )
        )
    }

    fn deterministic_bytes(
        report: &SimulationReport,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut bytes = serde_json::to_vec(&report.events)?;
        bytes.extend(serde_json::to_vec(&report.summary)?);
        bytes.extend(serde_json::to_vec(&report.people)?);
        bytes.extend(serde_json::to_vec(&report.households)?);
        bytes.extend(report.chronicle.as_bytes());
        Ok(bytes)
    }

    fn events_jsonl(report: &SimulationReport) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = Vec::new();
        for event in &report.events {
            bytes.extend(serde_json::to_vec(event)?);
            bytes.push(b'\n');
        }
        Ok(bytes)
    }

    fn golden_root() -> PathBuf {
        workspace_root().join("golden/era-01/smoke")
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }
}
