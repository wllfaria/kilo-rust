use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use errno::errno;
use std::collections::HashMap;
use std::io::Result;

use crate::keyboard::*;
use crate::screen::*;
use kilo_rust::*;

pub type EditorResult<T, E> = std::result::Result<T, E>;
pub enum EditorError {
    KeyReadFail,
}

#[derive(Copy, Clone)]
enum EditorKey {
    Up,
    Left,
    Down,
    Right,
    Home,
    End,
}

pub struct Editor {
    screen: Screen,
    keyboard: Keyboard,
    cursor: Position,
    keymap: HashMap<char, EditorKey>,
    rows: Vec<String>,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let mut keymap = HashMap::with_capacity(4);
        keymap.insert('w', EditorKey::Up);
        keymap.insert('a', EditorKey::Left);
        keymap.insert('s', EditorKey::Down);
        keymap.insert('d', EditorKey::Right);

        Ok(Self {
            screen: Screen::new()?,
            keyboard: Keyboard {},
            cursor: Default::default(),
            keymap,
            rows: vec!["Hello, World!".to_string()],
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
        self.screen.draw_rows(&self.rows)?;
        Ok(())
    }

    pub fn process_keypress(&mut self) -> Result<bool> {
        let bounds = self.screen.bounds();
        if let Ok(c) = self.keyboard.read() {
            match c {
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => return Ok(true),
                KeyEvent {
                    code: KeyCode::Up, ..
                } => self.move_cursor(EditorKey::Up),
                KeyEvent {
                    code: KeyCode::Left,
                    ..
                } => self.move_cursor(EditorKey::Left),
                KeyEvent {
                    code: KeyCode::Down,
                    ..
                } => self.move_cursor(EditorKey::Down),
                KeyEvent {
                    code: KeyCode::Right,
                    ..
                } => self.move_cursor(EditorKey::Right),
                KeyEvent {
                    code: KeyCode::PageUp,
                    ..
                } => {
                    for _ in 0..bounds.y {
                        self.move_cursor(EditorKey::Up);
                    }
                }
                KeyEvent {
                    code: KeyCode::PageDown,
                    ..
                } => {
                    for _ in 0..bounds.y {
                        self.move_cursor(EditorKey::Down);
                    }
                }
                KeyEvent {
                    code: KeyCode::Home,
                    ..
                } => self.move_cursor(EditorKey::Home),
                KeyEvent {
                    code: KeyCode::End, ..
                } => self.move_cursor(EditorKey::End),
                KeyEvent {
                    code: KeyCode::Char(key),
                    ..
                } => match key {
                    'w' | 'a' | 's' | 'd' => {
                        let c = *self.keymap.get(&key).unwrap();
                        self.move_cursor(c);
                    }
                    _ => {}
                },
                _ => {}
            }
        } else {
            self.die("unable to read keypress");
        }
        Ok(false)
    }

    pub fn die<S: Into<String>>(&mut self, message: S) {
        let _ = self.screen.clear();
        let _ = terminal::disable_raw_mode();
        eprintln!("{}: {}", message.into(), errno());
        std::process::exit(1);
    }

    fn move_cursor(&mut self, key: EditorKey) {
        let bounds = self.screen.bounds();
        match key {
            EditorKey::Up => self.cursor.y = self.cursor.y.saturating_sub(1),
            EditorKey::Right if self.cursor.x < bounds.x - 1 => self.cursor.x += 1,
            EditorKey::Down if self.cursor.y < bounds.y - 1 => self.cursor.y += 1,
            EditorKey::Left => self.cursor.x = self.cursor.x.saturating_sub(1),
            EditorKey::Home => self.cursor.x = 0,
            EditorKey::End => self.cursor.x = bounds.x,
            _ => {}
        }
    }
}
