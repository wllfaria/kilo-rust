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
            if self.process_keypress()? {
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
        if let Ok(c) = self.keyboard.read() {
            match c {
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => Ok(true),
                KeyEvent {
                    code: KeyCode::Char(key),
                    ..
                } => {
                    match key {
                        'w' | 'a' | 's' | 'd' => self.move_cursor(key),
                        _ => {}
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        } else {
            self.die("unable to read keypress");
            Ok(false)
        }
    }

    pub fn die<S: Into<String>>(&mut self, message: S) {
        let _ = self.screen.clear();
        let _ = terminal::disable_raw_mode();
        eprintln!("{}: {}", message.into(), errno());
        std::process::exit(1);
    }

    fn move_cursor(&mut self, key: char) {
        match key {
            'a' => self.cursor.x = self.cursor.x.saturating_sub(1),
            'd' => self.cursor.x += 1,
            'w' => self.cursor.y = self.cursor.y.saturating_sub(1),
            's' => self.cursor.y += 1,
            _ => self.die("invalid movement character"),
        }
    }
}
