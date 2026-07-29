"use client";

import { useRef, useState } from "react";
type Props = {
  showcase: {
    views: {
      slug: string;
      title: string;
      description: string;
      screen: string;
    }[];
  };
};

export function TerminalShowcase({ showcase }: Props) {
  const [activeSlug, setActiveSlug] = useState(showcase.views[0]?.slug ?? "");
  const tabRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const active =
    showcase.views.find((view) => view.slug === activeSlug) ?? showcase.views[0];

  function selectTab(index: number) {
    const view = showcase.views[index];
    if (!view) return;
    setActiveSlug(view.slug);
    tabRefs.current[index]?.focus();
  }

  return (
    <div className="terminal-showcase">
      <div className="terminal-view-tabs" role="tablist" aria-label="Terminal views">
        {showcase.views.map((view, index) => (
          <button
            aria-controls="terminal-screen"
            aria-selected={active?.slug === view.slug}
            id={`terminal-tab-${view.slug}`}
            key={view.slug}
            onClick={() => setActiveSlug(view.slug)}
            onKeyDown={(event) => {
              if (event.key === "ArrowRight") {
                event.preventDefault();
                selectTab((index + 1) % showcase.views.length);
              } else if (event.key === "ArrowLeft") {
                event.preventDefault();
                selectTab(
                  (index - 1 + showcase.views.length) % showcase.views.length
                );
              } else if (event.key === "Home") {
                event.preventDefault();
                selectTab(0);
              } else if (event.key === "End") {
                event.preventDefault();
                selectTab(showcase.views.length - 1);
              }
            }}
            ref={(button) => {
              tabRefs.current[index] = button;
            }}
            role="tab"
            tabIndex={active?.slug === view.slug ? 0 : -1}
            type="button"
          >
            <strong>{view.title}</strong>
            <span>{view.description}</span>
          </button>
        ))}
      </div>
      {active ? (
        <section
          aria-labelledby={`terminal-tab-${active.slug}`}
          className="terminal-screen-frame"
          id="terminal-screen"
          role="tabpanel"
        >
          <div className="terminal-screen-bar" aria-hidden="true">
            <span />
            <span />
            <span />
            <small>merra-tui / {active.slug}</small>
          </div>
          <pre tabIndex={0}>{active.screen}</pre>
        </section>
      ) : null}
    </div>
  );
}
