use open;

use crate::{SophiAction, SophiBase, util};

pub struct SophiDev {
    pub command: SophiBase,
}

impl SophiAction for SophiDev {
    fn action(&self) {
        let config = util::google::get_sophi_config();
        let args = &self.command.args;

        let app_arg_index = args.iter().position(|a| a == "-app");

        let app_name = match app_arg_index {
            Some(ind) => match args.get(ind + 1) {
                Some(nm) => Some(nm.to_string()),
                None => None,
            },
            None => None,
        };

        let app = config.get_app_or_default(app_name);
        println!("Opening {}", &app.dev_url);
        open::that(&app.dev_url).unwrap();
    }
}
