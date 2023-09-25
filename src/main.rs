use std::io::{stdout, Stdout};

use crossterm::cursor;
use crossterm::event;
use crossterm::style;
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
    terminal::enable_raw_mode().unwrap();

    let mut line_buffer = LineBuffer::default();
    let mut ctx = Context {
        stdout: stdout(),
        y_origin: cursor::position().unwrap().1,
        prompt_width: prefix.chars().count(),
    };

    execute!(ctx.stdout, style::Print(prefix)).unwrap();
    loop {
        if handle(&mut ctx, &mut line_buffer, event::read().unwrap()) {
            terminal::disable_raw_mode().unwrap();
            execute!(ctx.stdout, style::Print("\n")).unwrap();
            return line_buffer.buffer;
        }
    }
}

fn handle(ctx: &mut Context, line: &mut LineBuffer, event: event::Event) -> bool {
    match event {
        event::Event::Key(key_event) => {
            if key_event.modifiers == event::KeyModifiers::NONE
            // $ Need to split SHIFT off from NONE
                || key_event.modifiers == event::KeyModifiers::SHIFT
            {
                match key_event.code {
                    event::KeyCode::Char(c) => {
                        line.insert(c);
                        redraw_buffer(ctx, line);
                    }
                    event::KeyCode::Backspace => {
                        line.backspace();
                        redraw_buffer(ctx, line);
                    }
                    event::KeyCode::Delete => {
                        line.delete();
                        redraw_buffer(ctx, line);
                    }
                    event::KeyCode::Left => {
                        line.left();
                        update_cursor(ctx, line);
                    }
                    event::KeyCode::Right => {
                        line.right();
                        update_cursor(ctx, line);
                    }
                    event::KeyCode::Enter => {
                        return true;
                    }
                    _ => exit(1, "UNHANDLED KEY"),
                }
            } else {
                #[allow(clippy::match_single_binding)]
                match (key_event.modifiers, key_event.code) {
                    _ => exit(1, "NO MODIFIERS SUPPORTED YET"),
                }
            }
        }
        event::Event::Mouse(_) => exit(1, "MOUSE CAPTURE SHOULD BE DISABLED"),
        event::Event::Resize(_, _) => todo!(), // Need to reflow the text most likely
        event::Event::FocusGained => (),
        event::Event::FocusLost => (),
        event::Event::Paste(_) => exit(1, "BRACKETED PASTE SHOULD BE DISABLED"),
    }

    false
}

fn redraw_buffer(ctx: &mut Context, line: &LineBuffer) {
    update_cursor(ctx, line);
    execute!(ctx.stdout, cursor::SavePosition,).unwrap();
    queue!(
        ctx.stdout,
        terminal::Clear(terminal::ClearType::FromCursorDown),
        // $ This will cause a bug if the prompt is wider than the terminal, need to divide
        cursor::MoveTo(ctx.prompt_width as u16, ctx.y_origin),
        style::Print(&line.buffer)
    )
    .unwrap();
    execute!(ctx.stdout, cursor::RestorePosition).unwrap();
}

fn update_cursor(ctx: &mut Context, line: &LineBuffer) {
    let (x, y) = cursor_index_to_coord(ctx, line.cursor_index);
    execute!(ctx.stdout, cursor::MoveTo(x, y)).unwrap();
}

// $ This will probably cause a bug when the terminal is resized or when the user scrolls
fn cursor_index_to_coord(ctx: &mut Context, cursor_index: usize) -> (u16, u16) {
    let (terminal_width, _) = terminal::size().unwrap();
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
