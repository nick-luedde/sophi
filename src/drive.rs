use open;

use crate::{SophiAction, util};

pub struct SophiDrive {}

impl SophiAction for SophiDrive {
  fn action(&self) {
    let config = util::google::get_sophi_config();
    println!("Opening {}", config.drive_url);
    open::that(config.drive_url).unwrap();
  }
}