import type { Metadata } from "next";
import { TerminalShowcase } from "@/components/TerminalShowcase";
import { VillagePlayback } from "@/components/VillagePlayback";
import { getLocalHistoryShowcase } from "@/lib/content";

export const metadata: Metadata = {
  title: "Five Villages",
  description:
    "Explore Merra’s exact macro-to-local handoff, household migrations, road costs, and divergent five-village histories."
};

function formatNumber(value: number): string {
  return new Intl.NumberFormat("en-US").format(value);
}

export default function VillagesPage() {
  const showcase = getLocalHistoryShowcase();
  const maximumLiving = Math.max(
    ...showcase.settlements.map((settlement) => settlement.final_living_people)
  );
  const emptyVillage = showcase.settlements.find(
    (settlement) => settlement.final_living_people === 0
  );

  return (
    <div className="inner-page villages-page">
      <section className="page-hero villages-hero">
        <div className="shell">
          <p className="eyebrow">
            Era I · Cycle 05 · Year {showcase.projectionYear} + {showcase.years}
          </p>
          <div className="page-title-row">
            <div>
              <h1>
                Five villages.
                <br />
                Five consequences.
              </h1>
              <p>
                {showcase.description} This is a weighted detailed sample—not
                40,751 anonymous rows pretending to be lives.
              </p>
            </div>
            <div className="schema-stamp village-stamp">
              <span>{showcase.seed}</span>
              <small>root seed</small>
            </div>
          </div>
          <div className="run-meta terminal-run-meta">
            <div>
              <span>Macro people</span>
              <strong>{formatNumber(showcase.macroPopulation)}</strong>
            </div>
            <div>
              <span>Represented</span>
              <strong>{formatNumber(showcase.representedPopulation)}</strong>
            </div>
            <div>
              <span>Migrations</span>
              <strong>{showcase.migrations}</strong>
            </div>
            <div>
              <span>Located events</span>
              <strong>{showcase.locatedEvents}</strong>
            </div>
          </div>
        </div>
      </section>

      <section className="section village-playback-section">
        <div className="shell">
          <div className="world-section-heading">
            <p className="section-number">01 / Living history</p>
            <h2>Watch four generations redistribute five villages.</h2>
            <p>
              Every dot is one named person in the canonical detailed sample.
              Play all 60 years, scrub one year at a time, or jump to the first
              appearance of each generation. Births, deaths, and household
              migrations come directly from the checked event stream.
            </p>
          </div>
          <VillagePlayback
            connections={showcase.connections}
            playback={showcase.playback}
            settlements={showcase.settlements}
          />
        </div>
      </section>

      <section className="section village-consequence">
        <div className="shell">
          <div className="world-section-heading">
            <p className="section-number">02 / The consequence</p>
            <h2>Growth is comparative. Disappearance leaves evidence.</h2>
            <p>
              Fenstead attracts households and grows. Fenholm records births
              and deaths, then loses every sampled home. Empty does not mean
              erased: its population bridge, roads, events, and former
              households remain inspectable.
            </p>
          </div>
          <div className="village-outcome-grid">
            {showcase.settlements.map((settlement) => {
              const delta =
                settlement.final_living_people -
                settlement.initial_sample_people;
              const width =
                maximumLiving === 0
                  ? 0
                  : (settlement.final_living_people / maximumLiving) * 100;
              return (
                <article
                  className={
                    settlement.final_living_people === 0
                      ? "village-outcome is-empty"
                      : "village-outcome"
                  }
                  key={settlement.location_id}
                >
                  <div>
                    <p className="eyebrow">Place #{settlement.location_id}</p>
                    <h3>{settlement.name}</h3>
                  </div>
                  <div className="village-population">
                    <strong>
                      {settlement.initial_sample_people}
                      <span aria-hidden="true">→</span>
                      {settlement.final_living_people}
                    </strong>
                    <small>{delta >= 0 ? `+${delta}` : delta} sampled people</small>
                  </div>
                  <div className="village-bar" aria-hidden="true">
                    <span style={{ width: `${width}%` }} />
                  </div>
                  <dl>
                    <div>
                      <dt>Births / deaths</dt>
                      <dd>
                        {settlement.births} / {settlement.deaths}
                      </dd>
                    </div>
                    <div>
                      <dt>Arrivals / departures</dt>
                      <dd>
                        {settlement.arrivals} / {settlement.departures}
                      </dd>
                    </div>
                    <div>
                      <dt>Macro = represented</dt>
                      <dd>{formatNumber(settlement.macro_population)}</dd>
                    </div>
                  </dl>
                  {settlement.final_living_people === 0 ? (
                    <p className="empty-mark">No sampled household remains</p>
                  ) : null}
                </article>
              );
            })}
          </div>
          {emptyVillage ? (
            <p className="village-field-note">
              Field note: {emptyVillage.name} had {emptyVillage.births} births,
              {" "}{emptyVillage.deaths} deaths, {emptyVillage.arrivals} arrivals,
              and {emptyVillage.departures} departures before the final sampled
              household disappeared.
            </p>
          ) : null}
        </div>
      </section>

      <section className="section residence-model">
        <div className="shell">
          <div className="world-section-heading light-heading">
            <p className="section-number light">03 / Residence model</p>
            <h2>Kin first. Roads second. Seed only for a true tie.</h2>
            <p>
              A household owns one residence; every member derives place from
              it. The destination rule is lexicographic, so the evidence says
              which cause actually decided each move.
            </p>
          </div>
          <ol className="residence-rule-grid">
            <li>
              <span>01</span>
              <strong>Living kin</strong>
              <p>Choose the settlement with the strongest close-family support.</p>
            </li>
            <li>
              <span>02</span>
              <strong>Road cost</strong>
              <p>Among tied kin networks, minimize deterministic shortest paths.</p>
            </li>
            <li>
              <span>03</span>
              <strong>Seeded rank</strong>
              <p>Only an exact kin-and-road tie reaches the isolated random domain.</p>
            </li>
          </ol>
          <div className="residence-proof">
            <div>
              <span>Residence decisions</span>
              <strong>{showcase.residenceDecisions}</strong>
            </div>
            <div>
              <span>Boundary crossings</span>
              <strong>{showcase.migrations}</strong>
            </div>
            <div>
              <span>Pairwise paths</span>
              <strong>{showcase.connections.length}</strong>
            </div>
            <div>
              <span>Longest journey</span>
              <strong>
                {Math.max(
                  ...showcase.connections.map((connection) => connection.travel_days)
                )}
                d
              </strong>
            </div>
          </div>
        </div>
      </section>

      <section className="shell terminal-showcase-section villages-terminal">
        <div className="section-heading">
          <div>
            <p className="section-number">04 / Inspect the evidence</p>
            <h2>The overview tells the story. Every tab can prove it.</h2>
          </div>
          <p>
            The road view shows exact costs instead of invented geometry.
            Settlement rows reconcile macro and sampled scales. Migration rows
            preserve origins, reasons, routes, and causes. Household rows carry
            institutions and competing accounts of first contact.
          </p>
        </div>
        <TerminalShowcase showcase={showcase} />
      </section>

      <section className="reproduce-section village-reproduce">
        <div className="shell reproduce-grid">
          <div>
            <p className="section-number light">05 / Reproduce it</p>
            <h2>One handoff. No hidden spreadsheet.</h2>
            <p>
              The history command emits a versioned regional handoff. The
              villages command assigns every aggregate person exactly once,
              then writes machine records, a chronicle, and the terminal report.
            </p>
          </div>
          <div className="terminal-card terminal-command-card">
            <div className="terminal-bar">
              <span />
              <span />
              <span />
              <small>merra / bash</small>
            </div>
            <code>
              <span>$</span> {showcase.command}
            </code>
            <dl>
              <div>
                <dt>Projection</dt>
                <dd>Year {showcase.projectionYear}</dd>
              </div>
              <div>
                <dt>Detailed span</dt>
                <dd>{showcase.years} years</dd>
              </div>
              <div>
                <dt>Population equation</dt>
                <dd>
                  {formatNumber(showcase.macroPopulation)} ={" "}
                  {formatNumber(showcase.representedPopulation)}
                </dd>
              </div>
            </dl>
          </div>
        </div>
      </section>
    </div>
  );
}
