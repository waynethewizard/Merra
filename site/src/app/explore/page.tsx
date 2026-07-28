import type { Metadata } from "next";
import { EventExplorer } from "@/components/EventExplorer";
import { getFoundationRun } from "@/lib/content";

export const metadata: Metadata = {
  title: "Explore The First Clock",
  description:
    "Inspect the causal event chain and reproducibility manifest for Merra’s first deterministic run."
};

export default function ExplorePage() {
  const run = getFoundationRun();
  const commit = run.manifest.source.git_commit;

  return (
    <div className="inner-page">
      <section className="page-hero explorer-hero">
        <div className="shell">
          <p className="kicker">Run explorer / Golden evidence 001</p>
          <div className="page-title-row">
            <div>
              <h1>{run.title}</h1>
              <p>{run.description}</p>
            </div>
            <div className="schema-stamp">
              <span>Event schema</span>
              <strong>V{run.manifest.event_schema_version}</strong>
              <small>public contract</small>
            </div>
          </div>
          <div className="run-meta">
            <div>
              <span>Scenario</span>
              <strong>{run.manifest.scenario_id}</strong>
            </div>
            <div>
              <span>Seed</span>
              <strong>{run.manifest.seed}</strong>
            </div>
            <div>
              <span>Duration</span>
              <strong>{run.manifest.days} days</strong>
            </div>
            <div>
              <span>Events</span>
              <strong>{run.summary.event_count}</strong>
            </div>
          </div>
        </div>
      </section>

      <section className="shell explorer-section">
        <div className="section-heading">
          <div>
            <p className="section-number">01 / Causal record</p>
            <h2>What the world says happened</h2>
          </div>
          <p>
            Select an event to inspect its typed payload and causal predecessor.
            These are omniscient world events; later Merra systems will derive
            incomplete memories and conflicting records from them.
          </p>
        </div>
        <EventExplorer events={run.events} />
      </section>

      <section className="reproduce-section">
        <div className="shell reproduce-grid">
          <div>
            <p className="section-number light">02 / Reproduce it</p>
            <h2>No hidden history.</h2>
            <p>
              The scenario, seed, duration, source version, and output schemas
              travel together. Run the same command to regenerate the evidence.
            </p>
          </div>
          <div className="terminal-card">
            <div className="terminal-bar">
              <span />
              <span />
              <span />
              <small>merra / bash</small>
            </div>
            <code>
              <span>$</span> {run.command}
            </code>
            <dl>
              <div>
                <dt>Scenario hash</dt>
                <dd>{run.manifest.scenario_hash}</dd>
              </div>
              <div>
                <dt>Source</dt>
                <dd>{commit ? commit.slice(0, 12) : "working tree"}</dd>
              </div>
              <div>
                <dt>Runtime</dt>
                <dd>
                  Rust {run.manifest.rust_version} · Bevy{" "}
                  {run.manifest.bevy_version}
                </dd>
              </div>
            </dl>
          </div>
        </div>
      </section>

      <section className="shell chronicle-full section">
        <p className="section-number">03 / Rendered chronicle</p>
        <div className="chronicle-paper">
          <span className="paper-label">Machine evidence, made readable</span>
          <h2>Chronicle: The First Clock</h2>
          <div className="paper-rule" />
          <ul>
            <li>Scenario: {run.manifest.scenario_id}</li>
            <li>Seed: {run.manifest.seed}</li>
            <li>Calendar: {run.manifest.days} days per year</li>
            <li>Structured events: {run.summary.event_count}</li>
          </ul>
          <p>The clock advanced deterministically from Day 0 to Day 360.</p>
          <footer>Recorded at the close of Year 1</footer>
        </div>
      </section>
    </div>
  );
}
