# Assets

Era I is headless, so this directory primarily contains asset policy and the
versioned observatory media registry. The checked
[`observatory/media.json`](observatory/media.json) currently contains art briefs
and provenance placeholders, not distributable image files.

Before adding an asset:

1. verify that its license permits repository distribution and the intended
   commercial use;
2. record its creator, canonical source URL, license, and modifications in
   `ATTRIBUTION.md`;
3. preserve the original license or notice when required;
4. remove embedded private metadata;
5. prefer source formats and keep large generated media in release attachments.

When attaching an image to an observatory identity, also update the media
registry, change its status to `available`, provide a relative asset path, and
prefer a BLAKE3 hash. See [`observatory/README.md`](observatory/README.md).

Original Merra assets use CC BY 4.0 unless explicitly marked otherwise.
