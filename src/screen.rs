use crossterm::{
    cursor,
    style::{Color, Print, Stylize},
    terminal, QueueableCommand,
};
use std::io::{stdout, Result, Stdout, Write};

use kilo_rust::Position;

pub struct Screen {
    width: u16,
    height: u16,
    stdout: Stdout,
}

impl Screen {
    pub fn new() -> Result<Self> {
        let (columns, mut rows) = crossterm::terminal::size()?;
        rows -= 1;
        Ok(Self {
            width: columns,
            height: rows,
            stdout: stdout(),
        })
    }

    pub fn draw_rows(&mut self, rows: &[String], row_offset: u16, col_offset: u16) -> Result<()> {
        const VERSION: &str = env!("CARGO_PKG_VERSION");

        for row in 0..self.height {
            let file_row = (row + row_offset) as usize;
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
                let mut len = rows[file_row].len();
                if len < col_offset as usize {
                    continue;
                }
                len -= col_offset as usize;

                let start = col_offset as usize;
                let end = match len {
                    x if x > self.width as usize => self.width as usize + start,
                    _ => len + start,
                };
                self.stdout
                    .queue(cursor::MoveTo(0, row))?
                    .queue(Print(rows[file_row][start..end].to_string()))?;
            }
        }
        Ok(())
    }

    pub fn draw_status_bar(
        &mut self,
        rows: &[String],
        filename: &String,
        cursor_row: u16,
    ) -> Result<()> {
        let background = " ".on(Color::White);
        let name = match filename.is_empty() {
            true => String::from("[No Name]"),
            false => filename.to_string(),
        };
        let lines = format!("{}/{}", cursor_row + 1, rows.len());
        let status = format!("{name} - {} lines", rows.len());
        let status = match status.len() > self.width as usize - lines.len() {
            true => status[0..self.width as usize - lines.len() - 1].to_string(),
            false => status,
        };
        self.stdout
            .queue(cursor::MoveTo(0, self.height))?
            .queue(Print(status.clone().with(Color::Black).on(Color::White)))?
            .queue(cursor::MoveTo(self.width - lines.len() as u16, self.height))?
            .queue(Print(lines.clone().with(Color::Black).on(Color::White)))?;
        for i in status.len()..self.width as usize - lines.len() {
            self.stdout
                .queue(cursor::MoveTo(i as u16, self.height))?
                .queue(Print(background))?;
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

    pub fn move_to(&mut self, pos: &Position, row_offset: u16, col_offset: u16) -> Result<()> {
        self.stdout
            .queue(cursor::MoveTo(pos.x - col_offset, pos.y - row_offset))?;
        Ok(())
    }

    pub fn bounds(&self) -> Position {
        Position {
            x: self.width,
            y: self.height,
        }
    }
}
