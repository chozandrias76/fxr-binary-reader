use clap::{Parser, command};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use fxr_binary_reader::fxr::Header;
use fxr_binary_reader::fxr::fxr_parser_with_sections::{ParsedFXR, parse_fxr};
use fxr_binary_reader::fxr::view::view::{StructNode, build_reflection_tree};
use log::debug;
use memmap2::Mmap;
use ratatui::layout::Rect;
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::{Frame, Terminal};
use std::error::Error;
use std::ops::Deref;
use std::path::PathBuf;
use std::process::ExitCode;
use zerocopy::IntoBytes;
mod gui;
use gui::{render_ui, terminal_draw_loop};

use std::fs::File;
use std::io::{self, Read};
use std::time::Duration;

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

fn flatten_tree_mut<'a>(
    node: &'a mut StructNode,
    depth: usize,
    out: &mut Vec<(usize, *mut StructNode)>,
) {
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
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let bin_path = PathBuf::from("D:\\Elden Ring Tools\\fxr-binary-reader\\f000302421.fxr");
    let file = File::open(bin_path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let data = &mmap.as_bytes();

    let fxr: ParsedFXR<'_> = parse_fxr(data).unwrap();
    let header_view: StructNode = build_reflection_tree(&fxr.header.deref(), "Header");
    let state = AppState::new(header_view);

    terminal_draw_loop(&mut terminal, state)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

