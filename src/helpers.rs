const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const COLOR_RESET: &str = "\x1b[0m";

pub fn log_green(scope: &str, message: impl std::fmt::Display) {
    println!("{GREEN}[{scope}] {message}{COLOR_RESET}");
}

pub fn log_red(scope: &str, message: impl std::fmt::Display) {
    eprintln!("{RED}[{scope}] {message}{COLOR_RESET}");
}
