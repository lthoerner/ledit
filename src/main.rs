use std::io::{stdout, Stdout};

use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal;
use crossterm::{execute, queue};

#[derive(Default)]
struct LineBuffer {
    buffer: String,
    cursor_index: usize,
}

struct Context {
    stdout: Stdout,
    y_origin: u16,
    prompt_width: usize,
}

impl LineBuffer {
    fn insert(&mut self, c: char) {
        // TODO: Add bounds check? Maybe unnecessary
        self.buffer.insert(self.cursor_index, c);
        self.right();
    }

    fn insert_str(&mut self, s: &str) {
        self.buffer.insert_str(self.cursor_index, s);
        for _ in 0..s.chars().count() {
            self.right();
        }
    }

    fn left(&mut self) {
        // The cursor position should never go below 0 (underflow)
        self.cursor_index = self.cursor_index.saturating_sub(1);
    }

    fn right(&mut self) {
        // The cursor position should never overrun the length of the buffer
        if self.cursor_index == self.buffer.chars().count() {
            return;
        }

        self.cursor_index = self.cursor_index.saturating_add(1);
    }

    fn backspace(&mut self) {
        // Backspace should do nothing if the cursor is at the start of the line
        if self.cursor_index == 0 {
            return;
        }

        self.left();
        self.delete();
    }

    fn delete(&mut self) {
        self.buffer.remove(self.cursor_index);
    }
}

fn main() {
    let input = prompt("$ ");
    println!("{input}");
}

fn prompt(prefix: &str) -> String {
    let mut line_buffer = LineBuffer::default();
    let mut ctx = Context {
        stdout: stdout(),
        y_origin: cursor::position().unwrap().1,
        prompt_width: prefix.chars().count(),
    };

    terminal::enable_raw_mode().unwrap();
    execute!(ctx.stdout, Print(prefix)).unwrap();
    loop {
        if handle(&mut ctx, &mut line_buffer, event::read().unwrap()) {
            terminal::disable_raw_mode().unwrap();
            execute!(ctx.stdout, Print("\n")).unwrap();
            return line_buffer.buffer;
        }
    }
}

fn handle(ctx: &mut Context, line: &mut LineBuffer, event: Event) -> bool {
    match event {
        Event::Key(key_event) => {
            if key_event.modifiers == KeyModifiers::NONE {
                match key_event.code {
                    KeyCode::Char(c) => {
                        line.insert(c);
                        redraw_buffer(ctx, line);
                    }
                    KeyCode::Backspace => {
                        line.backspace();
                        redraw_buffer(ctx, line);
                    }
                    KeyCode::Delete => {
                        line.delete();
                        redraw_buffer(ctx, line);
                    }
                    KeyCode::Left => {
                        line.left();
                        update_cursor(ctx, line);
                    }
                    KeyCode::Right => {
                        line.right();
                        update_cursor(ctx, line);
                    }
                    KeyCode::Enter => {
                        return true;
                    }
                    _ => exit(1, "UNSUPPORTED KEY"),
                }
            } else if key_event.modifiers == KeyModifiers::SHIFT {
                match key_event.code {
                    KeyCode::Char(c) => {
                        line.insert(c);
                        redraw_buffer(ctx, line);
                    }
                    KeyCode::Right => {
                        line.insert_str("DEBUG ");
                        redraw_buffer(ctx, line);
                    }
                    _ => exit(1, "UNSUPPORTED KEY COMBINATION"),
                }
            } else {
                #[allow(clippy::match_single_binding)]
                match (key_event.modifiers, key_event.code) {
                    _ => exit(1, "UNSUPPORTED KEY COMBINATION"),
                }
            }
        }
        Event::Mouse(_) => exit(1, "MOUSE CAPTURE SHOULD BE DISABLED"),
        Event::Resize(_, _) => todo!(), // Need to reflow the text most likely
        Event::FocusGained => (),
        Event::FocusLost => (),
        Event::Paste(_) => exit(1, "BRACKETED PASTE SHOULD BE DISABLED"),
    }

    false
}

fn redraw_buffer(ctx: &mut Context, line: &LineBuffer) {
    update_cursor(ctx, line);
    execute!(ctx.stdout, cursor::SavePosition).unwrap();
    queue!(
        ctx.stdout,
        terminal::Clear(terminal::ClearType::FromCursorDown),
        // $ This will cause a bug if the prompt is wider than the terminal, need to divide
        cursor::MoveTo(ctx.prompt_width as u16, ctx.y_origin),
        Print(&line.buffer)
    )
    .unwrap();
    execute!(ctx.stdout, cursor::RestorePosition).unwrap();
}

fn update_cursor(ctx: &mut Context, line: &LineBuffer) {
    let terminal_height = terminal::size().unwrap().1;
    let (x, y) = cursor_index_to_coord(ctx, line.cursor_index);
    // If the cursor must overrun the end of the terminal window, the terminal must be scrolled down
    if y > terminal_height {
        // TODO: Viewport calculations and scrolling
    }

    execute!(ctx.stdout, cursor::MoveTo(x, y)).unwrap();
}

// $ This will probably cause a bug when the terminal is resized or when the user scrolls
fn cursor_index_to_coord(ctx: &mut Context, cursor_index: usize) -> (u16, u16) {
    let terminal_width = terminal::size().unwrap().0;
    let true_position = ctx.prompt_width + cursor_index;

    let x = (true_position % terminal_width as usize) as u16;
    let y = ctx.y_origin + (true_position / terminal_width as usize) as u16;

    (x, y)
}

fn exit(code: i32, msg: &str) -> ! {
    terminal::disable_raw_mode().unwrap();
    eprintln!("\n{}", msg);
    std::process::exit(code);
}
