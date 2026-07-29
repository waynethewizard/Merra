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

const snapshot = content as unknown as {
  foundationRun: GoldenRun;
  currentCycle: CycleRecord;
  terminalShowcase: TerminalShowcase;
  worldGenesisShowcase: WorldGenesisShowcase;
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
