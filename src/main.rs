use std::io::{stdout, stdin};
use std::io::Write;
use crossterm::event::KeyCode;
use crossterm::{cursor, style, terminal};
use crossterm::{tty::IsTty, event::{Event, KeyEvent}};

fn main() {

    if !stdout().is_tty() {
        todo!();
    }

    crossterm::terminal::enable_raw_mode().unwrap();

    crossterm::execute!(stdout(), cursor::Hide, terminal::Clear(terminal::ClearType::All)).unwrap();

    let mut s = String::new();
    loop {
        match crossterm::event::read().unwrap() {
            Event::Key(KeyEvent {
                code, ..
            }) => {
                if let KeyCode::Backspace = code {
                    crossterm::terminal::disable_raw_mode().unwrap();
                    break;
                }
                crossterm::execute!(stdout(), terminal::Clear(terminal::ClearType::CurrentLine), cursor::MoveTo(0, 0), style::Print(format!("{:?}", code))).unwrap();
            }
            _ => ()
        }
    }


    println!("Hello, world!");
}
