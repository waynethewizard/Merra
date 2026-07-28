import fs from "node:fs";
import path from "node:path";
import {
  getCurrentCycle,
  getFoundationRun,
  repoRoot,
  type WorldEvent
} from "../src/lib/content";

const errors: string[] = [];

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) errors.push(message);
}

const run = getFoundationRun();
const cycle = getCurrentCycle();

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
for (const event of run.events as WorldEvent[]) {
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

console.log(
  `Validated ${run.events.length} events, seed ${run.manifest.seed}, and ${cycle.title}.`
);
