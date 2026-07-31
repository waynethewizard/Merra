//! Versioned media registry for observatory portraits, objects, places, and lore.

use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::observatory::EntityRef;

const CANONICAL_MEDIA: &str = include_str!("../../../assets/observatory/media.json");
pub const OBSERVATORY_MEDIA_SCHEMA_V1: u32 = 1;

/// Portable on-disk registry of media assigned to stable observatory identities.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MediaManifestV1 {
    pub schema_version: u32,
    pub entries: Vec<MediaEntryV1>,
}

/// Whether an art brief is waiting for an asset or resolves to a checked file.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaStatusV1 {
    Planned,
    Available,
}

/// One serializable art brief or available asset.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MediaEntryV1 {
    /// Typed stable identity, such as `person:25` or `location:20`.
    pub key: String,
    pub status: MediaStatusV1,
    /// Path relative to the manifest. Required when status is `available`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset: Option<PathBuf>,
    /// Optional BLAKE3 hash for an available asset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blake3: Option<String>,
    pub caption: String,
    pub alt_text: String,
    pub provenance: MediaProvenanceV1,
}

/// Rights and source evidence kept next to every future asset.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MediaProvenanceV1 {
    pub creator: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    pub license: String,
    pub modifications: String,
}

/// Validated and path-resolved media record consumed by the renderer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaEntry {
    pub entity: EntityRef,
    pub status: MediaStatusV1,
    pub asset: Option<PathBuf>,
    pub blake3: Option<String>,
    pub caption: String,
    pub alt_text: String,
    pub provenance: MediaProvenanceV1,
}

/// Validated lookup table keyed by typed domain identity.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MediaCatalog {
    entries: BTreeMap<EntityRef, MediaEntry>,
}

impl MediaCatalog {
    /// Loads and validates the checked canonical art briefs embedded in the binary.
    pub fn canonical() -> Result<Self, MediaError> {
        let manifest =
            serde_json::from_str(CANONICAL_MEDIA).map_err(|source| MediaError::Json {
                path: PathBuf::from("assets/observatory/media.json"),
                source,
            })?;
        Self::from_manifest(manifest, None)
    }

    /// Loads a custom JSON manifest and resolves assets relative to its directory.
    pub fn load(path: &Path) -> Result<Self, MediaError> {
        let bytes = fs::read(path).map_err(|source| MediaError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let manifest = serde_json::from_slice(&bytes).map_err(|source| MediaError::Json {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_manifest(manifest, path.parent())
    }

    /// Validates an already decoded manifest, primarily for tools and tests.
    pub fn from_manifest(
        manifest: MediaManifestV1,
        base: Option<&Path>,
    ) -> Result<Self, MediaError> {
        if manifest.schema_version != OBSERVATORY_MEDIA_SCHEMA_V1 {
            return Err(MediaError::UnsupportedSchema(manifest.schema_version));
        }
        let mut entries = BTreeMap::new();
        for entry in manifest.entries {
            let entity = entry
                .key
                .parse::<EntityRef>()
                .map_err(|error| invalid(&entry.key, error.to_string()))?;
            if entries.contains_key(&entity) {
                return Err(MediaError::DuplicateKey(entry.key));
            }
            validate_text(&entry.key, "caption", &entry.caption)?;
            validate_text(&entry.key, "alt_text", &entry.alt_text)?;
            validate_text(&entry.key, "creator", &entry.provenance.creator)?;
            validate_text(&entry.key, "license", &entry.provenance.license)?;
            validate_text(&entry.key, "modifications", &entry.provenance.modifications)?;
            if let Some(url) = &entry.provenance.source_url
                && !(url.starts_with("https://") || url.starts_with("http://"))
            {
                return Err(invalid(&entry.key, "source_url must be an HTTP(S) URL"));
            }
            let asset = entry
                .asset
                .as_deref()
                .map(|asset| resolve_asset(&entry.key, asset, base))
                .transpose()?;
            if entry.status == MediaStatusV1::Available && asset.is_none() {
                return Err(invalid(
                    &entry.key,
                    "available media requires an asset path",
                ));
            }
            if entry.status == MediaStatusV1::Available {
                let Some(path) = asset.as_deref() else {
                    return Err(invalid(&entry.key, "available asset path is missing"));
                };
                if !path.is_file() {
                    return Err(invalid(
                        &entry.key,
                        format!("available asset does not exist: {}", path.display()),
                    ));
                }
                if let Some(expected) = &entry.blake3 {
                    let bytes = fs::read(path).map_err(|source| MediaError::Io {
                        path: path.to_path_buf(),
                        source,
                    })?;
                    let actual = blake3::hash(&bytes).to_hex().to_string();
                    if &actual != expected {
                        return Err(invalid(
                            &entry.key,
                            format!("BLAKE3 mismatch: expected {expected}, found {actual}"),
                        ));
                    }
                }
            } else if entry.blake3.is_some() {
                return Err(invalid(
                    &entry.key,
                    "planned media cannot declare a BLAKE3 hash",
                ));
            }
            entries.insert(
                entity,
                MediaEntry {
                    entity,
                    status: entry.status,
                    asset,
                    blake3: entry.blake3,
                    caption: entry.caption,
                    alt_text: entry.alt_text,
                    provenance: entry.provenance,
                },
            );
        }
        Ok(Self { entries })
    }

    #[must_use]
    pub fn entry(&self, entity: EntityRef) -> Option<&MediaEntry> {
        self.entries.get(&entity)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn resolve_asset(key: &str, asset: &Path, base: Option<&Path>) -> Result<PathBuf, MediaError> {
    if asset.as_os_str().is_empty()
        || asset.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(invalid(
            key,
            "asset must be a non-empty relative path without parent traversal",
        ));
    }
    let extension = asset
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(
        extension.as_str(),
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg"
    ) {
        return Err(invalid(
            key,
            "asset extension must be png, jpg, jpeg, webp, gif, or svg",
        ));
    }
    Ok(base.map_or_else(|| asset.to_path_buf(), |base| base.join(asset)))
}

fn validate_text(key: &str, field: &str, value: &str) -> Result<(), MediaError> {
    if value.trim().is_empty() {
        Err(invalid(key, format!("{field} must not be blank")))
    } else {
        Ok(())
    }
}

fn invalid(key: &str, reason: impl Into<String>) -> MediaError {
    MediaError::InvalidEntry {
        key: key.to_owned(),
        reason: reason.into(),
    }
}

#[derive(Debug, Error)]
pub enum MediaError {
    #[error("cannot read media registry at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot decode media registry at {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("unsupported observatory media schema {0}")]
    UnsupportedSchema(u32),
    #[error("duplicate observatory media key `{0}`")]
    DuplicateKey(String),
    #[error("invalid observatory media entry `{key}`: {reason}")]
    InvalidEntry { key: String, reason: String },
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        MediaCatalog, MediaEntryV1, MediaManifestV1, MediaProvenanceV1, MediaStatusV1,
        OBSERVATORY_MEDIA_SCHEMA_V1,
    };
    use crate::observatory::EntityRef;

    #[test]
    fn canonical_registry_contains_typed_art_briefs() -> Result<(), Box<dyn std::error::Error>> {
        let catalog = MediaCatalog::canonical()?;
        assert_eq!(catalog.len(), 8);
        let person = "person:1".parse::<EntityRef>()?;
        let Some(entry) = catalog.entry(person) else {
            return Err(std::io::Error::other("person:1 art brief is missing").into());
        };
        assert_eq!(entry.status, MediaStatusV1::Planned);
        assert!(entry.caption.contains("Garin Thorn"));
        assert!(entry.asset.is_none());
        Ok(())
    }

    #[test]
    fn available_assets_resolve_and_verify_hashes() -> Result<(), Box<dyn std::error::Error>> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let base = root.join("site/public");
        let bytes = std::fs::read(base.join("favicon.svg"))?;
        let hash = blake3::hash(&bytes).to_hex().to_string();
        let catalog = MediaCatalog::from_manifest(
            MediaManifestV1 {
                schema_version: OBSERVATORY_MEDIA_SCHEMA_V1,
                entries: vec![entry(
                    "location:20",
                    MediaStatusV1::Available,
                    Some(PathBuf::from("favicon.svg")),
                    Some(hash),
                )],
            },
            Some(&base),
        )?;
        let location = "location:20".parse::<EntityRef>()?;
        assert!(catalog.entry(location).is_some_and(|entry| {
            entry
                .asset
                .as_ref()
                .is_some_and(|path| path.ends_with("favicon.svg"))
        }));
        Ok(())
    }

    #[test]
    fn registry_rejects_duplicates_and_parent_traversal() {
        let duplicate = MediaManifestV1 {
            schema_version: OBSERVATORY_MEDIA_SCHEMA_V1,
            entries: vec![
                entry("item:1", MediaStatusV1::Planned, None, None),
                entry("item:1", MediaStatusV1::Planned, None, None),
            ],
        };
        assert!(MediaCatalog::from_manifest(duplicate, None).is_err());

        let traversal = MediaManifestV1 {
            schema_version: OBSERVATORY_MEDIA_SCHEMA_V1,
            entries: vec![entry(
                "item:1",
                MediaStatusV1::Planned,
                Some(PathBuf::from("../private.png")),
                None,
            )],
        };
        assert!(MediaCatalog::from_manifest(traversal, None).is_err());
    }

    fn entry(
        key: &str,
        status: MediaStatusV1,
        asset: Option<PathBuf>,
        blake3: Option<String>,
    ) -> MediaEntryV1 {
        MediaEntryV1 {
            key: key.to_owned(),
            status,
            asset,
            blake3,
            caption: String::from("A test caption."),
            alt_text: String::from("A test image description."),
            provenance: MediaProvenanceV1 {
                creator: String::from("Test creator"),
                source_url: Some(String::from("https://example.com/source")),
                license: String::from("CC BY 4.0"),
                modifications: String::from("None"),
            },
        }
    }
}
