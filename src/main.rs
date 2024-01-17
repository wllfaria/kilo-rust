use crossterm::{event::Event::Key, terminal};
use std::io::Result;

mod editor;

use editor::*;

fn main() -> Result<()> {
    terminal::enable_raw_mode()?;
    let mut editor = Editor::new()?;

    loop {
        if editor.refresh_screen().is_err() {
            editor.die("editor_refresh_screen");
        };
        if editor.process_keypress() {
            editor.clear_screen()?;
            break;
        }
    }
    terminal::disable_raw_mode()?;
    Ok(())
}
