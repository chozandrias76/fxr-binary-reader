use crate::{AppState, FocusedSection, file_entries, flatten_tree_mut};
use crossterm::event::{self, Event, KeyCode};
use fxr_binary_reader::fxr::{
    fxr_parser_with_sections::parse_fxr,
    view::view::{StructNode, build_reflection_tree},
};
use memmap2::Mmap;
use ratatui::{
    Frame, Terminal,
    layout::Rect,
    prelude::{Backend, CrosstermBackend},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};
use std::{env, fs::File, io, ops::Deref, path::PathBuf, time::Duration};
use zerocopy::IntoBytes;

pub fn render_ui(frame: &mut Frame, state: &AppState) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints(
            [
                ratatui::layout::Constraint::Percentage(state.node_pane_width), // Dynamic width for the node list
                ratatui::layout::Constraint::Percentage(100 - state.node_pane_width), // Remaining width for the fields
            ]
            .as_ref(),
        )
        .split(frame.area());

    #[cfg(feature = "debug")]
    render_debug_overlays(frame, &chunks);

    // Render the node list
    let visible_nodes = state
        .flattened
        .iter()
        .enumerate()
        .skip(state.node_scroll_offset)
        .take(chunks[0].height as usize - 2) // Adjust for borders
        .map(|(i, &(depth, ref ptr))| {
            let node = ptr;
            let indent = "  ".repeat(depth);
            let icon = if node.borrow().children.is_empty() {
                "Ø"
            } else if node.borrow().is_expanded {
                "▼"
            } else {
                "▶"
            };
            let label = format!("{}{} {}", indent, icon, node.borrow().name);
            let style = if i == state.selected_node {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            ListItem::new(label).style(style)
        })
        .collect::<Vec<_>>();

    let list =
        List::new(visible_nodes).block(Block::default().borders(Borders::ALL).title("Nodes"));
    frame.render_widget(list, chunks[0]);

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

        frame.render_widget(
            Block::default().style(Style::default().bg(ratatui::style::Color::Gray)),
            scrollbar_rect,
        );
    }

    // Render the fields of the selected node
    let selected_node = &*state.flattened[state.selected_node].1;
    let visible_fields = selected_node
        .borrow()
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
    frame.render_widget(fields_list, chunks[1]);

    // Render the scrollbar for the fields list
    if selected_node.borrow().fields.len() > chunks[1].height as usize - 2 {
        let scrollbar_height = (chunks[1].height as f32 - 2.0) * (chunks[1].height as f32 - 2.0)
            / selected_node.borrow().fields.len() as f32;
        let scrollbar_offset = (chunks[1].height as f32 - 2.0) * state.field_scroll_offset as f32
            / selected_node.borrow().fields.len() as f32;

        let scrollbar_rect = Rect::new(
            chunks[1].x + chunks[1].width - 1,
            chunks[1].y + 1 + scrollbar_offset as u16,
            1,
            scrollbar_height as u16,
        );

        frame.render_widget(
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
    files: (Vec<PathBuf>, Vec<String>), // Use PathBuf instead of String
    mut selected: usize,                // Add selected index as a parameter
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    loop {
        let mut list_state = ListState::default();
        list_state.select(Some(selected));
        terminal.draw(|f| {
            let size = f.area();
            let items: Vec<ListItem> = files
                .0
                .iter()
                .map(|file| ListItem::new(file.display().to_string())) // Display the path as a string
                .collect();
            let list = List::new(items)
                .block(
                    Block::default()
                        .title("Select a File")
                        .borders(Borders::ALL),
                )
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, size, &mut list_state);
        })?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Up => {
                            if !files.0.is_empty() {
                                selected = selected.saturating_sub(1);
                            }
                        }
                        KeyCode::Down => {
                            if !files.0.is_empty() {
                                selected = selected.saturating_add(1).min(files.0.len() - 1);
                            }
                        }
                        KeyCode::Right | KeyCode::Enter => {
                            let selected_route = files.0.get(selected).unwrap_or(&current_dir);
                            if selected_route.is_dir() {
                                let dir_entries = file_entries(selected_route)?; // Pass PathBuf directly
                                return file_selection_loop(terminal, dir_entries, 0);
                            }
                            if selected_route.is_file() {
                                return Ok(selected_route.clone());
                            }
                        }
                        KeyCode::Left => {
                            if let Some(parent) = files
                                .0
                                .get(selected)
                                .unwrap_or(&current_dir)
                                .parent()
                                .unwrap()
                                .parent()
                            {
                                let dir_entries = file_entries(&parent.to_path_buf())?;
                                let current_dir_name = files.0[selected].file_name();
                                let new_selected = dir_entries
                                    .0
                                    .iter()
                                    .position(|entry| entry.file_name() == current_dir_name)
                                    .unwrap_or(0);
                                // panic!("Parent directory selected: {:?}", parent);
                                return file_selection_loop(terminal, dir_entries, new_selected);
                            }
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
) -> Result<(), anyhow::Error> {
    // Resolve the selected file path relative to the current directory
    let current_dir = env::current_dir()?;
    let bin_path = current_dir.join(&state.selected_file); // Ensure the path is correctly joined

    // Open the selected file
    let file = File::open(&bin_path)
        .map_err(|e| anyhow::anyhow!("Failed to open file '{}': {}", bin_path.display(), e))?;
    let mmap = unsafe { Mmap::map(&file)? };
    let data = &mmap.as_bytes();

    // Parse the file
    let fxr = parse_fxr(data)
        .map_err(|e| anyhow::anyhow!("Failed to parse file '{}': {}", bin_path.display(), e))?;
    let mut header_view: StructNode = build_reflection_tree(&fxr.header.deref(), "Header");
    let mut flattened = vec![];
    let fields: Vec<(String, String)> = header_view.fields.clone();
    flatten_tree_mut(&mut header_view, 0, &mut flattened);
    state.flattened = flattened;
    state.fields = fields;

    loop {
        terminal
            .draw(|f: &mut Frame<'_>| render_ui(f, &state))
            .map_err(anyhow::Error::new)?;

        if event::poll(Duration::from_millis(10000)).map_err(anyhow::Error::new)? {
            let terminal_size = terminal.size().map_err(anyhow::Error::new)?; // Get the terminal size

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

            match event::read().map_err(anyhow::Error::new)? {
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
                                if mouse_event.column >= chunks[0].x
                                    && mouse_event.column < chunks[0].x + chunks[0].width
                                    && mouse_event.row >= chunks[0].y
                                    && mouse_event.row < chunks[0].y + chunks[0].height
                                {
                                    state.dragging = Some(FocusedSection::Nodes);
                                } else if mouse_event.column >= chunks[1].x
                                    && mouse_event.column < chunks[1].x + chunks[1].width
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
    }
    Ok(())
}
