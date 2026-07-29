import type { Metadata } from "next";
import Link from "next/link";
import { ItemLineageExplorer } from "@/components/ItemLineageExplorer";
import { getItemLineageShowcase } from "@/lib/content";

export const metadata: Metadata = {
  title: "Object Lineages",
  description:
    "Trace Merra’s working heirlooms through ownership, custody, repair, transformation, place, and four generations of material provenance."
};

function formatNumber(value: number): string {
  return new Intl.NumberFormat("en-US").format(value);
}

export default function ObjectsPage() {
  const showcase = getItemLineageShowcase();

  return (
    <div className="inner-page objects-page">
      <section className="page-hero objects-hero">
        <div className="shell">
          <p className="eyebrow">
            Canonical evidence · Year {showcase.projectionYear} +{" "}
            {showcase.years} · Seed {showcase.seed}
          </p>
          <div className="page-title-row">
            <div>
              <h1>
                An object can
                <br />
                have ancestors.
              </h1>
              <p>
                {showcase.description} People inherit more than names: they
                inherit things that remember use, damage, repair, movement, and
                the material identities that came before.
              </p>
            </div>
            <div className="object-generation-stamp">
              <span>G0</span>
              <i aria-hidden="true">→</i>
              <strong>G3</strong>
              <small>four generations</small>
            </div>
          </div>
          <div className="object-hero-stats">
            <div>
              <span>Stable identities</span>
              <strong>{showcase.summary.items}</strong>
            </div>
            <div>
              <span>Active heirlooms</span>
              <strong>{showcase.summary.activeItems}</strong>
            </div>
            <div>
              <span>Repairs</span>
              <strong>{showcase.summary.repairs}</strong>
            </div>
            <div>
              <span>Reworks</span>
              <strong>{showcase.summary.transformations}</strong>
            </div>
            <div>
              <span>Owner transfers</span>
              <strong>{showcase.summary.transfers}</strong>
            </div>
          </div>
        </div>
      </section>

      <section className="section object-lineage-section">
        <div className="shell">
          <div className="world-section-heading">
            <p className="section-number">01 / Follow the material</p>
            <h2>Trace the thing, not just the household holding it.</h2>
            <p>
              Choose any surviving sickle, then inspect each identity in its
              provenance chain. A source edge says what physically became the
              next object; it does not pretend the old and new identities are
              interchangeable.
            </p>
          </div>
          <ItemLineageExplorer
            biography={showcase.biography}
            featuredItemId={showcase.featuredItemId}
            items={showcase.items}
            settlements={showcase.settlements}
          />
        </div>
      </section>

      <section className="section object-rules-section">
        <div className="shell">
          <div className="world-section-heading light-heading">
            <p className="section-number light">02 / Identity rules</p>
            <h2>Continuity is explicit. So is change.</h2>
            <p>
              Final state alone cannot tell a history. Merra records which acts
              preserve identity, which create descendants, and which
              relationship answers “whose?” versus “held by whom?”
            </p>
          </div>
          <div className="object-rule-grid">
            <article>
              <span>01</span>
              <p className="eyebrow">Repair</p>
              <h3>The same thing survives.</h3>
              <p>
                Condition is restored and the repair count rises, but the stable
                item ID remains unchanged.
              </p>
              <strong>Item #46 → Item #46</strong>
            </article>
            <article>
              <span>02</span>
              <p className="eyebrow">Rework</p>
              <h3>A descendant begins.</h3>
              <p>
                The source becomes transformed. A new identity is introduced
                with a typed material edge back to it.
              </p>
              <strong>Item #31 → Item #46</strong>
            </article>
            <article>
              <span>03</span>
              <p className="eyebrow">Possession</p>
              <h3>Ownership is not custody.</h3>
              <p>
                Legal title and physical custody are separate facts, each with
                its own transfer event and causal evidence.
              </p>
              <strong>Event #2735 / #2736</strong>
            </article>
          </div>
        </div>
      </section>

      <section className="shell object-terminal-section">
        <div className="section-heading">
          <div>
            <p className="section-number">03 / Inspect the evidence</p>
            <h2>The biography and the graph share one record.</h2>
          </div>
          <p>
            The browser above is generated from the checked provenance graph.
            The terminal below exposes the same selected heirloom, including
            the events where transfer, custody, repair, use, and productive
            work meet.
          </p>
        </div>
        <div className="terminal-showcase object-terminal-showcase">
          <div className="terminal-screen-frame">
            <div className="terminal-screen-bar" aria-hidden="true">
              <span />
              <span />
              <span />
              <small>merra-tui / items / focus #{showcase.featuredItemId}</small>
            </div>
            <pre tabIndex={0}>{showcase.terminalScreen}</pre>
          </div>
        </div>
      </section>

      <section className="reproduce-section object-reproduce">
        <div className="shell reproduce-grid">
          <div>
            <p className="section-number light">04 / Reproduce it</p>
            <h2>Every heirloom is queryable evidence.</h2>
            <p>
              The scenario emits the complete final provenance graph in
              `items.json`; lifecycle events remain ordered in the local event
              stream. The site reads the checked Seed 42 golden directly.
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
                <dd>era-01-item-lineage</dd>
              </div>
              <div>
                <dt>Final records</dt>
                <dd>{formatNumber(showcase.summary.items)} item identities</dd>
              </div>
              <div>
                <dt>Source depth</dt>
                <dd>
                  G0–G{showcase.summary.maximumGeneration} · typed provenance
                </dd>
              </div>
            </dl>
          </div>
        </div>
        <div className="shell object-next-link">
          <p>These heirlooms belong to the same households you can watch move.</p>
          <Link href="/villages/">
            Return to the five villages <span aria-hidden="true">→</span>
          </Link>
        </div>
      </section>
    </div>
  );
}
