import Link from "next/link";

export function Wordmark() {
  return (
    <Link className="wordmark" href="/" aria-label="Merra home">
      <span className="wordmark-rune" aria-hidden="true">
        M
      </span>
      <span>
        <strong>Merra</strong>
        <small>A world that remembers</small>
      </span>
    </Link>
  );
}
