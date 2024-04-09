use std::io::{stdout, stdin};
use std::io::Write;
use crossterm::event::KeyCode;
use crossterm::{cursor, style, terminal};
use crossterm::{tty::IsTty, event::{Event, KeyEvent}};
mod periodic_table;
mod equations;
fn main() {

    if !stdout().is_tty() {
        todo!();
    }

    crossterm::terminal::enable_raw_mode().unwrap();

    crossterm::execute!(stdout(), cursor::Hide, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0)).unwrap();

    let mut s = String::new();
    loop {
        match crossterm::event::read().unwrap() {
            Event::Key(KeyEvent {
                code, ..
            }) => {
                match code {
                    KeyCode::Esc => {
                        crossterm::terminal::disable_raw_mode().unwrap();
                        break;
                    }
                    KeyCode::Char(ch) => {
                        crossterm::execute!(stdout(), style::Print(ch)).unwrap();
                    }
                    KeyCode::Enter => {
                        crossterm::execute!(stdout(), cursor::MoveToNextLine(1), style::Print("> ")).unwrap();
                    }
                    _ => ()
                }
            }
            _ => ()
        }
    }


    println!("Hello, world!");
}
