//! Command prompt for xi-term. currently this is
//! heavily inspired by vim and is just designed to
//! get a simple base to work off of.

use std::io::Error;
use std::io::Write;
use termion::event::{Event, Key};

use crate::core::{Command, ParseCommandError, FromPrompt};
use termion::clear::CurrentLine as ClearLine;
use termion::cursor::Goto;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CommandPromptMode {
    // Parse commands from user-input
    Command,
    // Switch directly to search-mode
    Find,
}

#[derive(Debug)]
pub struct CommandPrompt {
    mode: CommandPromptMode,
    dex: usize,
    chars: String,
}

impl CommandPrompt {
    pub fn new(mode: CommandPromptMode) -> CommandPrompt {
        CommandPrompt{mode, dex: 0, chars: Default::default()}
    }

    /// Process a terminal event for the command prompt.
    pub fn handle_input(&mut self, input: &Event) -> Result<Option<Command>, ParseCommandError> {
        match input {
            Event::Key(Key::Char('\n')) => self.finalize(),
            Event::Key(Key::Backspace) | Event::Key(Key::Ctrl('h')) => Ok(self.back()),
            Event::Key(Key::Delete) => Ok(self.delete()),
            Event::Key(Key::Left) => Ok(self.left()),
            Event::Key(Key::Right) => Ok(self.right()),
            Event::Key(Key::Char(chr)) => Ok(self.new_key(*chr)),
            _ => Ok(None),
        }
    }

    fn left(&mut self) -> Option<Command> {
        if self.dex > 0 {
            self.dex -= 1;
        }
        None
    }

    fn right(&mut self) -> Option<Command> {
        if self.dex < self.chars.len() {
            self.dex += 1;
        }
        None
    }

    fn delete(&mut self) -> Option<Command> {
        if self.dex < self.chars.len() {
            self.chars.remove(self.dex);
        }
        None
    }

    fn back(&mut self) -> Option<Command> {
        if !self.chars.is_empty() {
            self.dex -= 1;
            self.chars.remove(self.dex);
            None
        } else {
            Some(Command::Cancel)
        }
    }

    /// Gets called when any character is pressed.
    fn new_key(&mut self, chr: char) -> Option<Command> {
        self.chars.insert(self.dex, chr);
        self.dex += 1;
        None
    }

    /// Gets called when return is pressed,
    fn finalize(&mut self) -> Result<Option<Command>, ParseCommandError> {
        match self.mode {
            CommandPromptMode::Find => {
                if self.chars.is_empty() {
                    Err(ParseCommandError::ExpectedArgument{cmd: "find".to_string()})
                } else {
                    Ok(Some(Command::Find(self.chars.clone())))
                }
            },
            CommandPromptMode::Command => Ok(Some(Command::from_prompt(&self.chars)?)),
        }
        
    }

    pub fn render<W: Write>(&mut self, w: &mut W, row: u16) -> Result<(), Error> {
        let mode_indicator = match self.mode {
            CommandPromptMode::Find => "find",
            CommandPromptMode::Command => "",
        };

        let cursor_start = (self.dex + 2 + mode_indicator.len()) as u16;

        if let Err(err) = write!(
            w,
            "{}{}{}:{}{}",
            Goto(1, row),
            ClearLine,
            mode_indicator,
            self.chars,
            Goto(cursor_start, row)
        ) {
            error!("failed to render status bar: {:?}", err);
        }
        Ok(())
    }
}
