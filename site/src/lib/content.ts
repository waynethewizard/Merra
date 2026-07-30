import content from "@/generated/content.json";

export type EventPayload = {
  type: string;
  [key: string]: string | number;
};

export type WorldEvent = {
  id: number;
  time: { day: number };
  kind: string;
  actors: number[];
  location: number | null;
  causes: number[];
  tags: string[];
  payload: EventPayload;
};

export type RunManifest = {
  schema_version: number;
  event_schema_version: number;
  scenario_schema_version: number;
  merra_version: string;
  bevy_version: string;
  rust_version: string;
  source: {
    git_commit: string | null;
    dirty: boolean | null;
  };
  scenario_id: string;
  scenario_hash: string;
  seed: number;
  years: number;
  days: number;
};

export type RunSummary = {
  schema_version: number;
  scenario_id: string;
  seed: number;
  elapsed_days: number;
  elapsed_years: number;
  event_count: number;
};

export type GoldenRun = {
  slug: string;
  title: string;
  description: string;
  command: string;
  manifest: RunManifest;
  summary: RunSummary;
  events: WorldEvent[];
  chronicle: string;
};

export type CycleRecord = {
  era: number;
  cycle: number;
  slug: string;
  title: string;
  status: string;
  started: string;
  completed: string;
  codeTag: string;
  scenario: string;
  seeds: number[];
  body: string;
};

export type TerminalShowcase = {
  title: string;
  description: string;
  command: string;
  scenarioId: string;
  seed: number;
  years: number;
  eventCount: number;
  initialPopulation: number;
  births: number;
  livingPopulation: number;
  deaths: number;
  householdCount: number;
  views: {
    slug: string;
    title: string;
    description: string;
    screen: string;
  }[];
};

export type WorldGenesisShowcase = {
  title: string;
  description: string;
  command: string;
  seed: number;
  years: number;
  world: {
    regions: number;
    landRegions: number;
    islandRegions: number;
    riverRegions: number;
    biomes: number;
    features: number;
    places: number;
    routes: number;
  };
  history: {
    totalPopulation: number;
    populationCohorts: number;
    settlements: number;
    cultures: number;
    faiths: number;
    institutions: number;
    mixedLineagePopulations: number;
    firstContactYear: number;
    eventCount: number;
  };
  atlasSvg: string;
  tuiScreen: string;
  stages: {
    name: string;
    result: string;
  }[];
  lineages: {
    name: string;
    homeland: string;
    mortality: number;
    power: number;
    speed: number;
    sustenance: number;
  }[];
  cultures: {
    name: string;
    foundedYear: number;
    ritualDays: number;
  }[];
  faiths: {
    name: string;
    foundedYear: number;
  }[];
  lore: {
    title: string;
    text: string;
    confidence: number;
  }[];
  startingRegion: {
    settlementCount: number;
    eventCount: number;
    summary: string;
  };
};

export type LocalHistoryShowcase = {
  title: string;
  description: string;
  command: string;
  seed: number;
  projectionYear: number;
  years: number;
  macroPopulation: number;
  representedPopulation: number;
  livingPeople: number;
  births: number;
  deaths: number;
  residenceDecisions: number;
  migrations: number;
  locatedEvents: number;
  settlements: {
    location_id: number;
    name: string;
    macro_population: number;
    represented_population: number;
    initial_sample_people: number;
    final_living_people: number;
    births: number;
    deaths: number;
    arrivals: number;
    departures: number;
    active_households: number;
  }[];
  connections: {
    from: number;
    to: number;
    travel_cost: number;
    travel_days: number;
    route_ids: number[];
    path: number[];
  }[];
  playback: LocalHistoryPlayback;
  views: {
    slug: string;
    title: string;
    description: string;
    screen: string;
  }[];
};

export type LocalPlaybackPerson = {
  id: number;
  name: string;
  generation: number;
  starting_age_years: number;
  birth_day: number | null;
  death_day: number | null;
  parent_ids: number[];
};

export type LocalPlaybackEvent =
  | {
      type: "household_settled";
      event_id: number;
      day: number;
      household_id: number;
      origin_location_ids: number[];
      destination_location_id: number;
      traveler_ids: number[];
      route_ids: number[];
      travel_cost: number;
      travel_days: number;
      living_kin_support: number;
      reason:
        | "macro_projection"
        | "living_kin"
        | "shortest_journey"
        | "seeded_tie_break";
    }
  | {
      type: "person_born";
      event_id: number;
      day: number;
      person_id: number;
      household_id: number;
      location_id: number;
    }
  | {
      type: "person_died";
      event_id: number;
      day: number;
      person_id: number;
      age_years: number;
      location_id: number;
    };

export type LocalHistoryPlayback = {
  schema_version: number;
  seed: number;
  projection_year: number;
  elapsed_years: number;
  days_per_year: number;
  people: LocalPlaybackPerson[];
  events: LocalPlaybackEvent[];
};

export type HistoryLoreShowcase = {
  seed: number;
  startYear: number;
  projectionYear: number;
  endYear: number;
  recordedEvents: number;
  localLocatedEvents: number;
  firstContact: {
    year: number;
    eventId: number;
    routeEventId: number;
    locationId: number;
    record: string;
  };
  milestones: {
    years: string;
    phase: string;
    title: string;
    description: string;
    evidenceScope: string;
    eventIds: number[];
  }[];
  claims: {
    id: number;
    title: string;
    text: string;
    sourceCulture: string;
    sourceFaith: string | null;
    confidence: number;
    aboutEventIds: number[];
  }[];
  macroChronicle: string;
  localChronicle: string;
};

export type ItemHolder = {
  type: "person" | "household" | "institution" | "settlement" | "polity";
  id: number;
};

export type ItemSource = {
  item_id: number;
  role: "material" | "component" | "pattern";
};

export type ItemRecord = {
  id: number;
  archetype_id: string;
  name: string;
  introduced_day: number;
  introduction_event_id: number;
  sources?: ItemSource[];
  lineage_generation: number;
  condition_per_10_000: number;
  repairs: number;
  status: "active" | "lost" | "transformed" | "destroyed" | "consumed";
  owner: ItemHolder;
  custody: ItemHolder;
  current_location_id: number | null;
};

export type ItemBiographyEntry = {
  year: number;
  eventId: number;
  text: string;
};

export type ItemLineageShowcase = {
  title: string;
  description: string;
  command: string;
  seed: number;
  projectionYear: number;
  years: number;
  summary: {
    items: number;
    activeItems: number;
    transfers: number;
    repairs: number;
    transformations: number;
    maximumGeneration: number;
  };
  items: ItemRecord[];
  settlements: {
    id: number;
    name: string;
  }[];
  featuredItemId: number;
  biography: ItemBiographyEntry[];
  terminalScreen: string;
  chronicle: string;
};

const snapshot = content as unknown as {
  foundationRun: GoldenRun;
  currentCycle: CycleRecord;
  terminalShowcase: TerminalShowcase;
  worldGenesisShowcase: WorldGenesisShowcase;
  localHistoryShowcase: LocalHistoryShowcase;
  historyLoreShowcase: HistoryLoreShowcase;
  itemLineageShowcase: ItemLineageShowcase;
};

export function getFoundationRun(): GoldenRun {
  return snapshot.foundationRun;
}

export function getCurrentCycle(): CycleRecord {
  return snapshot.currentCycle;
}

export function getTerminalShowcase(): TerminalShowcase {
  return snapshot.terminalShowcase;
}

export function getWorldGenesisShowcase(): WorldGenesisShowcase {
  return snapshot.worldGenesisShowcase;
}

export function getLocalHistoryShowcase(): LocalHistoryShowcase {
  return snapshot.localHistoryShowcase;
}

export function getHistoryLoreShowcase(): HistoryLoreShowcase {
  return snapshot.historyLoreShowcase;
}

export function getItemLineageShowcase(): ItemLineageShowcase {
  return snapshot.itemLineageShowcase;
}
