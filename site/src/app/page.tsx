import Link from "next/link";
import { getFoundationRun } from "@/lib/content";

const cycles = [
  "Time & death",
  "The household",
  "Five villages",
  "The harvest",
  "What the dead leave",
  "The first crown",
  "The disputed heir"
];

export default function HomePage() {
  const run = getFoundationRun();

  return (
    <>
      <section className="hero">
        <div className="hero-contours" aria-hidden="true" />
        <div className="shell hero-grid">
          <div className="hero-copy">
            <div className="status-pill">
              <span />
              Now building · Era I, Cycle 01
            </div>
            <p className="kicker">An open-source historical simulation</p>
            <h1>
              A world that
              <br />
              keeps its <em>scars.</em>
            </h1>
            <p className="hero-lede">
              Merra is a world simulator where kingdoms possess a past, people
              know only what reaches them, and history survives as memory,
              record, rumor, and legend.
            </p>
            <div className="button-row">
              <Link className="button button-primary" href="/explore/">
                Explore the first run
                <span aria-hidden="true">→</span>
              </Link>
              <a
                className="button button-secondary"
                href="https://github.com/waynethewizard/Merra"
              >
                View source on GitHub
                <span aria-hidden="true">↗</span>
              </a>
            </div>
            <div className="hero-facts">
              <div>
                <strong>Rust</strong>
                <span>Portable rules</span>
              </div>
              <div>
                <strong>Bevy ECS</strong>
                <span>Headless simulation</span>
              </div>
              <div>
                <strong>Seed 42</strong>
                <span>Reproducible evidence</span>
              </div>
            </div>
          </div>

          <div className="hero-art" aria-label="A causal event chain">
            <div className="orbital orbital-one" />
            <div className="orbital orbital-two" />
            <div className="map-label map-label-west">The first clock</div>
            <div className="map-label map-label-east">360 days</div>
            <div className="timeline-card">
              <div className="timeline-card-header">
                <span>RUN / 001</span>
                <span className="live-dot">deterministic</span>
              </div>
              {run.events.map((event, index) => (
                <div className="timeline-event" key={event.id}>
                  <div className="timeline-node">
                    <span>{event.id}</span>
                    {index < run.events.length - 1 ? <i /> : null}
                  </div>
                  <div>
                    <small>Day {event.time.day}</small>
                    <strong>
                      {event.kind
                        .split("_")
                        .map(
                          (part) =>
                            part.charAt(0).toUpperCase() + part.slice(1)
                        )
                        .join(" ")}
                    </strong>
                  </div>
                </div>
              ))}
              <div className="timeline-seal">
                <span>MER</span>
                <p>
                  Event schema
                  <br />
                  verified / v1
                </p>
              </div>
            </div>
          </div>
        </div>
        <div className="hero-index shell">
          <span>41° 07′ N</span>
          <i />
          <span>Every history begins with a clock.</span>
          <i />
          <span>ERA / 01</span>
        </div>
      </section>

      <section className="manifesto section" id="world">
        <div className="shell manifesto-grid">
          <div>
            <p className="section-number">01 / The premise</p>
            <h2>
              History is not
              <br />
              a quest trigger.
            </h2>
          </div>
          <div className="manifesto-copy">
            <p className="large-copy">
              The world continues without the player. Harvests fail. Families
              migrate. Institutions preserve old grievances. A bridge changes
              trade; a disputed inheritance becomes a war.
            </p>
            <p>
              Merra simulates causes rather than selecting dramatic outcomes.
              Every meaningful change becomes structured evidence that later
              systems can witness, misunderstand, record, erase, and turn into
              folklore.
            </p>
          </div>
        </div>

        <div className="shell causal-chain">
          {[
            ["01", "Events", "What happened"],
            ["02", "Witnesses", "Who could know"],
            ["03", "Records", "What was preserved"],
            ["04", "Memory", "What was changed"],
            ["05", "Legend", "What survived"]
          ].map(([number, title, subtitle], index) => (
            <div className="causal-step" key={title}>
              <span>{number}</span>
              <div>
                <strong>{title}</strong>
                <small>{subtitle}</small>
              </div>
              {index < 4 ? <i aria-hidden="true">→</i> : null}
            </div>
          ))}
        </div>
      </section>

      <section className="first-evidence section">
        <div className="shell evidence-intro">
          <div>
            <p className="section-number light">02 / First evidence</p>
            <h2>The First Clock</h2>
          </div>
          <p>
            Before people can live and die, the world needs a trustworthy
            calendar. This tiny run proves the foundation: explicit inputs,
            ordered events, causal links, and byte-stable output.
          </p>
        </div>
        <div className="shell run-stats">
          <div>
            <span>Scenario</span>
            <strong>{run.manifest.scenario_id}</strong>
          </div>
          <div>
            <span>Root seed</span>
            <strong>{run.manifest.seed}</strong>
          </div>
          <div>
            <span>Elapsed</span>
            <strong>{run.manifest.days} days</strong>
          </div>
          <div>
            <span>Evidence</span>
            <strong>{run.summary.event_count} events</strong>
          </div>
        </div>
        <div className="shell chronicle-preview">
          <div className="chronicle-quote">
            <span className="quote-mark">“</span>
            <blockquote>
              The clock advanced deterministically from Day 0 to Day 360.
            </blockquote>
            <p>
              Chronicle excerpt · {run.title} · Seed {run.manifest.seed}
            </p>
          </div>
          <div className="chronicle-action">
            <p>
              Follow the event chain, inspect the raw payloads, and reproduce
              the run from public source.
            </p>
            <Link href="/explore/">
              Open the run explorer <span>→</span>
            </Link>
          </div>
        </div>
      </section>

      <section className="era section">
        <div className="shell era-grid">
          <div className="era-title">
            <p className="section-number">03 / The work ahead</p>
            <span>Era I</span>
            <h2>
              The First
              <br />
              Hundred Years
            </h2>
            <p>
              Seven causal slices. One readable century. Each cycle ends with
              working software, tests, reproducible evidence, and an honest
              public record.
            </p>
            <Link href="/chronicle/">Read the development chronicle →</Link>
          </div>
          <ol className="cycle-list">
            {cycles.map((cycle, index) => (
              <li className={index === 0 ? "current" : ""} key={cycle}>
                <span>{String(index + 1).padStart(2, "0")}</span>
                <strong>{cycle}</strong>
                <small>
                  {index === 0
                    ? "Foundation in progress"
                    : index === cycles.length - 1
                      ? "Era finale"
                      : "Planned"}
                </small>
              </li>
            ))}
          </ol>
        </div>
      </section>

      <section className="open-invitation">
        <div className="shell invitation-grid">
          <div>
            <p className="section-number light">Built in the open</p>
            <h2>
              Inspect the machinery.
              <br />
              Question the model.
            </h2>
          </div>
          <div>
            <p>
              Merra’s code, scenarios, architectural decisions, design
              principles, and selected histories are public. The project is
              early by design—and every durable layer should be explainable.
            </p>
            <a
              className="button button-paper"
              href="https://github.com/waynethewizard/Merra"
            >
              Visit the repository <span>↗</span>
            </a>
          </div>
        </div>
      </section>
    </>
  );
}
