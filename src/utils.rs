use colored::Colorize;

pub fn print_success(msg: &str) {
    println!("{} {}", "[SUCCESS]".green().bold(), msg);
}

pub fn print_error(msg: &str) {
    println!("{} {}", "[ERROR]".red().bold(), msg);
}

pub fn print_info(msg: &str) {
    println!("{} {}", "[INFO]".blue().bold(), msg);
}

pub fn print_warning(msg: &str) {
    println!("{} {}", "[WARNING]".yellow().bold(), msg);
}

// Additional utility functions for more specific use cases
pub fn print_step(msg: &str) {
    println!("{} {}", "[STEP]".cyan().bold(), msg);
}

// pub fn print_debug(msg: &str) {
//     println!("{} {}", "[DEBUG]".magenta().bold(), msg);
// }

// pub fn print_progress(msg: &str) {
//     println!("{} {}", "[PROGRESS]".white().bold(), msg);
// }
