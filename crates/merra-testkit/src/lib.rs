//! Shared fixtures and deterministic assertions for Merra tests.

use merra_core::{CalendarConfig, PopulationConfigV1, SCENARIO_SCHEMA_V1, ScenarioV1, SimDuration};
use merra_sim::{Simulation, SimulationError, SimulationReport};

/// Returns the canonical foundation smoke scenario.
#[must_use]
pub fn smoke_scenario() -> ScenarioV1 {
    ScenarioV1 {
        schema_version: SCENARIO_SCHEMA_V1,
        id: String::from("era-01-smoke"),
        title: String::from("The First Clock"),
        calendar: CalendarConfig { days_per_year: 360 },
        population: PopulationConfigV1 {
            initial_people: 0,
            minimum_starting_age: 0,
            maximum_starting_age: 0,
            mortality_bands: Vec::new(),
        },
    }
}

/// Runs the smoke scenario to completion without filesystem metadata.
pub fn run_smoke(seed: u64, years: u32) -> Result<SimulationReport, SimulationError> {
    let scenario = smoke_scenario();
    let duration = SimDuration::from_years(years, scenario.calendar);
    let mut simulation = Simulation::from_scenario(scenario, seed)?;
    simulation.advance(duration)?;
    simulation.finish()?;
    Ok(simulation.report())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use merra_core::ScenarioV1;
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

    fn deterministic_bytes(
        report: &SimulationReport,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut bytes = serde_json::to_vec(&report.events)?;
        bytes.extend(serde_json::to_vec(&report.summary)?);
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
