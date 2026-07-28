import fs from "node:fs";
import path from "node:path";

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

export const repoRoot = path.resolve(process.cwd(), "..");

if (!fs.existsSync(path.join(repoRoot, "Cargo.toml"))) {
  throw new Error("Run site commands from the site package directory.");
}

function readJson<T>(filePath: string): T {
  return JSON.parse(fs.readFileSync(filePath, "utf8")) as T;
}

function parseTomlValue(value: string): string | number | number[] {
  const trimmed = value.trim();
  if (trimmed.startsWith("\"") && trimmed.endsWith("\"")) {
    return trimmed.slice(1, -1);
  }
  if (trimmed.startsWith("[") && trimmed.endsWith("]")) {
    return trimmed
      .slice(1, -1)
      .split(",")
      .map((item) => item.trim())
      .filter(Boolean)
      .map(Number);
  }
  const numeric = Number(trimmed);
  return Number.isNaN(numeric) ? trimmed : numeric;
}

export function parseTomlFrontmatter(source: string): {
  data: Record<string, string | number | number[]>;
  body: string;
} {
  if (!source.startsWith("+++\n")) {
    throw new Error("Expected TOML frontmatter.");
  }

  const end = source.indexOf("\n+++\n", 4);
  if (end === -1) {
    throw new Error("TOML frontmatter is not closed.");
  }

  const header = source.slice(4, end);
  const data: Record<string, string | number | number[]> = {};
  for (const line of header.split("\n")) {
    const separator = line.indexOf("=");
    if (separator === -1) continue;
    const key = line.slice(0, separator).trim();
    data[key] = parseTomlValue(line.slice(separator + 1));
  }

  return {
    data,
    body: source.slice(end + 5).trim()
  };
}

export function getFoundationRun(): GoldenRun {
  const directory = path.join(
    repoRoot,
    "golden",
    "era-01",
    "the-first-clock"
  );
  const events = fs
    .readFileSync(path.join(directory, "events.jsonl"), "utf8")
    .trim()
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line) as WorldEvent);

  return {
    slug: "the-first-clock",
    title: "The First Clock",
    description:
      "Merra’s smallest reproducible history: one explicit calendar, one seed, and a causal record of time beginning.",
    command:
      "cargo merra run --scenario scenarios/era-01/smoke.ron --seed 42 --years 1 --output runs/foundation-smoke",
    manifest: readJson<RunManifest>(path.join(directory, "manifest.json")),
    summary: readJson<RunSummary>(path.join(directory, "summary.json")),
    events,
    chronicle: fs.readFileSync(path.join(directory, "chronicle.md"), "utf8")
  };
}

export function getCurrentCycle(): CycleRecord {
  const source = fs.readFileSync(
    path.join(
      repoRoot,
      "docs",
      "devlog",
      "era-01",
      "cycles",
      "01-time-and-death.md"
    ),
    "utf8"
  );
  const { data, body } = parseTomlFrontmatter(source);

  return {
    era: Number(data.era),
    cycle: Number(data.cycle),
    slug: String(data.slug),
    title: String(data.title),
    status: String(data.status),
    started: String(data.started),
    scenario: String(data.scenario),
    seeds: Array.isArray(data.seeds) ? data.seeds : [],
    body
  };
}

export function getRoadmap(): string {
  return fs.readFileSync(path.join(repoRoot, "docs", "roadmap.md"), "utf8");
}
