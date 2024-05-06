use crossterm::event::{
    KeyCode, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::terminal::ClearType;
use crossterm::{cursor, style, terminal};
use crossterm::{
    event::{Event, KeyEvent},
    tty::IsTty,
};
use std::fs::File;
use std::io::Write;
use std::io::{stdin, stdout};
use std::num::NonZeroUsize;

use crate::equations::parse::{parse_equation, Token};
use crate::equations::subscript_util;
use crate::periodic_table::PeriodicTable;
mod equations;
mod periodic_table;
mod tui;

fn main() {
    if !stdout().is_tty() {
        todo!();
    }

    crossterm::terminal::enable_raw_mode().unwrap();

    crossterm::execute!(
        stdout(),
        cursor::Hide,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .unwrap();

    let mut s = String::new();

    let mut is_subscript = false;

    let p = PeriodicTable::from_json(File::open("./PeriodicTableJSON.json").unwrap()).unwrap();
    let mut tokens: Vec<equations::parse::Token> = vec![];

    let mut symbols = String::new();

    loop {
        match crossterm::event::read().unwrap() {
            Event::Key(KeyEvent { code, .. }) => match code {
                KeyCode::Esc => {
                    crossterm::terminal::disable_raw_mode().unwrap();
                    break;
                }
                KeyCode::Backspace => {
                    crossterm::execute!(
                        stdout(),
                        cursor::MoveLeft(1),
                        style::Print(" "),
                        cursor::MoveLeft(1)
                    )
                    .unwrap();

                    if let Some(Token::Element { subscript, .. }) = tokens.last_mut() {
                        if subscript.get() > 1 {
                            *subscript = NonZeroUsize::new(1).unwrap();
                            continue;
                        }
                    }
                    tokens.pop();
                    symbols.pop();
                }
                KeyCode::Char(mut ch) => {
                    if ch.is_whitespace() {
                        crossterm::execute!(stdout(), style::Print(ch)).unwrap();
                        continue;
                    }
                    if ch == '`' {
                        is_subscript = true;
                        continue;
                    }
                    if ch == '+' {
                        if !symbols.is_empty() {
                            tokens.push(Token::Element {
                                subscript: NonZeroUsize::new(1).unwrap(),
                                element: p.by_symbol(&symbols).unwrap().number,
                            });
                            symbols.clear();
                        }

                        tokens.push(Token::Plus);
                        crossterm::execute!(stdout(), style::Print(ch)).unwrap();
                        continue;
                    }
                    if ch == '=' {
                        if !symbols.is_empty() {
                            tokens.push(Token::Element {
                                subscript: NonZeroUsize::new(1).unwrap(),
                                element: p.by_symbol(&symbols).unwrap().number,
                            });
                            symbols.clear();
                        }
                        tokens.push(Token::Arrow);
                        crossterm::execute!(stdout(), style::Print(ch)).unwrap();
                        continue;
                    }
                    if is_subscript {
                        is_subscript = false;
                        if ch.is_numeric() {
                            let digit = ch.to_digit(10).unwrap();
                            ch = subscript_util(digit);

                            if let Some(Token::Element { subscript, .. }) = tokens.last_mut() {
                                *subscript = NonZeroUsize::new(digit as usize).unwrap();
                            } else {
                                tokens.push(Token::Element {
                                    subscript: NonZeroUsize::new(digit as usize).unwrap(),
                                    element: p.by_symbol(&symbols).unwrap().number,
                                });
                                symbols.clear();
                            }
                        }
                    } else {
                        if ch.is_alphabetic() {
                            symbols.push(ch);
                        } else {
                            tokens.push(Token::Element {
                                subscript: NonZeroUsize::new(1).unwrap(),
                                element: p.by_symbol(&symbols).unwrap().number,
                            });
                            symbols.clear();
                        }
                    }
                    crossterm::execute!(stdout(), style::Print(ch)).unwrap();
                }
                KeyCode::Enter => {
                    if !symbols.is_empty() {
                        tokens.push(Token::Element {
                            subscript: NonZeroUsize::new(1).unwrap(),
                            element: p.by_symbol(&symbols).unwrap().number,
                        });
                        symbols.clear();
                    }

                    crossterm::execute!(stdout(), cursor::MoveToNextLine(1), style::Print("> "))
                        .unwrap();
                    crossterm::terminal::disable_raw_mode().unwrap();
                    let eqn = parse_equation(tokens.into_iter()).unwrap().balanced().unwrap();
                    panic!("{}", eqn.to_string(&p).unwrap());
                }
                _ => (),
            },
            _ => (),
        }
    }

    println!("Hello, world!");
}
