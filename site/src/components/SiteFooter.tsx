import Link from "next/link";
import { Wordmark } from "./Wordmark";

export function SiteFooter() {
  return (
    <footer className="site-footer">
      <div className="shell footer-grid">
        <div>
          <Wordmark />
          <p>
            An open-source historical simulation built in Rust and Bevy. Follow
            the evidence as a world learns to keep a past.
          </p>
        </div>
        <div className="footer-links">
          <span>Project</span>
          <Link href="/history/">History &amp; lore</Link>
          <Link href="/villages/">Village playback</Link>
          <Link href="/objects/">Object lineages</Link>
          <Link href="/explore/">Run explorer</Link>
          <Link href="/chronicle/">Development chronicle</Link>
          <a href="https://github.com/waynethewizard/Merra">Source code</a>
        </div>
        <div className="footer-note">
          <span>Current era</span>
          <strong>I · The First Hundred Years</strong>
          <p>Headless, deterministic, and deliberately small.</p>
        </div>
      </div>
      <div className="shell footer-base">
        <span>Code: MIT or Apache-2.0</span>
        <span>Original prose and art: CC BY 4.0</span>
      </div>
    </footer>
  );
}
