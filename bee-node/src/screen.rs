// TODO: handle proper screen exit when pressing CTRL-C

use bee_common::constants::{BEE_DISPLAYED_NAME, BEE_DISPLAYED_VERSION};

use std::io::{stdout, Write};

use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::cursor::{Hide, Show, MoveTo};
use crossterm::{execute, ExecutableCommand};
use crossterm::style::{SetForegroundColor, SetBackgroundColor, Color, ResetColor, Print};

pub fn init() {
    execute!(std::io::stdout(), EnterAlternateScreen).expect("error entering alternate screen");
    execute!(std::io::stdout(), Hide).expect("error hiding cursor");
    execute!(std::io::stdout(), MoveTo(0, 0)).expect("error moving cursor");

    header().expect("error printing logo");

    execute!(std::io::stdout(), MoveTo(0, 2)).expect("error moving cursor");
}

pub fn exit() {
    execute!(std::io::stdout(), LeaveAlternateScreen).expect("error leaving alternate screen");
    execute!(std::io::stdout(), Show).expect("error showing cursor");
}

fn header() -> crossterm::Result<()> {
    stdout()
    .execute(SetForegroundColor(Color::Black))?
    .execute(SetBackgroundColor(Color::Yellow))?
    .execute(Print(format!("IOTA Foundation, {} Version {}", BEE_DISPLAYED_NAME, BEE_DISPLAYED_VERSION)))?
    .execute(ResetColor)?;

    Ok(())
}