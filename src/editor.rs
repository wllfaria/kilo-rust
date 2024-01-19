use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use errno::errno;
use std::io::Result;
use std::path::Path;
use std::time::SystemTime;

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
    filename: String,
    keyboard: Keyboard,
    cursor: Position,
    rows: Vec<String>,
    row_offset: u16,
    col_offset: u16,
    status_msg: String,
    status_msg_timestamp: i32,
    dirty: u16,
    quit_attempts: u8,
}

impl Editor {
    pub fn new<P: AsRef<Path>>(filename: Option<P>) -> Result<Self> {
        let mut rows = Vec::new();
        let mut name = String::new();
        match filename {
            None => (),
            Some(path) => {
                name = path
                    .as_ref()
                    .file_name()
                    .unwrap()
                    .to_os_string()
                    .into_string()
                    .unwrap();
                rows = std::fs::read_to_string(path)
                    .unwrap()
                    .split('\n')
                    .map(|x| x.into())
                    .collect();
            }
        };

        Ok(Self {
            screen: Screen::new()?,
            keyboard: Keyboard {},
            cursor: Default::default(),
            rows,
            row_offset: 0,
            col_offset: 0,
            filename: name,
            status_msg: String::new(),
            status_msg_timestamp: 0,
            dirty: 0,
            quit_attempts: 3,
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

    pub fn rows_to_string(&self) -> String {
        let mut rows = String::new();
        for (idx, row) in self.rows.iter().enumerate() {
            rows.push_str(row);
            if idx < self.rows.len() - 1 {
                rows.push('\n');
            }
        }
        rows
    }

    pub fn save(&mut self) -> Result<()> {
        if self.filename.is_empty() {
            return Ok(());
        };
        let rows = self.rows_to_string();
        std::fs::write(&self.filename, rows)?;
        self.set_status_msg(format!("{} {}L written", self.filename, self.rows.len()));
        self.dirty = 0;
        Ok(())
    }

    pub fn row_insert_char(&mut self, row: usize, at: usize, c: char) {
        let at = std::cmp::min(at, self.rows[row].len());
        let mut left = self.rows[row][0..at].to_string();
        left.push(c);
        let right = self.rows[row][at..].to_string();
        self.rows[row] = left + &right;
        self.dirty += 1;
    }

    pub fn del_row(&mut self, at: usize) {
        if at >= self.rows.len() {
            return;
        }
        self.rows.remove(at);
        self.dirty += 1;
    }

    pub fn insert_row(&mut self, at: usize) {
        if at > self.rows.len() {
            return;
        }
        let left = self.rows[at - 1][0..self.cursor.x as usize].to_string();
        let right = self.rows[at - 1][self.cursor.x as usize..].to_string();
        self.rows[at - 1] = left;
        self.rows.insert(at, String::new());
        self.rows[at] = right;
        self.cursor.y += 1;
        self.cursor.x = 0;
    }

    pub fn row_append_string(&mut self, at: usize) {
        if at > self.rows.len() {
            return;
        }
        let mut left = self.rows[at - 1].clone();
        let right = self.rows[at].clone();
        left.push_str(&right);
        self.rows[at - 1] = left;
        self.dirty += 1;
    }

    pub fn insert_char(&mut self, c: char) {
        if self.cursor.y as usize == self.rows.len() {
            self.rows.push(String::new());
        }
        self.row_insert_char(self.cursor.y as usize, self.cursor.x as usize, c);
        self.cursor.x += 1;
    }

    pub fn row_del_char(&mut self, row: usize, at: usize) {
        let at = std::cmp::min(at, self.rows[row].len());
        let mut left = self.rows[row][0..at].to_string();
        left.pop();
        let right = self.rows[row][at..].to_string();
        self.rows[row] = left + &right;
        self.dirty += 1;
    }

    pub fn del_char(&mut self) {
        if self.cursor.y as usize == self.rows.len() {
            return;
        }
        if self.cursor.x == 0 && self.cursor.y == 0 {
            return;
        }
        if self.cursor.x > 0 {
            self.row_del_char(self.cursor.y as usize, self.cursor.x as usize);
            self.cursor.x -= 1;
            return;
        } else {
            let len = self.rows[self.cursor.y as usize - 1].len() as u16;
            self.row_append_string(self.cursor.y as usize);
            self.del_row(self.cursor.y as usize);
            self.cursor.y -= 1;
            self.cursor.x = len;
        }
    }

    pub fn set_status_msg(&mut self, message: String) {
        self.status_msg = message;
        self.status_msg_timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i32;
    }

    pub fn refresh_screen(&mut self) -> Result<()> {
        self.scroll();
        self.screen.clear()?;
        self.screen
            .draw_rows(&self.rows, self.row_offset, self.col_offset)?;
        self.screen
            .draw_status_bar(&self.rows, &self.filename, self.cursor.y, self.dirty)?;
        self.screen
            .draw_msg_bar(&self.status_msg, self.status_msg_timestamp)?;
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
                } => {
                    if self.dirty == 0 {
                        return Ok(true);
                    }
                    if self.quit_attempts == 0 {
                        return Ok(true);
                    }
                    self.set_status_msg(format!(
                        "WARNING!!! File has unsaved changes. Press Ctrl-Q {} more times to quit.",
                        self.quit_attempts
                    ));
                    self.quit_attempts -= 1;
                    return Ok(false);
                }
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                } => self.insert_row(self.cursor.y as usize + 1),
                KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                } => self.del_char(),
                KeyEvent {
                    code: KeyCode::Delete,
                    ..
                } => {
                    self.move_cursor(EditorKey::Right);
                    self.del_char();
                }
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
                    code: KeyCode::Char('s'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => self.save()?,
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
                } => self.insert_char(key),
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
            EditorKey::Right => match self.cursor.x as usize {
                _ if self.cursor.x as usize >= row_len
                    && self.cursor.y < self.rows.len() as u16 - 1 =>
                {
                    self.cursor.y += 1;
                    self.cursor.x = 0;
                }
                _ => self.cursor.x += 1,
            },
            EditorKey::Down if self.cursor.y < self.rows.len() as u16 - 1 => self.cursor.y += 1,
            EditorKey::Left => match self.cursor.x {
                _ if self.cursor.x > 0 => self.cursor.x -= 1,
                _ if self.cursor.y > 0 => {
                    self.cursor.y -= 1;
                    self.cursor.x = self.rows[self.cursor.y as usize].len() as u16;
                }
                _ => {}
            },
            EditorKey::Home => self.cursor.x = 0,
            EditorKey::End => self.cursor.x = bounds.x,
            _ => {}
        }
        let y = std::cmp::min(self.cursor.y, self.rows.len() as u16 - 1);
        let row_len = self.rows[y as usize].len();
        self.cursor.x = std::cmp::min(self.cursor.x, row_len as u16);
    }
}
