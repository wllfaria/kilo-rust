use crate::*;
use crossterm::event::{read, KeyEvent};

pub fn editor_read_key() -> Result<KeyEvent> {
    loop {
        if let Ok(event) = read() {
            if let Key(key) = event {
                return Ok(key);
            }
        } else {
            die("read");
            break;
        }
    }
    unreachable!();
}
