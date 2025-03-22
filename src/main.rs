use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use fxr_binary_reader::fxr::{
    fxr_parser_with_sections::{ParsedFXR, parse_fxr},
    view::view::{StructNode, build_reflection_tree},
};
use log::debug;
use memmap2::Mmap;
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::{env, fs, ops::Deref};
use zerocopy::IntoBytes;
mod gui;
use gui::{file_selection_loop, terminal_draw_loop};
use std::{fs::File, io};

enum FocusedSection {
    Nodes,
    Fields,
}
struct AppState {
    root: StructNode,
    flattened: Vec<(usize, *mut StructNode)>,
    fields: Vec<(String, String)>,
    selected: usize,
    node_scroll_offset: usize,
    field_scroll_offset: usize,
    focused_section: FocusedSection,
    dragging: Option<FocusedSection>, // Track which section's scrollbar is being dragged
    resizing: bool,                   // Track if the user is resizing the panes
    node_pane_width: u16,             // Width of the "Nodes" pane in percentage
}

impl AppState {
    fn new(mut root: StructNode) -> Self {
        let mut flattened = vec![];
        let fields: Vec<(String, String)> = root.fields.clone();
        flatten_tree_mut(&mut root, 0, &mut flattened);
        Self {
            root,
            fields,
            flattened,
            selected: 0,
            node_scroll_offset: 0,
            field_scroll_offset: 0,
            focused_section: FocusedSection::Fields,
            dragging: None,
            resizing: false,
            node_pane_width: 70, // Default to 70% width for the "Nodes" pane
        }
    }
}

fn flatten_tree_mut(node: &mut StructNode, depth: usize, out: &mut Vec<(usize, *mut StructNode)>) {
    out.push((depth, node));
    if node.is_expanded {
        for child in &mut node.children {
            flatten_tree_mut(child, depth + 1, out);
        }
    }
    debug!(
        "Flattened tree: {:#?}",
        out.iter()
            .map(|(d, n)| (d, unsafe { &**n }.name.clone()))
            .collect::<Vec<_>>()
    );
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable raw mode and set up the terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Ensure cleanup happens even if the program panics
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // Get the current directory
        let current_dir = env::current_dir()?;
        let files: Vec<String> = fs::read_dir(&current_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .filter_map(|entry: fs::DirEntry| {
                let path = entry.path();
                let extension = path.extension()?.to_str()?;
                if extension == "fxr" {
                    Some(entry.file_name().to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect();

        // Let the user pick a file
        let selected_file = file_selection_loop(&mut terminal, &files)?;

        // Open the selected file
        let bin_path = current_dir.join(selected_file);
        let file = File::open(bin_path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let data = &mmap.as_bytes();

        let fxr: ParsedFXR<'_> = parse_fxr(data).unwrap();
        let header_view: StructNode = build_reflection_tree(&fxr.header.deref(), "Header");
        let state = AppState::new(header_view);

        // Enter the main draw loop
        terminal_draw_loop(&mut terminal, state)?;

        Ok::<(), Box<dyn std::error::Error>>(())
    }));

    // Cleanup terminal state
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Handle any errors that occurred during execution
    if let Err(err) = result {
        if let Some(string) = err.downcast_ref::<String>() {
            eprintln!("Application encountered a panic: {}", string);
        } else if let Some(&str_slice) = err.downcast_ref::<&str>() {
            eprintln!("Application encountered a panic: {}", str_slice);
        } else {
            eprintln!("Application encountered an unknown panic.");
        }
        return Err(Box::new(std::io::Error::other(
            "Application crashed due to a panic",
        )));
    }

    Ok(())
}
