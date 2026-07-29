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
    "05-five-villages.md"
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
  completed: String(cycleData.completed ?? ""),
  codeTag: String(cycleData.code_tag ?? ""),
  scenario: String(cycleData.scenario),
  seeds: Array.isArray(cycleData.seeds) ? cycleData.seeds : [],
  body: cycleBody
};
const dynastyDirectory = path.join(
  repoRoot,
  "golden",
  "era-01",
  "dynasty-seed-42"
);
const dynastySummary = readJson<{
  scenario_id: string;
  seed: number;
  elapsed_years: number;
  event_count: number;
  initial_population: number;
  living_population: number;
  deaths: number;
}>(path.join(dynastyDirectory, "summary.json"));
const terminalViews = [
  {
    slug: "overview",
    title: "Overview",
    description:
      "Population, generations, surname survival, and one featured life at a glance.",
    file: "tui-overview.txt"
  },
  {
    slug: "history",
    title: "History",
    description:
      "Story events first, with resolved names, households, payloads, and causes.",
    file: "tui-history.txt"
  },
  {
    slug: "people",
    title: "People",
    description:
      "Searchable biographies that distinguish current state from recorded history.",
    file: "tui-people.txt"
  },
  {
    slug: "lineage",
    title: "Lineage",
    description:
      "Children remain grouped under the partnership that actually produced them.",
    file: "tui-lineage.txt"
  },
  {
    slug: "households",
    title: "Households",
    description:
      "Current membership and reconstructed moves, births, deaths, and dissolution.",
    file: "tui-households.txt"
  }
].map(({ file, ...view }) => ({
  ...view,
  screen: fs.readFileSync(path.join(dynastyDirectory, file), "utf8")
}));
const dynastyChronicle = fs.readFileSync(
  path.join(dynastyDirectory, "chronicle.md"),
  "utf8"
);
const householdCount = Number(
  dynastyChronicle.match(/Families: (\d+) households formed/)?.[1]
);
const terminalShowcase = {
  title: "Four Generations of Thorn and Fen",
  description:
    "A story-first terminal field report over the complete authoritative record for Cycle 2.",
  command: "cargo tui",
  scenarioId: dynastySummary.scenario_id,
  seed: dynastySummary.seed,
  years: dynastySummary.elapsed_years,
  eventCount: dynastySummary.event_count,
  initialPopulation: dynastySummary.initial_population,
  births:
    dynastySummary.living_population +
    dynastySummary.deaths -
    dynastySummary.initial_population,
  livingPopulation: dynastySummary.living_population,
  deaths: dynastySummary.deaths,
  householdCount,
  views: terminalViews
};

const worldGenesisDirectory = path.join(
  repoRoot,
  "golden",
  "era-01",
  "first-histories-seed-42"
);
const worldSummary = readJson<{
  seed: number;
  regions: number;
  land_regions: number;
  island_regions: number;
  river_regions: number;
  biome_count: number;
  feature_count: number;
  location_count: number;
  route_count: number;
  locked_sea_routes: number;
}>(path.join(worldGenesisDirectory, "world-summary.json"));
const historySummary = readJson<{
  seed: number;
  elapsed_years: number;
  total_population: number;
  population_cohorts: number;
  settlements: number;
  cultures: number;
  faiths: number;
  institutions: number;
  mixed_lineage_populations: number;
  first_contact_year: number;
  event_count: number;
}>(path.join(worldGenesisDirectory, "history-summary.json"));
const cultures = readJson<
  {
    name: string;
    founded_year: number;
    ritual_days_per_year: number;
  }[]
>(path.join(worldGenesisDirectory, "cultures.json"));
const faiths = readJson<
  {
    name: string;
    founded_year: number;
  }[]
>(path.join(worldGenesisDirectory, "faiths.json"));
const lore = readJson<
  {
    title: string;
    text: string;
    confidence_per_10_000: number;
  }[]
>(path.join(worldGenesisDirectory, "lore.json"));
const startingRegion = readJson<{
  settlement_ids: number[];
  event_ids: number[];
  summary: string;
}>(path.join(worldGenesisDirectory, "starting-region.json"));
const atlasSvg = fs.readFileSync(
  path.join(worldGenesisDirectory, "history-atlas.svg"),
  "utf8"
);
const worldTuiScreen = fs.readFileSync(
  path.join(worldGenesisDirectory, "tui-world.txt"),
  "utf8"
);
const worldGenesisShowcase = {
  title: "Before Memory / The First Histories",
  description:
    "One deterministic world generated before its peoples: terrain, climate, water, resources, mythic traces, places, then six centuries of migration and cultural history.",
  command:
    "cargo merra worldgen --template scenarios/era-01/before-memory.ron --seed 42 --output runs/before-memory-42",
  seed: worldSummary.seed,
  years: historySummary.elapsed_years,
  world: {
    regions: worldSummary.regions,
    landRegions: worldSummary.land_regions,
    islandRegions: worldSummary.island_regions,
    riverRegions: worldSummary.river_regions,
    biomes: worldSummary.biome_count,
    features: worldSummary.feature_count,
    places: worldSummary.location_count,
    routes: worldSummary.route_count
  },
  history: {
    totalPopulation: historySummary.total_population,
    populationCohorts: historySummary.population_cohorts,
    settlements: historySummary.settlements,
    cultures: historySummary.cultures,
    faiths: historySummary.faiths,
    institutions: historySummary.institutions,
    mixedLineagePopulations: historySummary.mixed_lineage_populations,
    firstContactYear: historySummary.first_contact_year,
    eventCount: historySummary.event_count
  },
  atlasSvg,
  tuiScreen: worldTuiScreen,
  stages: [
    {
      name: "Deep structure",
      result: "Tectonic plates and integer elevation establish a reproducible landmass."
    },
    {
      name: "Living surface",
      result: "Climate, drainage, rivers, biomes, and resources constrain habitation."
    },
    {
      name: "Meaning",
      result: "Mythic traces and affordances make places culturally legible."
    },
    {
      name: "Peoples",
      result: "Three human cohorts and one orc cohort begin in separate homelands."
    },
    {
      name: "History",
      result: "Migration, settlement, institutions, navigation, and belief run for 600 years."
    },
    {
      name: "Playable region",
      result: "Five connected settlements preserve local evidence of world-scale history."
    }
  ],
  lineages: [
    {
      name: "Humans",
      homeland: "Continental watersheds",
      mortality: 1,
      power: 1,
      speed: 1,
      sustenance: 1
    },
    {
      name: "Orcs",
      homeland: "Remote island valley",
      mortality: 0.75,
      power: 1.25,
      speed: 1,
      sustenance: 1.125
    }
  ],
  cultures: cultures.map((culture) => ({
    name: culture.name,
    foundedYear: culture.founded_year,
    ritualDays: culture.ritual_days_per_year
  })),
  faiths: faiths.map((faith) => ({
    name: faith.name,
    foundedYear: faith.founded_year
  })),
  lore: lore.map((claim) => ({
    title: claim.title,
    text: claim.text,
    confidence: claim.confidence_per_10_000 / 100
  })),
  startingRegion: {
    settlementCount: startingRegion.settlement_ids.length,
    eventCount: startingRegion.event_ids.length,
    summary: startingRegion.summary
  }
};

const localHistoryDirectory = path.join(
  repoRoot,
  "golden",
  "era-01",
  "five-villages-seed-42"
);
const localHistorySummary = readJson<{
  seed: number;
  projection_year: number;
  elapsed_years: number;
  settlements: number;
  macro_population: number;
  represented_population: number;
  living_sample_people: number;
  births: number;
  deaths: number;
  residence_decisions: number;
  household_migrations: number;
  located_events: number;
}>(path.join(localHistoryDirectory, "summary.json"));
const localSettlements = readJson<
  {
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
  }[]
>(path.join(localHistoryDirectory, "settlements.json"));
const localConnections = readJson<
  {
    from: number;
    to: number;
    travel_cost: number;
    travel_days: number;
    route_ids: number[];
    path: number[];
  }[]
>(path.join(localHistoryDirectory, "connections.json"));
type LocalPlaybackPerson = {
  id: number;
  name: string;
  generation: number;
  starting_age_years: number;
  birth_day: number | null;
  death_day: number | null;
  parent_ids: number[];
};
type LocalPlaybackEvent =
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
      reason: string;
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
const localPlayback = readJson<{
  schema_version: number;
  seed: number;
  projection_year: number;
  elapsed_years: number;
  days_per_year: number;
  people: LocalPlaybackPerson[];
  events: LocalPlaybackEvent[];
}>(path.join(localHistoryDirectory, "playback.json"));
const localTerminalViews = [
  {
    slug: "overview",
    title: "Overview",
    description:
      "The comparative consequence first: one village grows while another empties.",
    file: "tui-overview.txt"
  },
  {
    slug: "roads",
    title: "Roads",
    description:
      "Exact shortest paths and pairwise costs without invented map geometry.",
    file: "tui-roads.txt"
  },
  {
    slug: "settlements",
    title: "Settlements",
    description:
      "Macro reconciliation, local vital events, migration, and surviving homes.",
    file: "tui-settlements.txt"
  },
  {
    slug: "migrations",
    title: "Migrations",
    description:
      "Origins, destination, kin support, road cost, route, and causal evidence.",
    file: "tui-migrations.txt"
  },
  {
    slug: "households",
    title: "Households",
    description:
      "Residence, represented cohorts, institutions, faiths, and inherited claims.",
    file: "tui-households.txt"
  }
].map(({ file, ...view }) => ({
  ...view,
  screen: fs.readFileSync(path.join(localHistoryDirectory, file), "utf8")
}));
const localHistoryShowcase = {
  title: "Five Villages After First Contact",
  description:
    "A Year 600 aggregate region becomes 60 years of located household history, with every macro person reconciled and every move explainable.",
  command:
    "cargo tui villages --input runs/five-villages-42 --snapshot --view overview",
  seed: localHistorySummary.seed,
  projectionYear: localHistorySummary.projection_year,
  years: localHistorySummary.elapsed_years,
  macroPopulation: localHistorySummary.macro_population,
  representedPopulation: localHistorySummary.represented_population,
  livingPeople: localHistorySummary.living_sample_people,
  births: localHistorySummary.births,
  deaths: localHistorySummary.deaths,
  residenceDecisions: localHistorySummary.residence_decisions,
  migrations: localHistorySummary.household_migrations,
  locatedEvents: localHistorySummary.located_events,
  settlements: localSettlements,
  connections: localConnections,
  playback: localPlayback,
  views: localTerminalViews
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
assert(
  ["planned", "in_progress", "complete"].includes(cycle.status),
  "cycle status must be planned, in_progress, or complete"
);
if (cycle.status === "complete") {
  assert(Boolean(cycle.completed), "a complete cycle requires a completion date");
  assert(Boolean(cycle.codeTag), "a complete cycle requires a code tag");
  assert(
    cycle.codeTag ===
      `era-${String(cycle.era).padStart(2, "0")}-cycle-${String(cycle.cycle).padStart(2, "0")}`,
    "cycle code tag must match its era and cycle number"
  );
}
assert(cycle.seeds.length > 0, "cycle requires at least one reproducible seed");
assert(
  fs.existsSync(path.join(repoRoot, cycle.scenario)),
  `missing cycle scenario: ${cycle.scenario}`
);
assert(
  terminalShowcase.scenarioId === "era-01-dynasty" &&
    terminalShowcase.seed === 42 &&
    terminalShowcase.years === 60,
  "terminal showcase must use the canonical Cycle 2 run"
);
assert(
  terminalShowcase.eventCount === 644 &&
    terminalShowcase.births === 49 &&
    terminalShowcase.livingPopulation === 45 &&
    terminalShowcase.deaths === 20,
  "terminal showcase outcomes must match canonical evidence"
);
assert(terminalViews.length === 5, "terminal showcase requires all five views");
for (const view of terminalViews) {
  assert(!view.screen.includes("\u001b"), `${view.slug} snapshot contains ANSI`);
  assert(
    view.screen.trimEnd().split("\n").length === 36,
    `${view.slug} snapshot must be the canonical 120x36 screen`
  );
}
assert(
  terminalViews[0]?.screen.includes("Gorse       2 people · EXTINCT") &&
    terminalViews[3]?.screen.includes("Garin Fen #14  [CURRENT PARTNER]"),
  "terminal showcase must retain surname and union-aware story evidence"
);
assert(
  worldSummary.regions === 12_288 &&
    worldSummary.land_regions > 4_800 &&
    worldSummary.island_regions > 0 &&
    worldSummary.river_regions > 0,
  "world genesis must retain the canonical continent, island, and rivers"
);
assert(
  worldSummary.locked_sea_routes === 1,
  "world genesis must begin with one inaccessible sea route"
);
assert(
  historySummary.first_contact_year === 293 &&
    historySummary.mixed_lineage_populations === 4 &&
    historySummary.settlements === 24,
  "history showcase must retain canonical first-contact evidence"
);
assert(
  startingRegion.settlement_ids.length === 5,
  "the starting region must contain exactly five settlements"
);
assert(
  cultures.some((culture) => culture.name === "Keepers of the Ring") &&
    cultures.some((culture) => culture.name === "Tidebound"),
  "the showcase must contain both isolated and contact cultures"
);
assert(lore.length >= 2, "first contact requires competing lore claims");
assert(
  atlasSvg.trimStart().startsWith("<svg") &&
    !atlasSvg.toLowerCase().includes("<script"),
  "the generated atlas must be a script-free SVG"
);
assert(!worldTuiScreen.includes("\u001b"), "world TUI snapshot contains ANSI");
assert(
  localHistorySummary.settlements === 5 &&
    localSettlements.length === 5 &&
    localConnections.length === 10,
  "local history must preserve five settlements and ten pairwise connections"
);
assert(
  localHistorySummary.macro_population ===
    localHistorySummary.represented_population &&
    localSettlements.every(
      (settlement) =>
        settlement.macro_population === settlement.represented_population
    ),
  "local household allocations must exactly reconcile aggregate populations"
);
assert(
  localSettlements.some(
    (settlement) =>
      settlement.name === "Fenstead" &&
      settlement.final_living_people > settlement.initial_sample_people
  ) &&
    localSettlements.some(
      (settlement) =>
        settlement.name === "Fenholm" &&
        settlement.initial_sample_people > 0 &&
        settlement.final_living_people === 0
    ),
  "local showcase must retain the growing and empty village contrast"
);
assert(
  localPlayback.schema_version === 1 &&
    localPlayback.seed === localHistorySummary.seed &&
    localPlayback.projection_year === localHistorySummary.projection_year &&
    localPlayback.elapsed_years === localHistorySummary.elapsed_years &&
    localPlayback.days_per_year === 360,
  "local playback metadata must match the canonical local history"
);
const playbackPeople = new Map(
  localPlayback.people.map((person) => [person.id, person])
);
assert(
  playbackPeople.size === 108 &&
    localPlayback.people.length === playbackPeople.size,
  "local playback requires 108 uniquely identified sampled people"
);
const generationCounts = [0, 1, 2, 3].map(
  (generation) =>
    localPlayback.people.filter((person) => person.generation === generation)
      .length
);
assert(
  generationCounts.join(",") === "30,30,26,22",
  "local playback must retain the canonical four generations"
);

const selectedLocationIds = new Set(
  localSettlements.map((settlement) => settlement.location_id)
);
const livingPlaybackPeople = new Set<number>();
const seenPlaybackPeople = new Set<number>();
const playbackLocations = new Map<number, number>();
let previousPlaybackEventId = 0;
let previousPlaybackDay = 0;
let playbackBirths = 0;
let playbackDeaths = 0;
let playbackSettlements = 0;
let playbackMigrations = 0;
for (const event of localPlayback.events) {
  assert(
    event.event_id > previousPlaybackEventId && event.day >= previousPlaybackDay,
    `playback event ${event.event_id} is not in stable causal order`
  );
  previousPlaybackEventId = event.event_id;
  previousPlaybackDay = event.day;
  if (event.type === "household_settled") {
    playbackSettlements += 1;
    assert(
      selectedLocationIds.has(event.destination_location_id),
      `playback household ${event.household_id} settled outside the five villages`
    );
    if (
      event.origin_location_ids.some(
        (origin) => origin !== event.destination_location_id
      )
    ) {
      playbackMigrations += 1;
    }
    for (const personId of event.traveler_ids) {
      assert(
        playbackPeople.has(personId),
        `playback settlement references unknown person ${personId}`
      );
      livingPlaybackPeople.add(personId);
      seenPlaybackPeople.add(personId);
      playbackLocations.set(personId, event.destination_location_id);
    }
  } else if (event.type === "person_born") {
    playbackBirths += 1;
    const person = playbackPeople.get(event.person_id);
    assert(
      person?.birth_day === event.day,
      `playback birth for person ${event.person_id} disagrees with person metadata`
    );
    livingPlaybackPeople.add(event.person_id);
    seenPlaybackPeople.add(event.person_id);
    playbackLocations.set(event.person_id, event.location_id);
  } else {
    playbackDeaths += 1;
    const person = playbackPeople.get(event.person_id);
    assert(
      person?.death_day === event.day &&
        playbackLocations.get(event.person_id) === event.location_id,
      `playback death for person ${event.person_id} disagrees with lived location`
    );
    livingPlaybackPeople.delete(event.person_id);
  }
}
assert(
  localPlayback.events.length === 164 &&
    playbackSettlements === 52 &&
    playbackBirths === localHistorySummary.births &&
    playbackDeaths === localHistorySummary.deaths &&
    playbackMigrations === localHistorySummary.household_migrations,
  "local playback event totals must match the canonical local summary"
);
assert(
  seenPlaybackPeople.size === playbackPeople.size &&
    livingPlaybackPeople.size === localHistorySummary.living_sample_people,
  "local playback must place every sampled life and reconcile final survivors"
);
for (const settlement of localSettlements) {
  const livingHere = [...livingPlaybackPeople].filter(
    (personId) => playbackLocations.get(personId) === settlement.location_id
  ).length;
  assert(
    livingHere === settlement.final_living_people,
    `playback final population disagrees with ${settlement.name}`
  );
}
const generationStarts = [1, 2, 3].map((generation) => {
  const firstBirth = localPlayback.people
    .filter((person) => person.generation === generation)
    .map((person) => person.birth_day)
    .filter((day): day is number => day !== null)
    .sort((first, second) => first - second)[0];
  return firstBirth === undefined
    ? -1
    : Math.floor(firstBirth / localPlayback.days_per_year);
});
assert(
  generationStarts.join(",") === "2,22,42",
  "local playback generation milestones must remain Year +2, +22, and +42"
);
assert(
  localTerminalViews.length === 5,
  "local history showcase requires all five terminal views"
);
for (const view of localTerminalViews) {
  assert(!view.screen.includes("\u001b"), `${view.slug} local snapshot contains ANSI`);
  assert(
    view.screen.trimEnd().split("\n").length <= 36,
    `${view.slug} local snapshot exceeds the canonical 120x36 screen`
  );
}

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
  `${JSON.stringify({
    foundationRun,
    currentCycle,
    terminalShowcase,
    worldGenesisShowcase,
    localHistoryShowcase
  })}\n`
);

console.log(
  `Validated ${run.events.length} foundation events, ${terminalViews.length + localTerminalViews.length} terminal views, the ${worldSummary.regions.toLocaleString("en-US")}-region world atlas, and ${localPlayback.people.length} replayable village lives.`
);
