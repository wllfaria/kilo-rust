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
        const VERSION: &str = env!("CARGO_PKG_VERSION");

        for row in 0..self.height {
            if row == self.height / 3 {
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
        }
        self.stdout.queue(cursor::MoveTo(0, 0))?;
        Ok(())
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.clear()?;
        self.draw_rows()?;
        self.stdout.queue(cursor::MoveTo(0, 0))?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))?
            .queue(cursor::MoveTo(0, 0))?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.stdout.flush()
    }
}
