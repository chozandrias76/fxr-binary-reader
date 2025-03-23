use crate::{AppState, file_entries};
use crossterm::event::{self, Event, KeyCode};
use fxr_binary_reader::fxr::{
    fxr_parser_with_sections::{ParsedFXR, parse_fxr},
    view::view::build_reflection_tree,
};
use memmap2::Mmap;
use ratatui::{
    Terminal,
    prelude::{Backend, CrosstermBackend},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};
use ratatui_tree_widget::{Tree, TreeItem};
use std::{
    any::type_name,
    env,
    fs::File,
    io,
    ops::Deref,
    path::PathBuf,
    time::{Duration, Instant},
};
use zerocopy::IntoBytes;

const HIGHLIGHT_STYLE: Style = Style {
    fg: Some(ratatui::style::Color::Yellow),
    bg: Some(ratatui::style::Color::Blue),
    underline_color: Some(ratatui::style::Color::Red),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::ITALIC,
};

pub fn build_root_tree(state: &AppState) -> TreeItem<'static> {
    // Example: Build the root tree dynamically
    let header_tree = build_reflection_tree(
        &state
            .fxr
            .as_ref()
            .expect("Could not get FXR")
            .header
            .deref(),
        "Header",
    );
    let section_tree = build_reflection_tree(
        &state
            .fxr
            .as_ref()
            .expect("Could not get FXR")
            .section1_tree
            .as_ref()
            .expect("Could not get Section 1 Tree")
            .section1
            .deref(),
        "Section1Container",
    );

    TreeItem::new("FXR File", vec![header_tree, section_tree])
}

pub fn file_selection_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    files: (Vec<PathBuf>, Vec<String>), // Use PathBuf instead of String
    mut selected: usize,                // Add selected index as a parameter
) -> Result<PathBuf, anyhow::Error> {
    let current_dir = env::current_dir().unwrap();

    loop {
        let mut list_state = ListState::default();
        list_state.select(Some(selected));
        terminal
            .draw(|frame| {
                render_files_list_state(&files, list_state, frame);
            })
            .unwrap();

        if crossterm::event::poll(Duration::from_millis(50)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Up => {
                            increment_selected(&files, &mut selected);
                        }
                        KeyCode::Down => {
                            decrement_selected(&files, &mut selected);
                        }
                        KeyCode::Right | KeyCode::Enter => {
                            return terminal_enter_file_or_dir(
                                terminal,
                                &files,
                                selected,
                                &current_dir,
                            );
                        }
                        KeyCode::Left => {
                            return parent_pathbuf(terminal, &files, selected, &current_dir);
                        }
                        KeyCode::Esc => {
                            return Err(anyhow::anyhow!("File selection canceled"));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn parent_pathbuf<B: Backend>(
    terminal: &mut Terminal<B>,
    files: &(Vec<PathBuf>, Vec<String>),
    selected: usize,
    current_dir: &PathBuf,
) -> Result<PathBuf, anyhow::Error> {
    if let Some(parent) = files
        .0
        .get(selected)
        .unwrap_or(current_dir)
        .parent()
        .unwrap()
        .parent()
    {
        let dir_entries = file_entries(&parent.to_path_buf()).unwrap();
        let current_dir_name = files.0[selected].file_name();
        let new_selected = dir_entries
            .0
            .iter()
            .position(|entry| entry.file_name() == current_dir_name)
            .unwrap_or(0);
        return file_selection_loop(terminal, dir_entries, new_selected);
    }
    Ok(current_dir.to_path_buf())
}

fn terminal_enter_file_or_dir<B: Backend>(
    terminal: &mut Terminal<B>,
    files: &(Vec<PathBuf>, Vec<String>),
    selected: usize,
    current_dir: &PathBuf,
) -> Result<PathBuf, anyhow::Error> {
    let selected_route = files.0.get(selected).unwrap_or(current_dir);
    if selected_route.is_dir() {
        let dir_entries = file_entries(selected_route).unwrap();
        file_selection_loop(terminal, dir_entries, 0)
    } else if selected_route.is_file() {
        return Ok(selected_route.clone());
    } else {
        anyhow::bail!("Selected path is neither a file nor a directory");
    }
}

fn decrement_selected(files: &(Vec<PathBuf>, Vec<String>), selected: &mut usize) {
    if !files.0.is_empty() {
        *selected = (*selected + 1).min(files.0.len() - 1);
    }
}

fn increment_selected(files: &(Vec<PathBuf>, Vec<String>), selected: &mut usize) {
    if !files.0.is_empty() {
        *selected = selected.saturating_sub(1);
    }
}

fn render_files_list_state<B: Backend>(
    files: &(Vec<PathBuf>, Vec<String>),
    mut list_state: ListState,
    f: &mut ratatui::Frame<'_, B>,
) {
    let size = f.size();
    let items: Vec<ListItem> = files
        .0
        .iter()
        .enumerate()
        .map(|(i, _file)| ListItem::new(files.1[i].to_string())) // Display the path as a string
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Select a File")
                .borders(Borders::ALL),
        )
        .highlight_style(
            HIGHLIGHT_STYLE, // Set background color
        )
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, size, &mut list_state);
}

fn get_type_name<T>(_: &T) -> &'static str {
    type_name::<T>()
}

fn get_class_name<'a, T>(instance: &T) -> &'a str {
    let full_type_name = get_type_name(instance);
    full_type_name.split("::").last().unwrap_or(full_type_name)
}

pub fn terminal_draw_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut state: AppState,
) -> Result<(), anyhow::Error> {
    let (bin_path, file) = current_bin_path(&state.selected_file)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let data = &mmap.as_bytes();

    // Parse the file
    let root_tree = build(data, bin_path);

    // Initialize TreeState
    state.tree_state.toggle(vec![0]); // Expand the root node

    let mut last_key_time = Instant::now(); // Track the last key press time

    loop {
        // Render the UI
        terminal
            .draw(|f| {
                let size = f.size();
                let chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Horizontal)
                    .constraints(
                        [
                            ratatui::layout::Constraint::Percentage(100), // Full width for the tree
                        ]
                        .as_ref(),
                    )
                    .split(size);

                // Render the tree
                let tree_widget = Tree::new(vec![root_tree.clone()])
                    .block(Block::default().borders(Borders::ALL).title("Nodes"))
                    .highlight_style(HIGHLIGHT_STYLE);
                f.render_stateful_widget(tree_widget, chunks[0], &mut state.tree_state);
            })
            .map_err(anyhow::Error::new)?;

        // Handle input events
        if event::poll(Duration::from_millis(149)).map_err(anyhow::Error::new)? {
            if let Event::Key(key) = event::read().map_err(anyhow::Error::new)? {
                if last_key_time.elapsed() >= Duration::from_millis(150) {
                    // Debounce threshold
                    last_key_time = Instant::now(); // Update the last key press time
                    match key.code {
                        KeyCode::Char('q') => break, // Quit the loop
                        KeyCode::Up => state.tree_state.key_up(&[root_tree.clone()]),
                        KeyCode::Down => state.tree_state.key_down(&[root_tree.clone()]),
                        KeyCode::Left => state.tree_state.key_left(),
                        KeyCode::Right => state.tree_state.key_right(),
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

fn build<'a>(data: &&'a [u8], bin_path: PathBuf) -> TreeItem<'a> {
    let fxr = parse_fxr(data)
        .map_err(|e| anyhow::anyhow!("Failed to parse file '{}': {}", bin_path.display(), e))
        .unwrap();

    // Build reflection trees for the header and section
    let header = fxr.header.deref();
    let header_tree: TreeItem = build_reflection_tree(header, get_class_name(header));
    let mut children = vec![];
    let section1_tree = build_section_1_tree(&fxr);

    let section4_tree = build_section_4_tree(fxr);

    children.push(header_tree);
    if let Some(section_tree) = section1_tree {
        children.push(section_tree);
    }
    if let Some(section_tree) = section4_tree {
        children.push(section_tree);
    }

    // Combine the trees into a single root

    TreeItem::new("FXR File", children)
}

fn build_section_4_tree<'a>(fxr: ParsedFXR<'a>) -> Option<TreeItem<'a>> {
    if let Some(section4_tree) = &fxr.section4_tree {
        let section4 = section4_tree.container.deref();
        let mut section_tree: TreeItem = build_reflection_tree(section4, get_class_name(section4));

        if let Some(section5_entries) = &section4_tree.section5_entries {
            section5_entries.deref().iter().for_each(|section5_entry| {
                section_tree.add_child(build_reflection_tree(
                    section5_entry,
                    get_class_name(section5_entry),
                ));
            });
        }

        if let Some(section6_entries) = &section4_tree.section6_entries {
            section6_entries.deref().iter().for_each(|section6_entry| {
                section_tree.add_child(build_reflection_tree(
                    section6_entry,
                    get_class_name(section6_entry),
                ));
            });
        }

        Some(section_tree)
    } else {
        None
    }
}

fn build_section_1_tree<'a>(
    fxr: &fxr_binary_reader::fxr::fxr_parser_with_sections::ParsedFXR<'a>,
) -> Option<TreeItem<'a>> {
    if let Some(section1_tree) = &fxr.section1_tree {
        let section1 = section1_tree.section1.deref();
        let mut section_tree: TreeItem = build_reflection_tree(section1, get_class_name(section1));
        if let Some(section2) = &section1_tree.section2.as_deref() {
            let section2_tree: TreeItem = build_reflection_tree(section2, get_class_name(section2));
            section_tree.add_child(section2_tree);
        }
        if let Some(section3) = &section1_tree.section3 {
            section3.deref().iter().for_each(|section_3_entry| {
                section_tree.add_child(build_reflection_tree(
                    section_3_entry,
                    get_class_name(section_3_entry),
                ));
            });
        }
        Some(section_tree)
    } else {
        None
    }
}

pub fn current_bin_path(selected_file: &PathBuf) -> Result<(PathBuf, File), anyhow::Error> {
    let current_dir = env::current_dir()?;
    let bin_path = current_dir.join(selected_file);
    let file = File::open(&bin_path)
        .map_err(|e| anyhow::anyhow!("Failed to open file '{}': {}", bin_path.display(), e))?;
    Ok((bin_path, file))
}
