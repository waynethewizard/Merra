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
        EventKindV1, EventPayloadV1, HistoricalEventKindV1, HistoryConfigV1, HouseholdId,
        LineageId, LocalHistoryConfigV1, LocalHistoryPlaybackV1, LocalPlaybackEventV1, PersonId,
        ScenarioV1, WorldEventV1, WorldGenesisConfigV1,
    };
    use merra_sim::{SimulationReport, regional_history, run_history, run_local_history};
    use merra_worldgen::{generate_world, summarize_world};

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
    fn canonical_world_and_first_histories_match_golden_evidence()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = workspace_root();
        let world_config: WorldGenesisConfigV1 =
            ron::de::from_bytes(&fs::read(root.join("scenarios/era-01/before-memory.ron"))?)?;
        let history_config: HistoryConfigV1 = ron::de::from_bytes(&fs::read(
            root.join("scenarios/era-01/first-histories.ron"),
        )?)?;
        let world = generate_world(&world_config, 42)?;
        let repeated_world = generate_world(&world_config, 42)?;
        assert_eq!(world, repeated_world);
        let world_summary = summarize_world(&world);
        let history = run_history(&world, history_config, 42)?;
        let repeated_history = run_history(
            &world,
            ron::de::from_bytes(&fs::read(
                root.join("scenarios/era-01/first-histories.ron"),
            )?)?,
            42,
        )?;
        assert_eq!(history, repeated_history);

        assert_eq!(world_summary.regions, 12_288);
        assert_eq!(world_summary.land_regions, 5_898);
        assert_eq!(world_summary.island_regions, 471);
        assert_eq!(world_summary.locked_sea_routes, 1);
        assert_eq!(history.summary.elapsed_years, 600);
        assert_eq!(history.summary.first_contact_year, Some(293));
        assert_eq!(history.summary.mixed_lineage_populations, 4);
        assert_eq!(history.starting_region.settlement_ids.len(), 5);
        assert!(
            history
                .events
                .iter()
                .any(|event| { event.kind == HistoricalEventKindV1::FirstContact })
        );
        assert!(
            history
                .events
                .iter()
                .all(|event| event.causes.iter().all(|cause| cause.0 < event.id.0))
        );
        assert!(history.populations.iter().all(|population| {
            population
                .lineage
                .iter()
                .map(|share| u32::from(share.parts_per_10_000))
                .sum::<u32>()
                == 10_000
                && (population.cultures.is_empty()
                    || population
                        .cultures
                        .iter()
                        .map(|share| u32::from(share.parts_per_10_000))
                        .sum::<u32>()
                        == 10_000)
                && (population.faiths.is_empty()
                    || population
                        .faiths
                        .iter()
                        .map(|share| u32::from(share.parts_per_10_000))
                        .sum::<u32>()
                        == 10_000)
        }));
        let founders: Vec<_> = history
            .populations
            .iter()
            .filter(|population| population.founded_year == 0)
            .collect();
        assert_eq!(founders.len(), 4);
        assert_eq!(
            founders
                .iter()
                .filter(|population| population.lineage[0].id == LineageId(1))
                .count(),
            3
        );
        assert_eq!(
            founders
                .iter()
                .filter(|population| population.lineage[0].id == LineageId(2))
                .count(),
            1
        );

        let golden = root.join("golden/era-01/first-histories-seed-42");
        assert_eq!(
            serde_json::to_string_pretty(&world_summary)? + "\n",
            fs::read_to_string(golden.join("world-summary.json"))?
        );
        assert_eq!(
            serde_json::to_string_pretty(&history.summary)? + "\n",
            fs::read_to_string(golden.join("history-summary.json"))?
        );
        assert_eq!(
            history.chronicle,
            fs::read_to_string(golden.join("chronicle.md"))?
        );

        let local_config: LocalHistoryConfigV1 =
            ron::de::from_bytes(&fs::read(root.join("scenarios/era-01/five-villages.ron"))?)?;
        let regional = regional_history(&history);
        let local = run_local_history(&world, &regional, local_config.clone(), 42)?;
        let repeated_local = run_local_history(&world, &regional, local_config, 42)?;
        assert_eq!(local, repeated_local);
        let playback = LocalHistoryPlaybackV1::from_report(&local);
        assert_eq!(playback.people.len(), 108);
        assert_eq!(playback.events.len(), 164);
        assert_eq!(
            playback
                .events
                .iter()
                .filter(|event| matches!(event, LocalPlaybackEventV1::HouseholdSettled { .. }))
                .count(),
            52
        );
        assert_eq!(
            playback
                .events
                .iter()
                .filter(|event| matches!(event, LocalPlaybackEventV1::PersonBorn { .. }))
                .count(),
            78
        );
        assert_eq!(
            playback
                .events
                .iter()
                .filter(|event| matches!(event, LocalPlaybackEventV1::PersonDied { .. }))
                .count(),
            34
        );
        assert_eq!(local.summary.settlements, 5);
        assert_eq!(local.summary.macro_population, 40_751);
        assert_eq!(
            local.summary.represented_population,
            local.summary.macro_population
        );
        assert_eq!(local.connections.len(), 10);
        assert!(local.summary.household_migrations > 0);
        assert!(
            local
                .households
                .iter()
                .all(|household| household.residence_id.is_some())
        );
        assert!(
            local
                .events
                .iter()
                .enumerate()
                .all(|(index, event)| event.id.0 == index as u64 + 1
                    && event.causes.iter().all(|cause| cause.0 < event.id.0))
        );
        assert!(local.events.iter().all(|event| {
            !matches!(
                event.kind,
                EventKindV1::PersonBorn | EventKindV1::PersonDied
            ) || event.location.is_some()
        }));
        assert!(
            local
                .household_contexts
                .iter()
                .any(|context| !context.institution_ids.is_empty())
        );
        assert!(
            local
                .household_contexts
                .iter()
                .any(|context| !context.lore_claim_ids.is_empty())
        );
        let fenholm = local
            .settlements
            .iter()
            .find(|settlement| settlement.name == "Fenholm");
        assert!(fenholm.is_some_and(|settlement| {
            settlement.initial_sample_people > 0 && settlement.final_living_people == 0
        }));
        let fenstead = local
            .settlements
            .iter()
            .find(|settlement| settlement.name == "Fenstead");
        assert!(fenstead.is_some_and(|settlement| {
            settlement.final_living_people > settlement.initial_sample_people
        }));
        let local_golden = root.join("golden/era-01/five-villages-seed-42");
        assert_eq!(
            serde_json::to_string_pretty(&local.summary)? + "\n",
            fs::read_to_string(local_golden.join("summary.json"))?
        );
        assert_eq!(
            serde_json::to_string_pretty(&local.settlements)? + "\n",
            fs::read_to_string(local_golden.join("settlements.json"))?
        );
        assert_eq!(
            serde_json::to_string_pretty(&local.connections)? + "\n",
            fs::read_to_string(local_golden.join("connections.json"))?
        );
        assert_eq!(
            local.chronicle,
            fs::read_to_string(local_golden.join("chronicle.md"))?
        );
        assert_eq!(
            serde_json::to_string_pretty(&playback)? + "\n",
            fs::read_to_string(local_golden.join("playback.json"))?
        );
        Ok(())
    }

    #[test]
    fn world_history_cohort_preserves_structural_invariants()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = workspace_root();
        let world_config: WorldGenesisConfigV1 =
            ron::de::from_bytes(&fs::read(root.join("scenarios/era-01/before-memory.ron"))?)?;
        let history_config: HistoryConfigV1 = ron::de::from_bytes(&fs::read(
            root.join("scenarios/era-01/first-histories.ron"),
        )?)?;
        let local_config: LocalHistoryConfigV1 =
            ron::de::from_bytes(&fs::read(root.join("scenarios/era-01/five-villages.ron"))?)?;
        let mut contacts = 0;
        let mut runs_with_an_empty_village = 0;
        for seed in 1..=20 {
            let world = generate_world(&world_config, seed)?;
            let summary = summarize_world(&world);
            assert_eq!(summary.regions, 12_288);
            assert!(summary.land_regions > 5_000);
            assert!(summary.island_regions > 300);
            assert_eq!(summary.locked_sea_routes, 1);
            let history = run_history(&world, history_config.clone(), seed)?;
            contacts += usize::from(history.summary.first_contact_year.is_some());
            assert_eq!(history.summary.elapsed_years, 600);
            assert!(history.summary.settlements >= 5);
            assert_eq!(history.starting_region.settlement_ids.len(), 5);
            assert!(history.events.iter().enumerate().all(|(index, event)| {
                event.id.0 == index as u64 + 1
                    && event.causes.iter().all(|cause| cause.0 < event.id.0)
            }));
            let local = run_local_history(
                &world,
                &regional_history(&history),
                local_config.clone(),
                seed,
            )?;
            assert_eq!(local.summary.settlements, 5);
            assert_eq!(
                local.summary.macro_population,
                local.summary.represented_population
            );
            assert!(local.summary.household_migrations > 0);
            assert!(local.events.iter().all(|event| {
                !matches!(
                    event.kind,
                    EventKindV1::PersonBorn | EventKindV1::PersonDied
                ) || event.location.is_some()
            }));
            runs_with_an_empty_village += usize::from(
                local
                    .settlements
                    .iter()
                    .any(|settlement| settlement.final_living_people == 0),
            );
        }
        assert!(contacts > 0);
        assert!(runs_with_an_empty_village > 0);
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
                | (
                    EventKindV1::HouseholdSettled,
                    EventPayloadV1::HouseholdSettled { .. }
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
