use crossterm::{
    cursor,
    event::{read, KeyCode, KeyEvent, KeyModifiers},
    style::Print,
    terminal, QueueableCommand,
};
use errno::errno;
use std::io::{stdout, Result, Stdout, Write};

use crate::*;

pub struct Editor {
    width: u16,
    height: u16,
    stdout: Stdout,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let (columns, rows) = crossterm::terminal::size()?;
        Ok(Self {
            width: columns,
            height: rows,
            stdout: stdout(),
        })
    }

    pub fn process_keypress(&mut self) -> bool {
        let c = self.read_key();

        matches!(
            c,
            Ok(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                ..
            })
        )
    }

    pub fn read_key(&mut self) -> Result<KeyEvent> {
        loop {
            if let Ok(event) = read() {
                if let Key(key) = event {
                    return Ok(key);
                }
            } else {
                self.die("read");
                break;
            }
        }
        unreachable!();
    }

    pub fn draw_rows(&mut self) -> Result<()> {
        for row in 0..self.height {
            self.stdout
                .queue(cursor::MoveTo(0, row))?
                .queue(Print("~".to_string()))?;
        }

        Ok(())
    }

    pub fn clear_screen(&mut self) -> Result<()> {
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))?
            .queue(cursor::MoveTo(0, 0))?
            .flush()
    }

    pub fn refresh_screen(&mut self) -> Result<()> {
        self.clear_screen()?;
        self.draw_rows()?;

        self.stdout.queue(cursor::MoveTo(0, 0))?.flush()
    }

    // TODO: Die shouldn't belong to editor
    pub fn die<S: Into<String>>(&mut self, message: S) {
        let _ = self.clear_screen();
        let _ = terminal::disable_raw_mode();
        eprintln!("{}: {}", message.into(), errno());
        std::process::exit(1);
    }
}
