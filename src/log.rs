use log::{Level, LevelFilter};
use std::io::Write;

// TODO: Make it exe arguments at last

/// Initialize the logger with a custom formatter (cargo-style)
pub fn init() {
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let level = record.level();
            let (prefix, color) = match level {
                Level::Error => ("error", "\x1b[31m"),   // Red
                Level::Warn => ("warning", "\x1b[33m"),  // Yellow
                Level::Info => ("info", "\x1b[36m"),     // Cyan
                Level::Debug => ("debug", "\x1b[90m"),   // Dim
                Level::Trace => ("trace", "\x1b[90m"),   // Dim
            };
            let reset = "\x1b[0m";
            
            // Format file location
            let location = if let Some(file) = record.file() {
                if let Some(line) = record.line() {
                    format!("[{}:{}]", file, line)
                } else {
                    format!("[{}]", file)
                }
            } else {
                String::new()
            };
            
            // Cargo-style format: "error: message [file:line]"
            writeln!(
                buf,
                "{}{}{}: {}{}",
                color,
                prefix,
                reset,
                record.args(),
                if !location.is_empty() {
                    format!(" {}", location)
                } else {
                    String::new()
                }
            )
        })
        .filter_level(LevelFilter::Info)
        .init();
}

/// Log an error message with optional line number
#[macro_export]
macro_rules! log_error {
    ($msg:expr) => {
        log::error!("{}", $msg);
    };
    ($msg:expr, $line:expr) => {
        log::error!("{} (at line {})", $msg, $line);
    };
    ($($arg:tt)*) => {
        log::error!($($arg)*);
    };
}

/// Log a warning message
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        log::warn!($($arg)*);
    };
}

/// Log an info message
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        log::info!($($arg)*);
    };
}

/// Log a debug message
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        log::debug!($($arg)*);
    };
}

/// Log a trace message
#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        log::trace!($($arg)*);
    };
}

/// Log an error with location information (cargo-style)
pub fn _error_at(msg: &str, file: &str, line: usize, col: usize) {
    log::error!("{}", msg);
    eprintln!("  --> {}:{}:{}", file, line, col);
}

/// Log a warning with location information (cargo-style)
pub fn _warning_at(msg: &str, file: &str, line: usize, col: usize) {
    log::warn!("{}", msg);
    eprintln!("  --> {}:{}:{}", file, line, col);
}

/// Log a parsing error with position information
pub fn _log_parse_error(msg: &str, line: usize, column: usize) {
    log::error!(
        "Parse error at line {}:{} - {}",
        line,
        column,
        msg
    );
}

/// Log a lexing error with position information
pub fn _log_lex_error(msg: &str, line: usize, column: usize) {
    log::error!(
        "Lex error at line {}:{} - {}",
        line,
        column,
        msg
    );
}

/// Log a success message
pub fn _log_success(msg: &str) {
    log::info!("✓ {}", msg);
}

/// Log a section header (for test output)
pub fn _log_section(title: &str) {
    let separator = "=".repeat(60);
    log::info!("\n{}\n{}\n{}", separator, title, separator);
}

/// Log test results
pub fn _log_test_result(test_name: &str, success: bool, details: Option<&str>) {
    if success {
        log::info!("✓ {} - PASSED", test_name);
        if let Some(details) = details {
            log::debug!("  {}", details);
        }
    } else {
        log::error!("✗ {} - FAILED", test_name);
        if let Some(details) = details {
            log::error!("  {}", details);
        }
    }
}

/// Log AST output in a formatted way
pub fn _log_ast<T: std::fmt::Debug>(label: &str, ast: &T) {
    log::debug!("{} AST:\n{:#?}", label, ast);
}

/// Log parser progress
pub fn _log_parser_step(step: &str) {
    log::trace!("Parser: {}", step);
}

/// Log token information (for debugging)
pub fn _log_token(token: &str, line: usize, column: usize) {
    log::trace!("Token: '{}' at {}:{}", token, line, column);
}

