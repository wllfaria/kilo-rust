use crossterm::{cursor, style::Print, terminal, QueueableCommand};
use std::io::{stdout, Result, Stdout, Write};

use kilo_rust::Position;

pub struct Screen {
    width: u16,
    height: u16,
    stdout: Stdout,
}

impl Screen {
    pub fn new() -> Result<Self> {
        let (columns, rows) = crossterm::terminal::size()?;
        Ok(Self {
            width: columns,
            height: rows,
            stdout: stdout(),
        })
    }

    pub fn draw_rows(&mut self, rows: &[String], offset: usize) -> Result<()> {
        const VERSION: &str = env!("CARGO_PKG_VERSION");

        for row in 0..self.height {
            let file_row = row as usize + offset;
            if row >= rows.len() as u16 {
                if rows.is_empty() && row == self.height / 3 {
                    let mut welcome = format!("Kilo editor -- version {VERSION}");
                    welcome.truncate(self.width as usize);

                    if welcome.len() < self.width as usize {
                        let padding = ((self.width - welcome.len() as u16) / 2) as u16;
                        self.stdout
                            .queue(cursor::MoveTo(0, row))?
                            .queue(Print("~".to_string()))?
                            .queue(cursor::MoveTo(padding, row))?
                            .queue(Print(welcome))?;
                    } else {
                        self.stdout
                            .queue(cursor::MoveTo(0, row))?
                            .queue(Print(welcome))?;
                    }
                } else {
                    self.stdout
                        .queue(cursor::MoveTo(0, row))?
                        .queue(Print("~".to_string()))?;
                }
            } else {
                let len = rows[file_row].len().min(self.width as usize);
                self.stdout
                    .queue(cursor::MoveTo(0, row))?
                    .queue(Print(rows[file_row][0..len].to_string()))?;
            }
        }
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.stdout.flush()
    }

    pub fn move_to(&mut self, pos: &Position, offset: usize) -> Result<()> {
        self.stdout
            .queue(cursor::MoveTo(pos.x, pos.y - offset as u16))?;
        Ok(())
    }

    pub fn bounds(&self) -> Position {
        Position {
            x: self.width,
            y: self.height,
        }
    }
}
