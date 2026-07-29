import Link from "next/link";
import { Wordmark } from "./Wordmark";

export function SiteHeader() {
  return (
    <header className="site-header">
      <div className="shell header-inner">
        <Wordmark />
        <nav aria-label="Primary navigation">
          <Link href="/world/">World atlas</Link>
          <Link href="/villages/">Five villages</Link>
          <Link href="/objects/">Object lineages</Link>
          <Link href="/history/">History &amp; lore</Link>
          <Link href="/chronicle/">Chronicle</Link>
          <Link href="/terminal/">Terminal</Link>
          <Link href="/explore/">Explore a run</Link>
          <a
            className="nav-github"
            href="https://github.com/waynethewizard/Merra"
          >
            GitHub
            <span aria-hidden="true">↗</span>
          </a>
        </nav>
      </div>
    </header>
  );
}
