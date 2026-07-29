import type { Metadata } from "next";
import Link from "next/link";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { getHistoryLoreShowcase } from "@/lib/content";

export const metadata: Metadata = {
  title: "History & Lore",
  description:
    "Read Merra’s canonical Year 0–660 history alongside the competing cultural claims that remember first contact."
};

function formatEventReferences(scope: string, eventIds: number[]): string {
  if (eventIds.length === 0) return scope;
  const prefix = eventIds.length === 1 ? "#" : "#";
  return `${scope} ${prefix}${eventIds.join(", #")}`;
}

function withoutTitle(source: string): string {
  return source.replace(/^# .+\n+/, "");
}

export default function HistoryPage() {
  const showcase = getHistoryLoreShowcase();

  return (
    <main className="inner-page history-page">
      <section className="page-hero history-page-hero">
        <div className="shell">
          <p className="eyebrow">
            Canonical record · Seed {showcase.seed} · Years {showcase.startYear}–
            {showcase.endYear}
          </p>
          <div className="page-title-row">
            <div>
              <h1>
                History happened once.
                <br />
                Memory made it plural.
              </h1>
              <p>
                Read the causal record first, then the stories cultures tell
                about it. Facts and lore share evidence here without being
                collapsed into the same kind of truth.
              </p>
            </div>
            <div className="schema-stamp history-stamp">
              <span>{showcase.endYear}</span>
              <small>last year</small>
            </div>
          </div>
          <div className="history-hero-stats">
            <div>
              <span>Macro events</span>
              <strong>{showcase.recordedEvents}</strong>
            </div>
            <div>
              <span>First contact</span>
              <strong>Year {showcase.firstContact.year}</strong>
            </div>
            <div>
              <span>Located events</span>
              <strong>{showcase.localLocatedEvents}</strong>
            </div>
            <div>
              <span>Competing claims</span>
              <strong>{showcase.claims.length}</strong>
            </div>
          </div>
        </div>
      </section>

      <section className="section history-record-section">
        <div className="shell">
          <div className="world-section-heading">
            <p className="section-number">01 / The recorded history</p>
            <h2>Six centuries become a region with a past.</h2>
            <p>
              These milestones are derived from the ordered event stream and
              the final local summary. Event references are stable evidence,
              not decorative dates.
            </p>
          </div>
          <ol className="history-timeline">
            {showcase.milestones.map((milestone) => (
              <li
                className={
                  milestone.phase === "First contact"
                    ? "history-milestone is-contact"
                    : "history-milestone"
                }
                key={`${milestone.years}-${milestone.title}`}
              >
                <div className="history-milestone-time">
                  <strong>{milestone.years}</strong>
                  <span>{milestone.phase}</span>
                </div>
                <i aria-hidden="true" />
                <article>
                  <h3>{milestone.title}</h3>
                  <p>{milestone.description}</p>
                  <small>
                    {formatEventReferences(
                      milestone.evidenceScope,
                      milestone.eventIds
                    )}
                  </small>
                </article>
              </li>
            ))}
          </ol>
        </div>
      </section>

      <section className="section lore-reader-section">
        <div className="shell">
          <div className="world-section-heading light-heading">
            <p className="section-number light">02 / The remembered history</p>
            <h2>One first contact. Two inherited truths.</h2>
            <p>
              Lore claims reference the event; they do not replace it. Their
              confidence belongs to the culture making the claim, not to an
              omniscient narrator.
            </p>
          </div>

          <div className="lore-record">
            <div className="lore-record-heading">
              <p className="eyebrow">Authoritative record</p>
              <strong>Event #{showcase.firstContact.eventId}</strong>
            </div>
            <p>{showcase.firstContact.record}</p>
            <dl>
              <div>
                <dt>When</dt>
                <dd>Year {showcase.firstContact.year}</dd>
              </div>
              <div>
                <dt>Where</dt>
                <dd>Fenstead · Location #{showcase.firstContact.locationId}</dd>
              </div>
              <div>
                <dt>Direct cause</dt>
                <dd>Sea route event #{showcase.firstContact.routeEventId}</dd>
              </div>
            </dl>
          </div>

          <div className="lore-branch" aria-hidden="true">
            <span>Later remembered as</span>
          </div>

          <div className="lore-claim-grid">
            {showcase.claims.map((claim, index) => (
              <article className="lore-claim" key={claim.id}>
                <div className="lore-claim-meta">
                  <span>Account {String(index + 1).padStart(2, "0")}</span>
                  <small>Claim #{claim.id}</small>
                </div>
                <p className="eyebrow">
                  {claim.sourceCulture}
                  {claim.sourceFaith ? ` · ${claim.sourceFaith}` : ""}
                </p>
                <h3>{claim.title}</h3>
                <blockquote>“{claim.text}”</blockquote>
                <div className="claim-confidence">
                  <div>
                    <span>Claimed confidence</span>
                    <strong>{claim.confidence}%</strong>
                  </div>
                  <div
                    aria-label={`${claim.confidence}% claimed confidence`}
                    className="claim-confidence-track"
                    role="img"
                  >
                    <span style={{ width: `${claim.confidence}%` }} />
                  </div>
                </div>
                <small className="claim-evidence">
                  About event #{claim.aboutEventIds.join(", #")}
                </small>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="section chronicle-reader-section">
        <div className="shell">
          <div className="world-section-heading">
            <p className="section-number">03 / The chronicles</p>
            <h2>Read the reports the simulation actually wrote.</h2>
            <p>
              The first chronicle closes the six-century macro history. The
              second follows the selected region for sixty detailed years and
              preserves both demographic consequences and inherited claims.
            </p>
          </div>
          <div className="chronicle-reader-grid">
            <article className="history-document macro-document">
              <header>
                <span>Document 01</span>
                <strong>Macro history · Years 0–600</strong>
              </header>
              <div className="history-document-body">
                <ReactMarkdown remarkPlugins={[remarkGfm]}>
                  {withoutTitle(showcase.macroChronicle)}
                </ReactMarkdown>
              </div>
            </article>
            <article className="history-document local-document">
              <header>
                <span>Document 02</span>
                <strong>Five villages · Years 600–660</strong>
              </header>
              <div className="history-document-body">
                <ReactMarkdown remarkPlugins={[remarkGfm]}>
                  {withoutTitle(showcase.localChronicle)}
                </ReactMarkdown>
              </div>
            </article>
          </div>
        </div>
      </section>

      <section className="history-next-section">
        <div className="shell history-next-grid">
          <div>
            <p className="eyebrow">Keep following the evidence</p>
            <h2>Read the past. Then watch its people move.</h2>
          </div>
          <div>
            <Link className="button button-paper" href="/villages/">
              Watch the five villages <span>→</span>
            </Link>
            <Link className="history-text-link" href="/world/">
              Return to the world atlas →
            </Link>
          </div>
        </div>
      </section>
    </main>
  );
}
