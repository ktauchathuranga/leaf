use colored::Colorize;

pub fn print_success(msg: &str) {
    println!("{} {}", "✓".green(), msg);
}

pub fn print_error(msg: &str) {
    println!("{} {}", "✗".red(), msg);
}

pub fn print_info(msg: &str) {
    println!("{} {}", "ℹ".blue(), msg);
}

pub fn print_warning(msg: &str) {
    println!("{} {}", "⚠".yellow(), msg);
}

