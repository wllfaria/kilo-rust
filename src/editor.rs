use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use errno::errno;
use std::io::Result;

use crate::keyboard::*;
use crate::screen::*;

pub type EditorResult<T, E> = std::result::Result<T, E>;
pub enum EditorError {
    KeyReadFail,
}

pub struct Editor {
    screen: Screen,
    keyboard: Keyboard,
}

impl Editor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            screen: Screen::new()?,
            keyboard: Keyboard {},
        })
    }

    pub fn start(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        loop {
            if self.screen.refresh().is_err() {
                self.die("editor_refresh_screen");
            };
            self.screen.flush()?;
            if self.process_keypress() {
                self.screen.clear()?;
                break;
            }
        }
        terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn process_keypress(&mut self) -> bool {
        let c = self.keyboard.read();

        match c {
            Ok(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => true,
            Err(EditorError::KeyReadFail) => {
                self.die("unable to read keypress");
                false
            }
            _ => false,
        }
    }

    pub fn die<S: Into<String>>(&mut self, message: S) {
        let _ = self.screen.clear();
        let _ = terminal::disable_raw_mode();
        eprintln!("{}: {}", message.into(), errno());
        std::process::exit(1);
    }
}
