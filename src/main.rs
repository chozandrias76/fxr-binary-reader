use chrono::DateTime;
use crash_handler::{CrashContext, CrashEventResult, CrashHandler, make_crash_event};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use fxr_binary_reader::fxr::view::view::StructNode;
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::{cell::RefCell, env, fs, os::windows::fs::MetadataExt, rc::Rc, sync::Mutex};
mod gui;
use gui::{file_selection_loop, terminal_draw_loop};
use std::{fs::File, io};

enum FocusedSection {
    Nodes,
    Fields,
}
struct AppState {
    flattened: Vec<(usize, Rc<RefCell<StructNode>>)>,
    fields: Vec<(String, String)>,
    selected_file: String,
    selected_node: usize,
    node_scroll_offset: usize,
    field_scroll_offset: usize,
    focused_section: FocusedSection,
    dragging: Option<FocusedSection>, // Track which section's scrollbar is being dragged
    resizing: bool,                   // Track if the user is resizing the panes
    node_pane_width: u16,             // Width of the "Nodes" pane in percentage
}

impl AppState {
    fn new(selected_file: String) -> Self {
        Self {
            selected_file,
            selected_node: 0,
            flattened: Vec::new(),
            fields: Vec::new(),
            node_scroll_offset: 0,
            field_scroll_offset: 0,
            focused_section: FocusedSection::Fields,
            dragging: None,
            resizing: false,
            node_pane_width: 70, // Default to 70% width for the "Nodes" pane
        }
    }
}

fn flatten_tree_mut(
    node: &mut StructNode,
    depth: usize,
    out: &mut Vec<(usize, Rc<RefCell<StructNode>>)>,
) {
    out.push((depth, Rc::new(RefCell::new(node.clone()))));
    if node.is_expanded {
        for child in &mut node.children {
            flatten_tree_mut(child, depth + 1, out);
        }
    }
}

fn setup() -> Result<(), Box<dyn std::error::Error>> {
    let log_file = File::create("./fxr_binary_reader.log")?;
    let subscriber = tracing_subscriber::fmt()
        .with_writer(Mutex::new(log_file))
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    std::panic::set_hook(Box::new(|panic_info| {
        tracing::error!("Application panicked: {}", panic_info);
    }));

    let handler = CrashHandler::attach(unsafe {
        make_crash_event(move |context: &CrashContext| {
            tracing::error!(
                "Exception: {:x} at {:x}",
                context.exception_code,
                (*(*context.exception_pointers).ExceptionRecord).ExceptionAddress as usize
            );

            CrashEventResult::Handled(true)
        })
    })
    .unwrap();
    std::mem::forget(handler);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = setup();
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
        let files = file_entries(&current_dir)?;

        // Let the user pick a file
        let selected_file = file_selection_loop(&mut terminal, &files)?;

        let state = AppState::new(selected_file);

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

    if let Err(_err) = result {
        let partial = "Application crashed due to a panic.".to_string();
        let message = match env::var("RUST_BACKTRACE").unwrap_or_default().as_str() {
            "1" => "Check fxr_binary_reader.log for more information..".to_string(),
            _ => "Run with `RUST_BACKTRACE=1` for more information.".to_string(),
        };
        let full_message = format!("{} {}", partial, message);
        return Err(Box::new(std::io::Error::other(full_message.to_string())));
    }
    drop(subscriber);
    Ok(())
}

fn file_entries(
    current_dir: &std::path::PathBuf,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    fn format_file_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if size >= GB {
            format!("{:.2} GB", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.2} MB", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.2} KB", size as f64 / KB as f64)
        } else {
            format!("{} B", size)
        }
    }
    let files: Vec<String> = fs::read_dir(current_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ft: fs::FileType| {
            ft.is_file()
        }).unwrap_or(false) && (
            entry.file_name().into_string().map(|name| name.ends_with(".fxr")).unwrap_or(false)
        ))
        .map(|entry: fs::DirEntry| {
            let metadata = entry.metadata().unwrap();
            let file_attributes = metadata.file_attributes();
            let creation_time = metadata.creation_time();
            let last_access_time = metadata.last_access_time();
            let last_write_time = metadata.last_write_time();
            let file_size = metadata.file_size();

            format!(
                "{:<30} | Attrs: {:<10} | Created: {:<20} | Accessed: {:<20} | Modified: {:<20} | Size: {:<10}",
                entry.file_name().to_string_lossy(),
                file_attributes,
                DateTime::from_timestamp(
                    ((creation_time - 116444736000000000) / 10_000_000) as i64,
                    0
                )
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Invalid Date".to_string()),
                DateTime::from_timestamp(
                    ((last_access_time - 116444736000000000) / 10_000_000) as i64,
                    0
                )
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Invalid Date".to_string()),
                DateTime::from_timestamp(
                    ((last_write_time - 116444736000000000) / 10_000_000) as i64,
                    0
                )
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Invalid Date".to_string()),
                format_file_size(file_size),
            )
        })
        .collect();
    Ok(files)
}
