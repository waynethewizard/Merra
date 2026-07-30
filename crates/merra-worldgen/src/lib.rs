//! Deterministic, Bevy-independent continent generation and atlas rendering.

use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, VecDeque},
};

use merra_core::{
    BiomeV1, CellResourceV1, CoordinateV1, FeatureId, FeatureKindV1, GenerationPassV1, LandformV1,
    LocationId, LocationRecordV1, PlaceAffordanceV1, PlaceGraphV1, RegionId, RngDomain, RouteId,
    RouteKindV1, RouteRecordV1, SurfaceCellV1, SurfaceWorldV1, WORLD_GENESIS_SCHEMA_V1,
    WorldFeatureV1, WorldGenesisConfigV1, WorldGenesisError, WorldGenesisSummaryV1, rng_for_domain,
};
use rand::RngExt;
use thiserror::Error;

const GENERATOR_VERSION: &str = "merra-worldgen-v1";
const PLACE_NAMES: &[&str] = &[
    "Alder", "Barrow", "Cairn", "Dun", "Esker", "Fen", "Gorse", "Hallow", "Isen", "Juniper",
    "Keld", "Lark", "Mere", "Nettle", "Oak", "Pine", "Quill", "Rill", "Stone", "Thorn", "Umber",
    "Vale", "Wold", "Yarrow",
];
const PLACE_SUFFIXES: &[&str] = &[
    "ford", "mere", "watch", "haven", "cross", "stead", "reach", "gate", "rest", "holm",
];

/// Layer rendered by the ANSI-free atlas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtlasLayer {
    /// Terrain, rivers, places, and mythic traces.
    Terrain,
    /// Climate-derived biomes.
    Biome,
    /// Relative habitability.
    Habitability,
    /// Geological and ecological resources.
    Resources,
    /// Mythic features.
    Mythic,
}

impl AtlasLayer {
    /// Stable display name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Terrain => "terrain",
            Self::Biome => "biome",
            Self::Habitability => "habitability",
            Self::Resources => "resources",
            Self::Mythic => "mythic",
        }
    }
}

/// World generation failure.
#[derive(Debug, Error)]
pub enum GenerationError {
    #[error(transparent)]
    Config(#[from] WorldGenesisError),
    #[error("world dimensions overflowed addressable memory")]
    DimensionsOverflow,
    #[error("generator could not produce enough valid places")]
    InsufficientPlaces,
    #[error("generated place graph has no locked sea route")]
    MissingSeaRoute,
    #[error("could not serialize deterministic generation evidence: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone)]
struct Plate {
    coordinate: CoordinateV1,
    motion_x: i16,
    motion_y: i16,
}

#[derive(Clone)]
struct WorkingCell {
    coordinate: CoordinateV1,
    plate: u16,
    elevation: i16,
    temperature: i16,
    precipitation: u16,
    landform: LandformV1,
    biome: BiomeV1,
    flow_to: Option<usize>,
    drainage: u32,
    river: bool,
    island: bool,
    habitability: u16,
    resources: Vec<CellResourceV1>,
    feature_ids: Vec<FeatureId>,
}

/// Generates one complete surface and generic place graph.
pub fn generate_world(
    config: &WorldGenesisConfigV1,
    seed: u64,
) -> Result<SurfaceWorldV1, GenerationError> {
    config.validate()?;
    let cell_count = usize::from(config.width)
        .checked_mul(usize::from(config.height))
        .ok_or(GenerationError::DimensionsOverflow)?;
    let template_hash = hash_json(config)?;

    let plates = make_plates(config, seed);
    let plates_hash = hash_debug(
        &plates
            .iter()
            .map(|plate| {
                (
                    plate.coordinate.x,
                    plate.coordinate.y,
                    plate.motion_x,
                    plate.motion_y,
                )
            })
            .collect::<Vec<_>>(),
    );

    let (land, island) = land_masks(config, seed, cell_count);
    let mut cells = make_elevation(config, seed, &plates, &land, &island);
    let elevation_hash = hash_cells(&cells, |cell| i64::from(cell.elevation));

    apply_climate(config, seed, &mut cells);
    let climate_hash = hash_cells(&cells, |cell| {
        i64::from(cell.temperature) * 10_001 + i64::from(cell.precipitation)
    });

    apply_hydrology(config, &mut cells);
    let hydrology_hash = hash_cells(&cells, |cell| {
        i64::from(cell.drainage) * 2 + i64::from(cell.river)
    });

    classify_and_resource(seed, &mut cells);
    let ecology_hash = hash_cells(&cells, |cell| {
        i64::from(cell.habitability) * 31 + cell.resources.len() as i64
    });

    let mut features = physical_features(&cells);
    place_mythic_traces(config, seed, &mut cells, &mut features);
    let feature_hash = hash_json(&features)?;

    let places = build_place_graph(config, seed, &cells, &features)?;
    let place_hash = hash_json(&places)?;
    let passes = vec![
        GenerationPassV1 {
            name: String::from("tectonics"),
            input_hash: template_hash,
            output_hash: plates_hash.clone(),
        },
        GenerationPassV1 {
            name: String::from("elevation"),
            input_hash: plates_hash,
            output_hash: elevation_hash.clone(),
        },
        GenerationPassV1 {
            name: String::from("climate"),
            input_hash: elevation_hash,
            output_hash: climate_hash.clone(),
        },
        GenerationPassV1 {
            name: String::from("hydrology"),
            input_hash: climate_hash,
            output_hash: hydrology_hash.clone(),
        },
        GenerationPassV1 {
            name: String::from("ecology"),
            input_hash: hydrology_hash,
            output_hash: ecology_hash.clone(),
        },
        GenerationPassV1 {
            name: String::from("mythic-traces"),
            input_hash: ecology_hash,
            output_hash: feature_hash.clone(),
        },
        GenerationPassV1 {
            name: String::from("places"),
            input_hash: feature_hash,
            output_hash: place_hash,
        },
    ];

    let cells = cells
        .into_iter()
        .enumerate()
        .map(|(index, cell)| SurfaceCellV1 {
            id: region_id(index),
            coordinate: cell.coordinate,
            plate: cell.plate,
            elevation: cell.elevation,
            temperature: cell.temperature,
            precipitation: cell.precipitation,
            landform: cell.landform,
            biome: cell.biome,
            flow_to: cell.flow_to.map(region_id),
            drainage: cell.drainage,
            river: cell.river,
            island: cell.island,
            habitability: cell.habitability,
            resources: cell.resources,
            feature_ids: cell.feature_ids,
        })
        .collect();

    Ok(SurfaceWorldV1 {
        schema_version: WORLD_GENESIS_SCHEMA_V1,
        template_id: config.id.clone(),
        title: config.title.clone(),
        seed,
        width: config.width,
        height: config.height,
        cells,
        features,
        places,
        passes,
    })
}

/// Returns compact deterministic measurements.
#[must_use]
pub fn summarize_world(world: &SurfaceWorldV1) -> WorldGenesisSummaryV1 {
    let land = world
        .cells
        .iter()
        .filter(|cell| !matches!(cell.landform, LandformV1::Ocean | LandformV1::Lake))
        .count();
    let biomes = world
        .cells
        .iter()
        .filter(|cell| cell.landform != LandformV1::Ocean)
        .map(|cell| format!("{:?}", cell.biome))
        .collect::<BTreeSet<_>>();
    WorldGenesisSummaryV1 {
        schema_version: WORLD_GENESIS_SCHEMA_V1,
        template_id: world.template_id.clone(),
        seed: world.seed,
        regions: world.cells.len(),
        land_regions: land,
        island_regions: world.cells.iter().filter(|cell| cell.island).count(),
        river_regions: world.cells.iter().filter(|cell| cell.river).count(),
        biome_count: biomes.len(),
        feature_count: world.features.len(),
        location_count: world.places.locations.len(),
        route_count: world.places.routes.len(),
        locked_sea_routes: world
            .places
            .routes
            .iter()
            .filter(|route| route.kind == RouteKindV1::Sea && route.locked)
            .count(),
    }
}

/// Hashes the complete portable world.
pub fn world_hash(world: &SurfaceWorldV1) -> Result<String, serde_json::Error> {
    hash_json(world)
}

fn make_plates(config: &WorldGenesisConfigV1, seed: u64) -> Vec<Plate> {
    let mut rng = rng_for_domain(seed, RngDomain::Tectonics);
    (0..config.plate_count)
        .map(|_| Plate {
            coordinate: CoordinateV1 {
                x: rng.random_range(0..config.width),
                y: rng.random_range(0..config.height),
            },
            motion_x: rng.random_range(-100_i16..=100_i16),
            motion_y: rng.random_range(-100_i16..=100_i16),
        })
        .collect()
}

fn land_masks(
    config: &WorldGenesisConfigV1,
    seed: u64,
    cell_count: usize,
) -> (Vec<bool>, Vec<bool>) {
    let target_land =
        cell_count.saturating_mul(usize::from(config.land_fraction_per_10_000)) / 10_000;
    let target_island =
        target_land.saturating_mul(usize::from(config.island_land_fraction_per_10_000)) / 10_000;
    let main_target = target_land.saturating_sub(target_island);
    let island_center = (
        i64::from(config.width) * 88 / 100,
        i64::from(config.height) * 66 / 100,
    );
    let main_center = (
        i64::from(config.width) * 39 / 100,
        i64::from(config.height) / 2,
    );
    let main_max_x = island_center
        .0
        .saturating_sub(i64::from(config.island_separation))
        .saturating_sub(i64::from(config.width) / 14);
    let mut main_scores = Vec::new();
    let mut island_scores = Vec::new();
    for y in 1..config.height.saturating_sub(1) {
        for x in 1..config.width.saturating_sub(1) {
            let index = index_of(config.width, x, y);
            let noise = signed_noise(seed, x, y, 0x454c_4556);
            if i64::from(x) <= main_max_x {
                let dx = i64::from(x) - main_center.0;
                let dy = i64::from(y) - main_center.1;
                let ellipse = dx * dx * 10_000
                    / (i64::from(config.width) * i64::from(config.width) / 9)
                    + dy * dy * 10_000 / (i64::from(config.height) * i64::from(config.height) / 5);
                main_scores.push((Reverse(50_000_i64 - ellipse + i64::from(noise)), index));
            }
            let dx = i64::from(x) - island_center.0;
            let dy = i64::from(y) - island_center.1;
            let distance = dx * dx + dy * dy;
            island_scores.push((distance * 1_000 - i64::from(noise), index));
        }
    }
    main_scores.sort_unstable();
    island_scores.sort_unstable();
    let mut land = vec![false; cell_count];
    let mut island = vec![false; cell_count];
    for (_, index) in main_scores.into_iter().take(main_target) {
        land[index] = true;
    }
    for (_, index) in island_scores.into_iter().take(target_island) {
        land[index] = true;
        island[index] = true;
    }
    (land, island)
}

fn make_elevation(
    config: &WorldGenesisConfigV1,
    seed: u64,
    plates: &[Plate],
    land: &[bool],
    island: &[bool],
) -> Vec<WorkingCell> {
    let mut cells = Vec::with_capacity(land.len());
    for y in 0..config.height {
        for x in 0..config.width {
            let index = index_of(config.width, x, y);
            let coordinate = CoordinateV1 { x, y };
            let mut nearest = (i64::MAX, 0_usize);
            let mut second = (i64::MAX, 0_usize);
            for (plate_index, plate) in plates.iter().enumerate() {
                let dx = i64::from(x) - i64::from(plate.coordinate.x);
                let dy = i64::from(y) - i64::from(plate.coordinate.y);
                let distance = dx * dx + dy * dy;
                if distance < nearest.0 {
                    second = nearest;
                    nearest = (distance, plate_index);
                } else if distance < second.0 {
                    second = (distance, plate_index);
                }
            }
            let plate = &plates[nearest.1];
            let other = &plates[second.1];
            let boundary = (second.0 - nearest.0).unsigned_abs() < 140;
            let convergence = i32::from(plate.motion_x - other.motion_x)
                + i32::from(plate.motion_y - other.motion_y);
            let uplift = if boundary {
                convergence.clamp(-180, 260)
            } else {
                0
            };
            let fine = i32::from(signed_noise(seed, x, y, 0x4e4f_4953));
            let elevation = if land[index] {
                (350 + fine * 2 + uplift * 4 + if island[index] { 90 } else { 0 })
                    .clamp(1, i32::from(i16::MAX)) as i16
            } else {
                (-500 + fine).clamp(i32::from(i16::MIN), -1) as i16
            };
            cells.push(WorkingCell {
                coordinate,
                plate: nearest.1 as u16,
                elevation,
                temperature: 0,
                precipitation: 0,
                landform: LandformV1::Ocean,
                biome: BiomeV1::Ocean,
                flow_to: None,
                drainage: u32::from(land[index]),
                river: false,
                island: island[index],
                habitability: 0,
                resources: Vec::new(),
                feature_ids: Vec::new(),
            });
        }
    }
    cells
}

fn apply_climate(config: &WorldGenesisConfigV1, seed: u64, cells: &mut [WorkingCell]) {
    for cell in cells {
        let latitude = (i32::from(cell.coordinate.y) * 20_000 / i32::from(config.height) - 10_000)
            .unsigned_abs() as i32;
        let altitude = i32::from(cell.elevation.max(0));
        let variation = i32::from(signed_noise(
            seed,
            cell.coordinate.x,
            cell.coordinate.y,
            0x434c_494d,
        ));
        cell.temperature = (8_500 - latitude * 7 / 10 - altitude * 2 + variation * 3)
            .clamp(-10_000, 10_000) as i16;
        if cell.elevation <= 0 {
            cell.precipitation = 10_000;
            continue;
        }
        let west_ocean = i32::from(cell.coordinate.x) * 70;
        let mountain_shadow = altitude * 5;
        cell.precipitation =
            (8_500 - west_ocean - mountain_shadow + variation * 4).clamp(250, 10_000) as u16;
    }
}

fn apply_hydrology(config: &WorldGenesisConfigV1, cells: &mut [WorkingCell]) {
    let mut distance_to_ocean = vec![u32::MAX; cells.len()];
    let mut queue = VecDeque::new();
    for (index, cell) in cells.iter().enumerate() {
        if cell.elevation <= 0 {
            distance_to_ocean[index] = 0;
            queue.push_back(index);
        }
    }
    while let Some(index) = queue.pop_front() {
        let coordinate = cells[index].coordinate;
        let next_distance = distance_to_ocean[index].saturating_add(1);
        for neighbor in neighbors(config.width, config.height, coordinate) {
            if next_distance < distance_to_ocean[neighbor] {
                distance_to_ocean[neighbor] = next_distance;
                queue.push_back(neighbor);
            }
        }
    }
    for index in 0..cells.len() {
        if cells[index].elevation <= 0 {
            continue;
        }
        let coordinate = cells[index].coordinate;
        let current_distance = distance_to_ocean[index];
        let mut best: Option<(i16, u32, usize)> = None;
        for neighbor in neighbors(config.width, config.height, coordinate) {
            let candidate = cells[neighbor].elevation;
            let candidate_distance = distance_to_ocean[neighbor];
            if candidate_distance < current_distance
                && best.is_none_or(|(best_elevation, best_distance, best_index)| {
                    (candidate, candidate_distance, neighbor)
                        < (best_elevation, best_distance, best_index)
                })
            {
                best = Some((candidate, candidate_distance, neighbor));
            }
        }
        cells[index].flow_to = best.map(|(_, _, target)| target);
    }

    let mut order: Vec<_> = (0..cells.len()).collect();
    order.sort_unstable_by_key(|index| (Reverse(distance_to_ocean[*index]), *index));
    for index in order {
        if let Some(target) = cells[index].flow_to {
            cells[target].drainage = cells[target].drainage.saturating_add(cells[index].drainage);
        }
    }
    for cell in cells {
        if cell.elevation > 0 && cell.flow_to.is_none() {
            cell.landform = LandformV1::Lake;
            cell.biome = BiomeV1::Lake;
        }
        cell.river = cell.elevation > 0 && cell.drainage >= 30;
    }
}

fn classify_and_resource(seed: u64, cells: &mut [WorkingCell]) {
    for cell in cells {
        if cell.elevation <= 0 {
            cell.landform = LandformV1::Ocean;
            cell.biome = BiomeV1::Ocean;
            continue;
        }
        if cell.landform == LandformV1::Lake {
            cell.habitability = 6_000;
            continue;
        }
        cell.landform = if cell.elevation > 1_050 {
            LandformV1::Mountain
        } else if cell.elevation > 650 {
            LandformV1::Highland
        } else if cell.flow_to.is_some_and(|target| target == 0) {
            LandformV1::Coast
        } else {
            LandformV1::Lowland
        };
        cell.biome = if cell.elevation > 1_050 {
            BiomeV1::Alpine
        } else if cell.temperature < -1_500 {
            BiomeV1::Tundra
        } else if cell.precipitation < 1_800 {
            BiomeV1::Desert
        } else if cell.river && cell.precipitation > 6_000 {
            BiomeV1::Wetland
        } else if cell.temperature < 2_000 {
            BiomeV1::BorealForest
        } else if cell.precipitation > 4_500 {
            BiomeV1::TemperateForest
        } else {
            BiomeV1::Grassland
        };
        let water = if cell.river || cell.precipitation > 4_500 {
            3_000
        } else {
            0
        };
        let terrain_penalty = match cell.landform {
            LandformV1::Mountain => 4_000,
            LandformV1::Highland => 1_500,
            _ => 0,
        };
        let biome_base: i32 = match cell.biome {
            BiomeV1::TemperateForest | BiomeV1::Grassland => 6_000,
            BiomeV1::Wetland | BiomeV1::BorealForest => 4_800,
            BiomeV1::Desert | BiomeV1::Tundra => 2_000,
            BiomeV1::Alpine => 800,
            _ => 0,
        };
        cell.habitability = (biome_base + water - terrain_penalty).clamp(0, 10_000) as u16;

        let resource_roll = unsigned_noise(seed, cell.coordinate.x, cell.coordinate.y, 0x5245_534f);
        if matches!(cell.biome, BiomeV1::Grassland | BiomeV1::TemperateForest) {
            cell.resources.push(CellResourceV1 {
                resource: String::from("food"),
                amount_per_10_000: (4_000 + resource_roll % 4_000) as u16,
            });
        }
        if matches!(cell.biome, BiomeV1::TemperateForest | BiomeV1::BorealForest) {
            cell.resources.push(CellResourceV1 {
                resource: String::from("timber"),
                amount_per_10_000: (5_000 + resource_roll % 3_000) as u16,
            });
        }
        if matches!(cell.landform, LandformV1::Highland | LandformV1::Mountain) {
            cell.resources.push(CellResourceV1 {
                resource: String::from("stone"),
                amount_per_10_000: (5_500 + resource_roll % 3_500) as u16,
            });
            if resource_roll.is_multiple_of(7) {
                cell.resources.push(CellResourceV1 {
                    resource: String::from("ore"),
                    amount_per_10_000: (2_500 + resource_roll % 5_000) as u16,
                });
            }
        }
    }
}

fn physical_features(cells: &[WorkingCell]) -> Vec<WorldFeatureV1> {
    let mountains: Vec<_> = cells
        .iter()
        .enumerate()
        .filter(|(_, cell)| cell.landform == LandformV1::Mountain)
        .map(|(index, _)| region_id(index))
        .collect();
    let rivers: Vec<_> = cells
        .iter()
        .enumerate()
        .filter(|(_, cell)| cell.river)
        .map(|(index, _)| region_id(index))
        .collect();
    let mut features = Vec::new();
    if !mountains.is_empty() {
        features.push(WorldFeatureV1 {
            id: FeatureId(1),
            name: String::from("The Long Uplands"),
            kind: FeatureKindV1::MountainRange,
            regions: mountains,
            description: String::from(
                "A continent-spanning family of uplifts produced by converging plates.",
            ),
        });
    }
    if !rivers.is_empty() {
        features.push(WorldFeatureV1 {
            id: FeatureId(features.len() as u64 + 1),
            name: String::from("The River Veins"),
            kind: FeatureKindV1::River,
            regions: rivers,
            description: String::from(
                "Connected high-drainage regions carrying water toward lower ground.",
            ),
        });
    }
    features
}

fn place_mythic_traces(
    config: &WorldGenesisConfigV1,
    seed: u64,
    cells: &mut [WorkingCell],
    features: &mut Vec<WorldFeatureV1>,
) {
    let mut candidates: Vec<_> = cells
        .iter()
        .enumerate()
        .filter(|(_, cell)| cell.elevation > 0 && cell.landform != LandformV1::Lake)
        .map(|(index, cell)| {
            (
                Reverse(unsigned_noise(
                    seed,
                    cell.coordinate.x,
                    cell.coordinate.y,
                    0x4d59_5448,
                )),
                index,
            )
        })
        .collect();
    candidates.sort_unstable();
    let mut used = BTreeSet::new();
    for motif in &config.mythic_motifs {
        for ordinal in 0..motif.count {
            let Some((_, index)) = candidates.iter().find(|(_, index)| used.insert(*index)) else {
                break;
            };
            let feature_id = FeatureId(features.len() as u64 + 1);
            cells[*index].feature_ids.push(feature_id);
            features.push(WorldFeatureV1 {
                id: feature_id,
                name: format!("{} {}", motif.name, ordinal.saturating_add(1)),
                kind: FeatureKindV1::MythicTrace {
                    motif_id: motif.id.clone(),
                },
                regions: vec![region_id(*index)],
                description: format!(
                    "An observable {} with no authoritative account of its makers or cause.",
                    motif.name.to_lowercase()
                ),
            });
        }
    }
}

fn build_place_graph(
    config: &WorldGenesisConfigV1,
    seed: u64,
    cells: &[WorkingCell],
    features: &[WorldFeatureV1],
) -> Result<PlaceGraphV1, GenerationError> {
    let island_target = usize::from(config.place_count / 5).max(4);
    let continent_target = usize::from(config.place_count).saturating_sub(island_target);
    let continent = select_places(cells, false, continent_target, config.width);
    let island = select_places(cells, true, island_target, config.width);
    if continent.len() < 3 || island.is_empty() {
        return Err(GenerationError::InsufficientPlaces);
    }
    let mut indices = continent;
    indices.extend(island);
    let locations: Vec<_> = indices
        .iter()
        .enumerate()
        .map(|(ordinal, index)| {
            let cell = &cells[*index];
            let roll =
                unsigned_noise(seed, cell.coordinate.x, cell.coordinate.y, 0x504c_4143) as usize;
            let name = format!(
                "{}{}",
                PLACE_NAMES[roll % PLACE_NAMES.len()],
                PLACE_SUFFIXES[(roll / PLACE_NAMES.len()) % PLACE_SUFFIXES.len()]
            );
            let food = resource_amount(cell, "food").max(cell.habitability);
            let water = if cell.river || cell.precipitation > 4_500 {
                9_000
            } else {
                4_000
            };
            let material = resource_amount(cell, "stone")
                .max(resource_amount(cell, "timber"))
                .max(2_000);
            LocationRecordV1 {
                id: LocationId(ordinal as u64 + 1),
                name,
                region: Some(region_id(*index)),
                tags: vec![
                    if cell.island { "island" } else { "continent" }.to_owned(),
                    if cell.island {
                        "isolated_homeland"
                    } else {
                        "primary_homeland"
                    }
                    .to_owned(),
                    format!("{:?}", cell.biome).to_lowercase(),
                ],
                carrying_capacity: 1_200 + u32::from(cell.habitability) * 2,
                hazard_per_10_000: 10_000_u16.saturating_sub(cell.habitability) / 5,
                affordances: vec![
                    PlaceAffordanceV1 {
                        id: String::from("food"),
                        value_per_10_000: food,
                    },
                    PlaceAffordanceV1 {
                        id: String::from("fresh_water"),
                        value_per_10_000: water,
                    },
                    PlaceAffordanceV1 {
                        id: String::from("construction"),
                        value_per_10_000: material,
                    },
                    PlaceAffordanceV1 {
                        id: String::from("navigation"),
                        value_per_10_000: if cell.island || cell.river {
                            7_500
                        } else {
                            2_500
                        },
                    },
                ],
                feature_ids: features
                    .iter()
                    .filter(|feature| feature.regions.contains(&region_id(*index)))
                    .map(|feature| feature.id)
                    .collect(),
            }
        })
        .collect();
    let by_region: BTreeMap<_, _> = locations
        .iter()
        .filter_map(|location| location.region.map(|region| (region, location.id)))
        .collect();
    let selected: Vec<_> = indices
        .iter()
        .map(|index| (*index, by_region[&region_id(*index)]))
        .collect();
    let mut edge_set = BTreeSet::new();
    let mut route_specs = Vec::new();
    for (source_index, source_id) in &selected {
        let source = &cells[*source_index];
        let mut nearest: Vec<_> = selected
            .iter()
            .filter(|(_, candidate_id)| candidate_id != source_id)
            .filter(|(candidate_index, _)| cells[*candidate_index].island == source.island)
            .map(|(candidate_index, candidate_id)| {
                (
                    distance(source.coordinate, cells[*candidate_index].coordinate),
                    *candidate_id,
                )
            })
            .collect();
        nearest.sort_unstable();
        for (distance, candidate_id) in nearest.into_iter().take(2) {
            let endpoints = ordered_pair(*source_id, candidate_id);
            if edge_set.insert(endpoints) {
                route_specs.push((endpoints, RouteKindV1::Land, distance, false));
            }
        }
    }
    let mut cross: Vec<_> = selected
        .iter()
        .filter(|(index, _)| !cells[*index].island)
        .flat_map(|(first_index, first_id)| {
            selected
                .iter()
                .filter(|(index, _)| cells[*index].island)
                .map(move |(second_index, second_id)| {
                    (
                        distance(
                            cells[*first_index].coordinate,
                            cells[*second_index].coordinate,
                        ),
                        ordered_pair(*first_id, *second_id),
                    )
                })
        })
        .collect();
    cross.sort_unstable();
    if let Some((distance, endpoints)) = cross.first().copied() {
        route_specs.push((endpoints, RouteKindV1::Sea, distance, true));
    }
    let routes = route_specs
        .into_iter()
        .enumerate()
        .map(
            |(index, (endpoints, kind, travel_cost, locked))| RouteRecordV1 {
                id: RouteId(index as u64 + 1),
                endpoints,
                kind,
                travel_cost,
                capacity: if kind == RouteKindV1::Sea { 600 } else { 1_200 },
                locked,
                required_capability: locked.then(|| String::from("navigation")),
            },
        )
        .collect::<Vec<_>>();
    if !routes
        .iter()
        .any(|route| route.kind == RouteKindV1::Sea && route.locked)
    {
        return Err(GenerationError::MissingSeaRoute);
    }
    Ok(PlaceGraphV1 { locations, routes })
}

fn select_places(cells: &[WorkingCell], island: bool, target: usize, width: u16) -> Vec<usize> {
    let mut candidates: Vec<_> = cells
        .iter()
        .enumerate()
        .filter(|(_, cell)| {
            cell.island == island
                && cell.elevation > 0
                && cell.landform != LandformV1::Lake
                && cell.habitability >= 1_500
        })
        .map(|(index, cell)| (Reverse(cell.habitability), index))
        .collect();
    candidates.sort_unstable();
    let mut selected: Vec<usize> = Vec::new();
    for (_, index) in candidates {
        let coordinate = cells[index].coordinate;
        if selected.iter().all(|chosen| {
            distance(coordinate, cells[*chosen].coordinate) >= u32::from(width / 20).max(4)
        }) {
            selected.push(index);
            if selected.len() == target {
                break;
            }
        }
    }
    selected
}

fn neighbors(width: u16, height: u16, coordinate: CoordinateV1) -> Vec<usize> {
    let mut result = Vec::with_capacity(8);
    for dy in -1_i32..=1 {
        for dx in -1_i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let x = i32::from(coordinate.x) + dx;
            let y = i32::from(coordinate.y) + dy;
            if x >= 0 && y >= 0 && x < i32::from(width) && y < i32::from(height) {
                result.push(index_of(width, x as u16, y as u16));
            }
        }
    }
    result
}

fn region_id(index: usize) -> RegionId {
    RegionId(index as u64 + 1)
}

fn index_of(width: u16, x: u16, y: u16) -> usize {
    usize::from(y) * usize::from(width) + usize::from(x)
}

fn distance(first: CoordinateV1, second: CoordinateV1) -> u32 {
    u32::from(first.x.abs_diff(second.x)) + u32::from(first.y.abs_diff(second.y))
}

fn ordered_pair(first: LocationId, second: LocationId) -> [LocationId; 2] {
    if first <= second {
        [first, second]
    } else {
        [second, first]
    }
}

fn resource_amount(cell: &WorkingCell, key: &str) -> u16 {
    cell.resources
        .iter()
        .find(|resource| resource.resource == key)
        .map_or(0, |resource| resource.amount_per_10_000)
}

fn unsigned_noise(seed: u64, x: u16, y: u16, domain: u32) -> u32 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"merra-world-noise-v1\0");
    hasher.update(&seed.to_le_bytes());
    hasher.update(&domain.to_le_bytes());
    hasher.update(&x.to_le_bytes());
    hasher.update(&y.to_le_bytes());
    let bytes = hasher.finalize();
    u32::from_le_bytes(bytes.as_bytes()[0..4].try_into().unwrap_or_default())
}

fn signed_noise(seed: u64, x: u16, y: u16, domain: u32) -> i16 {
    let value = unsigned_noise(seed, x, y, domain) % 401;
    value as i16 - 200
}

fn hash_json(value: &impl serde::Serialize) -> Result<String, serde_json::Error> {
    Ok(blake3::hash(&serde_json::to_vec(value)?)
        .to_hex()
        .to_string())
}

fn hash_debug(value: &impl std::fmt::Debug) -> String {
    blake3::hash(format!("{value:?}").as_bytes())
        .to_hex()
        .to_string()
}

fn hash_cells(cells: &[WorkingCell], value: impl Fn(&WorkingCell) -> i64) -> String {
    let mut hasher = blake3::Hasher::new();
    for cell in cells {
        hasher.update(&value(cell).to_le_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

/// Renders a deterministic publication-ready SVG atlas.
#[must_use]
pub fn render_svg(world: &SurfaceWorldV1) -> String {
    let scale = 6_u32;
    let map_width = u32::from(world.width) * scale;
    let map_height = u32::from(world.height) * scale;
    let full_width = map_width + 300;
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {full_width} {map_height}\" role=\"img\" aria-labelledby=\"title desc\">\n<title id=\"title\">{}</title>\n<desc id=\"desc\">Deterministic atlas for seed {}</desc>\n<rect width=\"100%\" height=\"100%\" fill=\"#101c18\"/>\n",
        escape_xml(&world.title),
        world.seed
    );
    let biome_order = [
        BiomeV1::Ocean,
        BiomeV1::Lake,
        BiomeV1::Tundra,
        BiomeV1::BorealForest,
        BiomeV1::TemperateForest,
        BiomeV1::Grassland,
        BiomeV1::Wetland,
        BiomeV1::Desert,
        BiomeV1::Alpine,
    ];
    for biome in biome_order {
        let mut path = String::new();
        for y in 0..world.height {
            let mut x = 0_u16;
            while x < world.width {
                let index = usize::from(y) * usize::from(world.width) + usize::from(x);
                if world.cells[index].biome != biome {
                    x = x.saturating_add(1);
                    continue;
                }
                let start = x;
                while x < world.width {
                    let candidate = usize::from(y) * usize::from(world.width) + usize::from(x);
                    if world.cells[candidate].biome != biome {
                        break;
                    }
                    x = x.saturating_add(1);
                }
                let run = u32::from(x.saturating_sub(start)) * scale;
                let left = u32::from(start) * scale;
                let top = u32::from(y) * scale;
                path.push_str(&format!("M{left} {top}h{run}v{scale}H{left}z"));
            }
        }
        if !path.is_empty() {
            svg.push_str(&format!(
                "<path fill=\"{}\" d=\"{path}\"/>",
                biome_color(biome)
            ));
        }
    }
    for cell in world.cells.iter().filter(|cell| cell.river) {
        let x = u32::from(cell.coordinate.x) * scale;
        let y = u32::from(cell.coordinate.y) * scale;
        svg.push_str(&format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"1.4\" fill=\"#75b7c9\"/>",
            x + scale / 2,
            y + scale / 2
        ));
    }
    let region_lookup: BTreeMap<_, _> = world
        .cells
        .iter()
        .map(|cell| (cell.id, cell.coordinate))
        .collect();
    for feature in &world.features {
        if !matches!(feature.kind, FeatureKindV1::MythicTrace { .. }) {
            continue;
        }
        if let Some(coordinate) = feature
            .regions
            .first()
            .and_then(|region| region_lookup.get(region))
        {
            svg.push_str(&format!(
                "<path d=\"M {} {} l 5 5 m -5 0 l 5 -5\" stroke=\"#e4be68\" stroke-width=\"1.5\"/>",
                u32::from(coordinate.x) * scale + 1,
                u32::from(coordinate.y) * scale + 1
            ));
        }
    }
    for location in &world.places.locations {
        if let Some(coordinate) = location
            .region
            .and_then(|region| region_lookup.get(&region))
        {
            let x = u32::from(coordinate.x) * scale + scale / 2;
            let y = u32::from(coordinate.y) * scale + scale / 2;
            svg.push_str(&format!(
                "<circle cx=\"{x}\" cy=\"{y}\" r=\"2.2\" fill=\"#f4ead4\" stroke=\"#8c3f31\" stroke-width=\"1\"/>"
            ));
        }
    }
    let panel_x = map_width + 28;
    svg.push_str(&format!(
        "<g fill=\"#f4ead4\" font-family=\"ui-monospace,monospace\"><text x=\"{panel_x}\" y=\"48\" font-family=\"Georgia,serif\" font-size=\"26\">{}</text><text x=\"{panel_x}\" y=\"75\" font-size=\"12\" fill=\"#c69d60\">WORLD GENESIS / SEED {}</text>",
        escape_xml(&world.title),
        world.seed
    ));
    let summary = summarize_world(world);
    let facts = [
        format!("{} regions", summary.regions),
        format!(
            "{} land · {} island",
            summary.land_regions, summary.island_regions
        ),
        format!("{} river regions", summary.river_regions),
        format!("{} candidate places", summary.location_count),
        format!("{} ambiguous features", summary.feature_count),
        format!("{} locked sea route", summary.locked_sea_routes),
    ];
    for (index, fact) in facts.iter().enumerate() {
        svg.push_str(&format!(
            "<text x=\"{panel_x}\" y=\"{}\" font-size=\"13\">{}</text>",
            120 + index * 28,
            escape_xml(fact)
        ));
    }
    svg.push_str(&format!(
        "<text x=\"{panel_x}\" y=\"330\" font-size=\"11\" fill=\"#96aa98\">□ place  × unexplained trace</text><text x=\"{panel_x}\" y=\"360\" font-size=\"11\" fill=\"#96aa98\">Generated from public source.</text></g></svg>\n"
    ));
    svg
}

/// Renders a compact ANSI-free atlas for tests, logs, and redirected output.
#[must_use]
pub fn render_snapshot(
    world: &SurfaceWorldV1,
    layer: AtlasLayer,
    width: u16,
    height: u16,
) -> String {
    if width < 60 || height < 16 {
        return String::from("Merra World Atlas · use at least 60×16\n");
    }
    let map_width = usize::from(width.saturating_sub(27)).min(usize::from(world.width));
    let map_height = usize::from(height.saturating_sub(6)).min(usize::from(world.height));
    let x_step = usize::from(world.width).div_ceil(map_width).max(1);
    let y_step = usize::from(world.height).div_ceil(map_height).max(1);
    let mut output = format!(
        "MERRA // WORLD ATLAS\n{} · seed {} · layer {}\n",
        world.title.to_uppercase(),
        world.seed,
        layer.name()
    );
    let place_regions: BTreeSet<_> = world
        .places
        .locations
        .iter()
        .filter_map(|location| location.region)
        .collect();
    for y in (0..usize::from(world.height))
        .step_by(y_step)
        .take(map_height)
    {
        for x in (0..usize::from(world.width))
            .step_by(x_step)
            .take(map_width)
        {
            let cell = &world.cells[y * usize::from(world.width) + x];
            let character = if place_regions.contains(&cell.id) {
                '●'
            } else {
                snapshot_character(cell, layer)
            };
            output.push(character);
        }
        output.push('\n');
    }
    let summary = summarize_world(world);
    output.push_str(&format!(
        "{} land · {} island · {} rivers · {} places · {} locked sea route\n",
        summary.land_regions,
        summary.island_regions,
        summary.river_regions,
        summary.location_count,
        summary.locked_sea_routes
    ));
    output.push_str("● candidate place  ✦ mythic trace  ≈ water  ▲ mountain\n");
    output
}

fn snapshot_character(cell: &SurfaceCellV1, layer: AtlasLayer) -> char {
    if !cell.feature_ids.is_empty() && layer == AtlasLayer::Mythic {
        return '✦';
    }
    match layer {
        AtlasLayer::Terrain => {
            if cell.river {
                '│'
            } else {
                match cell.landform {
                    LandformV1::Ocean => '≈',
                    LandformV1::Lake => '~',
                    LandformV1::Mountain => '▲',
                    LandformV1::Highland => '^',
                    LandformV1::Coast => '·',
                    LandformV1::Lowland => ',',
                }
            }
        }
        AtlasLayer::Biome => match cell.biome {
            BiomeV1::Ocean => '≈',
            BiomeV1::Lake => '~',
            BiomeV1::Tundra => '░',
            BiomeV1::BorealForest => '♠',
            BiomeV1::TemperateForest => '♣',
            BiomeV1::Grassland => '"',
            BiomeV1::Wetland => ';',
            BiomeV1::Desert => '·',
            BiomeV1::Alpine => '▲',
        },
        AtlasLayer::Habitability => match cell.habitability {
            0 => ' ',
            1..=2_499 => '░',
            2_500..=4_999 => '▒',
            5_000..=7_499 => '▓',
            _ => '█',
        },
        AtlasLayer::Resources => {
            if cell
                .resources
                .iter()
                .any(|resource| resource.resource == "ore")
            {
                '◆'
            } else if cell
                .resources
                .iter()
                .any(|resource| resource.resource == "timber")
            {
                '♣'
            } else if cell
                .resources
                .iter()
                .any(|resource| resource.resource == "food")
            {
                '•'
            } else if cell.landform == LandformV1::Ocean {
                '≈'
            } else {
                '·'
            }
        }
        AtlasLayer::Mythic => {
            if cell.landform == LandformV1::Ocean {
                '≈'
            } else {
                '·'
            }
        }
    }
}

fn biome_color(biome: BiomeV1) -> &'static str {
    match biome {
        BiomeV1::Ocean => "#173b4f",
        BiomeV1::Lake => "#376d7e",
        BiomeV1::Tundra => "#a9b5aa",
        BiomeV1::BorealForest => "#294f43",
        BiomeV1::TemperateForest => "#3f6747",
        BiomeV1::Grassland => "#8b9b57",
        BiomeV1::Wetland => "#607b5d",
        BiomeV1::Desert => "#c3a76f",
        BiomeV1::Alpine => "#8b8f88",
    }
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Generator implementation identifier stored in manifests.
#[must_use]
pub const fn generator_version() -> &'static str {
    GENERATOR_VERSION
}

#[cfg(test)]
mod tests {
    use merra_core::{MythicMotifConfigV1, WORLD_GENESIS_SCHEMA_V1, WorldGenesisConfigV1};

    use super::{AtlasLayer, generate_world, render_snapshot, render_svg, summarize_world};

    fn config() -> WorldGenesisConfigV1 {
        WorldGenesisConfigV1 {
            schema_version: WORLD_GENESIS_SCHEMA_V1,
            id: String::from("test-world"),
            title: String::from("A World Before Memory"),
            width: 64,
            height: 48,
            plate_count: 8,
            land_fraction_per_10_000: 4_800,
            island_land_fraction_per_10_000: 800,
            island_separation: 8,
            place_count: 16,
            mythic_motifs: vec![MythicMotifConfigV1 {
                id: String::from("stone-rings"),
                name: String::from("Stone Ring"),
                count: 3,
            }],
        }
    }

    #[test]
    fn identical_seeds_produce_identical_worlds() -> Result<(), Box<dyn std::error::Error>> {
        let first = generate_world(&config(), 42)?;
        let second = generate_world(&config(), 42)?;
        assert_eq!(first, second);
        assert_ne!(first, generate_world(&config(), 43)?);
        Ok(())
    }

    #[test]
    fn world_has_island_rivers_places_and_locked_contact() -> Result<(), Box<dyn std::error::Error>>
    {
        let world = generate_world(&config(), 42)?;
        let summary = summarize_world(&world);
        assert!(summary.land_regions > 0);
        assert!(summary.island_regions > 0);
        assert!(summary.river_regions > 0);
        assert!(summary.location_count >= 8);
        assert_eq!(summary.locked_sea_routes, 1);
        assert!(
            world
                .cells
                .iter()
                .all(|cell| cell.flow_to.is_none_or(|target| target != cell.id))
        );
        Ok(())
    }

    #[test]
    fn atlas_is_portable_and_ansi_free() -> Result<(), Box<dyn std::error::Error>> {
        let world = generate_world(&config(), 42)?;
        let snapshot = render_snapshot(&world, AtlasLayer::Terrain, 100, 30);
        assert!(snapshot.contains("WORLD ATLAS"));
        assert!(snapshot.contains("locked sea route"));
        assert!(!snapshot.contains('\u{1b}'));
        let svg = render_svg(&world);
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("WORLD GENESIS"));
        Ok(())
    }
}
