use std::io::Result;

mod editor;
mod keyboard;
mod screen;

use editor::*;

fn main() -> Result<()> {
    let mut args = std::env::args();
    let mut editor = Editor::new(args.nth(1))?;
    editor.start()?;
    Ok(())
}
