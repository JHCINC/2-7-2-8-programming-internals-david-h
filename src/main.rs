use crossterm::tty::IsTty;
use std::fs::File;

use std::io::stdout;

use crate::periodic_table::PeriodicTable;
use crate::tui::{ShouldExit, TUIAcceptor};
mod equations;
mod periodic_table;
mod tui;

fn main() {
    if !stdout().is_tty() {
        todo!();
    }

    let _s = String::new();

    let _is_subscript = false;

    let p = PeriodicTable::from_json(File::open("./PeriodicTableJSON.json").unwrap()).unwrap();
    let _tokens: Vec<equations::parse::Token> = vec![];

    let _symbols = String::new();

    let mut tui = TUIAcceptor::new(&p);
    while let ShouldExit::No = tui.handle_event(crossterm::event::read().unwrap()).unwrap() {
        // do nothing
    }
}
