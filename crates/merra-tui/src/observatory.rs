//! Unified, causally linked exploration state for the Merra historical observatory.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use merra_core::{
    CultureId, EventId, EventKindV1, EventPayloadV1, FaithId, FeatureId, HistoricalEventPayloadV1,
    HistoricalEventV1, HistoricalSubjectV1, HistoryConfigV1, HistoryManifestV1, HistorySummaryV1,
    HouseholdId, InstitutionId, ItemCustodyV1, ItemId, LocalHistoryConfigV1,
    LocalHistoryManifestV1, LocalHistoryPlaybackV1, LocalHistoryReportV1, LocalPlaybackEventV1,
    LocationId, PersonId, PolityId, PopulationId, PropertyOwnerV1, RegionId, RegionalHistoryV1,
    RouteId, SurfaceWorldV1, WorldGenesisConfigV1, WorldSubjectV1,
};
use merra_sim::{HistoricalReport, regional_history, run_history, run_local_history};
use merra_worldgen::generate_world;
use ratatui::layout::Rect;
use thiserror::Error;

use crate::media::{MediaCatalog, MediaEntry};

const CANONICAL_SEED: u64 = 42;
const WORLD_SCENARIO: &str = include_str!("../../../scenarios/era-01/before-memory.ron");
const HISTORY_SCENARIO: &str = include_str!("../../../scenarios/era-01/first-histories.ron");
const LOCAL_SCENARIO: &str = include_str!("../../../scenarios/era-01/item-lineage.ron");

/// The four complementary workspaces in the unified observatory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservatoryView {
    Atlas,
    Chronicle,
    Relations,
    Catalog,
}

impl ObservatoryView {
    pub(crate) const ALL: [Self; 4] =
        [Self::Atlas, Self::Chronicle, Self::Relations, Self::Catalog];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Atlas => "Atlas",
            Self::Chronicle => "Chronicle",
            Self::Relations => "Relations",
            Self::Catalog => "Catalog",
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Atlas => Self::Chronicle,
            Self::Chronicle => Self::Relations,
            Self::Relations => Self::Catalog,
            Self::Catalog => Self::Atlas,
        }
    }

    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::Atlas => Self::Catalog,
            Self::Chronicle => Self::Atlas,
            Self::Relations => Self::Chronicle,
            Self::Catalog => Self::Relations,
        }
    }
}

impl FromStr for ObservatoryView {
    type Err = ObservatoryError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "atlas" | "map" => Ok(Self::Atlas),
            "chronicle" | "history" | "timeline" => Ok(Self::Chronicle),
            "relations" | "graph" | "connections" => Ok(Self::Relations),
            "catalog" | "records" | "entities" => Ok(Self::Catalog),
            _ => Err(ObservatoryError::InvalidInput(format!(
                "unknown observatory workspace `{value}`"
            ))),
        }
    }
}

/// Physical and historical overlays available on the atlas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservatoryLayer {
    History,
    Terrain,
    Biome,
    Habitability,
    Resources,
    Mythic,
}

impl ObservatoryLayer {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::History => "history",
            Self::Terrain => "terrain",
            Self::Biome => "biome",
            Self::Habitability => "habitability",
            Self::Resources => "resources",
            Self::Mythic => "mythic",
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::History => Self::Terrain,
            Self::Terrain => Self::Biome,
            Self::Biome => Self::Habitability,
            Self::Habitability => Self::Resources,
            Self::Resources => Self::Mythic,
            Self::Mythic => Self::History,
        }
    }
}

/// Color presentation chosen for interactive and snapshot rendering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservatoryTheme {
    Archive,
    Monochrome,
}

impl FromStr for ObservatoryTheme {
    type Err = ObservatoryError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "archive" | "color" => Ok(Self::Archive),
            "mono" | "monochrome" | "no-color" => Ok(Self::Monochrome),
            _ => Err(ObservatoryError::InvalidInput(format!(
                "unknown observatory theme `{value}`"
            ))),
        }
    }
}

/// A stable cross-scale identity. Event scope is explicit because IDs overlap.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EntityRef {
    Region(RegionId),
    Feature(FeatureId),
    Location(LocationId),
    Route(RouteId),
    Population(PopulationId),
    Culture(CultureId),
    Faith(FaithId),
    Institution(InstitutionId),
    Polity(PolityId),
    Household(HouseholdId),
    Person(PersonId),
    Item(ItemId),
    MacroEvent(EventId),
    LocalEvent(EventId),
    Claim(u64),
}

impl EntityRef {
    #[must_use]
    pub const fn kind(self) -> &'static str {
        match self {
            Self::Region(_) => "region",
            Self::Feature(_) => "feature",
            Self::Location(_) => "location",
            Self::Route(_) => "route",
            Self::Population(_) => "population",
            Self::Culture(_) => "culture",
            Self::Faith(_) => "faith",
            Self::Institution(_) => "institution",
            Self::Polity(_) => "polity",
            Self::Household(_) => "household",
            Self::Person(_) => "person",
            Self::Item(_) => "item",
            Self::MacroEvent(_) => "macro-event",
            Self::LocalEvent(_) => "local-event",
            Self::Claim(_) => "claim",
        }
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        match self {
            Self::Region(id) => id.0,
            Self::Feature(id) => id.0,
            Self::Location(id) => id.0,
            Self::Route(id) => id.0,
            Self::Population(id) => id.0,
            Self::Culture(id) => id.0,
            Self::Faith(id) => id.0,
            Self::Institution(id) => id.0,
            Self::Polity(id) => id.0,
            Self::Household(id) => id.0,
            Self::Person(id) => id.0,
            Self::Item(id) => id.0,
            Self::MacroEvent(id) => id.0,
            Self::LocalEvent(id) => id.0,
            Self::Claim(id) => id,
        }
    }
}

impl fmt::Display for EntityRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.kind(), self.raw())
    }
}

impl FromStr for EntityRef {
    type Err = ObservatoryError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Some((kind, raw)) = value.split_once(':') else {
            return Err(ObservatoryError::InvalidInput(format!(
                "focus `{value}` must use <kind>:<id>"
            )));
        };
        let id = raw.parse::<u64>().map_err(|_| {
            ObservatoryError::InvalidInput(format!("focus `{value}` has an invalid numeric ID"))
        })?;
        match kind.to_ascii_lowercase().as_str() {
            "region" => Ok(Self::Region(RegionId(id))),
            "feature" => Ok(Self::Feature(FeatureId(id))),
            "location" | "place" | "settlement" => Ok(Self::Location(LocationId(id))),
            "route" => Ok(Self::Route(RouteId(id))),
            "population" => Ok(Self::Population(PopulationId(id))),
            "culture" => Ok(Self::Culture(CultureId(id))),
            "faith" => Ok(Self::Faith(FaithId(id))),
            "institution" => Ok(Self::Institution(InstitutionId(id))),
            "polity" => Ok(Self::Polity(PolityId(id))),
            "household" => Ok(Self::Household(HouseholdId(id))),
            "person" => Ok(Self::Person(PersonId(id))),
            "item" => Ok(Self::Item(ItemId(id))),
            "macro-event" | "history-event" => Ok(Self::MacroEvent(EventId(id))),
            "local-event" | "event" => Ok(Self::LocalEvent(EventId(id))),
            "claim" | "lore" => Ok(Self::Claim(id)),
            _ => Err(ObservatoryError::InvalidInput(format!(
                "unknown focus kind `{kind}`"
            ))),
        }
    }
}

/// One labeled edge in the typed relationship index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Relation {
    pub label: String,
    pub target: EntityRef,
}

/// One ordered macro or local event on the combined timeline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservatoryMoment {
    pub entity: EntityRef,
    pub year: u32,
    pub day: u64,
    pub label: String,
    pub location: Option<LocationId>,
    pub causes: Vec<EntityRef>,
    pub subjects: Vec<EntityRef>,
    pub tags: Vec<String>,
    pub debug: bool,
}

/// Exact sampled local population and activity at one global year.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LocalYearState {
    pub year: u32,
    pub residents: BTreeMap<LocationId, u32>,
    pub people: BTreeMap<PersonId, LocationId>,
    pub movements: Vec<PersonMovement>,
    pub births: usize,
    pub deaths: usize,
    pub migrations: usize,
    pub item_events: usize,
}

/// One exact named-person relocation visible during a local playback year.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonMovement {
    pub event_id: EventId,
    pub people: Vec<PersonId>,
    pub from: LocationId,
    pub to: LocationId,
}

/// One aggregate migration recorded during a macro-history year.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MacroMovement {
    pub event_id: EventId,
    pub population_id: PopulationId,
    pub people: u32,
    pub from: LocationId,
    pub to: LocationId,
}

/// Aggregate motion that can be projected onto the Atlas for one year.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MacroYearState {
    pub year: u32,
    pub movements: Vec<MacroMovement>,
}

/// A person's state at the selected point on the local timeline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PersonPhase {
    NotYetBorn,
    Living,
    Dead,
}

/// One time-aware row in the family-tree visualization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FamilyTreeRow {
    pub entity: EntityRef,
    pub label: String,
    pub phase: PersonPhase,
}

/// All causally connected evidence consumed by the observatory.
#[derive(Clone, Debug)]
pub struct ObservatoryData {
    pub world: SurfaceWorldV1,
    pub history: Option<HistoricalReport>,
    pub local: Option<LocalHistoryReportV1>,
}

impl ObservatoryData {
    /// Generates the canonical connected seed-42 history without writing run artifacts.
    pub fn canonical() -> Result<Self, ObservatoryError> {
        let world_config: WorldGenesisConfigV1 = ron::from_str(WORLD_SCENARIO)
            .map_err(|error| ObservatoryError::Generation(error.to_string()))?;
        let history_config: HistoryConfigV1 = ron::from_str(HISTORY_SCENARIO)
            .map_err(|error| ObservatoryError::Generation(error.to_string()))?;
        let local_config: LocalHistoryConfigV1 = ron::from_str(LOCAL_SCENARIO)
            .map_err(|error| ObservatoryError::Generation(error.to_string()))?;
        let world = generate_world(&world_config, CANONICAL_SEED)
            .map_err(|error| ObservatoryError::Generation(error.to_string()))?;
        let history = run_history(&world, history_config, CANONICAL_SEED)
            .map_err(|error| ObservatoryError::Generation(error.to_string()))?;
        let local = run_local_history(
            &world,
            &regional_history(&history),
            local_config,
            CANONICAL_SEED,
        )
        .map_err(|error| ObservatoryError::Generation(error.to_string()))?;
        Ok(Self {
            world,
            history: Some(history),
            local: Some(local),
        })
    }

    /// Loads a world-only, history, or connected history/local data set.
    pub fn load(
        world: Option<&Path>,
        history: Option<&Path>,
        local: Option<&Path>,
    ) -> Result<Self, ObservatoryError> {
        if world.is_some() && history.is_some() {
            return Err(ObservatoryError::InvalidInput(String::from(
                "--world and --history are mutually exclusive",
            )));
        }
        if local.is_some() && history.is_none() {
            return Err(ObservatoryError::InvalidInput(String::from(
                "--local requires --history",
            )));
        }
        if let Some(history_dir) = history {
            return load_history_data(history_dir, local);
        }
        if let Some(world_path) = world {
            let resolved = resolve_file(world_path, "world.json");
            return Ok(Self {
                world: read_json(&resolved)?,
                history: None,
                local: None,
            });
        }
        Self::canonical()
    }
}

/// Catalog group used by the records workspace.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CatalogKind {
    Places,
    Peoples,
    Beliefs,
    Institutions,
    Households,
    People,
    Items,
    Events,
    Claims,
}

impl CatalogKind {
    pub(crate) const ALL: [Self; 9] = [
        Self::Places,
        Self::Peoples,
        Self::Beliefs,
        Self::Institutions,
        Self::Households,
        Self::People,
        Self::Items,
        Self::Events,
        Self::Claims,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Places => "Places",
            Self::Peoples => "Peoples",
            Self::Beliefs => "Beliefs",
            Self::Institutions => "Institutions",
            Self::Households => "Households",
            Self::People => "People",
            Self::Items => "Items",
            Self::Events => "Events",
            Self::Claims => "Claims",
        }
    }
}

/// Which major pane receives list movement and activation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaneFocus {
    Primary,
    Detail,
    Timeline,
}

impl PaneFocus {
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Primary => Self::Detail,
            Self::Detail => Self::Timeline,
            Self::Timeline => Self::Primary,
        }
    }

    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::Primary => Self::Timeline,
            Self::Detail => Self::Primary,
            Self::Timeline => Self::Detail,
        }
    }
}

/// Last rendered rectangles used for deterministic mouse hit testing.
#[derive(Clone, Debug, Default)]
pub struct HitRegions {
    pub tabs: Vec<(ObservatoryView, Rect)>,
    pub primary: Rect,
    pub detail: Rect,
    pub timeline: Rect,
    pub rows: Vec<(EntityRef, Rect)>,
    pub search_rows: Vec<(usize, Rect)>,
}

/// Stateful navigation, playback, search, and indexes for the observatory.
#[derive(Clone, Debug)]
pub struct Observatory {
    pub(crate) data: ObservatoryData,
    pub(crate) media: MediaCatalog,
    pub(crate) names: BTreeMap<EntityRef, String>,
    pub(crate) relations: BTreeMap<EntityRef, Vec<Relation>>,
    pub(crate) moments: Vec<ObservatoryMoment>,
    pub(crate) macro_years: Vec<MacroYearState>,
    pub(crate) local_years: Vec<LocalYearState>,
    pub(crate) catalog: BTreeMap<CatalogKind, Vec<EntityRef>>,
    pub(crate) entity_locations: BTreeMap<EntityRef, LocationId>,
    pub(crate) view: ObservatoryView,
    pub(crate) layer: ObservatoryLayer,
    pub(crate) theme: ObservatoryTheme,
    pub(crate) pane: PaneFocus,
    pub(crate) focus: Option<EntityRef>,
    pub(crate) back_stack: Vec<EntityRef>,
    pub(crate) cursor_year: u32,
    pub(crate) maximum_year: u32,
    pub(crate) playing: bool,
    pub(crate) playback_direction: i8,
    pub(crate) query: String,
    pub(crate) searching: bool,
    pub(crate) search_results: Vec<EntityRef>,
    pub(crate) selection: usize,
    pub(crate) catalog_kind: CatalogKind,
    pub(crate) catalog_kind_index: usize,
    pub(crate) map_x: u16,
    pub(crate) map_y: u16,
    pub(crate) map_zoom: u8,
    pub(crate) detail_scroll: u16,
    pub(crate) family_tree: bool,
    pub(crate) show_help: bool,
    pub(crate) show_debug: bool,
    pub(crate) transition_epoch: u64,
    pub(crate) status: Option<String>,
    pub(crate) hits: HitRegions,
}

impl Observatory {
    /// Indexes one connected data set and chooses a consequence-first default focus.
    #[must_use]
    pub fn new(data: ObservatoryData) -> Self {
        let maximum_year = data.local.as_ref().map_or_else(
            || data.history.as_ref().map_or(0, |history| history.years),
            |local| local.summary.projection_year + local.summary.elapsed_years,
        );
        let mut observatory = Self {
            data,
            media: MediaCatalog::canonical().unwrap_or_default(),
            names: BTreeMap::new(),
            relations: BTreeMap::new(),
            moments: Vec::new(),
            macro_years: Vec::new(),
            local_years: Vec::new(),
            catalog: BTreeMap::new(),
            entity_locations: BTreeMap::new(),
            view: ObservatoryView::Atlas,
            layer: ObservatoryLayer::History,
            theme: ObservatoryTheme::Archive,
            pane: PaneFocus::Primary,
            focus: None,
            back_stack: Vec::new(),
            cursor_year: maximum_year,
            maximum_year,
            playing: false,
            playback_direction: 1,
            query: String::new(),
            searching: false,
            search_results: Vec::new(),
            selection: 0,
            catalog_kind: CatalogKind::Places,
            catalog_kind_index: 0,
            map_x: 0,
            map_y: 0,
            map_zoom: 1,
            detail_scroll: 0,
            family_tree: true,
            show_help: false,
            show_debug: false,
            transition_epoch: 0,
            status: None,
            hits: HitRegions::default(),
        };
        observatory.build_indexes();
        let initial = observatory
            .data
            .history
            .as_ref()
            .map(|history| EntityRef::Location(history.starting_region.anchor_location_id))
            .or_else(|| {
                observatory
                    .data
                    .world
                    .places
                    .locations
                    .first()
                    .map(|location| EntityRef::Location(location.id))
            });
        if let Some(initial) = initial {
            observatory.set_focus(initial, false);
        }
        observatory
    }

    #[must_use]
    pub const fn view(&self) -> ObservatoryView {
        self.view
    }

    #[must_use]
    pub const fn layer(&self) -> ObservatoryLayer {
        self.layer
    }

    #[must_use]
    pub const fn theme(&self) -> ObservatoryTheme {
        self.theme
    }

    pub fn set_theme(&mut self, theme: ObservatoryTheme) {
        self.theme = theme;
    }

    #[must_use]
    pub const fn pane(&self) -> PaneFocus {
        self.pane
    }

    #[must_use]
    pub const fn is_searching(&self) -> bool {
        self.searching
    }

    #[must_use]
    pub const fn help_visible(&self) -> bool {
        self.show_help
    }

    #[must_use]
    pub const fn is_playing(&self) -> bool {
        self.playing
    }

    #[must_use]
    pub const fn playback_direction(&self) -> i8 {
        self.playback_direction
    }

    #[must_use]
    pub const fn transition_epoch(&self) -> u64 {
        self.transition_epoch
    }

    #[must_use]
    pub const fn hit_regions(&self) -> &HitRegions {
        &self.hits
    }

    pub fn set_media_catalog(&mut self, media: MediaCatalog) {
        self.media = media;
    }

    #[must_use]
    pub fn media_entry(&self, entity: EntityRef) -> Option<&MediaEntry> {
        self.media.entry(entity)
    }

    #[must_use]
    pub fn media_count(&self) -> usize {
        self.media.len()
    }

    #[must_use]
    pub const fn cursor_year(&self) -> u32 {
        self.cursor_year
    }

    #[must_use]
    pub const fn maximum_year(&self) -> u32 {
        self.maximum_year
    }

    #[must_use]
    pub const fn focused(&self) -> Option<EntityRef> {
        self.focus
    }

    #[must_use]
    pub fn label(&self, entity: EntityRef) -> String {
        self.names
            .get(&entity)
            .cloned()
            .unwrap_or_else(|| format!("{} #{}", entity.kind(), entity.raw()))
    }

    #[must_use]
    pub fn relation_list(&self) -> &[Relation] {
        self.focus
            .and_then(|focus| self.relations.get(&focus))
            .map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn current_moment(&self) -> Option<&ObservatoryMoment> {
        let focus = self.focus?;
        self.moments.iter().find(|moment| moment.entity == focus)
    }

    #[must_use]
    pub fn local_state(&self) -> Option<&LocalYearState> {
        self.local_years
            .iter()
            .find(|state| state.year == self.cursor_year)
    }

    #[must_use]
    pub fn macro_state(&self) -> Option<&MacroYearState> {
        self.macro_years
            .iter()
            .find(|state| state.year == self.cursor_year)
    }

    #[must_use]
    pub(crate) fn person_phase(&self, id: PersonId) -> PersonPhase {
        let Some(local) = self.data.local.as_ref() else {
            return PersonPhase::NotYetBorn;
        };
        if self.cursor_year < local.summary.projection_year {
            return PersonPhase::NotYetBorn;
        }
        let Some(person) = local.people.iter().find(|person| person.id == id) else {
            return PersonPhase::NotYetBorn;
        };
        let elapsed = self
            .cursor_year
            .saturating_sub(local.summary.projection_year);
        let day = u64::from(elapsed).saturating_mul(self.local_days_per_year());
        if person.birth_day.is_some_and(|birth| birth > day) {
            PersonPhase::NotYetBorn
        } else if person.death_day.is_some_and(|death| death <= day) {
            PersonPhase::Dead
        } else {
            PersonPhase::Living
        }
    }

    #[must_use]
    pub(crate) fn family_tree_visible(&self) -> bool {
        self.family_tree
            && matches!(
                self.focus,
                Some(EntityRef::Person(_) | EntityRef::Household(_))
            )
            && self.data.local.is_some()
    }

    #[must_use]
    pub(crate) fn family_tree_rows(&self) -> Vec<FamilyTreeRow> {
        let Some(local) = self.data.local.as_ref() else {
            return Vec::new();
        };
        let mut seeds = BTreeSet::<PersonId>::new();
        match self.focus {
            Some(EntityRef::Person(person)) => {
                seeds.insert(person);
            }
            Some(EntityRef::Household(household)) => {
                if let Some(record) = local
                    .households
                    .iter()
                    .find(|record| record.id == household)
                {
                    seeds.extend(record.member_ids.iter().copied());
                }
                for event in &local.events {
                    match &event.payload {
                        EventPayloadV1::HouseholdFormed {
                            household_id,
                            member_ids,
                            ..
                        } if *household_id == household => {
                            seeds.extend(member_ids.iter().copied());
                        }
                        EventPayloadV1::PersonBorn {
                            person_id,
                            household_id,
                            ..
                        } if *household_id == household => {
                            seeds.insert(*person_id);
                        }
                        _ => {}
                    }
                }
            }
            _ => return Vec::new(),
        }
        let people = local
            .people
            .iter()
            .map(|person| (person.id, person))
            .collect::<BTreeMap<_, _>>();
        let mut children = BTreeMap::<PersonId, Vec<PersonId>>::new();
        for person in &local.people {
            for parent in &person.parent_ids {
                children.entry(*parent).or_default().push(person.id);
            }
        }
        let mut component = seeds.clone();
        let mut ancestors = seeds.iter().copied().collect::<Vec<_>>();
        while let Some(person) = ancestors.pop() {
            if let Some(record) = people.get(&person) {
                for parent in &record.parent_ids {
                    if component.insert(*parent) {
                        ancestors.push(*parent);
                    }
                }
            }
        }
        let mut descendants = seeds.into_iter().collect::<Vec<_>>();
        let mut traversed_descendants = BTreeSet::new();
        while let Some(person) = descendants.pop() {
            if !traversed_descendants.insert(person) {
                continue;
            }
            if let Some(record) = people.get(&person)
                && let Some(partner) = record.partner_id
            {
                component.insert(partner);
            }
            if let Some(children) = children.get(&person) {
                for child in children {
                    component.insert(*child);
                    descendants.push(*child);
                    if let Some(record) = people.get(child) {
                        component.extend(record.parent_ids.iter().copied());
                    }
                }
            }
        }
        let mut visible = component
            .into_iter()
            .filter_map(|id| {
                let record = people.get(&id)?;
                let phase = self.person_phase(id);
                (phase != PersonPhase::NotYetBorn || self.focus == Some(EntityRef::Person(id)))
                    .then_some((*record, phase))
            })
            .collect::<Vec<_>>();
        visible.sort_by(|(left, _), (right, _)| {
            (left.generation, left.name.as_str(), left.id).cmp(&(
                right.generation,
                right.name.as_str(),
                right.id,
            ))
        });
        let minimum_generation = visible.first().map_or(0, |(person, _)| person.generation);
        visible
            .iter()
            .enumerate()
            .map(|(index, (person, phase))| {
                let last_in_generation = visible
                    .get(index + 1)
                    .is_none_or(|(next, _)| next.generation != person.generation);
                let depth = usize::from(person.generation.saturating_sub(minimum_generation));
                let connector = if last_in_generation {
                    "└─"
                } else {
                    "├─"
                };
                let parents = person
                    .parent_ids
                    .iter()
                    .filter(|parent| self.person_phase(**parent) != PersonPhase::NotYetBorn)
                    .map(|parent| self.label(EntityRef::Person(*parent)))
                    .collect::<Vec<_>>();
                FamilyTreeRow {
                    entity: EntityRef::Person(person.id),
                    label: format!(
                        "G{} {}{} {} #{}{}",
                        person.generation,
                        "│ ".repeat(depth),
                        connector,
                        person.name,
                        person.id.0,
                        if parents.is_empty() {
                            String::new()
                        } else {
                            format!("  ← {}", parents.join(" + "))
                        }
                    ),
                    phase: *phase,
                }
            })
            .collect()
    }

    #[must_use]
    pub fn visible_moments(&self) -> Vec<&ObservatoryMoment> {
        let related = self.focus.map(|focus| {
            self.relations
                .get(&focus)
                .into_iter()
                .flatten()
                .map(|relation| relation.target)
                .collect::<BTreeSet<_>>()
        });
        self.moments
            .iter()
            .filter(|moment| moment.year <= self.cursor_year)
            .filter(|moment| self.show_debug || !moment.debug)
            .filter(|moment| {
                if self.query.is_empty() {
                    return true;
                }
                let query = self.query.to_ascii_lowercase();
                moment.label.to_ascii_lowercase().contains(&query)
                    || moment
                        .tags
                        .iter()
                        .any(|tag| tag.to_ascii_lowercase().contains(&query))
            })
            .filter(|moment| {
                if self.view != ObservatoryView::Relations {
                    return true;
                }
                related
                    .as_ref()
                    .is_none_or(|entities| entities.contains(&moment.entity))
            })
            .collect()
    }

    #[must_use]
    pub fn catalog_entities(&self) -> &[EntityRef] {
        self.catalog
            .get(&self.catalog_kind)
            .map_or(&[], Vec::as_slice)
    }

    pub fn set_view(&mut self, view: ObservatoryView) {
        if self.view != view {
            self.view = view;
            self.selection = 0;
            if let Some(focus) = self.focus {
                match view {
                    ObservatoryView::Catalog => {
                        if let Some((kind, index)) =
                            self.catalog.iter().find_map(|(kind, entities)| {
                                entities
                                    .iter()
                                    .position(|candidate| *candidate == focus)
                                    .map(|index| (*kind, index))
                            })
                        {
                            self.catalog_kind = kind;
                            self.catalog_kind_index = CatalogKind::ALL
                                .iter()
                                .position(|candidate| *candidate == kind)
                                .unwrap_or(0);
                            self.selection = index;
                        }
                    }
                    ObservatoryView::Chronicle => {
                        let focus_location = match focus {
                            EntityRef::Location(location) => Some(location),
                            _ => self.entity_locations.get(&focus).copied(),
                        };
                        if let Some(index) = self
                            .visible_moments()
                            .into_iter()
                            .enumerate()
                            .rev()
                            .find_map(|(index, moment)| {
                                (moment.year <= self.cursor_year
                                    && (moment.entity == focus
                                        || moment.subjects.contains(&focus)
                                        || focus_location.is_some_and(|location| {
                                            moment.location == Some(location)
                                        })))
                                .then_some(index)
                            })
                        {
                            self.selection = index;
                        }
                    }
                    ObservatoryView::Relations if self.family_tree_visible() => {
                        if let Some(index) = self
                            .family_tree_rows()
                            .iter()
                            .position(|row| row.entity == focus)
                        {
                            self.selection = index;
                        }
                    }
                    ObservatoryView::Atlas | ObservatoryView::Relations => {}
                }
            }
            self.transition_epoch = self.transition_epoch.saturating_add(1);
        }
    }

    pub fn next_view(&mut self) {
        self.set_view(self.view.next());
    }

    pub fn previous_view(&mut self) {
        self.set_view(self.view.previous());
    }

    pub fn next_layer(&mut self) {
        self.layer = self.layer.next();
        self.transition_epoch = self.transition_epoch.saturating_add(1);
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_debug(&mut self) {
        self.show_debug = !self.show_debug;
        self.selection = 0;
    }

    pub fn toggle_playback(&mut self) {
        self.toggle_playback_in_direction(1);
    }

    pub fn toggle_reverse_playback(&mut self) {
        self.toggle_playback_in_direction(-1);
    }

    fn toggle_playback_in_direction(&mut self, direction: i8) {
        if self.maximum_year == 0 {
            return;
        }
        if self.playing && self.playback_direction == direction {
            self.playing = false;
            return;
        }
        self.playback_direction = direction;
        if direction > 0 && self.cursor_year == self.maximum_year {
            self.cursor_year = 0;
        } else if direction < 0 && self.cursor_year == 0 {
            self.cursor_year = self.maximum_year;
        }
        self.playing = true;
    }

    pub fn playback_tick(&mut self) {
        if !self.playing {
            return;
        }
        if self.playback_direction < 0 {
            self.playback_tick_backward();
            return;
        }
        if self.cursor_year >= self.maximum_year {
            self.playing = false;
            return;
        }
        if self.cursor_year < self.local_start_year().unwrap_or(self.maximum_year + 1) {
            let next = self
                .moments
                .iter()
                .map(|moment| moment.year)
                .find(|year| *year > self.cursor_year)
                .unwrap_or(self.cursor_year.saturating_add(1));
            self.cursor_year = next.min(self.maximum_year);
        } else {
            self.cursor_year = self.cursor_year.saturating_add(1).min(self.maximum_year);
        }
        self.sync_focus_to_time();
    }

    fn playback_tick_backward(&mut self) {
        if self.cursor_year == 0 {
            self.playing = false;
            return;
        }
        if self.cursor_year > self.local_start_year().unwrap_or(self.maximum_year + 1) {
            self.cursor_year = self.cursor_year.saturating_sub(1);
            self.sync_focus_to_time();
            return;
        }
        let previous = self
            .moments
            .iter()
            .rev()
            .map(|moment| moment.year)
            .find(|year| *year < self.cursor_year)
            .unwrap_or_else(|| self.cursor_year.saturating_sub(1));
        self.cursor_year = previous;
        self.sync_focus_to_time();
    }

    pub fn set_year(&mut self, year: u32) {
        self.cursor_year = year.min(self.maximum_year);
        self.playing = false;
        self.sync_focus_to_time();
    }

    pub fn step_year(&mut self, forward: bool) {
        if forward {
            self.set_year(self.cursor_year.saturating_add(1));
        } else {
            self.set_year(self.cursor_year.saturating_sub(1));
        }
    }

    pub fn first_year(&mut self) {
        self.set_year(0);
    }

    pub fn last_year(&mut self) {
        self.set_year(self.maximum_year);
    }

    pub fn step_event(&mut self, forward: bool) {
        let candidate = if forward {
            self.moments
                .iter()
                .find(|moment| moment.year > self.cursor_year)
        } else {
            self.moments
                .iter()
                .rev()
                .find(|moment| moment.year < self.cursor_year)
        }
        .map(|moment| (moment.year, moment.entity));
        if let Some((year, entity)) = candidate {
            self.set_year(year);
            self.set_focus(entity, true);
        }
    }

    pub fn focus_entity(&mut self, entity: EntityRef) -> bool {
        if !self.names.contains_key(&entity) {
            return false;
        }
        if self.view == ObservatoryView::Catalog
            && let Some((kind, index)) = self.catalog.iter().find_map(|(kind, entities)| {
                entities
                    .iter()
                    .position(|candidate| *candidate == entity)
                    .map(|index| (*kind, index))
            })
        {
            self.catalog_kind = kind;
            self.catalog_kind_index = CatalogKind::ALL
                .iter()
                .position(|candidate| *candidate == kind)
                .unwrap_or(0);
            self.selection = index;
        }
        self.set_focus(entity, true);
        true
    }

    pub fn back(&mut self) {
        if let Some(previous) = self.back_stack.pop() {
            self.set_focus(previous, false);
        }
    }

    pub fn begin_search(&mut self) {
        self.searching = true;
        self.query.clear();
        self.selection = 0;
        self.refresh_search();
    }

    pub fn cancel_search(&mut self) {
        self.searching = false;
        self.query.clear();
        self.search_results.clear();
        self.selection = 0;
    }

    pub fn push_search(&mut self, character: char) {
        self.query.push(character);
        self.selection = 0;
        self.refresh_search();
    }

    pub fn pop_search(&mut self) {
        self.query.pop();
        self.selection = 0;
        self.refresh_search();
    }

    pub fn accept_search(&mut self) {
        if let Some(entity) = self.search_results.get(self.selection).copied() {
            self.view = match entity {
                EntityRef::MacroEvent(_) | EntityRef::LocalEvent(_) | EntityRef::Claim(_) => {
                    ObservatoryView::Chronicle
                }
                EntityRef::Region(_) | EntityRef::Feature(_) | EntityRef::Location(_) => {
                    ObservatoryView::Atlas
                }
                _ => ObservatoryView::Relations,
            };
            self.set_focus(entity, true);
        }
        self.searching = false;
        self.query.clear();
        self.search_results.clear();
        self.selection = 0;
        self.transition_epoch = self.transition_epoch.saturating_add(1);
    }

    pub fn move_selection(&mut self, forward: bool) {
        let count = if self.searching {
            self.search_results.len()
        } else {
            match self.view {
                ObservatoryView::Chronicle => self.visible_moments().len(),
                ObservatoryView::Relations if self.family_tree_visible() => {
                    self.family_tree_rows().len()
                }
                ObservatoryView::Relations => self.relation_list().len(),
                ObservatoryView::Catalog => self.catalog_entities().len(),
                ObservatoryView::Atlas => 0,
            }
        };
        if count == 0 {
            return;
        }
        if forward {
            self.selection = (self.selection + 1).min(count - 1);
        } else {
            self.selection = self.selection.saturating_sub(1);
        }
    }

    pub fn page_selection(&mut self, forward: bool) {
        for _ in 0..10 {
            self.move_selection(forward);
        }
    }

    pub fn scroll_detail(&mut self, forward: bool) {
        self.detail_scroll = if forward {
            self.detail_scroll.saturating_add(1)
        } else {
            self.detail_scroll.saturating_sub(1)
        };
    }

    pub fn page_detail(&mut self, forward: bool) {
        for _ in 0..10 {
            self.scroll_detail(forward);
        }
    }

    pub fn activate_selection(&mut self) {
        if self.searching {
            self.accept_search();
            return;
        }
        let entity = match self.view {
            ObservatoryView::Chronicle => self
                .visible_moments()
                .get(self.selection)
                .map(|moment| moment.entity),
            ObservatoryView::Relations if self.family_tree_visible() => self
                .family_tree_rows()
                .get(self.selection)
                .map(|row| row.entity),
            ObservatoryView::Relations => self
                .relation_list()
                .get(self.selection)
                .map(|edge| edge.target),
            ObservatoryView::Catalog => self.catalog_entities().get(self.selection).copied(),
            ObservatoryView::Atlas => {
                if matches!(self.focus, Some(EntityRef::Person(_))) {
                    self.show_family_tree();
                    return;
                }
                let location = match self.focus {
                    Some(EntityRef::Location(location)) => Some(location),
                    _ => None,
                };
                if let Some(location) = location
                    && let Some(person) = self.local_state().and_then(|state| {
                        state
                            .people
                            .iter()
                            .find_map(|(person, current)| (*current == location).then_some(*person))
                    })
                {
                    self.set_focus(EntityRef::Person(person), true);
                    return;
                }
                self.focus
            }
        };
        if let Some(entity) = entity {
            self.set_focus(entity, true);
        }
    }

    pub fn cycle_catalog(&mut self, forward: bool) {
        let count = CatalogKind::ALL.len();
        self.catalog_kind_index = if forward {
            (self.catalog_kind_index + 1) % count
        } else {
            (self.catalog_kind_index + count - 1) % count
        };
        self.catalog_kind = CatalogKind::ALL[self.catalog_kind_index];
        self.selection = 0;
    }

    pub fn cycle_pane(&mut self, forward: bool) {
        self.pane = if forward {
            self.pane.next()
        } else {
            self.pane.previous()
        };
    }

    pub fn set_pane(&mut self, pane: PaneFocus) {
        self.pane = pane;
    }

    pub fn toggle_family_tree(&mut self) {
        self.family_tree = !self.family_tree;
        self.selection = 0;
        if self.family_tree
            && let Some(focus) = self.focus
            && let Some(index) = self
                .family_tree_rows()
                .iter()
                .position(|row| row.entity == focus)
        {
            self.selection = index;
        }
        self.transition_epoch = self.transition_epoch.saturating_add(1);
    }

    pub fn show_family_tree(&mut self) {
        self.family_tree = true;
        self.set_view(ObservatoryView::Relations);
        self.selection = self
            .focus
            .and_then(|focus| {
                self.family_tree_rows()
                    .iter()
                    .position(|row| row.entity == focus)
            })
            .unwrap_or(0);
    }

    pub fn cycle_person_at_focus(&mut self, forward: bool) {
        let location = match self.focus {
            Some(EntityRef::Location(location)) => Some(location),
            Some(EntityRef::Person(person)) => self
                .local_state()
                .and_then(|state| state.people.get(&person).copied()),
            _ => None,
        };
        let Some(location) = location else {
            return;
        };
        let mut people = self
            .local_state()
            .into_iter()
            .flat_map(|state| &state.people)
            .filter(|(_, candidate)| **candidate == location)
            .map(|(person, _)| *person)
            .collect::<Vec<_>>();
        people.sort_by_key(|person| self.label(EntityRef::Person(*person)));
        if people.is_empty() {
            return;
        }
        let current = self.focus.and_then(|focus| match focus {
            EntityRef::Person(person) => people.iter().position(|candidate| *candidate == person),
            _ => None,
        });
        let index = if forward {
            current.map_or(0, |index| (index + 1) % people.len())
        } else {
            current.map_or(people.len() - 1, |index| {
                (index + people.len() - 1) % people.len()
            })
        };
        self.set_focus(EntityRef::Person(people[index]), true);
    }

    pub fn move_map_cursor(&mut self, dx: i16, dy: i16) {
        let maximum_x = self.data.world.width.saturating_sub(1);
        let maximum_y = self.data.world.height.saturating_sub(1);
        self.map_x = add_signed(self.map_x, dx).min(maximum_x);
        self.map_y = add_signed(self.map_y, dy).min(maximum_y);
        let index =
            usize::from(self.map_y) * usize::from(self.data.world.width) + usize::from(self.map_x);
        if let Some(cell) = self.data.world.cells.get(index) {
            self.set_focus(EntityRef::Region(cell.id), true);
        }
    }

    pub fn zoom_map(&mut self, inward: bool) {
        self.map_zoom = if inward {
            (self.map_zoom.saturating_mul(2)).min(4)
        } else {
            (self.map_zoom / 2).max(1)
        };
    }

    pub fn select_map_position(&mut self, column: u16, row: u16, area: Rect) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let (start_x, start_y, visible_width, visible_height) = self.map_window();
        let x = u32::from(column.saturating_sub(area.x)).saturating_mul(u32::from(visible_width))
            / u32::from(area.width);
        let y = u32::from(row.saturating_sub(area.y)).saturating_mul(u32::from(visible_height))
            / u32::from(area.height);
        self.map_x = start_x.saturating_add(
            u16::try_from(x)
                .unwrap_or(self.data.world.width.saturating_sub(1))
                .min(visible_width.saturating_sub(1)),
        );
        self.map_y = start_y.saturating_add(
            u16::try_from(y)
                .unwrap_or(self.data.world.height.saturating_sub(1))
                .min(visible_height.saturating_sub(1)),
        );
        let index =
            usize::from(self.map_y) * usize::from(self.data.world.width) + usize::from(self.map_x);
        if let Some(cell) = self.data.world.cells.get(index) {
            let region = cell.id;
            let location = self
                .data
                .world
                .places
                .locations
                .iter()
                .find(|location| location.region == Some(region))
                .map(|location| EntityRef::Location(location.id));
            self.set_focus(location.unwrap_or(EntityRef::Region(region)), true);
        }
    }

    pub fn select_timeline_position(&mut self, column: u16, area: Rect) {
        if area.width <= 2 || self.maximum_year == 0 {
            return;
        }
        let relative = column
            .saturating_sub(area.x.saturating_add(1))
            .min(area.width.saturating_sub(2));
        let year = u32::from(relative).saturating_mul(self.maximum_year)
            / u32::from(area.width.saturating_sub(2).max(1));
        self.set_year(year);
    }

    pub fn click_row(&mut self, column: u16, row: u16) -> bool {
        if self.searching
            && let Some((index, _)) = self
                .hits
                .search_rows
                .iter()
                .find(|(_, area)| contains(*area, column, row))
        {
            self.selection = *index;
            self.accept_search();
            return true;
        }
        if let Some((entity, _)) = self
            .hits
            .rows
            .iter()
            .find(|(_, area)| contains(*area, column, row))
        {
            let entity = *entity;
            if self.view == ObservatoryView::Catalog
                && let Some(index) = self
                    .catalog_entities()
                    .iter()
                    .position(|candidate| *candidate == entity)
            {
                self.selection = index;
            }
            self.set_focus(entity, true);
            return true;
        }
        false
    }

    fn set_focus(&mut self, entity: EntityRef, remember: bool) {
        if remember
            && let Some(current) = self.focus
            && current != entity
        {
            self.back_stack.push(current);
        }
        if self.focus != Some(entity) {
            self.detail_scroll = 0;
        }
        self.focus = Some(entity);
        let playback_location = match entity {
            EntityRef::Person(person) => self
                .local_state()
                .and_then(|state| state.people.get(&person).copied()),
            _ => None,
        };
        if let Some(location) =
            playback_location.or_else(|| self.entity_locations.get(&entity).copied())
        {
            self.sync_map_to_location(location);
        } else if let EntityRef::Region(region) = entity
            && let Some(cell) = self.data.world.cells.iter().find(|cell| cell.id == region)
        {
            self.map_x = cell.coordinate.x;
            self.map_y = cell.coordinate.y;
        }
        if self.view == ObservatoryView::Chronicle
            && let Some(index) = self
                .visible_moments()
                .iter()
                .position(|moment| moment.entity == entity)
        {
            self.selection = index;
        } else if self.view == ObservatoryView::Relations
            && self.family_tree_visible()
            && let Some(index) = self
                .family_tree_rows()
                .iter()
                .position(|row| row.entity == entity)
        {
            self.selection = index;
        }
        self.transition_epoch = self.transition_epoch.saturating_add(1);
    }

    fn sync_map_to_location(&mut self, location: LocationId) {
        if let Some(region) = self
            .data
            .world
            .places
            .locations
            .iter()
            .find(|candidate| candidate.id == location)
            .and_then(|candidate| candidate.region)
            && let Some(cell) = self.data.world.cells.iter().find(|cell| cell.id == region)
        {
            self.map_x = cell.coordinate.x;
            self.map_y = cell.coordinate.y;
        }
    }

    fn sync_focus_to_time(&mut self) {
        let location = match self.focus {
            Some(EntityRef::Person(person)) => self
                .local_state()
                .and_then(|state| state.people.get(&person).copied()),
            _ => None,
        };
        if let Some(location) = location {
            self.sync_map_to_location(location);
        }
    }

    fn local_start_year(&self) -> Option<u32> {
        self.data
            .local
            .as_ref()
            .map(|local| local.summary.projection_year)
    }

    pub(crate) fn map_window(&self) -> (u16, u16, u16, u16) {
        let zoom = u16::from(self.map_zoom.max(1));
        let visible_width = (self.data.world.width / zoom).max(1);
        let visible_height = (self.data.world.height / zoom).max(1);
        let start_x = self
            .map_x
            .saturating_sub(visible_width / 2)
            .min(self.data.world.width.saturating_sub(visible_width));
        let start_y = self
            .map_y
            .saturating_sub(visible_height / 2)
            .min(self.data.world.height.saturating_sub(visible_height));
        (start_x, start_y, visible_width, visible_height)
    }

    pub(crate) fn macro_days_per_year(&self) -> u64 {
        self.data
            .history
            .as_ref()
            .and_then(|history| {
                history.events.iter().find_map(|event| {
                    let HistoricalEventPayloadV1::HistoryCompleted { elapsed_years } =
                        &event.payload
                    else {
                        return None;
                    };
                    (*elapsed_years > 0)
                        .then(|| event.time.day() / u64::from(*elapsed_years))
                        .filter(|days| *days > 0)
                })
            })
            .unwrap_or(360)
    }

    pub(crate) fn local_days_per_year(&self) -> u64 {
        self.data.local.as_ref().map_or(360, |local| {
            u64::from(local.simulation_summary.days_per_year)
        })
    }

    fn refresh_search(&mut self) {
        let query = self.query.trim().to_ascii_lowercase();
        let mut results = self
            .names
            .iter()
            .filter(|(entity, label)| {
                query.is_empty()
                    || label.to_ascii_lowercase().contains(&query)
                    || entity.kind().contains(&query)
                    || format!("{}:{}", entity.kind(), entity.raw()).contains(&query)
            })
            .map(|(entity, label)| {
                let lower = label.to_ascii_lowercase();
                let rank = if lower == query {
                    0
                } else if lower.starts_with(&query) {
                    1
                } else if entity.kind().starts_with(&query) {
                    2
                } else {
                    3
                };
                (rank, label.clone(), *entity)
            })
            .collect::<Vec<_>>();
        results.sort();
        self.search_results = results
            .into_iter()
            .map(|(_, _, entity)| entity)
            .take(200)
            .collect();
        self.selection = self
            .selection
            .min(self.search_results.len().saturating_sub(1));
    }

    fn build_indexes(&mut self) {
        self.index_world();
        self.index_history();
        self.index_local();
        for relations in self.relations.values_mut() {
            relations.sort_by(|left, right| {
                (left.label.as_str(), left.target).cmp(&(right.label.as_str(), right.target))
            });
            relations.dedup();
        }
        self.moments
            .sort_by_key(|moment| (moment.year, moment.day, moment.entity));
        for entities in self.catalog.values_mut() {
            entities.sort_by_key(|entity| {
                (self.names.get(entity).cloned().unwrap_or_default(), *entity)
            });
            entities.dedup();
        }
        self.macro_years =
            build_macro_years(self.data.history.as_ref(), self.macro_days_per_year());
        self.local_years = build_local_years(self.data.local.as_ref());
    }

    fn index_world(&mut self) {
        let width = usize::from(self.data.world.width);
        for cell in &self.data.world.cells {
            let entity = EntityRef::Region(cell.id);
            self.names.insert(
                entity,
                format!("Region {},{}", cell.coordinate.x, cell.coordinate.y),
            );
            for feature in &cell.feature_ids {
                add_relation(
                    &mut self.relations,
                    entity,
                    "contains trace",
                    EntityRef::Feature(*feature),
                );
            }
            self.catalog
                .entry(CatalogKind::Places)
                .or_default()
                .push(entity);
        }
        for feature in &self.data.world.features {
            let entity = EntityRef::Feature(feature.id);
            self.names.insert(entity, feature.name.clone());
            for region in &feature.regions {
                add_relation(
                    &mut self.relations,
                    entity,
                    "spans",
                    EntityRef::Region(*region),
                );
            }
            self.catalog
                .entry(CatalogKind::Places)
                .or_default()
                .push(entity);
        }
        for location in &self.data.world.places.locations {
            let entity = EntityRef::Location(location.id);
            self.names.insert(entity, location.name.clone());
            if let Some(region) = location.region {
                add_relation(
                    &mut self.relations,
                    entity,
                    "occupies",
                    EntityRef::Region(region),
                );
                if let Some(cell) = self.data.world.cells.get(region.0 as usize) {
                    self.map_x = self.map_x.min(cell.coordinate.x);
                    self.map_y = self.map_y.min(cell.coordinate.y);
                }
            }
            for feature in &location.feature_ids {
                add_relation(
                    &mut self.relations,
                    entity,
                    "near",
                    EntityRef::Feature(*feature),
                );
            }
            self.entity_locations.insert(entity, location.id);
            self.catalog
                .entry(CatalogKind::Places)
                .or_default()
                .push(entity);
        }
        for route in &self.data.world.places.routes {
            let entity = EntityRef::Route(route.id);
            self.names.insert(
                entity,
                format!(
                    "{:?} route #{} · cost {}",
                    route.kind, route.id.0, route.travel_cost
                ),
            );
            for endpoint in route.endpoints {
                add_relation(
                    &mut self.relations,
                    entity,
                    "connects",
                    EntityRef::Location(endpoint),
                );
            }
            self.catalog
                .entry(CatalogKind::Places)
                .or_default()
                .push(entity);
        }
        debug_assert_eq!(
            width * usize::from(self.data.world.height),
            self.data.world.cells.len()
        );
    }

    fn index_history(&mut self) {
        let Some(history) = self.data.history.clone() else {
            return;
        };
        for population in &history.populations {
            let entity = EntityRef::Population(population.id);
            self.names.insert(entity, population.name.clone());
            self.entity_locations.insert(entity, population.location_id);
            add_relation(
                &mut self.relations,
                entity,
                "at",
                EntityRef::Location(population.location_id),
            );
            for culture in &population.cultures {
                add_relation(
                    &mut self.relations,
                    entity,
                    "culture",
                    EntityRef::Culture(culture.id),
                );
            }
            for faith in &population.faiths {
                add_relation(
                    &mut self.relations,
                    entity,
                    "faith",
                    EntityRef::Faith(faith.id),
                );
            }
            self.catalog
                .entry(CatalogKind::Peoples)
                .or_default()
                .push(entity);
        }
        for culture in &history.cultures {
            let entity = EntityRef::Culture(culture.id);
            self.names.insert(entity, culture.name.clone());
            add_relation(
                &mut self.relations,
                entity,
                "founded by",
                EntityRef::MacroEvent(culture.origin_event),
            );
            self.catalog
                .entry(CatalogKind::Beliefs)
                .or_default()
                .push(entity);
        }
        for faith in &history.faiths {
            let entity = EntityRef::Faith(faith.id);
            self.names.insert(entity, faith.name.clone());
            add_relation(
                &mut self.relations,
                entity,
                "founded by",
                EntityRef::MacroEvent(faith.origin_event),
            );
            if let Some(parent) = faith.parent_faith_id {
                add_relation(
                    &mut self.relations,
                    entity,
                    "schism from",
                    EntityRef::Faith(parent),
                );
            }
            if let Some(feature) = faith.source_feature_id {
                add_relation(
                    &mut self.relations,
                    entity,
                    "source trace",
                    EntityRef::Feature(feature),
                );
            }
            self.catalog
                .entry(CatalogKind::Beliefs)
                .or_default()
                .push(entity);
        }
        for institution in &history.institutions {
            let entity = EntityRef::Institution(institution.id);
            self.names.insert(entity, institution.name.clone());
            self.entity_locations
                .insert(entity, institution.location_id);
            add_relation(
                &mut self.relations,
                entity,
                "at",
                EntityRef::Location(institution.location_id),
            );
            add_relation(
                &mut self.relations,
                entity,
                "culture",
                EntityRef::Culture(institution.culture_id),
            );
            if let Some(faith) = institution.faith_id {
                add_relation(
                    &mut self.relations,
                    entity,
                    "faith",
                    EntityRef::Faith(faith),
                );
            }
            self.catalog
                .entry(CatalogKind::Institutions)
                .or_default()
                .push(entity);
        }
        for polity in &history.polities {
            let entity = EntityRef::Polity(polity.id);
            self.names.insert(entity, polity.name.clone());
            for location in &polity.location_ids {
                add_relation(
                    &mut self.relations,
                    entity,
                    "governs",
                    EntityRef::Location(*location),
                );
            }
            for culture in &polity.culture_ids {
                add_relation(
                    &mut self.relations,
                    entity,
                    "culture",
                    EntityRef::Culture(*culture),
                );
            }
            self.catalog
                .entry(CatalogKind::Institutions)
                .or_default()
                .push(entity);
        }
        for claim in &history.lore {
            let entity = EntityRef::Claim(claim.id);
            self.names.insert(entity, claim.title.clone());
            add_relation(
                &mut self.relations,
                entity,
                "source culture",
                EntityRef::Culture(claim.source_culture_id),
            );
            if let Some(faith) = claim.source_faith_id {
                add_relation(
                    &mut self.relations,
                    entity,
                    "source faith",
                    EntityRef::Faith(faith),
                );
            }
            for event in &claim.about_events {
                add_relation(
                    &mut self.relations,
                    entity,
                    "interprets",
                    EntityRef::MacroEvent(*event),
                );
            }
            self.catalog
                .entry(CatalogKind::Claims)
                .or_default()
                .push(entity);
        }
        for event in &history.events {
            self.index_macro_event(event);
        }
    }

    fn index_macro_event(&mut self, event: &HistoricalEventV1) {
        let entity = EntityRef::MacroEvent(event.id);
        let label = macro_event_label(event);
        self.names.insert(entity, label.clone());
        let subjects = event
            .subjects
            .iter()
            .map(historical_subject_ref)
            .collect::<Vec<_>>();
        let causes = event
            .causes
            .iter()
            .map(|id| EntityRef::MacroEvent(*id))
            .collect::<Vec<_>>();
        for subject in &subjects {
            add_relation(&mut self.relations, entity, "subject", *subject);
        }
        for cause in &causes {
            add_relation(&mut self.relations, entity, "caused by", *cause);
        }
        if let Some(location) = event.location {
            self.entity_locations.insert(entity, location);
            add_relation(
                &mut self.relations,
                entity,
                "at",
                EntityRef::Location(location),
            );
        }
        self.moments.push(ObservatoryMoment {
            entity,
            year: u32::try_from(event.time.day() / self.macro_days_per_year()).unwrap_or(u32::MAX),
            day: event.time.day(),
            label,
            location: event.location,
            causes,
            subjects,
            tags: event.tags.clone(),
            debug: false,
        });
        self.catalog
            .entry(CatalogKind::Events)
            .or_default()
            .push(entity);
    }

    fn index_local(&mut self) {
        let Some(local) = self.data.local.clone() else {
            return;
        };
        for settlement in &local.settlements {
            let entity = EntityRef::Location(settlement.location_id);
            self.names.insert(entity, settlement.name.clone());
            self.entity_locations.insert(entity, settlement.location_id);
        }
        for household in &local.households {
            let entity = EntityRef::Household(household.id);
            self.names.insert(entity, household.name.clone());
            if let Some(location) = household.residence_id {
                self.entity_locations.insert(entity, location);
                add_relation(
                    &mut self.relations,
                    entity,
                    "resides",
                    EntityRef::Location(location),
                );
            }
            for person in &household.member_ids {
                add_relation(
                    &mut self.relations,
                    entity,
                    "member",
                    EntityRef::Person(*person),
                );
            }
            self.catalog
                .entry(CatalogKind::Households)
                .or_default()
                .push(entity);
        }
        for context in &local.household_contexts {
            let entity = EntityRef::Household(context.household_id);
            self.entity_locations.insert(entity, context.residence_id);
            for culture in &context.culture_ids {
                add_relation(
                    &mut self.relations,
                    entity,
                    "inherits culture",
                    EntityRef::Culture(*culture),
                );
            }
            for faith in &context.faith_ids {
                add_relation(
                    &mut self.relations,
                    entity,
                    "inherits faith",
                    EntityRef::Faith(*faith),
                );
            }
            for institution in &context.institution_ids {
                add_relation(
                    &mut self.relations,
                    entity,
                    "inherits institution",
                    EntityRef::Institution(*institution),
                );
            }
            for claim in &context.lore_claim_ids {
                add_relation(
                    &mut self.relations,
                    entity,
                    "inherits claim",
                    EntityRef::Claim(*claim),
                );
            }
        }
        for person in &local.people {
            let entity = EntityRef::Person(person.id);
            self.names.insert(entity, person.name.clone());
            if let Some(household) = person.household_id {
                add_relation(
                    &mut self.relations,
                    entity,
                    "household",
                    EntityRef::Household(household),
                );
                if let Some(location) = local
                    .households
                    .iter()
                    .find(|candidate| candidate.id == household)
                    .and_then(|candidate| candidate.residence_id)
                {
                    self.entity_locations.insert(entity, location);
                }
            }
            if let Some(partner) = person.partner_id {
                add_relation(
                    &mut self.relations,
                    entity,
                    "partner",
                    EntityRef::Person(partner),
                );
            }
            for parent in &person.parent_ids {
                add_relation(
                    &mut self.relations,
                    entity,
                    "parent",
                    EntityRef::Person(*parent),
                );
            }
            self.catalog
                .entry(CatalogKind::People)
                .or_default()
                .push(entity);
        }
        for item in &local.items {
            let entity = EntityRef::Item(item.id);
            self.names.insert(entity, item.name.clone());
            if let Some(location) = item.current_location_id {
                self.entity_locations.insert(entity, location);
                add_relation(
                    &mut self.relations,
                    entity,
                    "at",
                    EntityRef::Location(location),
                );
            }
            for source in &item.sources {
                add_relation(
                    &mut self.relations,
                    entity,
                    format!("{:?} source", source.role).to_ascii_lowercase(),
                    EntityRef::Item(source.item_id),
                );
            }
            if let Some(owner) = owner_ref(&item.owner) {
                add_relation(&mut self.relations, entity, "owned by", owner);
            }
            if let Some(custody) = custody_ref(&item.custody) {
                add_relation(&mut self.relations, entity, "held by", custody);
            }
            self.catalog
                .entry(CatalogKind::Items)
                .or_default()
                .push(entity);
        }
        for event in &local.events {
            self.index_local_event(event, local.summary.projection_year);
        }
    }

    fn index_local_event(&mut self, event: &merra_core::WorldEventV1, projection_year: u32) {
        let entity = EntityRef::LocalEvent(event.id);
        let label = local_event_label(event);
        self.names.insert(entity, label.clone());
        let subjects = event
            .subjects
            .iter()
            .map(world_subject_ref)
            .chain(event.actors.iter().map(|id| EntityRef::Person(*id)))
            .collect::<Vec<_>>();
        let causes = event
            .causes
            .iter()
            .map(|id| EntityRef::LocalEvent(*id))
            .collect::<Vec<_>>();
        for subject in &subjects {
            add_relation(&mut self.relations, entity, "subject", *subject);
        }
        for cause in &causes {
            add_relation(&mut self.relations, entity, "caused by", *cause);
        }
        if let Some(location) = event.location {
            self.entity_locations.insert(entity, location);
            add_relation(
                &mut self.relations,
                entity,
                "at",
                EntityRef::Location(location),
            );
        }
        let local_year = event.time.day() / self.local_days_per_year();
        self.moments.push(ObservatoryMoment {
            entity,
            year: projection_year.saturating_add(u32::try_from(local_year).unwrap_or(u32::MAX)),
            day: event.time.day(),
            label,
            location: event.location,
            causes,
            subjects,
            tags: event.tags.clone(),
            debug: matches!(
                event.kind,
                EventKindV1::SimulationStarted
                    | EventKindV1::TimeAdvanced
                    | EventKindV1::SeasonBegan
                    | EventKindV1::SimulationCompleted
            ),
        });
        self.catalog
            .entry(CatalogKind::Events)
            .or_default()
            .push(entity);
    }
}

/// Failures surfaced before entering the interactive terminal.
#[derive(Debug, Error)]
pub enum ObservatoryError {
    #[error("cannot read observatory data at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot decode observatory data at {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("cannot generate canonical observatory data: {0}")]
    Generation(String),
    #[error("invalid observatory input: {0}")]
    InvalidInput(String),
}

fn load_history_data(
    history_dir: &Path,
    local_dir: Option<&Path>,
) -> Result<ObservatoryData, ObservatoryError> {
    if !history_dir.is_dir() {
        return Err(ObservatoryError::InvalidInput(format!(
            "history input must be a directory: {}",
            history_dir.display()
        )));
    }
    let world_path = history_dir.join("world.json");
    let world_bytes = read_bytes(&world_path)?;
    let world: SurfaceWorldV1 = decode_json(&world_path, &world_bytes)?;
    let history_manifest: HistoryManifestV1 = read_json(&history_dir.join("manifest.json"))?;
    let actual_world_hash = blake3::hash(&world_bytes).to_hex().to_string();
    if actual_world_hash != history_manifest.world_hash {
        return Err(ObservatoryError::InvalidInput(format!(
            "history world hash does not match {}",
            world_path.display()
        )));
    }
    let regional_path = history_dir.join("regional-history.json");
    let regional_bytes = read_bytes(&regional_path)?;
    let regional: RegionalHistoryV1 = decode_json(&regional_path, &regional_bytes)?;
    let summary: HistorySummaryV1 = read_json(&history_dir.join("summary.json"))?;
    let history = HistoricalReport {
        title: regional.history_title.clone(),
        seed: summary.seed,
        years: summary.elapsed_years,
        events: read_json_lines(&history_dir.join("events.jsonl"))?,
        populations: read_json(&history_dir.join("populations.json"))?,
        settlements: read_json(&history_dir.join("settlements.json"))?,
        cultures: read_json(&history_dir.join("cultures.json"))?,
        faiths: read_json(&history_dir.join("faiths.json"))?,
        institutions: read_json(&history_dir.join("institutions.json"))?,
        polities: read_json(&history_dir.join("polities.json"))?,
        lore: read_json(&history_dir.join("lore.json"))?,
        important_places: read_json(&history_dir.join("important-places.json"))?,
        starting_region: read_json(&history_dir.join("starting-region.json"))?,
        summary,
        chronicle: read_string(&history_dir.join("chronicle.md"))?,
        open_route_ids: regional.open_route_ids.clone(),
    };
    let local = if let Some(local_dir) = local_dir {
        if !local_dir.is_dir() {
            return Err(ObservatoryError::InvalidInput(format!(
                "local input must be a directory: {}",
                local_dir.display()
            )));
        }
        let manifest: LocalHistoryManifestV1 = read_json(&local_dir.join("manifest.json"))?;
        if manifest.world_hash != actual_world_hash {
            return Err(ObservatoryError::InvalidInput(String::from(
                "local history was generated from a different world",
            )));
        }
        let actual_regional_hash = blake3::hash(&regional_bytes).to_hex().to_string();
        if manifest.regional_history_hash != actual_regional_hash {
            return Err(ObservatoryError::InvalidInput(String::from(
                "local history was generated from a different regional handoff",
            )));
        }
        Some(read_json(&local_dir.join("local-history.json"))?)
    } else {
        None
    };
    Ok(ObservatoryData {
        world,
        history: Some(history),
        local,
    })
}

fn resolve_file(path: &Path, filename: &str) -> PathBuf {
    if path.is_dir() {
        path.join(filename)
    } else {
        path.to_path_buf()
    }
}

fn read_bytes(path: &Path) -> Result<Vec<u8>, ObservatoryError> {
    fs::read(path).map_err(|source| ObservatoryError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn read_string(path: &Path) -> Result<String, ObservatoryError> {
    fs::read_to_string(path).map_err(|source| ObservatoryError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn decode_json<T: serde::de::DeserializeOwned>(
    path: &Path,
    bytes: &[u8],
) -> Result<T, ObservatoryError> {
    serde_json::from_slice(bytes).map_err(|source| ObservatoryError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ObservatoryError> {
    let bytes = read_bytes(path)?;
    decode_json(path, &bytes)
}

fn read_json_lines<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> Result<Vec<T>, ObservatoryError> {
    let contents = read_string(path)?;
    contents
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str(line).map_err(|source| ObservatoryError::Json {
                path: path.to_path_buf(),
                source,
            })
        })
        .collect()
}

fn add_relation(
    relations: &mut BTreeMap<EntityRef, Vec<Relation>>,
    source: EntityRef,
    label: impl Into<String>,
    target: EntityRef,
) {
    let label = label.into();
    relations.entry(source).or_default().push(Relation {
        label: label.clone(),
        target,
    });
    relations.entry(target).or_default().push(Relation {
        label: inverse_relation(&label).to_owned(),
        target: source,
    });
}

fn inverse_relation(label: &str) -> &str {
    match label {
        "at" | "occupies" | "resides" => "contains",
        "contains trace" | "spans" | "near" => "related place",
        "connects" => "route",
        "culture" | "inherits culture" | "source culture" => "represented by",
        "faith" | "inherits faith" | "source faith" => "represented by",
        "founded by" => "founded",
        "schism from" => "descendant faith",
        "source trace" => "inspired",
        "governs" => "governed by",
        "inherits institution" => "inherited by",
        "inherits claim" => "held by",
        "interprets" => "interpreted by",
        "member" => "household",
        "household" => "member",
        "partner" => "partner",
        "parent" => "child",
        "owned by" => "owns",
        "held by" => "holds",
        "subject" => "event",
        "caused by" => "caused",
        _ if label.ends_with(" source") => "descendant",
        _ => "related",
    }
}

fn historical_subject_ref(subject: &HistoricalSubjectV1) -> EntityRef {
    match subject {
        HistoricalSubjectV1::Population(id) => EntityRef::Population(*id),
        HistoricalSubjectV1::Location(id) => EntityRef::Location(*id),
        HistoricalSubjectV1::Culture(id) => EntityRef::Culture(*id),
        HistoricalSubjectV1::Faith(id) => EntityRef::Faith(*id),
        HistoricalSubjectV1::Institution(id) => EntityRef::Institution(*id),
        HistoricalSubjectV1::Polity(id) => EntityRef::Polity(*id),
        HistoricalSubjectV1::Feature(id) => EntityRef::Feature(*id),
    }
}

fn world_subject_ref(subject: &WorldSubjectV1) -> EntityRef {
    match subject {
        WorldSubjectV1::Person(id) => EntityRef::Person(*id),
        WorldSubjectV1::Household(id) => EntityRef::Household(*id),
        WorldSubjectV1::Item(id) => EntityRef::Item(*id),
        WorldSubjectV1::Location(id) => EntityRef::Location(*id),
        WorldSubjectV1::Institution(id) => EntityRef::Institution(*id),
        WorldSubjectV1::Polity(id) => EntityRef::Polity(*id),
    }
}

fn owner_ref(owner: &PropertyOwnerV1) -> Option<EntityRef> {
    Some(match owner {
        PropertyOwnerV1::Person(id) => EntityRef::Person(*id),
        PropertyOwnerV1::Household(id) => EntityRef::Household(*id),
        PropertyOwnerV1::Institution(id) => EntityRef::Institution(*id),
        PropertyOwnerV1::Settlement(id) => EntityRef::Location(*id),
        PropertyOwnerV1::Polity(id) => EntityRef::Polity(*id),
    })
}

fn custody_ref(custody: &ItemCustodyV1) -> Option<EntityRef> {
    match custody {
        ItemCustodyV1::Person(id) => Some(EntityRef::Person(*id)),
        ItemCustodyV1::Household(id) => Some(EntityRef::Household(*id)),
        ItemCustodyV1::Institution(id) => Some(EntityRef::Institution(*id)),
        ItemCustodyV1::AtLocation(id) => Some(EntityRef::Location(*id)),
        ItemCustodyV1::Unknown => None,
    }
}

fn macro_event_label(event: &HistoricalEventV1) -> String {
    match &event.payload {
        HistoricalEventPayloadV1::HistoryStarted { .. } => String::from("History began"),
        HistoricalEventPayloadV1::PopulationSeeded {
            population_id,
            people,
        } => format!("{people} people seeded in population #{}", population_id.0),
        HistoricalEventPayloadV1::SettlementFounded { name, .. } => {
            format!("{name} was founded")
        }
        HistoricalEventPayloadV1::PopulationMigrated {
            people, from, to, ..
        } => {
            format!("{people} people migrated #{} → #{}", from.0, to.0)
        }
        HistoricalEventPayloadV1::CultureFounded { name, .. } => {
            format!("{name} emerged")
        }
        HistoricalEventPayloadV1::FaithFounded { name, .. } => format!("{name} was founded"),
        HistoricalEventPayloadV1::InstitutionFounded { name, .. } => {
            format!("{name} was established")
        }
        HistoricalEventPayloadV1::PolityFounded { name, .. } => format!("{name} formed"),
        HistoricalEventPayloadV1::RouteOpened { route_id, .. } => {
            format!("Route #{} opened", route_id.0)
        }
        HistoricalEventPayloadV1::SeaRouteOpened { route_id } => {
            format!("Sea route #{} opened", route_id.0)
        }
        HistoricalEventPayloadV1::FirstContact { .. } => String::from("First contact"),
        HistoricalEventPayloadV1::PopulationsMixed { location_id, .. } => {
            format!("Populations mixed at #{}", location_id.0)
        }
        HistoricalEventPayloadV1::FaithSpread { faith_id, .. } => {
            format!("Faith #{} spread", faith_id.0)
        }
        HistoricalEventPayloadV1::FaithSchism {
            parent_id,
            child_id,
        } => format!("Faith #{} split into #{}", parent_id.0, child_id.0),
        HistoricalEventPayloadV1::SettlementAbandoned { location_id } => {
            format!("Settlement #{} was abandoned", location_id.0)
        }
        HistoricalEventPayloadV1::HistoryCompleted { elapsed_years } => {
            format!("Macro history completed at Year {elapsed_years}")
        }
    }
}

fn local_event_label(event: &merra_core::WorldEventV1) -> String {
    match &event.payload {
        EventPayloadV1::SimulationStarted { .. } => String::from("Local history began"),
        EventPayloadV1::PopulationInitialized { people } => {
            format!("{people} sampled people entered local history")
        }
        EventPayloadV1::TimeAdvanced { .. } => String::from("The local clock advanced"),
        EventPayloadV1::SeasonBegan {
            season_name, year, ..
        } => format!("{season_name} began in local Year {year}"),
        EventPayloadV1::HouseholdFormed { name, .. } => format!("{name} formed"),
        EventPayloadV1::PartnershipFormed { partners, .. } => {
            format!("People #{} and #{} partnered", partners[0].0, partners[1].0)
        }
        EventPayloadV1::PartnershipEnded { partners, .. } => {
            format!("Partnership #{} + #{} ended", partners[0].0, partners[1].0)
        }
        EventPayloadV1::PersonBorn { name, .. } => format!("{name} was born"),
        EventPayloadV1::HouseholdDissolved { name, .. } => format!("{name} dissolved"),
        EventPayloadV1::HouseholdSettled {
            household_id,
            destination_location_id,
            ..
        } => format!(
            "Household #{} settled at #{}",
            household_id.0, destination_location_id.0
        ),
        EventPayloadV1::PersonDied {
            name, age_years, ..
        } => {
            format!("{name} died aged {age_years}")
        }
        EventPayloadV1::HouseholdWorkCompleted {
            work_tag,
            effective_labor,
            ..
        } => format!("{work_tag} work produced {effective_labor} labor"),
        EventPayloadV1::ItemIntroduced { name, .. } => format!("{name} entered history"),
        EventPayloadV1::ItemUsed {
            work_tag,
            condition_before_per_10_000,
            condition_after_per_10_000,
            ..
        } => format!(
            "{work_tag} use wore an item {}%→{}%",
            condition_before_per_10_000 / 100,
            condition_after_per_10_000 / 100
        ),
        EventPayloadV1::ItemRepaired {
            condition_after_per_10_000,
            ..
        } => format!(
            "An item was repaired to {}%",
            condition_after_per_10_000 / 100
        ),
        EventPayloadV1::ItemTransformed {
            source_item_ids,
            output_item_ids,
            ..
        } => format!(
            "{} source item(s) became {} descendant(s)",
            source_item_ids.len(),
            output_item_ids.len()
        ),
        EventPayloadV1::ItemOwnershipTransferred { .. } => String::from("Item ownership changed"),
        EventPayloadV1::ItemCustodyTransferred { .. } => String::from("Item custody changed"),
        EventPayloadV1::ItemRelocated { from, to, .. } => {
            format!("An item moved #{}→#{}", from.0, to.0)
        }
        EventPayloadV1::ItemLost { .. } => String::from("An item was lost"),
        EventPayloadV1::ItemRecovered { .. } => String::from("An item was recovered"),
        EventPayloadV1::ItemDestroyed { .. } => String::from("An item was destroyed"),
        EventPayloadV1::SimulationCompleted { elapsed_years, .. } => {
            format!("Local history completed after {elapsed_years} years")
        }
    }
}

fn build_macro_years(
    history: Option<&HistoricalReport>,
    days_per_year: u64,
) -> Vec<MacroYearState> {
    let Some(history) = history else {
        return Vec::new();
    };
    (0..=history.years)
        .map(|year| {
            let movements = history
                .events
                .iter()
                .filter(|event| {
                    u32::try_from(event.time.day() / days_per_year)
                        .is_ok_and(|event_year| event_year == year)
                })
                .filter_map(|event| {
                    let HistoricalEventPayloadV1::PopulationMigrated {
                        population_id,
                        from,
                        to,
                        people,
                    } = &event.payload
                    else {
                        return None;
                    };
                    Some(MacroMovement {
                        event_id: event.id,
                        population_id: *population_id,
                        people: *people,
                        from: *from,
                        to: *to,
                    })
                })
                .collect();
            MacroYearState { year, movements }
        })
        .collect()
}

fn build_local_years(local: Option<&LocalHistoryReportV1>) -> Vec<LocalYearState> {
    let Some(local) = local else {
        return Vec::new();
    };
    let playback = LocalHistoryPlaybackV1::from_report(local);
    let first = playback.projection_year;
    let last = first.saturating_add(playback.elapsed_years);
    (first..=last)
        .map(|year| local_state_at_year(local, &playback, year))
        .collect()
}

fn local_state_at_year(
    local: &LocalHistoryReportV1,
    playback: &LocalHistoryPlaybackV1,
    year: u32,
) -> LocalYearState {
    let elapsed = year.saturating_sub(playback.projection_year);
    let through_day = u64::from(elapsed).saturating_mul(u64::from(playback.days_per_year));
    let period_start =
        u64::from(elapsed.saturating_sub(1)).saturating_mul(u64::from(playback.days_per_year));
    let mut alive = playback
        .people
        .iter()
        .filter(|person| person.birth_day.is_none())
        .map(|person| person.id)
        .collect::<BTreeSet<_>>();
    let mut locations = BTreeMap::<PersonId, LocationId>::new();
    let mut births = 0;
    let mut deaths = 0;
    let mut migrations = 0;
    let mut movements = Vec::<PersonMovement>::new();
    for event in &playback.events {
        let day = match event {
            LocalPlaybackEventV1::HouseholdSettled { day, .. }
            | LocalPlaybackEventV1::PersonBorn { day, .. }
            | LocalPlaybackEventV1::PersonDied { day, .. } => *day,
        };
        if day > through_day {
            continue;
        }
        match event {
            LocalPlaybackEventV1::HouseholdSettled {
                event_id,
                destination_location_id,
                traveler_ids,
                origin_location_ids,
                ..
            } => {
                if !origin_location_ids.is_empty()
                    && origin_location_ids
                        .iter()
                        .any(|origin| *origin != *destination_location_id)
                {
                    migrations += 1;
                }
                let occurs_this_year = if elapsed == 0 {
                    day == 0
                } else {
                    day > period_start
                };
                if occurs_this_year {
                    let mut by_origin = BTreeMap::<LocationId, Vec<PersonId>>::new();
                    for person in traveler_ids {
                        if alive.contains(person)
                            && let Some(origin) = locations.get(person).copied()
                            && origin != *destination_location_id
                        {
                            by_origin.entry(origin).or_default().push(*person);
                        }
                    }
                    movements.extend(by_origin.into_iter().map(|(from, people)| PersonMovement {
                        event_id: *event_id,
                        people,
                        from,
                        to: *destination_location_id,
                    }));
                }
                for person in traveler_ids {
                    locations.insert(*person, *destination_location_id);
                }
            }
            LocalPlaybackEventV1::PersonBorn {
                person_id,
                location_id,
                ..
            } => {
                births += 1;
                alive.insert(*person_id);
                locations.insert(*person_id, *location_id);
            }
            LocalPlaybackEventV1::PersonDied { person_id, .. } => {
                deaths += 1;
                alive.remove(person_id);
            }
        }
    }
    let mut residents = BTreeMap::<LocationId, u32>::new();
    let mut people = BTreeMap::<PersonId, LocationId>::new();
    for person in alive {
        if let Some(location) = locations.get(&person) {
            *residents.entry(*location).or_default() += 1;
            people.insert(person, *location);
        }
    }
    let item_events = local
        .events
        .iter()
        .filter(|event| event.time.day() <= through_day)
        .filter(|event| {
            event
                .subjects
                .iter()
                .any(|subject| matches!(subject, WorldSubjectV1::Item(_)))
        })
        .count();
    LocalYearState {
        year,
        residents,
        people,
        movements,
        births,
        deaths,
        migrations,
        item_events,
    }
}

fn add_signed(value: u16, delta: i16) -> u16 {
    if delta.is_negative() {
        value.saturating_sub(delta.unsigned_abs())
    } else {
        value.saturating_add(delta.unsigned_abs())
    }
}

fn contains(area: Rect, column: u16, row: u16) -> bool {
    column >= area.x
        && column < area.x.saturating_add(area.width)
        && row >= area.y
        && row < area.y.saturating_add(area.height)
}
