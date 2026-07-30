import type { Metadata } from "next";
import { TerminalShowcase } from "@/components/TerminalShowcase";
import { getTerminalShowcase } from "@/lib/content";

export const metadata: Metadata = {
  title: "Terminal Field Report",
  description:
    "Explore Merra’s four-generation Cycle 2 history through story-first, reproducible terminal views."
};

export default function TerminalPage() {
  const showcase = getTerminalShowcase();

  return (
    <div className="inner-page terminal-page">
      <section className="page-hero terminal-hero">
        <div className="shell">
          <p className="kicker">Terminal field report / Golden evidence 002</p>
          <div className="page-title-row">
            <div>
              <h1>A dynasty becomes a story.</h1>
              <p>
                {showcase.description} Every number and relationship below is
                derived from the same deterministic simulation evidence.
              </p>
            </div>
            <div className="schema-stamp">
              <span>Root seed</span>
              <strong>{showcase.seed}</strong>
              <small>reproducible</small>
            </div>
          </div>
          <div className="run-meta terminal-run-meta">
            <div>
              <span>People</span>
              <strong>{showcase.initialPopulation + showcase.births}</strong>
            </div>
            <div>
              <span>Living</span>
              <strong>{showcase.livingPopulation}</strong>
            </div>
            <div>
              <span>Households</span>
              <strong>{showcase.householdCount}</strong>
            </div>
            <div>
              <span>Events</span>
              <strong>{showcase.eventCount}</strong>
            </div>
          </div>
        </div>
      </section>

      <section className="shell terminal-showcase-section">
        <div className="section-heading">
          <div>
            <p className="section-number">01 / Read the result</p>
            <h2>{showcase.title}</h2>
          </div>
          <p>
            The overview leads with outcomes. History hides clock mechanics
            until requested. Lineage keeps children with their actual parental
            union, and households distinguish present membership from the lives
            that passed through them.
          </p>
        </div>
        <TerminalShowcase showcase={showcase} />
      </section>

      <section className="reproduce-section">
        <div className="shell reproduce-grid">
          <div>
            <p className="section-number light">02 / Reproduce it</p>
            <h2>The screen is evidence.</h2>
            <p>
              The interactive inspector and these ANSI-free review snapshots
              share one renderer. Stable focus IDs make a person, household, or
              event directly reproducible.
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
                <dt>Scenario</dt>
                <dd>{showcase.scenarioId}</dd>
              </div>
              <div>
                <dt>Duration</dt>
                <dd>{showcase.years} years</dd>
              </div>
              <div>
                <dt>Focused lineage</dt>
                <dd>--view lineage --focus-person 1</dd>
              </div>
            </dl>
          </div>
        </div>
      </section>
    </div>
  );
}
