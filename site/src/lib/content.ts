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
  scenario: string;
  seeds: number[];
  body: string;
};

const snapshot = content as unknown as {
  foundationRun: GoldenRun;
  currentCycle: CycleRecord;
};

export function getFoundationRun(): GoldenRun {
  return snapshot.foundationRun;
}

export function getCurrentCycle(): CycleRecord {
  return snapshot.currentCycle;
}
