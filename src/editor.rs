use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use errno::errno;
use std::io::Result;

use crate::keyboard::*;
use crate::screen::*;
use kilo_rust::*;

pub type EditorResult<T, E> = std::result::Result<T, E>;
pub enum EditorError {
    KeyReadFail,
}

pub struct Editor {
    screen: Screen,
    keyboard: Keyboard,
    cursor: Position,
}

impl Editor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            screen: Screen::new()?,
            keyboard: Keyboard {},
            cursor: Default::default(),
        })
    }

    pub fn start(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        loop {
            if self.refresh_screen().is_err() {
                self.die("editor_refresh_screen");
            };
            self.screen.move_to(&self.cursor)?;
            self.screen.flush()?;
            if let Ok(_) = self.process_keypress() {
                self.screen.clear()?;
                break;
            }
        }
        terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn refresh_screen(&mut self) -> Result<()> {
        self.screen.clear()?;
        self.screen.draw_rows()?;
        Ok(())
    }

    pub fn process_keypress(&mut self) -> Result<bool> {
        let c = self.keyboard.read();

        match c {
            Ok(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => Ok(true),
            Err(EditorError::KeyReadFail) => {
                self.die("unable to read keypress");
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    pub fn die<S: Into<String>>(&mut self, message: S) {
        let _ = self.screen.clear();
        let _ = terminal::disable_raw_mode();
        eprintln!("{}: {}", message.into(), errno());
        std::process::exit(1);
    }
}
