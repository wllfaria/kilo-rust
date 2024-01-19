use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use errno::errno;
use std::collections::HashMap;
use std::io::Result;
use std::path::Path;

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
    row_offset: u16,
    col_offset: u16,
}

impl Editor {
    pub fn new<P: AsRef<Path>>(filename: Option<P>) -> Result<Self> {
        let rows = match filename {
            None => Vec::new(),
            Some(path) => std::fs::read_to_string(path)
                .unwrap()
                .split('\n')
                .map(|x| x.into())
                .collect(),
        };

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
            rows,
            row_offset: 0,
            col_offset: 0,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        loop {
            if self.refresh_screen().is_err() {
                self.die("editor_refresh_screen");
            };
            self.screen
                .move_to(&self.cursor, self.row_offset, self.col_offset)?;
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
        self.scroll();
        self.screen.clear()?;
        self.screen
            .draw_rows(&self.rows, self.row_offset, self.col_offset)?;
        Ok(())
    }

    pub fn scroll(&mut self) {
        let bounds = self.screen.bounds();
        if self.cursor.y < self.row_offset {
            self.row_offset = self.cursor.y;
        }
        if self.cursor.y >= self.row_offset + bounds.y {
            self.row_offset = self.cursor.y - bounds.y + 1;
        }
        if self.cursor.x < self.col_offset {
            self.col_offset = self.cursor.x;
        }
        if self.cursor.x >= self.col_offset + bounds.x {
            self.col_offset = self.cursor.x - bounds.x + 1;
        }
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
        let row_len = match self.rows.len() {
            0 => 0,
            r if self.cursor.y >= r as u16 => self.rows[self.rows.len() - 1].len(),
            _ => self.rows[self.cursor.y as usize].len(),
        };
        let bounds = self.screen.bounds();
        match key {
            EditorKey::Up => self.cursor.y = self.cursor.y.saturating_sub(1),
            EditorKey::Right if (self.cursor.x as usize) < row_len => self.cursor.x += 1,
            EditorKey::Down if self.cursor.y < self.rows.len() as u16 - 1 => self.cursor.y += 1,
            EditorKey::Left => self.cursor.x = self.cursor.x.saturating_sub(1),
            EditorKey::Home => self.cursor.x = 0,
            EditorKey::End => self.cursor.x = bounds.x,
            _ => {}
        }
        let y = std::cmp::min(self.cursor.y, self.rows.len() as u16 - 1);
        let row_len = self.rows[y as usize].len();
        self.cursor.x = std::cmp::min(self.cursor.x, row_len as u16);
    }
}
