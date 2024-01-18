use crossterm::event::{read, Event::Key, KeyEvent};

use crate::*;

pub struct Keyboard {}

impl Keyboard {
    pub fn read(&self) -> EditorResult<KeyEvent, EditorError> {
        loop {
            if let Ok(event) = read() {
                if let Key(key) = event {
                    return Ok(key);
                }
            } else {
                return Err(EditorError::KeyReadFail);
            }
        }
    }
}
