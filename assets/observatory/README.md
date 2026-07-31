# Observatory media registry

`media.json` maps stable typed identities from the historical observatory to
art briefs or available image assets. The default registry is embedded in the
`merra-tui` binary, while a custom registry can be loaded with:

```sh
cargo tui --media path/to/media.json
```

Schema version 1 records:

- `key`: an observatory identity such as `person:25`, `household:1`,
  `item:1`, `location:20`, `macro-event:37`, or `claim:1`;
- `status`: `planned` for an art brief or `available` for a checked asset;
- `asset`: an optional path relative to the manifest; it is required for an
  available entry and cannot be absolute or traverse a parent directory;
- `blake3`: an optional lowercase BLAKE3 hash, verified before an available
  asset is accepted;
- `caption` and `alt_text`: presentation copy and accessible description;
- `provenance`: required creator, license, and modifications plus an optional
  canonical HTTP(S) source URL.

Example available entry:

```json
{
  "key": "item:1",
  "status": "available",
  "asset": "items/thorn-harvest-sickle.webp",
  "blake3": "<lowercase BLAKE3 hash>",
  "caption": "The Thorn harvest sickle before its later rework.",
  "alt_text": "A repaired iron sickle with a polished wooden grip.",
  "provenance": {
    "creator": "Artist name",
    "source_url": "https://example.com/canonical-source",
    "license": "CC BY 4.0",
    "modifications": "Cropped and color balanced."
  }
}
```

Every distributed third-party file must also be listed in
[`../ATTRIBUTION.md`](../ATTRIBUTION.md). The terminal currently displays the
registry metadata inside a reserved media well; future graphics-protocol
rendering can consume the already validated resolved asset path.
