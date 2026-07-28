# Merra public site

The public site is a static Next.js application. It reads published development
records from `../docs` and selected deterministic simulation evidence from
`../golden` during the build. Those repository directories remain canonical.

```sh
npm install
npm run validate
npm run check
npm run dev
```

There is no runtime API, database, account system, or visitor-triggered
simulation. Railway builds the repository-level `site/Dockerfile` and serves
the exported files through Caddy.
