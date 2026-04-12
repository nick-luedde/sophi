use std::io::Write;

use colored::Colorize;
use console::Term;
pub struct LoadingBar {
    total_elements: usize,
    elements_done: usize,
    current_term_chars: usize,
    term: Term,
    pub verbose: bool,
}

impl LoadingBar {
    pub fn new() -> LoadingBar {
        LoadingBar {
            total_elements: 0,
            elements_done: 0,
            current_term_chars: 0,
            term: Term::stdout(),
            verbose: false,
        }
    }

    pub fn reset(&mut self, total_elements: usize) -> &mut LoadingBar {
        self.total_elements = total_elements;
        self.elements_done = 0;
        self.current_term_chars = 0;

        return self;
    }

    pub fn load(&mut self, num: Option<usize>, msg: &str) -> &mut LoadingBar {
        let incr = match num {
            Some(n) => n,
            None => 1,
        };

        self.elements_done = std::cmp::min(self.elements_done + incr, self.total_elements);

        self.render(&msg);

        return self;
    }

    pub fn render(&mut self, msg: &str) -> &mut LoadingBar {
        let pct = self.percent();

        // get number of loaded blocks out of total blocks
        const TOTAL_BLOCKS: usize = 40;

        const LOADED_BLOCK: &str = "|";
        const STANDARD_BLOCK: &str = "-";

        // of the total blocks, what number should be shown as done?
        let done_blocks = (pct * TOTAL_BLOCKS as f32).floor() as usize;
        let remaining_blocks: usize = TOTAL_BLOCKS - done_blocks;
        
        let nxt = format!(
            "[{}{}] {}% - ({})",
            LOADED_BLOCK.repeat(done_blocks).blue(),
            STANDARD_BLOCK.repeat(remaining_blocks).bright_black(),
            (pct * 100.00).floor(),
            msg.bright_black()
        );
        
        // The str includes non-printed characters (like for colors and stuff) which throws off the terminal char count
        let nxt_chars = console::strip_ansi_codes(&nxt).chars().count();

        if !self.verbose && self.current_term_chars > 0 {
            self.wipe();
        } else if self.verbose {
            println!("");
        }

        print!("{}", nxt);
        std::io::stdout().flush().unwrap();

        self.current_term_chars = nxt_chars;
        return self;
    }

    fn wipe(&self) {
        let (_, cols) = self.term.size();

        let chars = self.current_term_chars;

        let rows_used = chars.div_ceil(cols as usize);

        self.term.move_cursor_up(rows_used - 1).expect("[Term]: Failed to move cursor up!");
        self.term.move_cursor_left(chars).expect("[Term]: Failed to move cursor left!");
        self.term.clear_to_end_of_screen().expect("[Term]: Failed to clear to end of screen!");
    }

    fn percent(&self) -> f32 {
        return self.elements_done as f32 / self.total_elements as f32;
    }

    pub fn complete(&mut self, msg: &str) -> &mut LoadingBar {
        self.render(&msg);
        println!("");

        return self;
    }
}
