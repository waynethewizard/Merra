import type { Metadata } from "next";
import Link from "next/link";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { getCurrentCycle } from "@/lib/content";

export const metadata: Metadata = {
  title: "Development Chronicle",
  description:
    "The questions, evidence, design decisions, failures, and next steps behind each Merra development cycle."
};

export default function ChroniclePage() {
  const cycle = getCurrentCycle();

  return (
    <div className="inner-page chronicle-page">
      <section className="page-hero chronicle-hero">
        <div className="shell">
          <p className="kicker">The development chronicle</p>
          <div className="page-title-row">
            <div>
              <h1>Build the world. Keep the record.</h1>
              <p>
                Every cycle begins with a question and ends with inspectable
                evidence. This is the project’s public memory—including the
                wrong turns.
              </p>
            </div>
            <div className="issue-mark">
              <span>Current record</span>
              <strong>01</strong>
              <small>Era I · 2026</small>
            </div>
          </div>
        </div>
      </section>

      <section className="shell journal-layout">
        <aside className="journal-aside">
          <p className="section-number">Era I / Cycle 01</p>
          <h2>{cycle.title}</h2>
          <dl>
            <div>
              <dt>Status</dt>
              <dd>
                <span className="status-dot" />
                {cycle.status.replace("_", " ")}
              </dd>
            </div>
            <div>
              <dt>Started</dt>
              <dd>{cycle.started}</dd>
            </div>
            <div>
              <dt>Scenario</dt>
              <dd>{cycle.scenario}</dd>
            </div>
            <div>
              <dt>Featured seed</dt>
              <dd>{cycle.seeds.join(", ")}</dd>
            </div>
          </dl>
          <Link href="/explore/">Inspect this cycle’s evidence →</Link>
        </aside>
        <article className="journal-entry">
          <ReactMarkdown remarkPlugins={[remarkGfm]}>
            {cycle.body}
          </ReactMarkdown>
        </article>
      </section>
    </div>
  );
}
