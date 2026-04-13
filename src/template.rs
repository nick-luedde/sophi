use crate::{util::loading::LoadingBar, SophiAction, SophiBase};

use std::path::Path;
use std::fs;
use std::io;

use rust_embed::RustEmbed;

// This macro runs at compile time. It looks for the "templates" 
// folder in your project root and embeds it.
#[derive(RustEmbed)]
#[folder = "template/"]
struct TemplateAssets;

pub struct SophiTemplate {
    pub command: SophiBase,
}

impl SophiTemplate {
  fn scaffold(&self, loading: &mut LoadingBar) -> io::Result<()> {

    let asset_iter = TemplateAssets::iter();

    let total_elements = asset_iter.count();

    loading.reset(total_elements + 1);

    loading.load(None, "Setting up template files...");

    let path_arg = match self.command.args.first() {
        Some(s) => s,
        None => &String::from("")
    };

    let target_dir = Path::new(path_arg);

    if !target_dir.exists() {
      panic!("Target directory {} for template does not exist", path_arg);
    }

    for file_path in TemplateAssets::iter() {
      let path_str = file_path.as_ref();

      loading.load(None, &format!("Creating {}", path_str));

      let file = TemplateAssets::get(path_str).expect(&format!("Failed to get embedded file {}", path_str));

      let destination = target_dir.join(path_str);

      if let Some(parent) = destination.parent() {
        if !parent.exists() {
          fs::create_dir_all(parent).expect("Could not create parent directory for template!");
        }
      }

      fs::write(&destination, file.data).expect("Failed to write template file");
    }

    Ok(())
  }
}

impl SophiAction for SophiTemplate {
    fn action(&self) {
        let mut loading = LoadingBar::new();
        
        self.scaffold(&mut loading).expect("Failed to scaffold template!");
        
        loading.complete(&format!("Template generated at {}", self.command.args.first().unwrap()));   
    }
}