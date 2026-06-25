use std::borrow::Cow;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph},
};
use ratatui_image::{Resize, StatefulImage};

use crate::app::{App, LoginField, Screen};
use crate::download::DownloadStatus;
use crate::images::ImageManager;
use jellyfin_client::MediaItem;

const SIDEBAR_RATIO: f64 = 1.0 / 3.0;
const MIN_WIDTH_FOR_SIDEBAR: u16 = 80;

fn split_sidebar(area: Rect) -> (Option<Rect>, Rect) {
    if area.width < MIN_WIDTH_FOR_SIDEBAR {
        return (None, area);
    }
    let w = (area.width as f64 * SIDEBAR_RATIO) as u16;
    let chunks = Layout::horizontal([Constraint::Length(w), Constraint::Fill(1)]).split(area);
    (Some(chunks[0]), chunks[1])
}

pub fn render(frame: &mut Frame, app: &mut App, images: &mut ImageManager) {
    match app.screen {
        Screen::Login => render_login(frame, app),
        Screen::Home => render_home(frame, app, images),
        Screen::Library => render_library(frame, app, images),
        Screen::Search => render_search(frame, app, images),
    }

    if app.show_downloads {
        render_downloads_popup(frame, app);
    }
}

fn render_login(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let outer_block = Block::default()
        .title("Jellytui - Login")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(outer_block, area);

    let inner_area = centered_rect(60, 50, area);

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(2),
        Constraint::Length(2),
    ])
    .split(inner_area);

    let server_style = field_style(app.login_field == LoginField::ServerUrl);
    let server_block = Block::default()
        .title("Server URL")
        .borders(Borders::ALL)
        .border_style(server_style);
    let server_input = Paragraph::new(app.server_url_input.as_str()).block(server_block);
    frame.render_widget(server_input, chunks[0]);

    let username_style = field_style(app.login_field == LoginField::Username);
    let username_block = Block::default()
        .title("Username")
        .borders(Borders::ALL)
        .border_style(username_style);
    let username_input = Paragraph::new(app.username_input.as_str()).block(username_block);
    frame.render_widget(username_input, chunks[1]);

    let password_style = field_style(app.login_field == LoginField::Password);
    let password_block = Block::default()
        .title("Password")
        .borders(Borders::ALL)
        .border_style(password_style);
    let masked_password = "*".repeat(app.password_input.len());
    let password_input = Paragraph::new(masked_password).block(password_block);
    frame.render_widget(password_input, chunks[2]);

    let help_text = Line::from(vec![
        Span::raw("Tab: next field | Shift+Tab: prev | Enter: login | "),
        Span::styled("Esc: quit", Style::default().fg(Color::Red)),
    ]);
    let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[3]);

    if let Some(ref error) = app.login_error {
        let error_text = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
        frame.render_widget(error_text, chunks[4]);
    }

    if app.login_loading {
        let loading = Paragraph::new("Authenticating...").style(Style::default().fg(Color::Yellow));
        frame.render_widget(loading, chunks[4]);
    }

    set_cursor_for_input(frame, app, chunks);
}

fn frame_layout(
    frame_area: Rect,
    now_playing: bool,
    search_bar: bool,
) -> (Option<Rect>, Rect, Rect, Option<Rect>) {
    let mut constraints = Vec::with_capacity(4);
    if search_bar {
        constraints.push(Constraint::Length(3));
    }
    constraints.push(Constraint::Min(3));
    constraints.push(Constraint::Length(1));
    if now_playing {
        constraints.push(Constraint::Length(3));
    } else {
        constraints.push(Constraint::Length(0));
    }
    let chunks = Layout::vertical(constraints).split(frame_area);

    let mut idx = 0;
    let search_rect = if search_bar {
        let r = chunks[idx];
        idx += 1;
        Some(r)
    } else {
        None
    };
    let content = chunks[idx];
    let help = chunks[idx + 1];
    let footer = if now_playing { Some(chunks[idx + 2]) } else { None };
    (search_rect, content, help, footer)
}

fn selected_item(app: &App) -> Option<&MediaItem> {
    match app.screen {
        Screen::Home => app
            .home_sections
            .get(app.home_row)
            .and_then(|s| s.items.get(app.home_col)),
        Screen::Library => app.items.get(app.selected_index),
        Screen::Search => app.search_results.get(app.search_selected),
        Screen::Login => None,
    }
}

fn render_sidebar(frame: &mut Frame, app: &App, images: &mut ImageManager, area: Rect) {
    // Clear sidebar first to wipe stale content from previous selections/screens.
    // This must be the FIRST render call so that borders and all widgets draw on top.
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(item) = selected_item(app) else {
        return;
    };

    let bold = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let dim = Style::default().fg(Color::DarkGray);
    let text = Style::default().fg(Color::Gray);

    let mut lines: Vec<(Cow<'_, str>, Style)> = Vec::new();

    // Name
    if item.r#type == "Episode" {
        let ep_label = match (&item.series_name, &item.parent_index_number, &item.index_number) {
            (Some(series), Some(s), Some(e)) => Cow::Owned(format!("{} S{:02}E{:02}", series, s, e)),
            (Some(series), _, _) => Cow::Owned(series.clone()),
            _ => Cow::Borrowed(item.name.as_str()),
        };
        lines.push((ep_label, bold));
        if item.series_name.is_some() {
            lines.push((Cow::Borrowed(item.name.as_str()), text));
        }
    } else {
        lines.push((Cow::Borrowed(item.name.as_str()), bold));
    }

    // Original title
    if let Some(ref ot) = item.original_title
        && !ot.is_empty() && ot != &item.name {
            lines.push((Cow::Borrowed(ot.as_str()), dim));
        }

    // Year
    if let Some(y) = item.production_year {
        lines.push((Cow::Owned(y.to_string()), dim));
    }

    // Rating
    let mut rating_line = String::new();
    if let Some(ref r) = item.official_rating
        && !r.is_empty() {
            rating_line.push_str(r);
        }
    if let Some(cr) = item.community_rating {
        if !rating_line.is_empty() {
            rating_line.push_str("  ");
        }
        rating_line.push_str(&format!("{:.1}", cr));
    }
    if !rating_line.is_empty() {
        lines.push((Cow::Owned(rating_line), text));
    }

    // Blank line
    lines.push((Cow::Borrowed(""), dim));

    // Overview
    if let Some(ref ov) = item.overview
        && !ov.is_empty() {
            lines.push((Cow::Borrowed(ov.as_str()), text));
        }

    // Genres
    if !item.genres.is_empty() {
        lines.push((Cow::Owned(format!("Genres: {}", item.genres.join(", "))), dim));
    }

    // Studios
    if !item.studios.is_empty() {
        lines.push((
            Cow::Owned(format!(
                "Studio: {}",
                item.studios
                    .iter()
                    .map(|st| st.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            dim,
        ));
    }



    // Split inner into poster zone (top third) and text zone (bottom two-thirds)
    let chunks = Layout::vertical([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).split(inner);
    let poster_zone = chunks[0];
    let text_zone = chunks[1];

    if let Some(ref client) = app.client {
        let url = client.get_primary_image_url(&item.id, 600);
        images.ensure(&item.id, url);
    }
    // Clear poster zone first to wipe stale content.
    frame.render_widget(Clear, poster_zone);

    if let Some(protocol) = images.get_protocol(&item.id) {
        let fit = Resize::Fit(None);
        let rendered = protocol.size_for(fit.clone(), poster_zone);
        let centered = Rect {
            x: poster_zone.x + (poster_zone.width.saturating_sub(rendered.width)) / 2,
            y: poster_zone.y + (poster_zone.height.saturating_sub(rendered.height)) / 2,
            width: rendered.width.min(poster_zone.width),
            height: rendered.height.min(poster_zone.height),
        };
        frame.render_stateful_widget(
            StatefulImage::default().resize(fit),
            centered,
            protocol,
        );
    } else if !images.is_failed(&item.id) {
        let placeholder_text = Paragraph::new("Fetching poster...").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(placeholder_text, poster_zone);
    } else {
        let placeholder_text = Paragraph::new("No image available").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(placeholder_text, poster_zone);
    }

    // Render detail text in the bottom zone only
    let max_w = text_zone.width as usize;
    let mut y = text_zone.y;
    for (txt, style) in &lines {
        if y >= text_zone.y + text_zone.height {
            break;
        }
        let wrapped = if txt.len() > max_w && (*style == text || *style == dim) {
            wrap_text(txt, max_w)
        } else {
            vec![txt.to_string()]
        };
        for wline in &wrapped {
            if y >= text_zone.y + text_zone.height {
                break;
            }
            let rect = Rect {
                x: text_zone.x,
                y,
                width: text_zone.width,
                height: 1,
            };
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(wline, *style))),
                rect,
            );
            y += 1;
        }
    }
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for word in text.split(' ') {
        let last = result.last_mut();
        match last {
            Some(line) if line.len() + 1 + word.len() <= max_width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => {
                if word.len() > max_width {
                    let mut rem = word;
                    while rem.len() > max_width {
                        let split = rem
                            .char_indices()
                            .nth(max_width)
                            .map(|(i, _)| i)
                            .unwrap_or(rem.len());
                        result.push(rem[..split].to_string());
                        rem = &rem[split..];
                    }
                    if !rem.is_empty() {
                        result.push(rem.to_string());
                    }
                } else {
                    result.push(word.to_string());
                }
            }
        }
    }
    result
}

fn render_home(frame: &mut Frame, app: &mut App, images: &mut ImageManager) {
    let (sidebar, right) = split_sidebar(frame.area());
    let (_, content, help_area, footer_area) =
        frame_layout(right, app.now_playing.is_some(), false);

    if let Some(sb) = sidebar {
        render_sidebar(frame, app, images, sb);
    }

    let title = format!("Jellytui — {}", app.current_title());
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(content);
    frame.render_widget(block, content);

    if app.loading {
        let loading = Paragraph::new("Loading...").style(Style::default().fg(Color::Yellow));
        frame.render_widget(loading, inner);
    } else if let Some(ref error) = app.error_message {
        let error_text =
            Paragraph::new(error.as_str()).style(Style::default().fg(Color::Red));
        frame.render_widget(error_text, inner);
    } else if app.home_sections.is_empty() {
        let empty =
            Paragraph::new("No content yet. Press 'r' to refresh.").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, inner);
    } else {
        render_home_sections(frame, app, images, inner);
    }

    render_help(frame, help_area, home_help_text());
    if let Some(footer) = footer_area
        && app.now_playing.is_some() && footer.height > 0 {
            render_now_playing_footer(frame, app, footer);
        }
}

fn render_home_sections(
    frame: &mut Frame,
    app: &mut App,
    _images: &mut ImageManager,
    area: Rect,
) {
    let total = app.home_sections.len();
    if total == 0 {
        return;
    }

    let heights: Vec<u16> = app
        .home_sections
        .iter()
        .map(|s| 2 + s.items.len().min(100) as u16)
        .collect();

    let mut sel = app.home_row;
    if sel >= heights.len() {
        sel = heights.len() - 1;
        app.home_row = sel;
    }
    let mut scroll = sel;
    let mut used = heights[sel];
    for i in (0..sel).rev() {
        if used + heights[i] > area.height {
            break;
        }
        scroll = i;
        used += heights[i];
    }

    let mut y = area.y;
    for (idx, h) in heights.iter().copied().enumerate().skip(scroll) {
        if y >= area.y + area.height {
            break;
        }
        if y + h > area.y + area.height {
            break;
        }

        let section = &app.home_sections[idx];
        let item_count = section.items.len();
        let area = Rect {
            x: area.x,
            y,
            width: area.width,
            height: h,
        };
        let is_sel = idx == sel;
        if is_sel && app.home_col >= item_count && item_count > 0 {
            app.home_col = item_count - 1;
        }
        let border_style = if is_sel {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(section.title.clone())
            .title_alignment(Alignment::Center);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        for i in 0..item_count {
            let item = &section.items[i];
            let item_rect = Rect {
                x: inner.x,
                y: inner.y + i as u16,
                width: inner.width,
                height: 1,
            };
            let is_item_sel = is_sel && i == app.home_col;
            let item_style = if is_item_sel {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            let text = format!("{} {}", type_icon(item), poster_label(item));
            let line = Line::from(Span::styled(text, item_style));
            frame.render_widget(Paragraph::new(line), item_rect);
        }
        y += h;
    }
}

fn poster_label(item: &MediaItem) -> String {
    if item.r#type == "Episode" {
        match (&item.series_name, &item.parent_index_number, &item.index_number) {
            (Some(series), Some(s), Some(e)) => format!("{} S{:02}E{:02}", series, s, e),
            (Some(series), _, _) => series.clone(),
            _ => item.name.clone(),
        }
    } else {
        item.name.clone()
    }
}

fn type_icon(item: &MediaItem) -> &'static str {
    if item.is_folder {
        ">"
    } else {
        "-"
    }
}

fn render_item_list(
    frame: &mut Frame,
    items: &[MediaItem],
    selected: usize,
    area: Rect,
) {
    let max_visible = area.height as usize;
    let show = items.len().min(max_visible);
    let scroll = if selected >= max_visible {
        selected + 1 - max_visible
    } else {
        0
    };

    for i in 0..show {
        let idx = scroll + i;
        if idx >= items.len() {
            break;
        }
        let item = &items[idx];
        let rect = Rect {
            x: area.x,
            y: area.y + i as u16,
            width: area.width,
            height: 1,
        };
        let is_sel = idx == selected;
        let style = if is_sel {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let text = format!(" {}  {}", type_icon(item), poster_label(item));
        frame.render_widget(Paragraph::new(Line::from(Span::styled(text, style))), rect);
    }
}

fn render_library(frame: &mut Frame, app: &mut App, images: &mut ImageManager) {
    let (sidebar, right) = split_sidebar(frame.area());
    let (_, content, help_area, footer_area) =
        frame_layout(right, app.now_playing.is_some(), false);

    if let Some(sb) = sidebar {
        render_sidebar(frame, app, images, sb);
    }

    let title = format!("Jellytui — {}", app.current_title());
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(content);
    frame.render_widget(block, content);

    if app.loading {
        frame.render_widget(
            Paragraph::new("Loading...").style(Style::default().fg(Color::Yellow)),
            inner,
        );
    } else if let Some(ref error) = app.error_message {
        frame.render_widget(
            Paragraph::new(error.as_str()).style(Style::default().fg(Color::Red)),
            inner,
        );
    } else if app.items.is_empty() {
        frame.render_widget(
            Paragraph::new("Empty.").style(Style::default().fg(Color::DarkGray)),
            inner,
        );
    } else {
        render_item_list(frame, &app.items, app.selected_index, inner);
    }

    render_help(frame, help_area, library_help_text());
    if let Some(footer) = footer_area
        && app.now_playing.is_some() && footer.height > 0 {
            render_now_playing_footer(frame, app, footer);
        }
}

fn render_search(frame: &mut Frame, app: &mut App, images: &mut ImageManager) {
    let (sidebar, right) = split_sidebar(frame.area());
    let (search_rect, content, help_area, footer_area) =
        frame_layout(right, app.now_playing.is_some(), true);

    if let Some(sb) = sidebar {
        render_sidebar(frame, app, images, sb);
    }

    if let Some(rect) = search_rect {
        let search_block = Block::default()
            .title("Search")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let search_input = Paragraph::new(app.search_query.as_str()).block(search_block);
        frame.render_widget(search_input, rect);
        frame.set_cursor_position((rect.x + app.search_query.len() as u16 + 1, rect.y + 1));
    }

    let results_block = Block::default()
        .title(format!("Results ({})", app.search_results.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = results_block.inner(content);
    frame.render_widget(results_block, content);

    if app.loading {
        frame.render_widget(
            Paragraph::new("Searching...").style(Style::default().fg(Color::Yellow)),
            inner,
        );
    } else if let Some(ref error) = app.error_message {
        frame.render_widget(
            Paragraph::new(error.as_str()).style(Style::default().fg(Color::Red)),
            inner,
        );
    } else if app.search_results.is_empty() {
        let msg = if app.search_query.is_empty() {
            "Type to search..."
        } else {
            "No results found"
        };
        frame.render_widget(
            Paragraph::new(msg).style(Style::default().fg(Color::DarkGray)),
            inner,
        );
    } else {
        render_item_list(frame, &app.search_results, app.search_selected, inner);
    }

    render_help(frame, help_area, search_help_text());
    if let Some(footer) = footer_area
        && app.now_playing.is_some() && footer.height > 0 {
            render_now_playing_footer(frame, app, footer);
        }
}

fn render_help(frame: &mut Frame, area: Rect, text: &str) {
    if area.height == 0 {
        return;
    }
    let help = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, area);
}

fn home_help_text() -> &'static str {
    "h/j/k/l: navigate | Enter: open | /: search | d: downloads | r: refresh | q: quit"
}

fn library_help_text() -> &'static str {
    "h/j/k/l: navigate | Enter: open/play | D: download | d: downloads | Esc: back | q: quit"
}

fn search_help_text() -> &'static str {
    "Type to search | h/j/k/l or arrows: navigate | Enter: open/play | Esc: close"
}

fn render_now_playing_footer(frame: &mut Frame, app: &App, area: Rect) {
    let Some(ref playing) = app.now_playing else {
        return;
    };

    let duration = app.playback_duration_secs;
    let percent = if duration > 0.0 {
        (app.playback_position_secs / duration * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let status = if app.playback_paused { "Paused" } else { "Playing" };
    let label = if duration > 0.0 {
        format!(
            "{} / {}  •  {}",
            format_duration(app.playback_position_secs),
            format_duration(duration),
            status
        )
    } else {
        format!(
            "{}  •  {}",
            format_duration(app.playback_position_secs),
            status
        )
    };

    let display = poster_label(&playing.item);
    let title = format!("Now Playing — {}", display);
    let gauge_color = if app.playback_paused {
        Color::Yellow
    } else {
        Color::Green
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .gauge_style(Style::default().fg(gauge_color).bg(Color::DarkGray))
        .percent(percent.round() as u16)
        .label(Span::styled(label, Style::default().fg(Color::White)));

    frame.render_widget(gauge, area);
}

fn format_duration(seconds: f64) -> String {
    let total_seconds = seconds.max(0.0).floor() as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

fn field_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

fn set_cursor_for_input(frame: &mut Frame, app: &App, chunks: std::rc::Rc<[Rect]>) {
    let (chunk_idx, input_len) = match app.login_field {
        LoginField::ServerUrl => (0, app.server_url_input.len()),
        LoginField::Username => (1, app.username_input.len()),
        LoginField::Password => (2, app.password_input.len()),
    };

    let chunk = chunks[chunk_idx];
    frame.set_cursor_position((chunk.x + input_len as u16 + 1, chunk.y + 1));
}

fn render_downloads_popup(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let content_area = match app.screen {
        Screen::Library | Screen::Search => {
            let (_, right) = split_sidebar(area);
            right
        }
        _ => area,
    };
    let popup_area = centered_rect(70, 60, content_area);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(format!("Downloads ({})", app.downloads.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    if app.downloads.is_empty() {
        let empty = Paragraph::new("No downloads. Press D on a media item to download.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(empty, popup_area);
        return;
    }

    let inner_area = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::vertical(
        app.downloads
            .iter()
            .map(|_| Constraint::Length(3))
            .collect::<Vec<_>>(),
    )
    .split(inner_area);

    for (i, task) in app.downloads.iter().enumerate() {
        if i >= chunks.len() {
            break;
        }

        let (status_text, status_color) = match task.status {
            DownloadStatus::Pending => ("Pending", Color::DarkGray),
            DownloadStatus::Downloading => ("Downloading", Color::Yellow),
            DownloadStatus::Completed => ("Completed", Color::Green),
            DownloadStatus::Failed => ("Failed", Color::Red),
        };

        let title = format!("{} [{}]", task.item_name, status_text);

        let gauge = Gauge::default()
            .block(Block::default().title(title).borders(Borders::ALL))
            .gauge_style(Style::default().fg(status_color))
            .percent(task.progress as u16)
            .label(format!("{}%", task.progress));

        frame.render_widget(gauge, chunks[i]);
    }
}

