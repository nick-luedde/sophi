use std::env;

mod build;
mod dev;
mod drive;
mod help;
mod login;
mod logout;
mod push;
mod ship;
mod template;
mod unknown;
mod util;

// Cross compile
// https://stackoverflow.com/questions/31492799/cross-compile-a-rust-application-from-linux-to-windows
// sudo apt-get install mingw-w64

pub struct SophiBase {
    args: Vec<String>,
}

pub trait SophiAction {
    fn action(&self) -> ();
}

pub struct Sophi {}

impl Sophi {
    pub fn get_command_and_args() -> (String, Vec<String>) {
        let args: Vec<String> = env::args().skip(1).collect();

        let command = if args.len() >= 1 { &args[0] } else { "(none)" };
        let command_args: Vec<String> = if args.len() >= 2 {
            args[1..args.len()].to_vec()
        } else {
            vec![]
        };

        (command.to_string(), command_args)
    }

    fn run() {
        let (command, command_args) = Sophi::get_command_and_args();

        if command == "build" {
            let build = build::SophiBuild {
                command: { SophiBase { args: command_args } },
            };

            build.action();
            return;
        }

        if command == "push" {
            let push = push::SophiPush {
                command: SophiBase { args: command_args },
            };

            push.action();
            return;
        }

        if command == "ship" {
            let run = ship::SophiShip {};

            run.action();
            return;
        }

        if command == "template" {
            let template = template::SophiTemplate {
                command: SophiBase { args: command_args },
            };

            template.action();
            return;
        }

        if command == "dev" {
            let dev = dev::SophiDev {
                command: SophiBase { args: command_args },
            };

            dev.action();
            return;
        }

        if command == "drive" {
            let drive = drive::SophiDrive {};

            drive.action();
            return;
        }

        if command == "login" {
            let login = login::SophiLogin {};

            login.action();
            return;
        }

        if command == "logout" {
            let logout = logout::SophiLogout {};

            logout.action();
            return;
        }

        if command == "help" || command == "--help" {
            let help = help::SophiHelp {};

            help.action();
            return;
        }

        unknown::SophiUnknown::action(&command);
    }
}

fn main() {
    Sophi::run();
}
