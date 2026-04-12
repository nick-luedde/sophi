use crate::{util::loading::LoadingBar, SophiAction, util};

pub struct SophiLogout {}

impl SophiAction for SophiLogout {
    fn action(&self) {
        let mut loading = LoadingBar::new();
        loading.reset(1);
        loading.load(None, "Logging out of Apps Script API auth...");
        
        util::google::logout();
        
        loading.complete("Logged out");   
    }
}