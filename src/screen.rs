use crossterm::{cursor, style::Print, terminal, QueueableCommand};
use std::io::{stdout, Result, Stdout, Write};

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

    pub fn draw_rows(&mut self) -> Result<()> {
        for row in 0..self.height {
            self.stdout
                .queue(cursor::MoveTo(0, row))?
                .queue(Print("~".to_string()))?;
        }

        Ok(())
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.clear()?;
        self.draw_rows()?;
        self.stdout.queue(cursor::MoveTo(0, 0))?.flush()
    }

    pub fn clear(&mut self) -> Result<()> {
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))?
            .queue(cursor::MoveTo(0, 0))?
            .flush()
    }
}
