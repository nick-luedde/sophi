use colored::Colorize;
use std::io::{self, Write};

use crate::{
    SophiAction, SophiBase,
    util::{self, google, loading::LoadingBar, timer::SophiTimer},
};
pub struct SophiPush {
    pub command: SophiBase,
}

impl SophiPush {
    pub fn valid_args() -> Vec<String> {
        vec!["--app".to_string(), "--list".to_string()]
    }

    pub fn parse_push_args(all_args: &Vec<String>) -> Vec<String> {
        let valid = SophiPush::valid_args();
        let app_arg_index = all_args.iter().position(|a| a == "--app");

        let mut parsed_args: Vec<String> = all_args
            .iter()
            .filter(|a| valid.contains(a))
            .map(|a| a.to_string())
            .collect();

        if let Some(ind) = app_arg_index {
            let app_name = all_args
                .get(ind + 1)
                .expect("Missing push --app <app-name> argument");

            parsed_args.push(app_name.to_string());
        }

        return parsed_args;
    }
}

impl SophiAction for SophiPush {
    fn action(&self) {
        let args = &self.command.args;

        let is_list = args.contains(&"--list".to_string());

        let config = google::get_sophi_config();

        let app_name = if is_list {
            Some(google::select_app_from_config(Some(&config)))
        } else {
            let app_arg_index = args.iter().position(|a| a == "--app");

            let name = match app_arg_index {
                Some(ind) => match args.get(ind + 1) {
                    Some(nm) => Some(nm.to_string()),
                    None => Some(config.get_app_or_default(None).default.to_string()),
                },
                None => Some(config.get_app_or_default(None).default.to_string()),
            };

            name
        };

        // I'm just being lazy AF here....
        let nm = &app_name.clone().unwrap_or("default".to_string());

        let mut timer = SophiTimer::new();

        let mut loading = LoadingBar::new();
        loading.reset(3);
        util::google::auth();
        loading.load(None, "Authenticating...");
        io::stdout().flush().unwrap();

        loading.load(None, "Pushing source code to the Apps Script project...");
        io::stdout().flush().unwrap();
        util::google::push(app_name);

        loading.load(None, "Source code files pushed...");
        loading.complete("Push complete");
        println!("[{}] {}", nm, "Apps Script project updated".green());
        timer.stop().print_line();
    }
}
