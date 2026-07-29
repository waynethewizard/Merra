import Link from "next/link";
import { getWorldGenesisShowcase } from "@/lib/content";

function formatNumber(value: number): string {
  return new Intl.NumberFormat("en-US").format(value);
}

function formatFactor(value: number): string {
  return `${value.toFixed(3).replace(/0+$/, "").replace(/\.$/, "")}×`;
}

export default function WorldPage() {
  const showcase = getWorldGenesisShowcase();

  return (
    <>
      <section className="page-hero world-page-hero">
        <div className="shell">
          <p className="eyebrow">Era I · Cycles 03–04 · Seed {showcase.seed}</p>
          <div className="page-title-row">
            <div>
              <h1>
                A world before
                <br />
                its witnesses.
              </h1>
              <p>
                {showcase.description} This is generated evidence, not a
                painted concept map.
              </p>
            </div>
            <div className="schema-stamp world-stamp">
              <span>{showcase.years}</span>
              <small>years</small>
            </div>
          </div>
        </div>
      </section>

      <section className="world-atlas-section">
        <div className="shell">
          <div className="world-stat-grid" aria-label="Canonical world statistics">
            {[
              ["Regions", formatNumber(showcase.world.regions)],
              ["Land", formatNumber(showcase.world.landRegions)],
              ["Rivers", formatNumber(showcase.world.riverRegions)],
              ["Places", formatNumber(showcase.world.places)],
              ["Routes", formatNumber(showcase.world.routes)],
              ["Features", formatNumber(showcase.world.features)]
            ].map(([label, value]) => (
              <div key={label}>
                <span>{label}</span>
                <strong>{value}</strong>
              </div>
            ))}
          </div>
          <div className="atlas-frame">
            <div className="atlas-frame-heading">
              <div>
                <p className="eyebrow">Historical atlas / Year {showcase.years}</p>
                <h2>One landmass. One distant island. No inevitable contact.</h2>
              </div>
              <div className="atlas-legend">
                <span className="legend-human">Human</span>
                <span className="legend-orc">Orc</span>
                <span className="legend-mixed">Mixed</span>
              </div>
            </div>
            <div
              className="generated-atlas"
              dangerouslySetInnerHTML={{ __html: showcase.atlasSvg }}
            />
          </div>
        </div>
      </section>

      <section className="world-process section">
        <div className="shell">
          <div className="world-section-heading">
            <p className="section-number">01 / Generation order</p>
            <h2>The world provides causes. People make history inside it.</h2>
            <p>
              Geography is built first, but terrain never hard-codes a species
              or a theme. The history engine consumes a portable place graph
              with routes, affordances, and named homelands.
            </p>
          </div>
          <ol className="genesis-pipeline">
            {showcase.stages.map((stage, index) => (
              <li key={stage.name}>
                <span>{String(index + 1).padStart(2, "0")}</span>
                <strong>{stage.name}</strong>
                <p>{stage.result}</p>
              </li>
            ))}
          </ol>
        </div>
      </section>

      <section className="lineage-section section">
        <div className="shell">
          <div className="world-section-heading light-heading">
            <p className="section-number light">02 / Lineage is not culture</p>
            <h2>Bodies are inherited. Beliefs are learned.</h2>
            <p>
              Humans and orcs use the same simulation parameters. Orc
              differences are data—not conditional species logic—and their
              unusually religious society belongs to culture and faith, where
              it can change through contact.
            </p>
          </div>
          <div className="homeland-grid">
            {showcase.lineages.map((lineage) => (
              <article key={lineage.name}>
                <p className="eyebrow">{lineage.homeland}</p>
                <h3>{lineage.name}</h3>
                <dl>
                  <div>
                    <dt>Mortality</dt>
                    <dd>{formatFactor(lineage.mortality)}</dd>
                  </div>
                  <div>
                    <dt>Power</dt>
                    <dd>{formatFactor(lineage.power)}</dd>
                  </div>
                  <div>
                    <dt>Speed</dt>
                    <dd>{formatFactor(lineage.speed)}</dd>
                  </div>
                  <div>
                    <dt>Sustenance</dt>
                    <dd>{formatFactor(lineage.sustenance)}</dd>
                  </div>
                </dl>
              </article>
            ))}
          </div>
          <div className="culture-strip">
            {showcase.cultures.map((culture) => (
              <div key={culture.name}>
                <span>Year {culture.foundedYear}</span>
                <strong>{culture.name}</strong>
                <small>{culture.ritualDays} ritual days / year</small>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="contact-section section">
        <div className="shell contact-grid">
          <div>
            <p className="section-number">03 / Contingent contact</p>
            <h2>The sea opens in Year {showcase.history.firstContactYear}.</h2>
            <p>
              Navigation had to be learned before the locked maritime route
              could become history. Contact creates mixed communities and a
              new Tidebound culture; it does not erase either population.
            </p>
            <div className="history-stat-grid">
              {[
                ["Population", formatNumber(showcase.history.totalPopulation)],
                ["Settlements", showcase.history.settlements],
                ["Cultures", showcase.history.cultures],
                ["Faiths", showcase.history.faiths],
                ["Institutions", showcase.history.institutions],
                ["Mixed cohorts", showcase.history.mixedLineagePopulations]
              ].map(([label, value]) => (
                <div key={label}>
                  <strong>{value}</strong>
                  <span>{label}</span>
                </div>
              ))}
            </div>
          </div>
          <div className="contact-record">
            <p className="eyebrow">One event · two remembered truths</p>
            {showcase.lore.map((claim) => (
              <blockquote key={claim.title}>
                <strong>{claim.title}</strong>
                <p>“{claim.text}”</p>
                <small>claimed confidence · {claim.confidence}%</small>
              </blockquote>
            ))}
            <Link className="contact-history-link" href="/history/">
              Read the complete history &amp; lore →
            </Link>
          </div>
        </div>
      </section>

      <section className="starting-region section">
        <div className="shell starting-region-grid">
          <div>
            <p className="section-number light">04 / Zooming back in</p>
            <h2>A playable place with a world behind it.</h2>
            <p>{showcase.startingRegion.summary}</p>
            <dl>
              <div>
                <dt>Settlements</dt>
                <dd>{showcase.startingRegion.settlementCount}</dd>
              </div>
              <div>
                <dt>Relevant events</dt>
                <dd>{showcase.startingRegion.eventCount}</dd>
              </div>
              <div>
                <dt>Global events</dt>
                <dd>{showcase.history.eventCount}</dd>
              </div>
            </dl>
          </div>
          <div className="world-terminal-card">
            <div className="terminal-bar">
              <span />
              <span />
              <span />
              <small>cargo tui world · biome layer</small>
            </div>
            <pre>{showcase.tuiScreen}</pre>
          </div>
        </div>
      </section>

      <section className="world-reproduce">
        <div className="shell">
          <p className="eyebrow">Reproduce the evidence</p>
          <code>{showcase.command}</code>
          <p>
            Then run the history command documented with the golden evidence.
            The atlas, chronicle, event stream, and summaries are all derived
            from public scenarios.
          </p>
        </div>
      </section>
    </>
  );
}
