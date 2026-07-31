//! Ratatui rendering for the cross-scale historical observatory.

use std::collections::BTreeMap;

use merra_core::{BiomeV1, HistoricalEventPayloadV1, LandformV1, LocationId, RouteId, RouteKindV1};
use ratatui::{
    Frame, Terminal,
    backend::TestBackend,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

use crate::observatory::{
    CatalogKind, EntityRef, HitRegions, Observatory, ObservatoryLayer, ObservatoryTheme,
    ObservatoryView, PaneFocus,
};

#[derive(Clone, Copy)]
struct Palette {
    background: Color,
    ink: Color,
    dim: Color,
    border: Color,
    copper: Color,
    teal: Color,
    rust: Color,
    water: Color,
    lowland: Color,
    highland: Color,
    mountain: Color,
    forest: Color,
}

impl Palette {
    const fn for_theme(theme: ObservatoryTheme) -> Self {
        match theme {
            ObservatoryTheme::Archive => Self {
                background: Color::Rgb(18, 20, 22),
                ink: Color::Rgb(224, 214, 190),
                dim: Color::Rgb(122, 126, 122),
                border: Color::Rgb(99, 92, 78),
                copper: Color::Rgb(205, 137, 76),
                teal: Color::Rgb(85, 176, 166),
                rust: Color::Rgb(188, 84, 68),
                water: Color::Rgb(55, 93, 112),
                lowland: Color::Rgb(96, 127, 86),
                highland: Color::Rgb(142, 126, 82),
                mountain: Color::Rgb(177, 173, 160),
                forest: Color::Rgb(64, 131, 91),
            },
            ObservatoryTheme::Monochrome => Self {
                background: Color::Reset,
                ink: Color::Reset,
                dim: Color::DarkGray,
                border: Color::Gray,
                copper: Color::White,
                teal: Color::White,
                rust: Color::White,
                water: Color::Gray,
                lowland: Color::Gray,
                highland: Color::White,
                mountain: Color::White,
                forest: Color::Gray,
            },
        }
    }
}

/// Draws the complete observatory and records hit regions for mouse input.
pub fn render_observatory(frame: &mut Frame<'_>, app: &mut Observatory) {
    synchronize_visual_focus(app);
    app.hits = HitRegions::default();
    let area = frame.area();
    let palette = Palette::for_theme(app.theme);
    frame.render_widget(
        Block::new().style(Style::default().bg(palette.background).fg(palette.ink)),
        area,
    );
    if area.width < 60 || area.height < 18 {
        render_too_small(frame, area, palette);
        return;
    }

    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(4),
        Constraint::Length(1),
    ])
    .split(area);
    render_header(frame, rows[0], app, palette);
    render_tabs(frame, rows[1], app, palette);
    match app.view {
        ObservatoryView::Atlas => render_atlas(frame, rows[2], app, palette),
        ObservatoryView::Chronicle => render_chronicle(frame, rows[2], app, palette),
        ObservatoryView::Relations => render_relations(frame, rows[2], app, palette),
        ObservatoryView::Catalog => render_catalog(frame, rows[2], app, palette),
    }
    render_timeline(frame, rows[3], app, palette);
    render_pane_focus(frame.buffer_mut(), app, palette);
    render_footer(frame, rows[4], app, palette);
    if app.show_help {
        render_help(frame, area, app, palette);
    }
    if app.searching {
        render_search(frame, area, app, palette);
    }
}

fn render_pane_focus(buffer: &mut Buffer, app: &Observatory, palette: Palette) {
    let area = match app.pane {
        PaneFocus::Primary => app.hits.primary,
        PaneFocus::Detail => app.hits.detail,
        PaneFocus::Timeline => app.hits.timeline,
    };
    if area.width < 2 || area.height < 2 {
        return;
    }
    let right = area.x.saturating_add(area.width.saturating_sub(1));
    let bottom = area.y.saturating_add(area.height.saturating_sub(1));
    for x in area.x..=right {
        buffer[(x, area.y)].set_fg(palette.copper);
        buffer[(x, bottom)].set_fg(palette.copper);
    }
    for y in area.y..=bottom {
        buffer[(area.x, y)].set_fg(palette.copper);
        buffer[(right, y)].set_fg(palette.copper);
    }
}

fn synchronize_visual_focus(app: &mut Observatory) {
    let selected = match app.view {
        ObservatoryView::Chronicle => app
            .visible_moments()
            .get(app.selection)
            .map(|moment| moment.entity),
        ObservatoryView::Catalog => app.catalog_entities().get(app.selection).copied(),
        ObservatoryView::Atlas | ObservatoryView::Relations => None,
    };
    if let Some(entity) = selected
        && app.focus != Some(entity)
    {
        app.focus = Some(entity);
        app.detail_scroll = 0;
        app.transition_epoch = app.transition_epoch.saturating_add(1);
    }
}

/// Produces an ANSI-free screen for tests, documentation, and redirected output.
#[must_use]
pub fn render_observatory_snapshot(app: &Observatory, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = match Terminal::new(backend) {
        Ok(terminal) => terminal,
        Err(never) => match never {},
    };
    let mut snapshot = app.clone();
    let draw = terminal.draw(|frame| render_observatory(frame, &mut snapshot));
    if let Err(never) = draw {
        match never {}
    }
    let buffer = terminal.backend().buffer();
    let mut output = String::new();
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            line.push_str(buffer[(x, y)].symbol());
        }
        output.push_str(line.trim_end());
        output.push('\n');
    }
    output
}

fn render_too_small(frame: &mut Frame<'_>, area: Rect, palette: Palette) {
    frame.render_widget(
        Paragraph::new(vec![
            Line::styled(
                "MERRA // HISTORICAL OBSERVATORY",
                Style::default()
                    .fg(palette.copper)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::default(),
            Line::from(format!(
                "Terminal is {}×{}; use at least 60×18.",
                area.width, area.height
            )),
        ])
        .block(archive_block("Archive closed", palette))
        .alignment(Alignment::Center),
        area,
    );
}

fn render_header(frame: &mut Frame<'_>, area: Rect, app: &Observatory, palette: Palette) {
    let history = app.data.history.as_ref();
    let local = app.data.local.as_ref();
    let focus = app.focus.map_or_else(
        || String::from("the generated world"),
        |entity| app.label(entity),
    );
    let evidence = format!(
        "{} regions · {} places{}{}",
        app.data.world.cells.len(),
        app.data.world.places.locations.len(),
        history.map_or_else(String::new, |report| format!(
            " · {} macro events · {} people",
            report.events.len(),
            report.summary.total_population
        )),
        local.map_or_else(String::new, |report| format!(
            " · {} local lives · {} heirlooms",
            report.people.len(),
            report.items.len()
        )),
    );
    let text = Text::from(vec![
        Line::from(vec![
            Span::styled(
                "MERRA",
                Style::default()
                    .fg(palette.copper)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " // HISTORICAL OBSERVATORY",
                Style::default().fg(palette.ink),
            ),
            Span::styled(
                format!("   YEAR {:>3}", app.cursor_year),
                Style::default()
                    .fg(palette.teal)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if app.playing { "  ▶ PLAYING" } else { "" },
                Style::default().fg(palette.teal),
            ),
        ]),
        Line::styled(evidence, Style::default().fg(palette.dim)),
        Line::from(vec![
            Span::styled("Focus  ", Style::default().fg(palette.dim)),
            Span::styled(focus, Style::default().fg(palette.ink)),
            Span::styled(
                if app.back_stack.is_empty() {
                    String::new()
                } else {
                    format!("  ·  {} step(s) in trail", app.back_stack.len())
                },
                Style::default().fg(palette.dim),
            ),
        ]),
    ]);
    frame.render_widget(text, area);
}

fn render_tabs(frame: &mut Frame<'_>, area: Rect, app: &mut Observatory, palette: Palette) {
    let widths = [
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
    ];
    let tabs = Layout::horizontal(widths).split(area);
    for (index, view) in ObservatoryView::ALL.iter().enumerate() {
        let selected = app.view == *view;
        let style = if selected {
            Style::default()
                .fg(palette.background)
                .bg(palette.copper)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(palette.dim)
        };
        frame.render_widget(
            Paragraph::new(format!("{}  {}", index + 1, view.label()))
                .alignment(Alignment::Center)
                .style(style)
                .block(
                    Block::new()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(if selected {
                            palette.copper
                        } else {
                            palette.border
                        })),
                ),
            tabs[index],
        );
        app.hits.tabs.push((*view, tabs[index]));
    }
}

fn render_atlas(frame: &mut Frame<'_>, area: Rect, app: &mut Observatory, palette: Palette) {
    let columns = if area.width >= 100 {
        Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)]).split(area)
    } else {
        Layout::vertical([Constraint::Percentage(62), Constraint::Percentage(38)]).split(area)
    };
    let map_area = columns[0];
    let detail_area = columns[1];
    app.hits.primary = map_area;
    app.hits.detail = detail_area;

    let map_title = format!("World Atlas · {} · {}×", app.layer.label(), app.map_zoom);
    let map_block = archive_block(&map_title, palette);
    let inner = map_block.inner(map_area);
    frame.render_widget(map_block, map_area);
    render_world_map(frame.buffer_mut(), inner, app, palette);
    render_detail(frame, detail_area, app, palette);
}

fn render_world_map(buffer: &mut Buffer, area: Rect, app: &Observatory, palette: Palette) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let (start_x, start_y, visible_width, visible_height) = app.map_window();
    let locations_by_region = app
        .data
        .world
        .places
        .locations
        .iter()
        .filter_map(|location| location.region.map(|region| (region, location.id)))
        .collect::<BTreeMap<_, _>>();
    let selected_location = match app.focus {
        Some(EntityRef::Location(location)) => Some(location),
        _ => None,
    };

    for screen_y in 0..area.height {
        for screen_x in 0..area.width {
            let world_x = start_x.saturating_add(
                u16::try_from(
                    u32::from(screen_x).saturating_mul(u32::from(visible_width))
                        / u32::from(area.width),
                )
                .unwrap_or(0),
            );
            let world_y = start_y.saturating_add(
                u16::try_from(
                    u32::from(screen_y).saturating_mul(u32::from(visible_height))
                        / u32::from(area.height),
                )
                .unwrap_or(0),
            );
            let index =
                usize::from(world_y) * usize::from(app.data.world.width) + usize::from(world_x);
            if let Some(cell) = app.data.world.cells.get(index) {
                let (symbol, style) = atlas_cell(cell, app.layer, palette);
                buffer[(area.x + screen_x, area.y + screen_y)]
                    .set_symbol(symbol)
                    .set_style(style.bg(palette.background));
            }
        }
    }

    render_routes(
        buffer,
        area,
        app,
        palette,
        (start_x, start_y, visible_width, visible_height),
    );

    for (region, location) in locations_by_region {
        let Some(cell) = app
            .data
            .world
            .cells
            .iter()
            .find(|candidate| candidate.id == region)
        else {
            continue;
        };
        let Some((screen_x, screen_y)) = project_coordinate(
            cell.coordinate.x,
            cell.coordinate.y,
            area,
            start_x,
            start_y,
            visible_width,
            visible_height,
        ) else {
            continue;
        };
        let residents = app
            .local_state()
            .and_then(|state| state.residents.get(&location))
            .copied()
            .unwrap_or(0);
        let selected = selected_location == Some(location);
        let symbol = if selected {
            "◆"
        } else if residents > 0 {
            "●"
        } else if app.data.local.as_ref().is_some_and(|local| {
            local
                .settlements
                .iter()
                .any(|settlement| settlement.location_id == location)
        }) {
            "○"
        } else {
            "•"
        };
        let style = Style::default()
            .fg(if selected {
                palette.copper
            } else if residents > 0 {
                palette.teal
            } else {
                palette.ink
            })
            .bg(palette.background)
            .add_modifier(if selected {
                Modifier::BOLD
            } else {
                Modifier::empty()
            });
        buffer[(screen_x, screen_y)]
            .set_symbol(symbol)
            .set_style(style);
    }

    if let Some((screen_x, screen_y)) = project_coordinate(
        app.map_x,
        app.map_y,
        area,
        start_x,
        start_y,
        visible_width,
        visible_height,
    ) {
        let cell = &mut buffer[(screen_x, screen_y)];
        if selected_location.is_none() {
            cell.set_symbol("◇")
                .set_style(Style::default().fg(palette.copper).bg(palette.background));
        }
    }
}

fn render_routes(
    buffer: &mut Buffer,
    area: Rect,
    app: &Observatory,
    palette: Palette,
    window: (u16, u16, u16, u16),
) {
    let (start_x, start_y, visible_width, visible_height) = window;
    let coordinates = app
        .data
        .world
        .places
        .locations
        .iter()
        .filter_map(|location| {
            let region = location.region?;
            let cell = app
                .data
                .world
                .cells
                .iter()
                .find(|candidate| candidate.id == region)?;
            Some((location.id, (cell.coordinate.x, cell.coordinate.y)))
        })
        .collect::<BTreeMap<_, _>>();
    for route in &app.data.world.places.routes {
        if !route_is_open(route.id, route.locked, app) {
            continue;
        }
        let (Some(from), Some(to)) = (
            coordinates.get(&route.endpoints[0]),
            coordinates.get(&route.endpoints[1]),
        ) else {
            continue;
        };
        let (Some(start), Some(end)) = (
            project_coordinate(
                from.0,
                from.1,
                area,
                start_x,
                start_y,
                visible_width,
                visible_height,
            ),
            project_coordinate(
                to.0,
                to.1,
                area,
                start_x,
                start_y,
                visible_width,
                visible_height,
            ),
        ) else {
            continue;
        };
        let style = Style::default()
            .fg(match route.kind {
                RouteKindV1::Sea | RouteKindV1::River => palette.water,
                RouteKindV1::Land | RouteKindV1::Abstract => palette.dim,
            })
            .bg(palette.background);
        draw_line(buffer, start, end, "·", style);
    }
}

fn route_is_open(route_id: RouteId, locked: bool, app: &Observatory) -> bool {
    if !locked {
        return true;
    }
    let Some(history) = app.data.history.as_ref() else {
        return false;
    };
    history.events.iter().any(|event| {
        let matches = match &event.payload {
            HistoricalEventPayloadV1::RouteOpened {
                route_id: opened, ..
            }
            | HistoricalEventPayloadV1::SeaRouteOpened { route_id: opened } => *opened == route_id,
            _ => false,
        };
        matches
            && u32::try_from(event.time.day() / app.macro_days_per_year())
                .is_ok_and(|year| year <= app.cursor_year)
    })
}

fn atlas_cell(
    cell: &merra_core::SurfaceCellV1,
    layer: ObservatoryLayer,
    palette: Palette,
) -> (&'static str, Style) {
    let (symbol, color) = match layer {
        ObservatoryLayer::History | ObservatoryLayer::Terrain => {
            if cell.river {
                ("│", palette.water)
            } else {
                match cell.landform {
                    LandformV1::Ocean => ("≈", palette.water),
                    LandformV1::Lake => ("~", palette.water),
                    LandformV1::Mountain => ("▲", palette.mountain),
                    LandformV1::Highland => ("^", palette.highland),
                    LandformV1::Coast => ("·", palette.lowland),
                    LandformV1::Lowland => ("‚", palette.lowland),
                }
            }
        }
        ObservatoryLayer::Biome => match cell.biome {
            BiomeV1::Ocean => ("≈", palette.water),
            BiomeV1::Lake => ("~", palette.water),
            BiomeV1::Tundra => ("░", palette.mountain),
            BiomeV1::BorealForest | BiomeV1::TemperateForest => ("♣", palette.forest),
            BiomeV1::Grassland => ("\"", palette.lowland),
            BiomeV1::Wetland => (";", palette.teal),
            BiomeV1::Desert => ("·", palette.highland),
            BiomeV1::Alpine => ("▲", palette.mountain),
        },
        ObservatoryLayer::Habitability => match cell.habitability {
            0 => (" ", palette.dim),
            1..=2_499 => ("░", palette.dim),
            2_500..=4_999 => ("▒", palette.highland),
            5_000..=7_499 => ("▓", palette.lowland),
            _ => ("█", palette.teal),
        },
        ObservatoryLayer::Resources => {
            if cell
                .resources
                .iter()
                .any(|resource| resource.resource == "ore")
            {
                ("◆", palette.copper)
            } else if cell
                .resources
                .iter()
                .any(|resource| resource.resource == "timber")
            {
                ("♣", palette.forest)
            } else if cell.landform == LandformV1::Ocean {
                ("≈", palette.water)
            } else {
                ("·", palette.dim)
            }
        }
        ObservatoryLayer::Mythic => {
            if cell.feature_ids.is_empty() {
                if cell.landform == LandformV1::Ocean {
                    ("≈", palette.water)
                } else {
                    ("·", palette.dim)
                }
            } else {
                ("✦", palette.copper)
            }
        }
    };
    let color = if layer == ObservatoryLayer::History {
        match color {
            value if value == palette.water => value,
            _ => palette.dim,
        }
    } else {
        color
    };
    (symbol, Style::default().fg(color))
}

fn render_chronicle(frame: &mut Frame<'_>, area: Rect, app: &mut Observatory, palette: Palette) {
    let columns =
        Layout::horizontal([Constraint::Percentage(46), Constraint::Percentage(54)]).split(area);
    app.hits.primary = columns[0];
    app.hits.detail = columns[1];
    let moments = app
        .visible_moments()
        .into_iter()
        .map(|moment| (moment.entity, moment.year, moment.label.clone()))
        .collect::<Vec<_>>();
    let selected = app.selection.min(moments.len().saturating_sub(1));
    let chronicle_title = format!(
        "Chronicle · {} records{}",
        moments.len(),
        if app.show_debug {
            " · complete stream"
        } else {
            ""
        }
    );
    let inner = archive_block(&chronicle_title, palette);
    let list_area = inner.inner(columns[0]);
    frame.render_widget(inner, columns[0]);
    render_entity_rows(
        frame,
        list_area,
        app,
        &moments
            .iter()
            .map(|(entity, year, label)| (*entity, format!("Y{year:>3}  {label}")))
            .collect::<Vec<_>>(),
        selected,
        palette,
    );
    render_detail(frame, columns[1], app, palette);
}

fn render_relations(frame: &mut Frame<'_>, area: Rect, app: &mut Observatory, palette: Palette) {
    let columns =
        Layout::horizontal([Constraint::Percentage(66), Constraint::Percentage(34)]).split(area);
    app.hits.primary = columns[0];
    app.hits.detail = columns[1];
    let focus = app.focus;
    let label = focus.map_or_else(
        || String::from("Nothing selected"),
        |entity| app.label(entity),
    );
    let block = archive_block("Typed Relations · Enter follows an edge", palette);
    let inner = block.inner(columns[0]);
    frame.render_widget(block, columns[0]);
    let edges = app
        .relation_list()
        .iter()
        .map(|relation| {
            (
                relation.target,
                format!("├─ {:<20} {}", relation.label, app.label(relation.target)),
            )
        })
        .collect::<Vec<_>>();
    let title_area = Rect::new(inner.x, inner.y, inner.width, inner.height.min(3));
    frame.render_widget(
        Paragraph::new(vec![
            Line::styled(
                format!("◆ {label}"),
                Style::default()
                    .fg(palette.copper)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::styled(
                format!(
                    "{}:{} · {} linked records",
                    focus.map_or("none", EntityRef::kind),
                    focus.map_or(0, EntityRef::raw),
                    edges.len()
                ),
                Style::default().fg(palette.dim),
            ),
        ]),
        title_area,
    );
    let rows_area = Rect::new(
        inner.x.saturating_add(2),
        inner.y.saturating_add(title_area.height),
        inner.width.saturating_sub(2),
        inner.height.saturating_sub(title_area.height),
    );
    render_entity_rows(
        frame,
        rows_area,
        app,
        &edges,
        app.selection.min(edges.len().saturating_sub(1)),
        palette,
    );
    render_detail(frame, columns[1], app, palette);
}

fn render_catalog(frame: &mut Frame<'_>, area: Rect, app: &mut Observatory, palette: Palette) {
    let columns =
        Layout::horizontal([Constraint::Percentage(48), Constraint::Percentage(52)]).split(area);
    app.hits.primary = columns[0];
    app.hits.detail = columns[1];
    let catalog_title = format!("Catalog · {} · ←/→ category", app.catalog_kind.label());
    let block = archive_block(&catalog_title, palette);
    let inner = block.inner(columns[0]);
    frame.render_widget(block, columns[0]);
    let categories = CatalogKind::ALL
        .iter()
        .map(|kind| {
            if *kind == app.catalog_kind {
                Span::styled(
                    format!("[{}] ", short_catalog_label(*kind)),
                    Style::default()
                        .fg(palette.copper)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    format!("{} ", short_catalog_label(*kind)),
                    Style::default().fg(palette.dim),
                )
            }
        })
        .collect::<Vec<_>>();
    let category_area = Rect::new(inner.x, inner.y, inner.width, 1);
    frame.render_widget(Paragraph::new(Line::from(categories)), category_area);
    let list_area = Rect::new(
        inner.x,
        inner.y.saturating_add(1),
        inner.width,
        inner.height.saturating_sub(1),
    );
    let entities = app.catalog_entities().to_vec();
    let rows = entities
        .iter()
        .map(|entity| {
            (
                *entity,
                format!(
                    "{:<13} #{:<5} {}",
                    entity.kind(),
                    entity.raw(),
                    app.label(*entity)
                ),
            )
        })
        .collect::<Vec<_>>();
    render_entity_rows(
        frame,
        list_area,
        app,
        &rows,
        app.selection.min(rows.len().saturating_sub(1)),
        palette,
    );
    render_detail(frame, columns[1], app, palette);
}

fn render_entity_rows(
    frame: &mut Frame<'_>,
    area: Rect,
    app: &mut Observatory,
    rows: &[(EntityRef, String)],
    selected: usize,
    palette: Palette,
) {
    if area.height == 0 {
        return;
    }
    let height = usize::from(area.height);
    let start = selected
        .saturating_sub(height / 2)
        .min(rows.len().saturating_sub(height));
    let mut lines = Vec::new();
    for (visible_index, (index, (entity, label))) in
        rows.iter().enumerate().skip(start).take(height).enumerate()
    {
        let selected_row = index == selected;
        lines.push(Line::styled(
            format!("{} {label}", if selected_row { "▶" } else { " " }),
            if selected_row {
                Style::default()
                    .fg(palette.background)
                    .bg(palette.copper)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(palette.ink)
            },
        ));
        app.hits.rows.push((
            *entity,
            Rect::new(
                area.x,
                area.y
                    .saturating_add(u16::try_from(visible_index).unwrap_or(0)),
                area.width,
                1,
            ),
        ));
    }
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_detail(frame: &mut Frame<'_>, area: Rect, app: &Observatory, palette: Palette) {
    let focus = app.focus;
    let title = focus.map_or_else(
        || String::from("Evidence"),
        |entity| format!("{} #{}", title_case(entity.kind()), entity.raw()),
    );
    let block = archive_block(&title, palette);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let lines = detail_lines(app, focus, palette);
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((app.detail_scroll, 0)),
        inner,
    );
}

fn detail_lines(
    app: &Observatory,
    focus: Option<EntityRef>,
    palette: Palette,
) -> Vec<Line<'static>> {
    let Some(focus) = focus else {
        return vec![Line::styled(
            "Select a record to inspect its evidence.",
            Style::default().fg(palette.dim),
        )];
    };
    let mut lines = vec![
        Line::styled(
            app.label(focus),
            Style::default()
                .fg(palette.copper)
                .add_modifier(Modifier::BOLD),
        ),
        Line::styled(
            format!("{}:{}", focus.kind(), focus.raw()),
            Style::default().fg(palette.dim),
        ),
        Line::default(),
    ];
    let mut body = entity_details(app, focus);
    if body.is_empty() {
        body.push(String::from("No additional structured evidence."));
    }
    lines.extend(body.into_iter().map(Line::from));
    let relation_count = app.relations.get(&focus).map_or(0, Vec::len);
    lines.push(Line::default());
    lines.push(Line::styled(
        format!("{relation_count} typed relation(s) · press 3 to trace"),
        Style::default().fg(palette.teal),
    ));
    lines
}

fn entity_details(app: &Observatory, focus: EntityRef) -> Vec<String> {
    match focus {
        EntityRef::Region(id) => app
            .data
            .world
            .cells
            .iter()
            .find(|cell| cell.id == id)
            .map(|cell| {
                vec![
                    format!(
                        "Coordinate {},{} · plate {}",
                        cell.coordinate.x, cell.coordinate.y, cell.plate
                    ),
                    format!(
                        "{:?} / {:?} · elevation {}",
                        cell.landform, cell.biome, cell.elevation
                    ),
                    format!(
                        "Temperature {} · precipitation {} · habitability {}%",
                        cell.temperature,
                        cell.precipitation,
                        cell.habitability / 100
                    ),
                    format!(
                        "Resources: {}",
                        cell.resources
                            .iter()
                            .map(|resource| format!(
                                "{} {}%",
                                resource.resource,
                                resource.amount_per_10_000 / 100
                            ))
                            .collect::<Vec<_>>()
                            .join(" · ")
                    ),
                ]
            })
            .unwrap_or_default(),
        EntityRef::Feature(id) => app
            .data
            .world
            .features
            .iter()
            .find(|feature| feature.id == id)
            .map(|feature| {
                vec![
                    format!("{:?} · {} regions", feature.kind, feature.regions.len()),
                    feature.description.clone(),
                ]
            })
            .unwrap_or_default(),
        EntityRef::Location(id) => location_details(app, id),
        EntityRef::Route(id) => app
            .data
            .world
            .places
            .routes
            .iter()
            .find(|route| route.id == id)
            .map(|route| {
                vec![
                    format!(
                        "{:?} · #{} ↔ #{}",
                        route.kind, route.endpoints[0].0, route.endpoints[1].0
                    ),
                    format!(
                        "Cost {} · capacity {} · {}",
                        route.travel_cost,
                        route.capacity,
                        if route.locked {
                            "capability-gated"
                        } else {
                            "available from the epoch"
                        }
                    ),
                    String::from("Atlas connectors are schematic, not claimed terrain paths."),
                ]
            })
            .unwrap_or_default(),
        EntityRef::Population(id) => app
            .data
            .history
            .as_ref()
            .and_then(|history| history.populations.iter().find(|record| record.id == id))
            .map(|record| {
                vec![
                    format!(
                        "{} people at final macro state · founded Y{}",
                        record.people, record.founded_year
                    ),
                    format!("Location #{}", record.location_id.0),
                    String::from(
                        "Earlier annual population totals are not present; no interpolation is shown.",
                    ),
                ]
            })
            .unwrap_or_default(),
        EntityRef::Culture(id) => app
            .data
            .history
            .as_ref()
            .and_then(|history| history.cultures.iter().find(|record| record.id == id))
            .map(|record| {
                vec![
                    format!("Founded Year {} · event #{}", record.founded_year, record.origin_event.0),
                    format!(
                        "{} ritual days/year · preservation {}%",
                        record.ritual_days_per_year,
                        record.institutional_preservation_per_10_000 / 100
                    ),
                    format!(
                        "Faith transmission {}% · sacred contribution {}%",
                        record.faith_transmission_per_10_000 / 100,
                        record.sacred_contribution_per_10_000 / 100
                    ),
                ]
            })
            .unwrap_or_default(),
        EntityRef::Faith(id) => app
            .data
            .history
            .as_ref()
            .and_then(|history| history.faiths.iter().find(|record| record.id == id))
            .map(|record| {
                vec![
                    format!("Founded Year {} · event #{}", record.founded_year, record.origin_event.0),
                    format!(
                        "Parent: {} · source trace: {}",
                        record.parent_faith_id.map_or_else(
                            || String::from("none"),
                            |value| format!("#{}", value.0)
                        ),
                        record.source_feature_id.map_or_else(
                            || String::from("none"),
                            |value| format!("#{}", value.0)
                        )
                    ),
                ]
            })
            .unwrap_or_default(),
        EntityRef::Institution(id) => app
            .data
            .history
            .as_ref()
            .and_then(|history| {
                history
                    .institutions
                    .iter()
                    .find(|record| record.id == id)
            })
            .map(|record| {
                vec![
                    format!(
                        "Founded Year {} at location #{}",
                        record.founded_year, record.location_id.0
                    ),
                    format!(
                        "Culture #{} · faith {} · {}",
                        record.culture_id.0,
                        record.faith_id.map_or_else(
                            || String::from("none"),
                            |value| format!("#{}", value.0)
                        ),
                        record.dissolved_year.map_or_else(
                            || String::from("active at final state"),
                            |year| format!("dissolved Y{year}")
                        )
                    ),
                ]
            })
            .unwrap_or_default(),
        EntityRef::Polity(id) => app
            .data
            .history
            .as_ref()
            .and_then(|history| history.polities.iter().find(|record| record.id == id))
            .map(|record| {
                vec![
                    format!("Founded Year {}", record.founded_year),
                    format!(
                        "{} locations · {} cultures",
                        record.location_ids.len(),
                        record.culture_ids.len()
                    ),
                ]
            })
            .unwrap_or_default(),
        EntityRef::Household(id) => app
            .data
            .local
            .as_ref()
            .and_then(|local| local.households.iter().find(|record| record.id == id))
            .map(|record| {
                vec![
                    format!(
                        "{} current members · {} children born",
                        record.member_ids.len(),
                        record.children_born
                    ),
                    format!(
                        "Founded local Year {} · {}",
                        record.founded_day / app.local_days_per_year(),
                        record.dissolved_day.map_or_else(
                            || String::from("active at final state"),
                            |day| {
                                format!(
                                    "dissolved local Year {}",
                                    day / app.local_days_per_year()
                                )
                            }
                        )
                    ),
                    format!(
                        "Residence: {}",
                        record.residence_id.map_or_else(
                            || String::from("unplaced"),
                            |location| app.label(EntityRef::Location(location))
                        )
                    ),
                ]
            })
            .unwrap_or_default(),
        EntityRef::Person(id) => app
            .data
            .local
            .as_ref()
            .and_then(|local| local.people.iter().find(|record| record.id == id))
            .map(|record| {
                vec![
                    format!(
                        "Generation {} · age {} · {}",
                        record.generation,
                        record.final_age_years,
                        if record.alive { "living" } else { "dead" }
                    ),
                    format!(
                        "Parents: {}",
                        if record.parent_ids.is_empty() {
                            String::from("projected founder")
                        } else {
                            record
                                .parent_ids
                                .iter()
                                .map(|parent| app.label(EntityRef::Person(*parent)))
                                .collect::<Vec<_>>()
                                .join(" + ")
                        }
                    ),
                    format!(
                        "Household: {} · partner: {}",
                        record.household_id.map_or_else(
                            || String::from("none"),
                            |household| app.label(EntityRef::Household(household))
                        ),
                        record.partner_id.map_or_else(
                            || String::from("none"),
                            |partner| app.label(EntityRef::Person(partner))
                        )
                    ),
                ]
            })
            .unwrap_or_default(),
        EntityRef::Item(id) => app
            .data
            .local
            .as_ref()
            .and_then(|local| local.items.iter().find(|record| record.id == id))
            .map(|record| {
                let visible = u64::from(app.cursor_year.saturating_sub(
                    app.data
                        .local
                        .as_ref()
                        .map_or(0, |local| local.summary.projection_year),
                ))
                .saturating_mul(app.local_days_per_year())
                    >= record.introduced_day;
                let state = if visible {
                    format!(
                        "{:?} · condition {}%",
                        record.status,
                        record.condition_per_10_000 / 100
                    )
                } else {
                    String::from("not yet introduced at this year")
                };
                vec![
                    format!("Lineage G{} · {state}", record.lineage_generation),
                    format!(
                        "{} repairs · introduced local Year {} by event #{}",
                        record.repairs,
                        record.introduced_day / app.local_days_per_year(),
                        record.introduction_event_id.0
                    ),
                    format!(
                        "Sources: {}",
                        if record.sources.is_empty() {
                            String::from("none; original identity")
                        } else {
                            record
                                .sources
                                .iter()
                                .map(|source| format!("#{} ({:?})", source.item_id.0, source.role))
                                .collect::<Vec<_>>()
                                .join(" · ")
                        }
                    ),
                    format!("Owner {:?} · custody {:?}", record.owner, record.custody),
                ]
            })
            .unwrap_or_default(),
        EntityRef::MacroEvent(_) | EntityRef::LocalEvent(_) => app
            .moments
            .iter()
            .find(|moment| moment.entity == focus)
            .map(|moment| {
                vec![
                    format!("Authoritative event · global Year {}", moment.year),
                    format!(
                        "Location: {}",
                        moment.location.map_or_else(
                            || String::from("not located"),
                            |location| app.label(EntityRef::Location(location))
                        )
                    ),
                    format!(
                        "Subjects: {}",
                        moment
                            .subjects
                            .iter()
                            .map(|entity| app.label(*entity))
                            .collect::<Vec<_>>()
                            .join(" · ")
                    ),
                    format!(
                        "Caused by: {}",
                        if moment.causes.is_empty() {
                            String::from("no recorded causal parent")
                        } else {
                            moment
                                .causes
                                .iter()
                                .map(|entity| app.label(*entity))
                                .collect::<Vec<_>>()
                                .join(" · ")
                        }
                    ),
                    format!("Tags: {}", moment.tags.join(", ")),
                ]
            })
            .unwrap_or_default(),
        EntityRef::Claim(id) => app
            .data
            .history
            .as_ref()
            .and_then(|history| history.lore.iter().find(|claim| claim.id == id))
            .map(|claim| {
                vec![
                    String::from("INTERPRETATION — not authoritative fact"),
                    claim.text.clone(),
                    format!("Confidence {}%", claim.confidence_per_10_000 / 100),
                    format!(
                        "Source culture {} · source faith {}",
                        app.label(EntityRef::Culture(claim.source_culture_id)),
                        claim.source_faith_id.map_or_else(
                            || String::from("none"),
                            |faith| app.label(EntityRef::Faith(faith))
                        )
                    ),
                    format!(
                        "About authoritative event(s): {}",
                        claim
                            .about_events
                            .iter()
                            .map(|event| format!("#{}", event.0))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                ]
            })
            .unwrap_or_default(),
    }
}

fn location_details(app: &Observatory, id: LocationId) -> Vec<String> {
    let mut lines = app
        .data
        .world
        .places
        .locations
        .iter()
        .find(|location| location.id == id)
        .map(|location| {
            vec![
                format!(
                    "Capacity {} · hazard {}%",
                    location.carrying_capacity,
                    location.hazard_per_10_000 / 100
                ),
                format!("Tags: {}", location.tags.join(", ")),
            ]
        })
        .unwrap_or_default();
    if let Some(history) = app.data.history.as_ref()
        && let Some(settlement) = history
            .settlements
            .iter()
            .find(|settlement| settlement.location_id == id)
    {
        lines.push(format!(
            "Macro settlement founded Y{} · final population {}",
            settlement.founded_year, settlement.population
        ));
    }
    if let Some(local) = app.data.local.as_ref()
        && let Some(settlement) = local
            .settlements
            .iter()
            .find(|settlement| settlement.location_id == id)
    {
        let residents = app
            .local_state()
            .and_then(|state| state.residents.get(&id))
            .copied()
            .unwrap_or(0);
        lines.push(format!(
            "Local sample at Y{}: {} living",
            app.cursor_year, residents
        ));
        lines.push(format!(
            "Final evidence: {} births · {} deaths · {} arrivals · {} departures",
            settlement.births, settlement.deaths, settlement.arrivals, settlement.departures
        ));
    }
    lines
}

fn render_timeline(frame: &mut Frame<'_>, area: Rect, app: &mut Observatory, palette: Palette) {
    app.hits.timeline = area;
    let timeline_title = format!(
        "Archive Time · Year {} / {} · {}",
        app.cursor_year,
        app.maximum_year,
        if app.playing { "playing" } else { "paused" }
    );
    let block = archive_block(&timeline_title, palette);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || app.maximum_year == 0 {
        return;
    }
    let line_y = inner.y;
    let mut density = vec![0_usize; usize::from(inner.width)];
    for moment in &app.moments {
        let column = usize::try_from(
            moment
                .year
                .saturating_mul(u32::from(inner.width.saturating_sub(1)))
                / app.maximum_year.max(1),
        )
        .unwrap_or(0)
        .min(density.len().saturating_sub(1));
        density[column] = density[column].saturating_add(1);
    }
    let maximum = density.iter().copied().max().unwrap_or(1).max(1);
    for (index, count) in density.iter().enumerate() {
        let symbol = match count.saturating_mul(4) / maximum {
            0 => "─",
            1 => "·",
            2 => "•",
            _ => "▪",
        };
        let x = inner.x.saturating_add(u16::try_from(index).unwrap_or(0));
        frame.buffer_mut()[(x, line_y)]
            .set_symbol(symbol)
            .set_style(Style::default().fg(palette.dim).bg(palette.background));
    }
    for (year, symbol, color) in [
        (0, "│", palette.ink),
        (
            app.data
                .history
                .as_ref()
                .and_then(|history| history.summary.first_contact_year)
                .unwrap_or(0),
            "┼",
            palette.copper,
        ),
        (
            app.data
                .local
                .as_ref()
                .map_or(app.maximum_year, |local| local.summary.projection_year),
            "┼",
            palette.teal,
        ),
        (app.maximum_year, "│", palette.ink),
    ] {
        let x = timeline_x(year, inner, app.maximum_year);
        frame.buffer_mut()[(x, line_y)]
            .set_symbol(symbol)
            .set_style(Style::default().fg(color).bg(palette.background));
    }
    let cursor_x = timeline_x(app.cursor_year, inner, app.maximum_year);
    frame.buffer_mut()[(cursor_x, line_y)]
        .set_symbol("◆")
        .set_style(Style::default().fg(palette.copper).bg(palette.background));
    if inner.height > 1 {
        let contact = app
            .data
            .history
            .as_ref()
            .and_then(|history| history.summary.first_contact_year)
            .map_or_else(
                || String::from("no contact"),
                |year| format!("contact Y{year}"),
            );
        let projection = app.data.local.as_ref().map_or_else(
            || String::from("no local handoff"),
            |local| format!("local Y{}", local.summary.projection_year),
        );
        frame.render_widget(
            Paragraph::new(format!(
                "Y0  ·  {contact}  ·  {projection}  ·  Y{}",
                app.maximum_year
            ))
            .style(Style::default().fg(palette.dim))
            .alignment(Alignment::Center),
            Rect::new(inner.x, inner.y + 1, inner.width, 1),
        );
    }
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, app: &Observatory, palette: Palette) {
    let contextual = match app.view {
        ObservatoryView::Atlas => "hjkl move · +/- zoom · L layer",
        ObservatoryView::Chronicle => {
            if app.show_debug {
                "↑↓ records · Enter inspect · f story stream"
            } else {
                "↑↓ records · Enter inspect · f complete stream"
            }
        }
        ObservatoryView::Relations => "↑↓ edge · Enter follow · Esc back",
        ObservatoryView::Catalog => "←→ category · ↑↓ record · Enter inspect",
    };
    let status = app.status.as_deref().unwrap_or("");
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                "1–4 workspace · Tab pane · / search · [ ] event · Space play · ? help · q quit",
                Style::default().fg(palette.dim),
            ),
            Span::styled(format!("   {contextual}"), Style::default().fg(palette.ink)),
            Span::styled(
                if status.is_empty() {
                    String::new()
                } else {
                    format!("   {status}")
                },
                Style::default().fg(palette.rust),
            ),
        ])),
        area,
    );
}

fn render_help(frame: &mut Frame<'_>, area: Rect, app: &Observatory, palette: Palette) {
    let popup = centered(
        area,
        72.min(area.width.saturating_sub(4)),
        22.min(area.height - 2),
    );
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(vec![
            Line::styled(
                "THE ARCHIVE IS ONE CONNECTED TRAIL",
                Style::default()
                    .fg(palette.copper)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from(""),
            Line::from("1–4           switch workspace"),
            Line::from("Tab / ⇧Tab    move primary / detail / timeline pane focus"),
            Line::from("arrows, hjkl  move or scroll the active pane"),
            Line::from("Enter         follow the selected typed relation"),
            Line::from("Esc / b       close overlay / return through focus trail"),
            Line::from("/             search every named record"),
            Line::from("← →           year step; [ ] previous/next recorded event"),
            Line::from("Space         play/pause; macro jumps events, local advances yearly"),
            Line::from("L / + / -     atlas layer / zoom in / zoom out"),
            Line::from("f             reveal or hide clock/debug events"),
            Line::from("mouse         click records/map/timeline; wheel scrolls or zooms"),
            Line::from(""),
            Line::styled(
                "Copper = focus/milestone · teal = living/local · rust = warning",
                Style::default().fg(palette.teal),
            ),
            Line::from("Claims are always marked as interpretations, never authoritative facts."),
            Line::from(""),
            Line::styled(
                format!("Press ? or Esc to close · current pane {:?}", app.pane),
                Style::default().fg(palette.dim),
            ),
        ])
        .wrap(Wrap { trim: false })
        .block(archive_block("Field Guide", palette)),
        popup,
    );
}

fn render_search(frame: &mut Frame<'_>, area: Rect, app: &mut Observatory, palette: Palette) {
    let height = 16.min(area.height.saturating_sub(2));
    let width = 78.min(area.width.saturating_sub(4));
    let popup = centered(area, width, height);
    frame.render_widget(Clear, popup);
    let block = archive_block("Search the complete archive", palette);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    let query_area = Rect::new(inner.x, inner.y, inner.width, 2.min(inner.height));
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled("/", Style::default().fg(palette.copper)),
                Span::styled(app.query.clone(), Style::default().fg(palette.ink)),
                Span::styled("█", Style::default().fg(palette.copper)),
            ]),
            Line::styled(
                format!(
                    "{} result(s) · type a name, kind, or kind:id",
                    app.search_results.len()
                ),
                Style::default().fg(palette.dim),
            ),
        ]),
        query_area,
    );
    let results_area = Rect::new(
        inner.x,
        inner.y.saturating_add(query_area.height),
        inner.width,
        inner.height.saturating_sub(query_area.height),
    );
    let visible = usize::from(results_area.height);
    let start = app
        .selection
        .saturating_sub(visible / 2)
        .min(app.search_results.len().saturating_sub(visible));
    let mut lines = Vec::new();
    for (visible_index, (index, entity)) in app
        .search_results
        .iter()
        .enumerate()
        .skip(start)
        .take(visible)
        .enumerate()
    {
        lines.push(Line::styled(
            format!(
                "{} {:<13} #{:<5} {}",
                if index == app.selection { "▶" } else { " " },
                entity.kind(),
                entity.raw(),
                app.label(*entity)
            ),
            if index == app.selection {
                Style::default().fg(palette.background).bg(palette.copper)
            } else {
                Style::default().fg(palette.ink)
            },
        ));
        app.hits.search_rows.push((
            index,
            Rect::new(
                results_area.x,
                results_area
                    .y
                    .saturating_add(u16::try_from(visible_index).unwrap_or(0)),
                results_area.width,
                1,
            ),
        ));
    }
    frame.render_widget(Paragraph::new(lines), results_area);
}

fn archive_block<'a>(title: &'a str, palette: Palette) -> Block<'a> {
    Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(palette.border))
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(palette.ink),
        ))
        .style(Style::default().bg(palette.background).fg(palette.ink))
}

fn project_coordinate(
    x: u16,
    y: u16,
    area: Rect,
    start_x: u16,
    start_y: u16,
    visible_width: u16,
    visible_height: u16,
) -> Option<(u16, u16)> {
    if x < start_x
        || y < start_y
        || x >= start_x.saturating_add(visible_width)
        || y >= start_y.saturating_add(visible_height)
    {
        return None;
    }
    let screen_x = area.x.saturating_add(
        u16::try_from(
            u32::from(x - start_x).saturating_mul(u32::from(area.width))
                / u32::from(visible_width.max(1)),
        )
        .unwrap_or(0)
        .min(area.width.saturating_sub(1)),
    );
    let screen_y = area.y.saturating_add(
        u16::try_from(
            u32::from(y - start_y).saturating_mul(u32::from(area.height))
                / u32::from(visible_height.max(1)),
        )
        .unwrap_or(0)
        .min(area.height.saturating_sub(1)),
    );
    Some((screen_x, screen_y))
}

fn draw_line(buffer: &mut Buffer, start: (u16, u16), end: (u16, u16), symbol: &str, style: Style) {
    let (mut x, mut y) = (i32::from(start.0), i32::from(start.1));
    let (end_x, end_y) = (i32::from(end.0), i32::from(end.1));
    let dx = (end_x - x).abs();
    let sx = if x < end_x { 1 } else { -1 };
    let dy = -(end_y - y).abs();
    let sy = if y < end_y { 1 } else { -1 };
    let mut error = dx + dy;
    loop {
        if let (Ok(column), Ok(row)) = (u16::try_from(x), u16::try_from(y)) {
            buffer[(column, row)].set_symbol(symbol).set_style(style);
        }
        if x == end_x && y == end_y {
            break;
        }
        let twice = 2 * error;
        if twice >= dy {
            error += dy;
            x += sx;
        }
        if twice <= dx {
            error += dx;
            y += sy;
        }
    }
}

fn timeline_x(year: u32, area: Rect, maximum: u32) -> u16 {
    area.x.saturating_add(
        u16::try_from(
            year.saturating_mul(u32::from(area.width.saturating_sub(1))) / maximum.max(1),
        )
        .unwrap_or(0)
        .min(area.width.saturating_sub(1)),
    )
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

fn short_catalog_label(kind: CatalogKind) -> &'static str {
    match kind {
        CatalogKind::Places => "places",
        CatalogKind::Peoples => "peoples",
        CatalogKind::Beliefs => "beliefs",
        CatalogKind::Institutions => "instit.",
        CatalogKind::Households => "homes",
        CatalogKind::People => "people",
        CatalogKind::Items => "items",
        CatalogKind::Events => "events",
        CatalogKind::Claims => "claims",
    }
}

fn title_case(value: &str) -> String {
    let mut characters = value.chars();
    characters.next().map_or_else(String::new, |first| {
        first.to_uppercase().collect::<String>() + characters.as_str()
    })
}
