use crate::{
    Sophi, SophiAction, SophiBase,
    build::{self, SophiBuild},
    push::{self, SophiPush},
};
pub struct SophiShip {}

impl SophiAction for SophiShip {
    fn action(&self) {
        let (_, command_args) = Sophi::get_command_and_args();

        let valid_build_args = SophiBuild::valid_args();

        let build_args: Vec<String> = command_args
            .iter()
            .filter(|a| valid_build_args.contains(a))
            .map(|a| a.to_string())
            .collect();

        let push_args: Vec<String> = SophiPush::parse_push_args(&command_args);

        let build_args_to_use = if build_args.len() == 0 {
            vec!["-a".to_string()]
        } else {
            build_args
        };
        let push_args_to_use = push_args;

        let b = build::SophiBuild {
            command: SophiBase {
                args: build_args_to_use,
            },
        };
        let p = push::SophiPush {
            command: SophiBase {
                args: push_args_to_use,
            },
        };

        b.action();
        p.action();
    }
}
