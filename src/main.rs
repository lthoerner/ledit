use std::io::{stdout, Read, Stdout, Write};

use crossterm::cursor;
use crossterm::event;
use crossterm::style;
use crossterm::terminal;
use crossterm::{execute, queue};

fn main() {
    prompt("$ ");
}

fn prompt(prefix: &str) -> String {
    let forbidden_area = prefix.chars().count() as u16;
    let mut line_buffer = String::new();

    terminal::enable_raw_mode().unwrap();
    let mut stdout = stdout();

    eprint!("{}", prefix);
    loop {
        if handle(&mut stdout, &mut line_buffer, event::read().unwrap()) {
            terminal::disable_raw_mode().unwrap();
            return line_buffer;
        }
    }
}

fn handle(stdout: &mut Stdout, line: &mut String, event: event::Event) -> bool {
    match event {
        event::Event::Key(key_event) => {
            if key_event.modifiers == event::KeyModifiers::NONE {
                match key_event.code {
                    event::KeyCode::Char(c) => {
                        execute!(stdout, style::Print(c)).unwrap();
                        line.push(c);
                    }
                    event::KeyCode::Backspace => {
                        execute!(
                            stdout,
                            cursor::MoveLeft(1),
                            style::Print(" "),
                            cursor::MoveLeft(1)
                        )
                        .unwrap();
                        line.pop();
                    }
                    event::KeyCode::Left => execute!(stdout, cursor::MoveLeft(1)).unwrap(),
                    event::KeyCode::Right => execute!(stdout, cursor::MoveRight(1)).unwrap(),
                    event::KeyCode::Enter => {
                        execute!(stdout, cursor::MoveToNextLine(1)).unwrap();
                        return true;
                    }
                    _ => panic!("UNHANDLED KEY"),
                }
            } else {
                match (key_event.modifiers, key_event.code) {
                    _ => panic!("NO MODIFIERS SUPPORTED YET"),
                }
            }
        }
        event::Event::Mouse(_) => panic!("MOUSE CAPTURE SHOULD BE DISABLED"),
        event::Event::Resize(_, _) => todo!(), // Need to reflow the text most likely
        event::Event::FocusGained => (),
        event::Event::FocusLost => (),
        event::Event::Paste(_) => panic!("BRACKETED PASTE SHOULD BE DISABLED"),
    }

    false
}
