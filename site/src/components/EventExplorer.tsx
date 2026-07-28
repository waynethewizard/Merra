"use client";

import { useMemo, useState } from "react";
import type { WorldEvent } from "@/lib/content";

type Props = {
  events: WorldEvent[];
};

function humanize(value: string): string {
  return value
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function payloadSummary(event: WorldEvent): string {
  switch (event.payload.type) {
    case "simulation_started":
      return `Seed ${event.payload.seed} entered ${event.payload.scenario_id}.`;
    case "time_advanced":
      return `${event.payload.elapsed_days} days elapsed, from Day ${event.payload.from_day} to Day ${event.payload.to_day}.`;
    case "simulation_completed":
      return `The requested run closed after ${event.payload.elapsed_years} complete year.`;
    default:
      return "Structured evidence recorded by the simulation.";
  }
}

export function EventExplorer({ events }: Props) {
  const tags = useMemo(
    () => ["all", ...Array.from(new Set(events.flatMap((event) => event.tags)))],
    [events]
  );
  const [activeTag, setActiveTag] = useState("all");
  const [selectedId, setSelectedId] = useState(events[0]?.id ?? 0);

  const visibleEvents = events.filter(
    (event) => activeTag === "all" || event.tags.includes(activeTag)
  );
  const selected =
    events.find((event) => event.id === selectedId) ?? visibleEvents[0];

  return (
    <div className="explorer-grid">
      <section className="event-browser" aria-label="Event browser">
        <div className="filter-row">
          <span>Filter evidence</span>
          <div className="filter-buttons">
            {tags.map((tag) => (
              <button
                aria-pressed={activeTag === tag}
                className={activeTag === tag ? "active" : ""}
                key={tag}
                onClick={() => setActiveTag(tag)}
                type="button"
              >
                {tag}
              </button>
            ))}
          </div>
        </div>
        <ol className="event-list">
          {visibleEvents.map((event) => (
            <li key={event.id}>
              <button
                className={selected?.id === event.id ? "selected" : ""}
                onClick={() => setSelectedId(event.id)}
                type="button"
              >
                <span className="event-marker">{event.id}</span>
                <span className="event-main">
                  <small>Day {event.time.day}</small>
                  <strong>{humanize(event.kind)}</strong>
                  <span>{payloadSummary(event)}</span>
                </span>
                <span className="event-arrow" aria-hidden="true">
                  →
                </span>
              </button>
            </li>
          ))}
        </ol>
      </section>

      <aside className="evidence-card" aria-live="polite">
        <div className="eyebrow">Selected evidence</div>
        {selected ? (
          <>
            <div className="evidence-heading">
              <span>Event {selected.id}</span>
              <small>Day {selected.time.day}</small>
            </div>
            <h2>{humanize(selected.kind)}</h2>
            <p>{payloadSummary(selected)}</p>
            <dl>
              <div>
                <dt>Caused by</dt>
                <dd>
                  {selected.causes.length
                    ? selected.causes.map((cause) => `Event ${cause}`).join(", ")
                    : "Origin event"}
                </dd>
              </div>
              <div>
                <dt>Tags</dt>
                <dd>{selected.tags.join(", ")}</dd>
              </div>
              <div>
                <dt>Schema type</dt>
                <dd>{selected.payload.type}</dd>
              </div>
            </dl>
            <details>
              <summary>Inspect raw payload</summary>
              <pre>{JSON.stringify(selected.payload, null, 2)}</pre>
            </details>
          </>
        ) : null}
      </aside>
    </div>
  );
}
