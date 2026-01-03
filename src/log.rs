use log::{Level, LevelFilter};
use std::io::Write;

/// Initialize the logger with a custom formatter
pub fn init() {
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let level = record.level();
            let level_color = match level {
                Level::Error => "\x1b[31m",   // Red
                Level::Warn => "\x1b[33m",    // Yellow
                Level::Info => "\x1b[36m",    // Cyan
                Level::Debug => "\x1b[35m",   // Magenta
                Level::Trace => "\x1b[90m",   // Bright Black
            };
            let reset = "\x1b[0m";
            let level_symbol = match level {
                Level::Error => "ERROR",
                Level::Warn => "WARNING",
                Level::Info => "INFO",
                Level::Debug => "DEBUG",
                Level::Trace => "TRACE",
            };
            
            writeln!(
                buf,
                "{}{} {} {}{} {}",
                level_color,
                level_symbol,
                level,
                reset,
                if let Some(file) = record.file() {
                    format!("[{}:{}]", file, record.line().unwrap_or(0))
                } else {
                    String::new()
                },
                record.args()
            )
        })
        .filter_level(LevelFilter::Info)
        .init();
}

/// Log an error message
#[macro_export]
macro_rules! log_error {
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

/// Log a parsing error with position information
pub fn log_parse_error(msg: &str, line: usize, column: usize) {
    log::error!(
        "Parse error at line {}:{} - {}",
        line,
        column,
        msg
    );
}

/// Log a lexing error with position information
pub fn log_lex_error(msg: &str, line: usize, column: usize) {
    log::error!(
        "Lex error at line {}:{} - {}",
        line,
        column,
        msg
    );
}

/// Log a success message
pub fn log_success(msg: &str) {
    log::info!("✓ {}", msg);
}

/// Log a section header (for test output)
pub fn log_section(title: &str) {
    let separator = "=".repeat(60);
    log::info!("\n{}\n{}\n{}", separator, title, separator);
}

/// Log test results
pub fn log_test_result(test_name: &str, success: bool, details: Option<&str>) {
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
pub fn log_ast<T: std::fmt::Debug>(label: &str, ast: &T) {
    log::debug!("{} AST:\n{:#?}", label, ast);
}

/// Log parser progress
pub fn log_parser_step(step: &str) {
    log::trace!("Parser: {}", step);
}

/// Log token information (for debugging)
pub fn log_token(token: &str, line: usize, column: usize) {
    log::trace!("Token: '{}' at {}:{}", token, line, column);
}

