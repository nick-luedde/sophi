use colored::Colorize;

pub struct SophiUnknown {}

impl SophiUnknown {
    pub fn action(unknown_command: &str) {
        println!("{} [{}]", "Unknown sophi command".red(), unknown_command.red().bold());
        
        println!("Try {} for a list of commands!", ">sophi help".black().on_bright_green());
    }
}