"use client";

import {
  useEffect,
  useMemo,
  useState,
  type CSSProperties
} from "react";
import type {
  LocalHistoryPlayback,
  LocalHistoryShowcase,
  LocalPlaybackEvent,
  LocalPlaybackPerson
} from "@/lib/content";

type Settlement = LocalHistoryShowcase["settlements"][number];
type Connection = LocalHistoryShowcase["connections"][number];

type VillagePlaybackProps = {
  playback: LocalHistoryPlayback;
  settlements: Settlement[];
  connections: Connection[];
};

type DiagramPoint = {
  x: number;
  y: number;
};

type ReplayedState = {
  living: Set<number>;
  locations: Map<number, number>;
  eventsThisYear: LocalPlaybackEvent[];
};

const GENERATION_COLORS = ["#f4dca9", "#dd965e", "#8fc4a3", "#8da8d8"];
const SPEEDS = [1, 2, 4];
const CANONICAL_LAYOUT = new Map<number, DiagramPoint>([
  [20, { x: 390, y: 215 }],
  [11, { x: 145, y: 125 }],
  [17, { x: 165, y: 425 }],
  [28, { x: 705, y: 225 }],
  [27, { x: 860, y: 430 }]
]);

function replayToYear(
  playback: LocalHistoryPlayback,
  yearOffset: number
): ReplayedState {
  const living = new Set<number>();
  const locations = new Map<number, number>();
  const cutoff = yearOffset * playback.days_per_year;
  const priorCutoff =
    yearOffset === 0 ? Number.NEGATIVE_INFINITY : cutoff - playback.days_per_year;
  const eventsThisYear: LocalPlaybackEvent[] = [];

  for (const event of playback.events) {
    if (event.day > cutoff) break;
    if (event.day > priorCutoff) eventsThisYear.push(event);

    if (event.type === "household_settled") {
      for (const personId of event.traveler_ids) {
        living.add(personId);
        locations.set(personId, event.destination_location_id);
      }
    } else if (event.type === "person_born") {
      living.add(event.person_id);
      locations.set(event.person_id, event.location_id);
    } else {
      living.delete(event.person_id);
    }
  }

  return { living, locations, eventsThisYear };
}

function pointForPerson(personId: number, village: DiagramPoint): DiagramPoint {
  const angle = ((personId * 137.508) % 360) * (Math.PI / 180);
  const radius = 22 + ((personId * 29) % 59);
  return {
    x: village.x + Math.cos(angle) * radius,
    y: village.y + Math.sin(angle) * radius
  };
}

function generationLabel(generation: number): string {
  return generation === 0 ? "Founders" : `Generation ${generation}`;
}

function reasonLabel(reason: string): string {
  switch (reason) {
    case "macro_projection":
      return "initial projection";
    case "living_kin":
      return "living kin";
    case "shortest_journey":
      return "shortest road";
    default:
      return "seeded tie";
  }
}

function eventLabel(
  event: LocalPlaybackEvent,
  people: Map<number, LocalPlaybackPerson>,
  locationNames: Map<number, string>
): string {
  if (event.type === "person_born") {
    return `${people.get(event.person_id)?.name ?? `Person #${event.person_id}`} was born in ${
      locationNames.get(event.location_id) ?? `place #${event.location_id}`
    }.`;
  }
  if (event.type === "person_died") {
    return `${people.get(event.person_id)?.name ?? `Person #${event.person_id}`} died at age ${
      event.age_years
    } in ${locationNames.get(event.location_id) ?? `place #${event.location_id}`}.`;
  }

  const travelers = event.traveler_ids
    .map((personId) => people.get(personId)?.name ?? `Person #${personId}`)
    .join(" + ");
  const destination =
    locationNames.get(event.destination_location_id) ??
    `place #${event.destination_location_id}`;
  if (event.origin_location_ids.length === 0) {
    return `${travelers} began in ${destination}.`;
  }
  const origins = event.origin_location_ids
    .map(
      (locationId) =>
        locationNames.get(locationId) ?? `place #${locationId}`
    )
    .join(" + ");
  return `${travelers} moved ${origins} → ${destination} · ${reasonLabel(
    event.reason
  )}.`;
}

export function VillagePlayback({
  playback,
  settlements,
  connections
}: VillagePlaybackProps) {
  const [yearOffset, setYearOffset] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [speed, setSpeed] = useState(1);
  const [selectedPersonId, setSelectedPersonId] = useState(39);
  const maximumYear = playback.elapsed_years;

  const people = useMemo(
    () => new Map(playback.people.map((person) => [person.id, person])),
    [playback.people]
  );
  const locationNames = useMemo(
    () =>
      new Map(
        settlements.map((settlement) => [
          settlement.location_id,
          settlement.name
        ])
      ),
    [settlements]
  );
  const diagram = useMemo(() => {
    const fallback = [
      { x: 180, y: 160 },
      { x: 420, y: 120 },
      { x: 650, y: 180 },
      { x: 310, y: 430 },
      { x: 750, y: 420 }
    ];
    return new Map(
      settlements.map((settlement, index) => [
        settlement.location_id,
        CANONICAL_LAYOUT.get(settlement.location_id) ?? fallback[index]!
      ])
    );
  }, [settlements]);
  const replayed = useMemo(
    () => replayToYear(playback, yearOffset),
    [playback, yearOffset]
  );

  useEffect(() => {
    if (!playing) return;
    const timer = window.setInterval(() => {
      setYearOffset((current) => Math.min(current + 1, maximumYear));
    }, 720 / speed);
    return () => window.clearInterval(timer);
  }, [maximumYear, playing, speed]);

  useEffect(() => {
    if (yearOffset >= maximumYear) setPlaying(false);
  }, [maximumYear, yearOffset]);

  const livingPeople = playback.people.filter((person) =>
    replayed.living.has(person.id)
  );
  const livingByLocation = new Map<number, LocalPlaybackPerson[]>();
  for (const person of livingPeople) {
    const locationId = replayed.locations.get(person.id);
    if (locationId === undefined) continue;
    const group = livingByLocation.get(locationId) ?? [];
    group.push(person);
    livingByLocation.set(locationId, group);
  }
  const generationCounts = [0, 1, 2, 3].map(
    (generation) =>
      livingPeople.filter((person) => person.generation === generation).length
  );
  const generationMilestones = [0, 1, 2, 3].map((generation) => {
    if (generation === 0) {
      return { generation, year: 0, label: "Founders" };
    }
    const firstBirth = playback.people
      .filter((person) => person.generation === generation)
      .map((person) => person.birth_day)
      .filter((day): day is number => day !== null)
      .sort((first, second) => first - second)[0];
    return {
      generation,
      year:
        firstBirth === undefined
          ? maximumYear
          : Math.floor(firstBirth / playback.days_per_year),
      label: `G${generation} begins`
    };
  });
  const currentGeneration = generationMilestones
    .filter((milestone) => milestone.year <= yearOffset)
    .at(-1)?.generation;
  const selectedPerson = people.get(selectedPersonId);
  const selectedLocationId = replayed.locations.get(selectedPersonId);
  const selectedBorn =
    selectedPerson !== undefined &&
    (selectedPerson.birth_day === null ||
      selectedPerson.birth_day <= yearOffset * playback.days_per_year);
  const selectedDead =
    selectedPerson?.death_day !== null &&
    selectedPerson?.death_day !== undefined &&
    selectedPerson.death_day <= yearOffset * playback.days_per_year;
  const selectedAge =
    selectedPerson === undefined || !selectedBorn
      ? null
      : selectedPerson.starting_age_years +
        Math.floor(
          (Math.min(
            yearOffset * playback.days_per_year,
            selectedPerson.death_day ?? yearOffset * playback.days_per_year
          ) -
            (selectedPerson.birth_day ?? 0)) /
            playback.days_per_year
        );
  const directConnections = connections.filter(
    (connection) => connection.route_ids.length === 1
  );
  const displayedEvents = replayed.eventsThisYear
    .filter(
      (event) =>
        event.type !== "household_settled" ||
        event.origin_location_ids.length > 0 ||
        yearOffset === 0
    )
    .slice(0, 5);
  const moreEvents =
    replayed.eventsThisYear.filter(
      (event) =>
        event.type !== "household_settled" ||
        event.origin_location_ids.length > 0 ||
        yearOffset === 0
    ).length - displayedEvents.length;

  function togglePlayback() {
    if (yearOffset >= maximumYear) setYearOffset(0);
    setPlaying((current) => !current);
  }

  return (
    <div className="village-playback">
      <div className="playback-toolbar">
        <div className="playback-year">
          <span>Local year</span>
          <strong>{playback.projection_year + yearOffset}</strong>
          <small>+{yearOffset} after projection</small>
        </div>
        <div className="playback-buttons" aria-label="Playback controls">
          <button
            onClick={() => setYearOffset((current) => Math.max(0, current - 1))}
            type="button"
          >
            ← Year
          </button>
          <button
            className="playback-primary"
            onClick={togglePlayback}
            type="button"
          >
            {playing ? "Pause" : yearOffset === maximumYear ? "Replay" : "Play"}
          </button>
          <button
            onClick={() =>
              setYearOffset((current) => Math.min(maximumYear, current + 1))
            }
            type="button"
          >
            Year →
          </button>
          <button
            onClick={() =>
              setSpeed((current) => {
                const index = SPEEDS.indexOf(current);
                return SPEEDS[(index + 1) % SPEEDS.length]!;
              })
            }
            type="button"
          >
            {speed}× speed
          </button>
        </div>
      </div>

      <div className="playback-timeline">
        <input
          aria-label="Local history year"
          max={maximumYear}
          min={0}
          onChange={(event) => {
            setPlaying(false);
            setYearOffset(Number(event.target.value));
          }}
          type="range"
          value={yearOffset}
        />
        <div className="generation-jumps" aria-label="Generation milestones">
          {generationMilestones.map((milestone) => (
            <button
              aria-pressed={
                yearOffset !== maximumYear &&
                currentGeneration === milestone.generation
              }
              key={milestone.generation}
              onClick={() => {
                setPlaying(false);
                setYearOffset(milestone.year);
              }}
              style={{
                "--generation-color":
                  GENERATION_COLORS[milestone.generation]
              } as CSSProperties}
              type="button"
            >
              <span>G{milestone.generation}</span>
              <strong>{milestone.label}</strong>
              <small>Year {playback.projection_year + milestone.year}</small>
            </button>
          ))}
          <button
            aria-pressed={yearOffset === maximumYear}
            onClick={() => {
              setPlaying(false);
              setYearOffset(maximumYear);
            }}
            type="button"
          >
            <span>End</span>
            <strong>Final year</strong>
            <small>Year {playback.projection_year + maximumYear}</small>
          </button>
        </div>
      </div>

      <div className="playback-stage">
        <div className="playback-map">
          <svg
            aria-label={`Schematic village network in Year ${
              playback.projection_year + yearOffset
            }, showing ${livingPeople.length} living sampled people`}
            role="img"
            viewBox="0 0 1000 590"
          >
            <defs>
              <filter id="village-glow" x="-40%" y="-40%" width="180%" height="180%">
                <feGaussianBlur stdDeviation="10" />
              </filter>
            </defs>
            <g className="playback-roads">
              {directConnections.map((connection) => {
                const from = diagram.get(connection.from);
                const to = diagram.get(connection.to);
                if (!from || !to) return null;
                const middleX = (from.x + to.x) / 2;
                const middleY = (from.y + to.y) / 2;
                return (
                  <g key={`${connection.from}-${connection.to}`}>
                    <line
                      className={
                        connection.travel_cost > 20 ? "is-long-road" : undefined
                      }
                      x1={from.x}
                      x2={to.x}
                      y1={from.y}
                      y2={to.y}
                    />
                    <text x={middleX} y={middleY - 9}>
                      {connection.travel_cost} cost
                    </text>
                  </g>
                );
              })}
            </g>
            {settlements.map((settlement) => {
              const point = diagram.get(settlement.location_id);
              if (!point) return null;
              const inhabitants =
                livingByLocation.get(settlement.location_id) ?? [];
              return (
                <g
                  className={
                    inhabitants.length === 0
                      ? "playback-village is-empty"
                      : "playback-village"
                  }
                  key={settlement.location_id}
                >
                  <circle
                    className="village-glow"
                    cx={point.x}
                    cy={point.y}
                    filter="url(#village-glow)"
                    r={86}
                  />
                  <circle
                    className="village-ring"
                    cx={point.x}
                    cy={point.y}
                    r={91}
                  />
                  <text
                    className="village-name"
                    textAnchor="middle"
                    x={point.x}
                    y={point.y - 108}
                  >
                    {settlement.name}
                  </text>
                  <text
                    className="village-count"
                    textAnchor="middle"
                    x={point.x}
                    y={point.y - 88}
                  >
                    {inhabitants.length} living
                  </text>
                </g>
              );
            })}
            <g className="playback-people">
              {livingPeople.map((person) => {
                const locationId = replayed.locations.get(person.id);
                const village =
                  locationId === undefined ? undefined : diagram.get(locationId);
                if (!village) return null;
                const point = pointForPerson(person.id, village);
                const selected = person.id === selectedPersonId;
                return (
                  <g
                    aria-label={`${person.name}, generation ${person.generation}, ${
                      locationNames.get(locationId!) ?? `place #${locationId}`
                    }`}
                    className={
                      selected
                        ? "playback-person is-selected"
                        : "playback-person"
                    }
                    key={person.id}
                    onClick={() => setSelectedPersonId(person.id)}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        setSelectedPersonId(person.id);
                      }
                    }}
                    role="button"
                    style={{
                      "--person-x": `${point.x}px`,
                      "--person-y": `${point.y}px`,
                      "--person-color": GENERATION_COLORS[person.generation]
                    } as CSSProperties}
                    tabIndex={0}
                  >
                    <circle className="person-target" r={12} />
                    <circle className="person-dot" r={selected ? 7 : 5} />
                    <title>
                      {person.name} · G{person.generation} ·{" "}
                      {locationNames.get(locationId!)}
                    </title>
                  </g>
                );
              })}
            </g>
            <text className="schematic-note" x={22} y={568}>
              Schematic network · lines show direct selected-settlement routes, not map position
            </text>
          </svg>
          <p className="playback-pan-hint" aria-hidden="true">
            Swipe the network to see every village →
          </p>
          <div className="playback-legend" aria-label="Generation legend">
            {generationCounts.map((count, generation) => (
              <span key={generation}>
                <i
                  style={{
                    backgroundColor: GENERATION_COLORS[generation]
                  }}
                />
                G{generation} · {count}
              </span>
            ))}
          </div>
        </div>

        <aside className="playback-inspector">
          <div className="playback-total">
            <span>Living sample</span>
            <strong>{livingPeople.length}</strong>
            <small>Each dot is one named sampled person</small>
          </div>
          {selectedPerson ? (
            <div className="selected-person">
              <p className="eyebrow">Selected life · #{selectedPerson.id}</p>
              <h3>{selectedPerson.name}</h3>
              <p>
                {generationLabel(selectedPerson.generation)} ·{" "}
                {!selectedBorn
                  ? `not yet born`
                  : selectedDead
                    ? `died at ${selectedAge}`
                    : `age ${selectedAge}`}
              </p>
              <dl>
                <div>
                  <dt>Place</dt>
                  <dd>
                    {!selectedBorn
                      ? "—"
                      : locationNames.get(selectedLocationId ?? -1) ?? "unknown"}
                  </dd>
                </div>
                <div>
                  <dt>Born</dt>
                  <dd>
                    {selectedPerson.birth_day === null
                      ? "Before projection"
                      : `Year ${
                          playback.projection_year +
                          Math.floor(
                            selectedPerson.birth_day / playback.days_per_year
                          )
                        }`}
                  </dd>
                </div>
                <div>
                  <dt>Parents</dt>
                  <dd>
                    {selectedPerson.parent_ids.length === 0
                      ? "Projected founders"
                      : selectedPerson.parent_ids
                          .map(
                            (personId) =>
                              people.get(personId)?.name ?? `#${personId}`
                          )
                          .join(" + ")}
                  </dd>
                </div>
              </dl>
            </div>
          ) : null}
          <div className="year-events">
            <p className="eyebrow">
              In Year {playback.projection_year + yearOffset}
            </p>
            {displayedEvents.length === 0 ? (
              <p className="quiet-year">
                No recorded births, deaths, or household moves this year.
              </p>
            ) : (
              <ol>
                {displayedEvents.map((event) => (
                  <li key={event.event_id}>
                    {eventLabel(event, people, locationNames)}
                  </li>
                ))}
              </ol>
            )}
            {moreEvents > 0 ? <small>+{moreEvents} more events</small> : null}
          </div>
        </aside>
      </div>
    </div>
  );
}
