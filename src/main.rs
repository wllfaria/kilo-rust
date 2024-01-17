use crossterm::{event::Event::Key, terminal};
use std::io::Result;

mod input;
mod keyboard;
mod view;

use input::*;
use view::*;

fn main() -> Result<()> {
    terminal::enable_raw_mode()?;
    loop {
        if editor_refresh_screen().is_err() {
            die("editor_refresh_screen");
        };
        if editor_process_keypress() {
            break;
        }
    }
    terminal::disable_raw_mode()?;
    Ok(())
}
