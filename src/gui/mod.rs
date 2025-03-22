use std::error::Error;
use std::io;
use std::time::Duration;

use crate::{AppState, FocusedSection};
use crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Backend, CrosstermBackend};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::{Frame, Terminal};

pub fn render_ui(f: &mut Frame, state: &AppState) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints(
            [
                ratatui::layout::Constraint::Percentage(state.node_pane_width), // Dynamic width for the node list
                ratatui::layout::Constraint::Percentage(100 - state.node_pane_width), // Remaining width for the fields
            ]
            .as_ref(),
        )
        .split(f.area());

    #[cfg(feature = "debug")]
    render_debug_overlays(f, &chunks);

    // Render the node list
    let visible_nodes = state
        .flattened
        .iter()
        .enumerate()
        .skip(state.node_scroll_offset)
        .take(chunks[0].height as usize - 2) // Adjust for borders
        .map(|(i, &(depth, ptr))| {
            let node = unsafe { &*ptr };
            let indent = "  ".repeat(depth);
            let icon = if node.children.is_empty() {
                "Ø"
            } else if node.is_expanded {
                "▼"
            } else {
                "▶"
            };
            let label = format!("{}{} {}", indent, icon, node.name);
            let style = if i == state.selected {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            ListItem::new(label).style(style)
        })
        .collect::<Vec<_>>();

    let list =
        List::new(visible_nodes).block(Block::default().borders(Borders::ALL).title("Nodes"));
    f.render_widget(list, chunks[0]);

    // Render the scrollbar for the node list
    if state.flattened.len() > chunks[0].height as usize - 2 {
        let scrollbar_height = (chunks[0].height as f32 - 2.0) * (chunks[0].height as f32 - 2.0)
            / state.flattened.len() as f32;
        let scrollbar_offset = (chunks[0].height as f32 - 2.0) * state.node_scroll_offset as f32
            / state.flattened.len() as f32;

        let scrollbar_rect = Rect::new(
            chunks[0].x + chunks[0].width - 1,
            chunks[0].y + 1 + scrollbar_offset as u16,
            1,
            scrollbar_height as u16,
        );

        f.render_widget(
            Block::default().style(Style::default().bg(ratatui::style::Color::Gray)),
            scrollbar_rect,
        );
    }

    // Render the fields of the selected node
    let selected_node = unsafe { &*state.flattened[state.selected].1 };
    let visible_fields = selected_node
        .fields
        .iter()
        .skip(state.field_scroll_offset)
        .take(chunks[1].height as usize - 2) // Adjust for borders
        .map(|(key, value)| {
            let label = format!("{}: {}", key, value);
            ListItem::new(label)
        })
        .collect::<Vec<_>>();

    let fields_list =
        List::new(visible_fields).block(Block::default().borders(Borders::ALL).title("Fields"));
    f.render_widget(fields_list, chunks[1]);

    // Render the scrollbar for the fields list
    if selected_node.fields.len() > chunks[1].height as usize - 2 {
        let scrollbar_height = (chunks[1].height as f32 - 2.0) * (chunks[1].height as f32 - 2.0)
            / selected_node.fields.len() as f32;
        let scrollbar_offset = (chunks[1].height as f32 - 2.0) * state.field_scroll_offset as f32
            / selected_node.fields.len() as f32;

        let scrollbar_rect = Rect::new(
            chunks[1].x + chunks[1].width - 1,
            chunks[1].y + 1 + scrollbar_offset as u16,
            1,
            scrollbar_height as u16,
        );

        f.render_widget(
            Block::default().style(Style::default().bg(ratatui::style::Color::Gray)),
            scrollbar_rect,
        );
    }
}

#[cfg(feature = "debug")]
fn render_debug_overlays(f: &mut Frame<'_>, chunks: &std::rc::Rc<[Rect]>) {
    // Correctly calculate the boundary based on the layout
    let boundary_x = *chunks[0].x + *chunks[0].width;
    // The right edge of the "Nodes" pane

    // Debug overlay: Highlight scrollable areas and resize area
    // Highlight the "Nodes" pane scrollable area in green
    let nodes_debug_rect = Rect::new(
        *chunks[0].x,
        *chunks[0].y,
        *chunks[0].width,
        *chunks[0].height,
    );
    f.render_widget(
        Block::default().style(Style::default().bg(ratatui::style::Color::Green)),
        nodes_debug_rect,
    );

    // Highlight the "Fields" pane scrollable area in green
    let fields_debug_rect = Rect::new(
        *chunks[1].x,
        *chunks[1].y,
        *chunks[1].width,
        *chunks[1].height,
    );
    f.render_widget(
        Block::default().style(Style::default().bg(ratatui::style::Color::Green)),
        fields_debug_rect,
    );

    // Highlight the resize area in red
    let resize_debug_rect = Rect::new(boundary_x - 1, f.area().y, 2, f.area().height);
    f.render_widget(
        Block::default().style(Style::default().bg(ratatui::style::Color::Red)),
        resize_debug_rect,
    );
}

pub fn file_selection_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    files: &[String],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut selected = 0;

    loop {
        let mut list_state = ListState::default();
        list_state.select(Some(selected));
        terminal.draw(|f| {
            let size = f.area();
            let items: Vec<ListItem> = files
                .iter()
                .map(|file| ListItem::new(file.clone()))
                .collect();
            let list = List::new(items)
                .block(Block::default().title("Select a File").borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, size, &mut list_state);
        })?;

        // Use `poll` to wait for a key event with a timeout
        if crossterm::event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // Only handle key presses (ignore repeats)
                if key.kind == crossterm::event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Up => {
                            if selected > 0 {
                                selected -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if selected < files.len() - 1 {
                                selected += 1;
                            }
                        }
                        KeyCode::Enter => {
                            return Ok(files[selected].clone());
                        }
                        KeyCode::Esc => {
                            return Err("File selection canceled".into());
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

pub fn terminal_draw_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut state: AppState,
) -> Result<(), Box<dyn Error>> {
    Ok(loop {
        terminal.draw(|f| render_ui(f, &state))?;

        if event::poll(Duration::from_millis(10000))? {
            let terminal_size = terminal.size()?; // Get the terminal size

            let chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints(
                    [
                        ratatui::layout::Constraint::Percentage(state.node_pane_width), // Dynamic width for "Nodes"
                        ratatui::layout::Constraint::Percentage(100 - state.node_pane_width), // Remaining width for "Fields"
                    ]
                    .as_ref(),
                )
                .split(Rect::new(0, 0, terminal_size.width, terminal_size.height));

            match event::read()? {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Tab => {
                            // Toggle focus between "Nodes" and "Fields"
                            state.focused_section = match state.focused_section {
                                FocusedSection::Nodes => FocusedSection::Fields,
                                FocusedSection::Fields => FocusedSection::Nodes,
                            };
                        }
                        KeyCode::PageDown => {
                            let page_size = match state.focused_section {
                                FocusedSection::Nodes => chunks[0].height as usize - 2,
                                FocusedSection::Fields => chunks[1].height as usize - 2,
                            };

                            match state.focused_section {
                                FocusedSection::Nodes => {
                                    if state.node_scroll_offset + page_size < state.flattened.len()
                                    {
                                        state.node_scroll_offset += page_size;
                                    } else {
                                        state.node_scroll_offset =
                                            state.flattened.len().saturating_sub(page_size);
                                    }
                                }
                                FocusedSection::Fields => {
                                    if state.field_scroll_offset + page_size < state.fields.len() {
                                        state.field_scroll_offset += page_size;
                                    } else {
                                        state.field_scroll_offset =
                                            state.fields.len().saturating_sub(page_size);
                                    }
                                }
                            }
                        }
                        KeyCode::PageUp => {
                            let page_size = match state.focused_section {
                                FocusedSection::Nodes => chunks[0].height as usize - 2,
                                FocusedSection::Fields => chunks[1].height as usize - 2,
                            };

                            match state.focused_section {
                                FocusedSection::Nodes => {
                                    if state.node_scroll_offset > 0 {
                                        state.node_scroll_offset =
                                            state.node_scroll_offset.saturating_sub(page_size);
                                    }
                                }
                                FocusedSection::Fields => {
                                    if state.field_scroll_offset > 0 {
                                        state.field_scroll_offset =
                                            state.field_scroll_offset.saturating_sub(page_size);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Event::Mouse(mouse_event) => {
                    match mouse_event.kind {
                        crossterm::event::MouseEventKind::Down(
                            crossterm::event::MouseButton::Left,
                        ) => {
                            // Detect resizing near the boundary
                            let boundary_x =
                                (terminal_size.width as f32 * state.node_pane_width as f32 / 100.0)
                                    .round() as u16;
                            if (mouse_event.column as i16 - boundary_x as i16).abs() <= 1 {
                                state.resizing = true;
                            } else {
                                // Check if the click is in the "Nodes" or "Fields" pane for dragging
                                if mouse_event.column as u16 >= chunks[0].x
                                    && (mouse_event.column as u16) < chunks[0].x + chunks[0].width
                                    && mouse_event.row >= chunks[0].y
                                    && mouse_event.row < chunks[0].y + chunks[0].height
                                {
                                    state.dragging = Some(FocusedSection::Nodes);
                                } else if mouse_event.column as u16 >= chunks[1].x
                                    && (mouse_event.column as u16) < chunks[1].x + chunks[1].width
                                    && mouse_event.row >= chunks[1].y
                                    && mouse_event.row < chunks[1].y + chunks[1].height
                                {
                                    state.dragging = Some(FocusedSection::Fields);
                                }
                            }
                        }
                        crossterm::event::MouseEventKind::Drag(
                            crossterm::event::MouseButton::Left,
                        ) => {
                            if state.resizing {
                                // Adjust the width of the "Nodes" pane during resizing
                                let new_width = (mouse_event.column as f32
                                    / terminal_size.width as f32
                                    * 100.0)
                                    .round() as u16;
                                state.node_pane_width = new_width.clamp(20, 80); // Clamp between 20% and 80%
                            } else if let Some(dragging_section) = &state.dragging {
                                // Handle dragging (scrolling)
                                match dragging_section {
                                    FocusedSection::Nodes => {
                                        let relative_position =
                                            mouse_event.row.saturating_sub(chunks[0].y);
                                        let max_scroll_offset = state
                                            .flattened
                                            .len()
                                            .saturating_sub(chunks[0].height as usize - 2);

                                        state.node_scroll_offset = ((relative_position as f32
                                            / chunks[0].height as f32)
                                            * max_scroll_offset as f32)
                                            .round()
                                            as usize;
                                    }
                                    FocusedSection::Fields => {
                                        let relative_position =
                                            mouse_event.row.saturating_sub(chunks[1].y);
                                        let max_scroll_offset = state
                                            .fields
                                            .len()
                                            .saturating_sub(chunks[1].height as usize - 2);

                                        state.field_scroll_offset = ((relative_position as f32
                                            / chunks[1].height as f32)
                                            * max_scroll_offset as f32)
                                            .round()
                                            as usize;
                                    }
                                }
                            }
                        }
                        crossterm::event::MouseEventKind::Up(
                            crossterm::event::MouseButton::Left,
                        ) => {
                            // Stop resizing or dragging when the mouse button is released
                            state.resizing = false;
                            state.dragging = None;
                        }
                        crossterm::event::MouseEventKind::ScrollUp => {
                            // Handle scroll up
                            match state.focused_section {
                                FocusedSection::Nodes => {
                                    if state.node_scroll_offset > 0 {
                                        state.node_scroll_offset =
                                            state.node_scroll_offset.saturating_sub(1);
                                    }
                                }
                                FocusedSection::Fields => {
                                    if state.field_scroll_offset > 0 {
                                        state.field_scroll_offset =
                                            state.field_scroll_offset.saturating_sub(1);
                                    }
                                }
                            }
                        }
                        crossterm::event::MouseEventKind::ScrollDown => {
                            // Handle scroll down
                            match state.focused_section {
                                FocusedSection::Nodes => {
                                    if state.node_scroll_offset + 1 < state.flattened.len() {
                                        state.node_scroll_offset =
                                            state.node_scroll_offset.saturating_add(1);
                                    }
                                }
                                FocusedSection::Fields => {
                                    if state.field_scroll_offset + 1 < state.fields.len() {
                                        state.field_scroll_offset =
                                            state.field_scroll_offset.saturating_add(1);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    })
}
