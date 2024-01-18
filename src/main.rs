use std::io::Result;

mod editor;
mod keyboard;
mod screen;

use editor::*;

fn main() -> Result<()> {
    let mut editor = Editor::new()?;
    editor.start()?;
    Ok(())
}
