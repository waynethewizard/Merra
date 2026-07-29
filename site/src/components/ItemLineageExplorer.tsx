"use client";

import { useMemo, useState } from "react";
import type {
  ItemBiographyEntry,
  ItemHolder,
  ItemRecord
} from "@/lib/content";

type Props = {
  biography: ItemBiographyEntry[];
  featuredItemId: number;
  items: ItemRecord[];
  settlements: {
    id: number;
    name: string;
  }[];
};

function holderLabel(holder: ItemHolder): string {
  return `${holder.type.charAt(0).toUpperCase()}${holder.type.slice(1)} #${holder.id}`;
}

function statusLabel(status: ItemRecord["status"]): string {
  return `${status.charAt(0).toUpperCase()}${status.slice(1)}`;
}

function conditionLabel(value: number): string {
  return `${Math.round(value / 100)}%`;
}

function introductionYear(item: ItemRecord): number {
  return Math.floor(item.introduced_day / 360);
}

export function ItemLineageExplorer({
  biography,
  featuredItemId,
  items,
  settlements
}: Props) {
  const itemsById = useMemo(
    () => new Map(items.map((item) => [item.id, item])),
    [items]
  );
  const activeItems = useMemo(
    () =>
      items
        .filter((item) => item.status === "active")
        .sort((first, second) => first.id - second.id),
    [items]
  );
  const initialActive =
    activeItems.find((item) => item.id === featuredItemId) ?? activeItems[0];
  const [activeItemId, setActiveItemId] = useState(initialActive?.id ?? 0);
  const [selectedItemId, setSelectedItemId] = useState(initialActive?.id ?? 0);

  const activeItem =
    itemsById.get(activeItemId) ?? initialActive ?? items[0];
  const chain = useMemo(() => {
    const result: ItemRecord[] = [];
    let cursor: ItemRecord | undefined = activeItem;
    while (cursor) {
      result.unshift(cursor);
      const sourceId: number | undefined = cursor.sources?.[0]?.item_id;
      cursor = sourceId === undefined ? undefined : itemsById.get(sourceId);
    }
    return result;
  }, [activeItem, itemsById]);
  const selected =
    chain.find((item) => item.id === selectedItemId) ?? activeItem ?? chain[0];
  const selectedPlace =
    selected?.current_location_id === null
      ? null
      : settlements.find(
          (settlement) => settlement.id === selected?.current_location_id
        );
  const activeIndex = Math.max(
    0,
    activeItems.findIndex((item) => item.id === activeItem?.id)
  );

  function chooseLineage(itemId: number) {
    setActiveItemId(itemId);
    setSelectedItemId(itemId);
  }

  if (!activeItem || !selected) return null;

  return (
    <div className="object-lineage-explorer">
      <div className="object-picker">
        <div>
          <p className="eyebrow">
            Surviving lineage {String(activeIndex + 1).padStart(2, "0")} /{" "}
            {activeItems.length}
          </p>
          <h3>{activeItem.name}</h3>
        </div>
        <label>
          <span>Choose an active heirloom</span>
          <select
            onChange={(event) => chooseLineage(Number(event.target.value))}
            value={activeItem.id}
          >
            {activeItems.map((item) => (
              <option key={item.id} value={item.id}>
                #{item.id} · {item.name}
              </option>
            ))}
          </select>
        </label>
      </div>

      <div
        aria-label={`Provenance chain for ${activeItem.name}`}
        className="object-lineage-chain"
        role="list"
      >
        {chain.map((item, index) => (
          <div className="object-chain-step" key={item.id} role="listitem">
            {index > 0 ? (
              <div className="object-chain-join" aria-hidden="true">
                <span>reworked</span>
                <i>→</i>
              </div>
            ) : null}
            <button
              aria-pressed={selected.id === item.id}
              className={selected.id === item.id ? "is-selected" : ""}
              onClick={() => setSelectedItemId(item.id)}
              type="button"
            >
              <span>G{item.lineage_generation}</span>
              <strong>Item #{item.id}</strong>
              <small>
                {item.status === "active"
                  ? "Working descendant"
                  : `Became Item #${
                      chain[index + 1]?.id ?? "—"
                    }`}
              </small>
            </button>
          </div>
        ))}
      </div>

      <div className="object-record-grid" aria-live="polite">
        <article className="object-selected-record">
          <div className="object-record-heading">
            <p className="eyebrow">Selected identity</p>
            <span className={`object-status status-${selected.status}`}>
              {statusLabel(selected.status)}
            </span>
          </div>
          <h3>{selected.name}</h3>
          <p>
            Introduced as Item #{selected.id} in Year{" "}
            {introductionYear(selected)}, with its own stable record and event
            history.
          </p>
          <div className="object-condition">
            <div>
              <span>Final condition</span>
              <strong>{conditionLabel(selected.condition_per_10_000)}</strong>
            </div>
            <div
              aria-label={`${conditionLabel(selected.condition_per_10_000)} condition`}
              className="object-condition-track"
              role="img"
            >
              <span
                style={{ width: `${selected.condition_per_10_000 / 100}%` }}
              />
            </div>
          </div>
        </article>

        <dl className="object-fact-sheet">
          <div>
            <dt>Identity began</dt>
            <dd>
              Year {introductionYear(selected)} · Event #
              {selected.introduction_event_id}
            </dd>
          </div>
          <div>
            <dt>Repairs retained it</dt>
            <dd>{selected.repairs} same-identity repairs</dd>
          </div>
          <div>
            <dt>Legal owner</dt>
            <dd>{holderLabel(selected.owner)}</dd>
          </div>
          <div>
            <dt>Physical custody</dt>
            <dd>{holderLabel(selected.custody)}</dd>
          </div>
          <div>
            <dt>Last known place</dt>
            <dd>
              {selectedPlace?.name ??
                (selected.current_location_id === null
                  ? "Unknown"
                  : `Location #${selected.current_location_id}`)}
            </dd>
          </div>
          <div>
            <dt>Material source</dt>
            <dd>
              {selected.sources?.length
                ? selected.sources
                    .map(
                      (source) =>
                        `Item #${source.item_id} · ${source.role}`
                    )
                    .join(", ")
                : "Original introduction"}
            </dd>
          </div>
        </dl>
      </div>

      <section className="object-biography">
        <div className="object-biography-heading">
          <div>
            <p className="eyebrow">Last chapter / Item #{featuredItemId}</p>
            <h3>A tool changes hands, is mended, and works again.</h3>
          </div>
          <p>
            These are authoritative events from the selected terminal biography,
            not prose inferred from final state.
          </p>
        </div>
        <ol>
          {biography.map((entry) => (
            <li key={entry.eventId}>
              <div>
                <span>Year {entry.year}</span>
                <small>Event #{entry.eventId}</small>
              </div>
              <i aria-hidden="true" />
              <p>{entry.text}</p>
            </li>
          ))}
        </ol>
      </section>
    </div>
  );
}
