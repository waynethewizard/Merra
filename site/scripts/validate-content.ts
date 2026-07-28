import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const errors: string[] = [];
const siteRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  ".."
);
const repoRoot = path.resolve(siteRoot, "..");

type WorldEvent = {
  id: number;
  kind: string;
  causes: number[];
  payload: { type: string };
};

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) errors.push(message);
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

function parseTomlFrontmatter(source: string): {
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

const goldenDirectory = path.join(
  repoRoot,
  "golden",
  "era-01",
  "the-first-clock"
);
const events = fs
  .readFileSync(path.join(goldenDirectory, "events.jsonl"), "utf8")
  .trim()
  .split("\n")
  .filter(Boolean)
  .map((line) => JSON.parse(line) as WorldEvent);
const manifest = readJson<{
  schema_version: number;
  event_schema_version: number;
  scenario_id: string;
  scenario_hash: string;
  seed: number;
}>(path.join(goldenDirectory, "manifest.json"));
const summary = readJson<{
  schema_version: number;
  scenario_id: string;
  seed: number;
  event_count: number;
}>(path.join(goldenDirectory, "summary.json"));
const foundationRun = {
  slug: "the-first-clock",
  title: "The First Clock",
  description:
    "Merra’s smallest reproducible history: one explicit calendar, one seed, and a causal record of time beginning.",
  command:
    "cargo merra run --scenario scenarios/era-01/smoke.ron --seed 42 --years 1 --output runs/foundation-smoke",
  manifest,
  summary,
  events,
  chronicle: fs.readFileSync(
    path.join(goldenDirectory, "chronicle.md"),
    "utf8"
  )
};
const cycleSource = fs.readFileSync(
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
const { data: cycleData, body: cycleBody } =
  parseTomlFrontmatter(cycleSource);
const currentCycle = {
  era: Number(cycleData.era),
  cycle: Number(cycleData.cycle),
  slug: String(cycleData.slug),
  title: String(cycleData.title),
  status: String(cycleData.status),
  started: String(cycleData.started),
  scenario: String(cycleData.scenario),
  seeds: Array.isArray(cycleData.seeds) ? cycleData.seeds : [],
  body: cycleBody
};

const run = foundationRun;
const cycle = currentCycle;

assert(run.manifest.schema_version === 1, "manifest schema must be v1");
assert(run.manifest.event_schema_version === 1, "event schema must be v1");
assert(run.summary.schema_version === 1, "summary schema must be v1");
assert(
  run.summary.event_count === run.events.length,
  "summary event_count must match events.jsonl"
);
assert(
  run.manifest.seed === run.summary.seed,
  "manifest and summary seeds must match"
);
assert(
  run.manifest.scenario_id === run.summary.scenario_id,
  "manifest and summary scenario IDs must match"
);
assert(
  /^[a-f0-9]{64}$/.test(run.manifest.scenario_hash),
  "scenario hash must be a 64-character lowercase hex digest"
);
assert(cycle.era > 0 && cycle.cycle > 0, "cycle era and number are required");
assert(Boolean(cycle.slug && cycle.title), "cycle slug and title are required");
assert(Boolean(cycle.status && cycle.started), "cycle status and date are required");
assert(cycle.seeds.includes(run.manifest.seed), "golden seed must appear in cycle");

let previousId = 0;
const knownIds = new Set<number>();
for (const event of run.events) {
  assert(event.id > previousId, `event ${event.id} is not in stable ID order`);
  assert(
    event.causes.every((cause) => knownIds.has(cause)),
    `event ${event.id} references an unknown or future cause`
  );
  assert(Boolean(event.kind && event.payload.type), `event ${event.id} is untyped`);
  previousId = event.id;
  knownIds.add(event.id);
}

for (const relativePath of [
  "docs/roadmap.md",
  "docs/design-principles.md",
  "scenarios/era-01/smoke.ron"
]) {
  assert(
    fs.existsSync(path.join(repoRoot, relativePath)),
    `missing public source: ${relativePath}`
  );
}

if (errors.length) {
  console.error("Content validation failed:");
  for (const error of errors) console.error(`- ${error}`);
  process.exit(1);
}

const generatedDirectory = path.join(siteRoot, "src", "generated");
fs.mkdirSync(generatedDirectory, { recursive: true });
fs.writeFileSync(
  path.join(generatedDirectory, "content.json"),
  `${JSON.stringify({ foundationRun, currentCycle })}\n`
);

console.log(
  `Validated and embedded ${run.events.length} events, seed ${run.manifest.seed}, and ${cycle.title}.`
);
