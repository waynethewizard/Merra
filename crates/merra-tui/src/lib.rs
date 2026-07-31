//! Story-first terminal rendering and navigation for Merra simulation evidence.

mod local;
mod model;
mod observatory;
mod observatory_render;
mod render;

pub use local::{LocalInspector, LocalView, render_local_snapshot};
pub use model::{EventFilter, Focus, HouseholdSort, Inspector, PersonSort, View};
pub use observatory::{
    CatalogKind, EntityRef, Observatory, ObservatoryData, ObservatoryError, ObservatoryLayer,
    ObservatoryTheme, ObservatoryView, PaneFocus,
};
pub use observatory_render::{render_observatory, render_observatory_snapshot};
pub use render::{render, render_snapshot, snapshot, snapshot_view, snapshot_view_with_focus};

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use merra_core::{
        CalendarConfig, FamilyConfigV1, HistoryConfigV1, HouseholdId, LocalHistoryConfigV1,
        LocationId, PersonId, PopulationConfigV1, SCENARIO_SCHEMA_V1, ScenarioV1, SeasonConfigV1,
        WorldGenesisConfigV1,
    };
    use merra_sim::{
        SimulationReport, regional_history, run_history, run_local_history, run_years,
    };
    use merra_worldgen::generate_world;

    use super::{
        EntityRef, Focus, Inspector, LocalInspector, LocalView, Observatory, ObservatoryData,
        ObservatoryView, View, render_local_snapshot, render_observatory_snapshot, render_snapshot,
        snapshot, snapshot_view, snapshot_view_with_focus,
    };

    #[test]
    fn snapshots_are_dynamic_plain_and_responsive() -> Result<(), Box<dyn std::error::Error>> {
        let scenario = ScenarioV1 {
            schema_version: SCENARIO_SCHEMA_V1,
            id: String::from("tui-test"),
            title: String::from("A Small Test"),
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
            family: FamilyConfigV1::default(),
            items: Default::default(),
        };
        let report = run_years(scenario, 42, 1)?;
        let screen = snapshot(report.clone(), 100, 30);

        assert!(screen.contains("MERRA // A SMALL TEST"));
        assert!(screen.contains("0 founders + 0 births = 0 people"));
        assert!(!screen.contains('\u{1b}'));

        let compact = snapshot_view(report.clone(), 72, 20, View::Overview);
        assert!(compact.contains("MERRA // A SMALL TEST"));
        assert!(compact.contains("World at Year 1"));

        let tiny = snapshot_view(report, 40, 8, View::Overview);
        assert!(tiny.contains("use at least 60×16"));
        Ok(())
    }

    #[test]
    fn canonical_dynasty_showcase_tells_the_seed_42_story() -> Result<(), Box<dyn std::error::Error>>
    {
        let report = dynasty_report()?;
        let overview = snapshot_view(report.clone(), 120, 36, View::Overview);
        let lineage = snapshot_view_with_focus(
            report,
            120,
            36,
            View::Lineage,
            Some(Focus::Person(PersonId(1))),
        );

        assert!(overview.contains("Population 16 + 49 born → 45 living · 20 deaths"));
        assert!(overview.contains("65 people recorded · 16 founders · 49 births"));
        assert!(overview.contains("G0"));
        assert!(overview.contains("G3"));
        assert!(overview.contains("Fen"));
        assert!(overview.contains("Gorse"));
        assert!(overview.contains("EXTINCT"));
        assert!(overview.contains("Featured Life"));
        assert!(overview.contains("Garin Thorn"));

        assert!(lineage.contains("Garin Gorse #2"));
        assert!(lineage.contains("Mara Thorn #17"));
        assert!(lineage.contains("Garin Thorn #25"));
        assert!(lineage.contains("Runa Oak #11"));
        assert!(lineage.contains("Garin Fen #14"));
        assert!(lineage.contains("CURRENT PARTNER"));
        Ok(())
    }

    #[test]
    fn canonical_cycle_one_views_match_current_golden_screens()
    -> Result<(), Box<dyn std::error::Error>> {
        let report = scenario_report("scenarios/era-01/century.ron", 100)?;

        assert_eq!(
            snapshot_view(report.clone(), 120, 36, View::Overview),
            include_str!("../../../golden/era-01/century-seed-42/tui-overview.txt")
        );
        assert_eq!(
            snapshot_view(report.clone(), 120, 36, View::History),
            include_str!("../../../golden/era-01/century-seed-42/tui-history.txt")
        );
        assert_eq!(
            snapshot_view(report, 120, 36, View::People),
            include_str!("../../../golden/era-01/century-seed-42/tui-people.txt")
        );
        Ok(())
    }

    #[test]
    fn canonical_dynasty_views_match_golden_screens() -> Result<(), Box<dyn std::error::Error>> {
        let report = dynasty_report()?;

        assert_eq!(
            snapshot_view(report.clone(), 120, 36, View::Overview),
            include_str!("../../../golden/era-01/dynasty-seed-42/tui-overview.txt")
        );
        assert_eq!(
            snapshot_view(report.clone(), 120, 36, View::History),
            include_str!("../../../golden/era-01/dynasty-seed-42/tui-history.txt")
        );
        assert_eq!(
            snapshot_view(report.clone(), 120, 36, View::People),
            include_str!("../../../golden/era-01/dynasty-seed-42/tui-people.txt")
        );
        assert_eq!(
            snapshot_view_with_focus(
                report.clone(),
                120,
                36,
                View::Lineage,
                Some(Focus::Person(PersonId(1))),
            ),
            include_str!("../../../golden/era-01/dynasty-seed-42/tui-lineage.txt")
        );
        assert_eq!(
            snapshot_view_with_focus(
                report,
                120,
                36,
                View::Households,
                Some(Focus::Household(HouseholdId(1))),
            ),
            include_str!("../../../golden/era-01/dynasty-seed-42/tui-households.txt")
        );
        Ok(())
    }

    #[test]
    fn history_defaults_to_story_and_can_reveal_debug_events()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut inspector = Inspector::new(dynasty_report()?);
        inspector.set_view(View::History);
        let story = render_snapshot_from_inspector(&inspector);

        assert_eq!(inspector.visible_event_indices().len(), 161);
        assert!(story.contains("History · historical"));
        assert!(!story.contains("clock advanced"));
        inspector.cycle_event_filter();
        inspector.cycle_event_filter();
        inspector.cycle_event_filter();
        inspector.first();
        let debug = render_snapshot_from_inspector(&inspector);
        assert_eq!(inspector.visible_event_indices().len(), 644);
        assert!(debug.contains("History · all / debug"));
        assert!(debug.contains("simulation started"));
        Ok(())
    }

    #[test]
    fn search_sort_and_cross_navigation_remain_coherent() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut inspector = Inspector::new(dynasty_report()?);
        inspector.set_view(View::People);
        inspector.begin_search();
        for character in "Gorse".chars() {
            inspector.push_search_char(character);
        }
        inspector.accept_search();
        let filtered = render_snapshot_from_inspector(&inspector);
        assert!(filtered.contains("2 shown"));
        assert!(filtered.contains("search “Gorse”"));
        assert!(filtered.contains("Garin Gorse"));

        inspector.clear_search();
        inspector.cycle_sort();
        inspector.focus(Focus::Person(PersonId(1)));
        inspector.activate();
        assert_eq!(inspector.view(), View::Lineage);
        inspector.jump_to_household();
        assert_eq!(inspector.view(), View::Households);
        inspector.jump_to_related_event();
        assert_eq!(inspector.view(), View::History);

        inspector.set_view(View::Lineage);
        assert!(inspector.focus(Focus::Person(PersonId(2))));
        inspector.jump_to_household();
        assert_eq!(inspector.view(), View::Households);
        assert!(render_snapshot_from_inspector(&inspector).contains("Thorn household  #1"));
        assert!(!inspector.focus(Focus::Person(PersonId(9_999))));
        Ok(())
    }

    #[test]
    fn five_village_views_show_consequence_routes_and_historical_context()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let world_config: WorldGenesisConfigV1 = ron::de::from_bytes(&std::fs::read(
            root.join("scenarios/era-01/before-memory.ron"),
        )?)?;
        let history_config: HistoryConfigV1 = ron::de::from_bytes(&std::fs::read(
            root.join("scenarios/era-01/first-histories.ron"),
        )?)?;
        let local_config: LocalHistoryConfigV1 = ron::de::from_bytes(&std::fs::read(
            root.join("scenarios/era-01/five-villages.ron"),
        )?)?;
        let world = generate_world(&world_config, 42)?;
        let history = run_history(&world, history_config, 42)?;
        let local = run_local_history(&world, &regional_history(&history), local_config, 42)?;
        let mut inspector = LocalInspector::new(local);

        let overview = render_local_snapshot(&inspector, 120, 36);
        assert!(overview.contains("40751 macro people represented exactly"));
        assert!(overview.contains("Fenstead grew 12→37"));
        assert!(overview.contains("Fenholm changed 4→0 and emptied"));
        assert!(overview.contains("one residence per household"));
        assert_eq!(
            overview,
            include_str!("../../../golden/era-01/five-villages-seed-42/tui-overview.txt")
        );

        inspector.set_view(LocalView::Roads);
        let roads = render_local_snapshot(&inspector, 120, 36);
        assert!(roads.contains("PAIRWISE TRAVEL COST"));
        assert!(roads.contains("Longest: Junipercross → Fenstead → Yarrowmere → Fenholm"));
        assert_eq!(
            roads,
            include_str!("../../../golden/era-01/five-villages-seed-42/tui-roads.txt")
        );

        assert!(inspector.focus_location(LocationId(27)));
        let settlement = render_local_snapshot(&inspector, 120, 36);
        assert!(settlement.contains("No sampled household remains"));
        assert_eq!(
            settlement,
            include_str!("../../../golden/era-01/five-villages-seed-42/tui-settlements.txt")
        );
        inspector.activate();
        let households = render_local_snapshot(&inspector, 120, 36);
        assert!(households.contains("HISTORICAL INHERITANCE · Fenholm"));
        inspector.clear_filter();
        assert!(inspector.focus_household(HouseholdId(1)));
        let household = render_local_snapshot(&inspector, 120, 36);
        assert!(household.contains("Institutions:"));
        assert!(household.contains("Claims:"));
        assert_eq!(
            household,
            include_str!("../../../golden/era-01/five-villages-seed-42/tui-households.txt")
        );

        inspector.set_view(LocalView::Migrations);
        let migrations = render_local_snapshot(&inspector, 120, 36);
        assert_eq!(
            migrations,
            include_str!("../../../golden/era-01/five-villages-seed-42/tui-migrations.txt")
        );

        let compact = render_local_snapshot(&inspector, 72, 20);
        assert!(compact.contains("MERRA // FIVE VILLAGES"));
        assert!(!compact.contains('\u{1b}'));
        let tiny = render_local_snapshot(&inspector, 40, 8);
        assert!(tiny.contains("use at least 60×16"));

        let item_config: LocalHistoryConfigV1 = ron::de::from_bytes(&std::fs::read(
            root.join("scenarios/era-01/item-lineage.ron"),
        )?)?;
        let item_history = run_local_history(&world, &regional_history(&history), item_config, 42)?;
        let mut item_inspector = LocalInspector::new(item_history);
        item_inspector.set_view(LocalView::Items);
        let items = render_local_snapshot(&item_inspector, 120, 36);
        assert!(items.contains("AUTHORITATIVE PROVENANCE"));
        assert!(items.contains("Sources:"));
        assert!(items.contains("BIOGRAPHY"));
        assert_eq!(
            items,
            include_str!("../../../golden/era-01/item-lineage-seed-42/tui-items.txt")
        );
        Ok(())
    }

    #[test]
    fn historical_observatory_connects_every_scale_and_workspace()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut observatory = Observatory::new(ObservatoryData::canonical()?);
        assert_eq!(observatory.maximum_year(), 660);
        assert!(observatory.local_state().is_some());

        let atlas = render_observatory_snapshot(&observatory, 100, 30);
        assert!(atlas.contains("HISTORICAL OBSERVATORY"));
        assert!(atlas.contains("World Atlas · history"));
        assert!(atlas.contains("contact Y293"));
        assert!(atlas.contains("local Y600"));
        assert!(!atlas.contains('\u{1b}'));

        observatory.set_view(ObservatoryView::Chronicle);
        let chronicle = render_observatory_snapshot(&observatory, 100, 30);
        assert!(chronicle.contains("Chronicle"));
        assert!(chronicle.contains("Authoritative event"));

        assert!(observatory.focus_entity("item:1".parse::<EntityRef>()?));
        observatory.set_view(ObservatoryView::Relations);
        let relations = render_observatory_snapshot(&observatory, 100, 30);
        assert!(relations.contains("Typed Relations"));
        assert!(relations.contains("linked records"));

        observatory.set_view(ObservatoryView::Catalog);
        let catalog = render_observatory_snapshot(&observatory, 100, 30);
        assert!(catalog.contains("Catalog · Items"));
        assert!(catalog.contains("item"));
        Ok(())
    }

    #[test]
    fn observatory_keeps_macro_and_local_event_id_scopes_distinct()
    -> Result<(), Box<dyn std::error::Error>> {
        let macro_event = "macro-event:1".parse::<EntityRef>()?;
        let local_event = "event:1".parse::<EntityRef>()?;
        assert_ne!(macro_event, local_event);
        assert_eq!(macro_event.to_string(), "macro-event:1");
        assert_eq!(local_event.to_string(), "local-event:1");
        assert!("person".parse::<EntityRef>().is_err());
        Ok(())
    }

    fn render_snapshot_from_inspector(inspector: &Inspector) -> String {
        render_snapshot(inspector, 120, 36)
    }

    fn dynasty_report() -> Result<SimulationReport, Box<dyn std::error::Error>> {
        scenario_report("scenarios/era-01/dynasty.ron", 60)
    }

    fn scenario_report(
        scenario_path: &str,
        years: u32,
    ) -> Result<SimulationReport, Box<dyn std::error::Error>> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let bytes = std::fs::read(root.join(scenario_path))?;
        let scenario: ScenarioV1 = ron::de::from_bytes(&bytes)?;
        Ok(run_years(scenario, 42, years)?)
    }
}
