use std::io::stdout;
use std::num::NonZeroUsize;

use crossterm::event::{
    KeyCode, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::terminal::ClearType;
use crossterm::{cursor, style, terminal};
use crossterm::{
    event::{Event, KeyEvent},
    tty::IsTty,
};
use anyhow::anyhow;

use crate::equations::Equation;
use crate::equations::parse::{Token, parse_equation};
use crate::periodic_table::PeriodicTable;

#[derive(Clone, Debug)]
pub enum TUIToken {
    Coefficient(String),
    Element { element: String, subscript: String },
    Plus,
    Equals,
    Whitespace(usize),
}

pub struct TUIAcceptor<'a> {
    looking_for_subscript: bool,
    tokens: Vec<TUIToken>,
    table: &'a PeriodicTable,
}

impl<'a> TUIAcceptor<'a> {
    pub fn new(p: &'a PeriodicTable) -> Self {
        crossterm::terminal::enable_raw_mode().unwrap();

        crossterm::execute!(
            stdout(),
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )
        .unwrap();

        Self {
            looking_for_subscript: false,
            tokens: vec![],
            table: p,
        }
    }

    pub fn handle_key_event(&mut self, k: KeyEvent) -> anyhow::Result<()> {
        match k.code {
            KeyCode::Char('`') => self.subscript_start(),
            KeyCode::Backspace => self.backspace()?,
            KeyCode::Enter => self.handle_enter()?,
            KeyCode::Char('+') => self.handle_token('+', TUIToken::Plus)?,
            KeyCode::Char('=') => self.handle_token('=', TUIToken::Equals)?,
            KeyCode::Char(c) if c.is_whitespace() => self.handle_whitespace(c)?,
            KeyCode::Char(c) => self.accept_char(c)?,
            _ => (),
        }
        Ok(())
    }

    fn tui_token_process(&self, t: TUIToken) -> anyhow::Result<Option<Token>> {
        match t {
            TUIToken::Coefficient(n) => Ok(Some(Token::Coefficient(
                NonZeroUsize::new(n.parse()?).ok_or(anyhow::anyhow!("Zero coefficient"))?,
            ))),
            TUIToken::Element { element, subscript } => Ok(Some({
                let subscript: usize = if subscript.is_empty() {
                    1
                } else {
                    subscript.parse()?
                };

                let subscript = NonZeroUsize::new(subscript).ok_or(anyhow!("zero subscript"))?;
                let element = self.table.by_symbol(&element).ok_or(anyhow!("element {element} nonexistent"))?.number;
                Token::Element { subscript, element }
            })),
            TUIToken::Equals => Ok(Some(Token::Arrow)),
            TUIToken::Plus => Ok(Some(Token::Plus)),
            TUIToken::Whitespace(_) => Ok(None),
        }
    }

    fn handle_enter(&mut self) -> anyhow::Result<()> {
        let mut tokens = vec![];
        for v in std::mem::take(&mut self.tokens) {
            if let Some(v) = self.tui_token_process(v)? {
                tokens.push(v);
            }
        }
        
        let eqn = parse_equation(tokens.into_iter())?.balanced()?;
        self.close();
        panic!("{}", eqn.to_string(self.table)?);
        Ok(())
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
            Some(TUIToken::Element { subscript, element })
                if self.looking_for_subscript || (subscript.is_empty() && c.is_lowercase()) =>
            {
                // preceding element token

                if self.looking_for_subscript {
                    subscript.push(c);
                    self.emit(subscript_util(c.to_digit(10).unwrap()))?;
                } else {
                    element.push(c);
                    self.emit(c)?;
                }
            }
            Some(TUIToken::Coefficient(s)) if c.is_numeric() => {
                s.push(c);
                self.emit(c)?;
            }
            _ => {
                // no notable preceding token
                self.emit(c)?;
                if c.is_numeric() {
                    self.tokens.push(TUIToken::Coefficient(c.to_string()));
                } else if c.is_alphabetic() {
                    self.tokens.push(TUIToken::Element {
                        subscript: "".to_string(),
                        element: c.to_string(),
                    })
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
                TUIToken::Element { subscript, element } => {
                    if subscript.pop().is_none() {
                        element.pop();
                        element.is_empty()
                    } else {
                        false
                    }
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
