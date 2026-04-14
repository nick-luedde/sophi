use colored::Colorize;

use crate::SophiAction;

pub struct SophiHelp {}

impl SophiAction for SophiHelp {
    fn action(&self) {
        println!(
            "{}",
            "Use this tool to build Google Apps Script web app!".purple()
        );
        println!("");
        println!("./client & ./server directories are built into the ./build directory.");
        println!("");

        println!("");
        println!("{}", "[Command] build".blue());
        println!(
            "{}",
            "     [ARGS: kind] (--client, -c) | (--server, -s) | (--all, -a)".yellow()
        );
        println!(
            "{}",
            "      [ARGS: env] (--dev, -d) | (--test, -t) | (--prod, -p)".yellow()
        );
        println!("{}", "  [ARGS: options] (--minify, -m, --verbose, -v)".yellow());

        println!("");
        println!("{}", "Example: >sophi build -a -p -m".cyan());
        println!("{}", "  ^ builds minified versions of both the client and server sides of the app with production config into ./build/index.html and ./build/index.js".bold());

        println!("");
        println!("{}", "[Command] push".blue());
        println!("{}", "     [ARGS: app] (--app <app-name>)".yellow());
        println!("{}", "     [ARGS: list] (--list)".yellow());
        println!("");
        println!("{}", "Example: >sophi push".cyan());
        println!("{}", "  ^ combined with a sophi_config.json file, pushes source code to your Apps Script project".bold());

        println!("");
        println!("{}", "[Command] ship".blue());
        println!("{}", "     [ARGS: (build and push args)]".yellow());
        println!("");
        println!("{}", "Example: >sophi ship".cyan());
        println!(
            "{}",
            "  ^ combines the build and push commands".bold()
        );

        println!("");
        println!("{}", "[Command] dev".blue());
        println!("{}", "     [ARGS: none]".yellow());
        println!("");
        println!("{}", "Example: >sophi dev".cyan());
        println!(
            "{}",
            "  ^ opens the configured dev url for your Apps Script project".bold()
        );
        
        println!("");
        println!("{}", "[Command] drive".blue());
        println!("{}", "     [ARGS: none]".yellow());
        println!("");
        println!("{}", "Example: >sophi drive".cyan());
        println!(
            "{}",
            "  ^ opens the configured Google Drive directory for your Apps Script project".bold()
        );

        println!("");
        println!("{}", "[Command] login".blue());
        println!("{}", "     [ARGS: none]".yellow());
        println!("");
        println!("{}", "Example: >sophi login".cyan());
        println!(
            "{}",
            "  ^ runs auth workflow and securely caches credentials for pushing to your Apps Script project".bold()
        );

        println!("");
        println!("{}", "[Command] logout".blue());
        println!("{}", "     [ARGS: none]".yellow());
        println!("");
        println!("{}", "Example: >sophi logout".cyan());
        println!(
            "{}",
            "  ^ revokes credentials for pushing to your Apps Script project".bold()
        );
    }
}
