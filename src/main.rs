use chrono::DateTime;
use crash_handler::{CrashContext, CrashEventResult, CrashHandler, make_crash_event};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use fxr_binary_reader::fxr::fxr_parser_with_sections::{ParsedFXR, parse_fxr};
use ratatui::{Terminal, prelude::CrosstermBackend};
use ratatui_tree_widget::TreeState;
use std::{
    any::Any, env, error::Error, fs, io::Read, os::windows::fs::MetadataExt, path::PathBuf,
    sync::Mutex,
};
mod gui;
use gui::{file_selection_loop, terminal_draw_loop};
use std::{fs::File, io};

struct AppState<'a> {
    selected_file: PathBuf,
    fxr: Option<ParsedFXR<'a>>,
    tree_state: TreeState,
}

impl<'a> Default for AppState<'a> {
    fn default() -> Self {
        Self {
            selected_file: PathBuf::new(),
            // flattened: Vec::new(),
            fxr: None,
            tree_state: TreeState::default(),
        }
    }
}
fn load_file_data(file_path: &PathBuf) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut file = File::open(file_path)?;
    let mut file_data = Vec::new();
    file.read_to_end(&mut file_data)?;
    Ok(file_data)
}
impl<'a> AppState<'a> {
    fn new(selected_file: PathBuf, file_data: &'a [u8]) -> Result<Self, Box<dyn Error>> {
        let mut ret = Self::default();

        // Parse the file
        let fxr: ParsedFXR<'a> = parse_fxr(file_data)?;
        ret.fxr = Some(fxr);

        Ok(Self {
            selected_file,
            ..ret
        })
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
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = terminal_main_wrapper(&mut terminal);

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

fn terminal_main_wrapper<'a>(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<Result<(), Box<dyn Error>>, Box<(dyn Any + Send + 'a)>> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let current_dir = env::current_dir().unwrap();
        let files = file_entries(&current_dir).unwrap();

        let selected_file_index: usize = 0;
        let selected_file = file_selection_loop(terminal, files, selected_file_index);
        if selected_file.is_none() {
            // User canceled the file selection
            return Ok(());
        }
        let selected_file = selected_file.unwrap().unwrap();
        // Load file data
        let file_data = load_file_data(&selected_file).unwrap();

        // Initialize AppState with file data
        let state = AppState::new(selected_file, &file_data).unwrap();

        if terminal_draw_loop(terminal, state).is_some() {
            Ok(())
        } else {
            terminal_main_wrapper(terminal).unwrap()
        }
    }))
}
fn file_entries(
    current_dir: &std::path::PathBuf,
) -> Result<(Vec<PathBuf>, Vec<String>), Box<dyn std::error::Error>> {
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

    let mut files = Vec::new();
    let file_data: Vec<String> = fs::read_dir(current_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            (entry.file_type().unwrap().is_dir() && !entry.file_name().to_string_lossy().starts_with("."))
                || entry
                    .file_name()
                    .into_string()
                    .map(|name| name.ends_with(".fxr"))
                    .unwrap_or(false)
        })
        .map(|entry: fs::DirEntry| {
            let entry_as_pathbuf = entry.path();
            files.push(entry_as_pathbuf.clone()); // Store the PathBuf in the vector
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

    Ok((files, file_data)) // Return both the Vec<PathBuf> and the Vec<String>
}
