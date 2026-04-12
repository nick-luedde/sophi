use crate::{util::loading::LoadingBar, SophiAction, util};

pub struct SophiLogin {}

impl SophiAction for SophiLogin {
    fn action(&self) {
        let mut loading = LoadingBar::new();
        loading.reset(2);
        loading.load(None, "Logging into Apps Script API auth...");
        
        util::google::login();
        
        loading.load(None, "Storing token...");

        loading.complete("Logged in");   
    }
}