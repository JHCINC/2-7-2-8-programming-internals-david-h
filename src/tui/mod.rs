use std::fmt::Display;
use std::io::stdout;
use std::num::NonZeroUsize;

use anyhow::{anyhow, bail};
use crossterm::event::{KeyCode, KeyModifiers};

use crossterm::event::{Event, KeyEvent};
use crossterm::{cursor, style, terminal};

use crate::equations::parse::{parse_equation, Token};
use crate::equations::{Component, ComponentType};
use crate::periodic_table::PeriodicTable;

#[derive(Clone, Debug)]
pub enum TUIToken {
    Coefficient(String),
    Element(String),
    Subscript(String),
    Plus,
    Equals,
    LParen,
    RParen,
    Whitespace(usize),
}

pub enum ShouldExit {
    Yes,
    No,
}
pub struct TUIAcceptor<'a> {
    looking_for_subscript: bool,
    tokens: Vec<TUIToken>,
    table: &'a PeriodicTable,
    size: (u16, u16),
}

impl<'a> TUIAcceptor<'a> {
    pub fn new(p: &'a PeriodicTable) -> Self {
        crossterm::terminal::enable_raw_mode().unwrap();

        let size = crossterm::terminal::size().expect("Should be able to get size");
        crossterm::execute!(
            stdout(),
            cursor::SetCursorStyle::BlinkingUnderScore,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, size.1 - 2)
        )
        .unwrap();

        Self {
            looking_for_subscript: false,
            tokens: vec![],
            table: p,
            size,
        }
    }

    pub fn handle_event(&mut self, e: Event) -> anyhow::Result<ShouldExit> {
        match e {
            Event::Resize(w, h) => {
                self.size.0 = w;
                self.size.1 = h;
            }
            Event::Key(k) => return self.handle_key_event(k),
            _ => (),
        }
        Ok(ShouldExit::No)
    }

    pub fn handle_key_event(&mut self, k: KeyEvent) -> anyhow::Result<ShouldExit> {
        match k.code {
            KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                return Ok(ShouldExit::Yes);
            }
            KeyCode::Char('`') => self.subscript_start(),
            KeyCode::Backspace => self.backspace()?,
            KeyCode::Enter => {
                if let Err(e) = self.handle_enter() {
                    self.emit_str(e)?;
                }
            }
            KeyCode::Char('+') => self.handle_token('+', TUIToken::Plus)?,
            KeyCode::Char('=') => self.handle_token('=', TUIToken::Equals)?,
            KeyCode::Char('(') => self.handle_token('(', TUIToken::LParen)?,
            KeyCode::Char(')') => self.handle_token(')', TUIToken::RParen)?,
            KeyCode::Char(c) if c.is_whitespace() => self.handle_whitespace(c)?,
            KeyCode::Char(c) => self.accept_char(c)?,
            _ => (),
        }
        Ok(ShouldExit::No)
    }

    fn emit_str(&mut self, d: impl Display) -> anyhow::Result<()> {
        crossterm::execute!(
            stdout(),
            style::Print("> "),
            style::Print(d),
            terminal::ScrollUp(1),
            cursor::MoveTo(0, self.size.1 - 2)
        )
        .unwrap();
        Ok(())
    }

    fn handle_enter(&mut self) -> anyhow::Result<()> {
        let mut tokens = vec![];
        let mut stream = std::mem::take(&mut self.tokens).into_iter().peekable();

        let mut component_stack: Vec<ComponentType> = vec![];
        let mut is_parenthesised = false;
        while let Some(t) = stream.next().clone() {
            match t {
                TUIToken::LParen => {
                    if is_parenthesised {
                        bail!("Already parenthesised");
                    }
                    component_stack.push(ComponentType::Multiple(
                        vec![],
                        NonZeroUsize::new(1).unwrap(),
                    ));
                    is_parenthesised = true;
                }
                TUIToken::RParen => {
                    is_parenthesised = false;
                }
                TUIToken::Element(elem) => {
                    let subscript = NonZeroUsize::new(1).unwrap();
                    let element = self
                        .table
                        .by_symbol(&elem)
                        .ok_or(anyhow!("element {elem} nonexistent"))?
                        .number;
                    if !is_parenthesised {
                        component_stack
                            .push(ComponentType::Element(Component { element, subscript }))
                    } else if let Some(ComponentType::Multiple(vec, _)) = component_stack.last_mut()
                    {
                        vec.push(Component { element, subscript });
                    } else {
                        unreachable!()
                    }
                }
                TUIToken::Subscript(n) => {
                    let subscript =
                        NonZeroUsize::new(n.parse()?).ok_or(anyhow!("Zero subscript"))?;

                    if let Some(last) = component_stack.last_mut() {
                        match last {
                            ComponentType::Element(c) => {
                                c.subscript = subscript;
                            }
                            ComponentType::Multiple(vals, sub) => {
                                if !is_parenthesised {
                                    *sub = subscript;
                                } else {
                                    vals.last_mut()
                                        .ok_or(anyhow!("subscript without preceding"))?
                                        .subscript = subscript;
                                }
                            }
                        }
                    } else {
                        bail!("subscript without preceding");
                    }
                }
                t => {
                    for t in std::mem::take(&mut component_stack) {
                        tokens.push(Token::Component(t));
                    }
                    match t {
                        TUIToken::Coefficient(n) => tokens.push(Token::Coefficient(
                            NonZeroUsize::new(n.parse()?)
                                .ok_or(anyhow::anyhow!("Zero coefficient"))?,
                        )),

                        TUIToken::Equals => tokens.push(Token::Arrow),
                        TUIToken::Plus => tokens.push(Token::Plus),
                        TUIToken::Whitespace(_) => (),
                        _ => (),
                    }
                }
            }
        }
        for t in std::mem::take(&mut component_stack) {
            tokens.push(Token::Component(t));
        }

        let eqn = parse_equation(tokens.into_iter())?;
        crossterm::execute!(
            stdout(),
            terminal::ScrollUp(1),
            cursor::MoveTo(0, self.size.1 - 2)
        )
        .unwrap();

        self.emit_str(eqn.balanced()?.to_string(self.table)?)?;

        Ok(())
        // self.close();
        // panic!("{}", eqn.to_string(self.table)?);
        // Ok(())
    }

    fn handle_token(&mut self, c: char, token: TUIToken) -> anyhow::Result<()> {
        self.tokens.push(token);
        self.emit(c)
    }

    fn handle_whitespace(&mut self, c: char) -> anyhow::Result<()> {
        if let Some(TUIToken::Whitespace(n)) = self.tokens.last_mut() {
            *n += 1;
        } else {
            self.tokens.push(TUIToken::Whitespace(1));
        }

        self.looking_for_subscript = false;
        self.emit(c)
    }

    fn accept_char(&mut self, c: char) -> anyhow::Result<()> {
        if self.looking_for_subscript && !c.is_numeric() {
            self.looking_for_subscript = false;
        }

        match self.tokens.last_mut() {
            // matches the case we:
            // # have an element token preceding
            // # are looking for a subscript token OR
            // # the subscript is empty (we are adding
            //   to the element string & the next letter
            //   is lowercase. If it is not, it's a new
            //   element).
            Some(TUIToken::Element(elem)) if c.is_lowercase() => {
                // preceding element token
                elem.push(c);
                self.emit(c)?;
            }
            Some(TUIToken::Subscript(subscript)) if self.looking_for_subscript => {
                subscript.push(c);
                self.emit(subscript_util(c.to_digit(10).unwrap()))?;
            }
            Some(TUIToken::Coefficient(s)) if c.is_numeric() => {
                s.push(c);
                self.emit(c)?;
            }
            _ => {
                // no notable preceding token

                if c.is_numeric() {
                    if self.looking_for_subscript {
                        self.emit(subscript_util(c.to_digit(10).unwrap()))?;
                        self.tokens.push(TUIToken::Subscript(c.to_string()));
                    } else {
                        self.emit(c)?;
                        self.tokens.push(TUIToken::Coefficient(c.to_string()));
                    }
                } else if c.is_alphabetic() {
                    self.emit(c)?;
                    self.tokens.push(TUIToken::Element(c.to_string()))
                }
            }
        }

        Ok(())
    }

    fn subscript_start(&mut self) {
        self.looking_for_subscript = true;
    }

    fn backspace(&mut self) -> anyhow::Result<()> {
        crossterm::execute!(
            stdout(),
            cursor::MoveLeft(1),
            style::Print(" "),
            cursor::MoveLeft(1)
        )?;
        let should_remove = if let Some(last) = self.tokens.last_mut() {
            match last {
                TUIToken::Whitespace(n) => {
                    *n -= 1;
                    *n == 0
                }
                TUIToken::Element(element) => {
                    element.pop();
                    element.is_empty()
                }
                TUIToken::Subscript(subscript) => {
                    subscript.pop();
                    subscript.is_empty()
                }
                _ => true,
            }
        } else {
            false
        };

        if should_remove {
            self.tokens.pop();
        }

        Ok(())
    }

    fn emit(&self, c: char) -> anyhow::Result<()> {
        crossterm::execute!(stdout(), style::Print(c))?;
        Ok(())
    }

    fn close(&self) {
        crossterm::execute!(stdout(), cursor::MoveToNextLine(1), style::Print("> ")).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
    }
}

impl<'a> Drop for TUIAcceptor<'a> {
    fn drop(&mut self) {
        self.close();
    }
}

pub fn subscript_util(digit: u32) -> char {
    assert!(digit < 10); // only digits from 0 to 9
    char::from_u32('\u{2080}' as u32 + digit).unwrap()
}
pub fn subscript_num(num: u32) -> String {
    let mut s = String::new();

    let num_digits = num.ilog10() + 1;
    let nth_digit = |n| (num / 10u32.pow(num_digits - n)) % 10;

    for i in 0..num_digits {
        s.push(subscript_util(nth_digit(i + 1)));
    }
    s
}

#[cfg(test)]
mod tests {
    use crate::tui::subscript_num;

    #[test]
    fn subscript_test() {
        assert_eq!(subscript_num(12), "₁₂");
    }
}
