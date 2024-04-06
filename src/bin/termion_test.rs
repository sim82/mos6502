use std::io::{stdin, stdout};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn main() {
    let stdin = stdin();
    let mut _stdout = stdout().into_raw_mode().unwrap();

    // write!(
    //     stdout,
    //     "{}{}q to exit. Type stuff, use alt, and so on.{}",
    //     termion::clear::All,
    //     termion::cursor::Goto(1, 1),
    //     termion::cursor::Hide
    // )
    // .unwrap();
    // stdout.flush().unwrap();

    let mut keys = stdin.keys();
    'outer: loop {
        loop {
            let Some(c) = keys.next() else { break };
            // write!(
            //     stdout,
            //     "{}{}",
            //     termion::cursor::Goto(1, 1),
            //     termion::clear::CurrentLine
            // )
            // .unwrap();

            match c.unwrap() {
                Key::Char('q') => break 'outer,
                Key::Char(c) => println!("xxx: {}", c),
                Key::Alt(c) => println!("^{}", c),
                Key::Ctrl(c) => println!("*{}", c),
                Key::Esc => println!("ESC"),
                Key::Left => println!("←"),
                Key::Right => println!("→"),
                Key::Up => println!("↑"),
                Key::Down => println!("↓"),
                Key::Backspace => println!("×"),
                _ => {}
            }
            // stdout.flush().unwrap();
        }
    }
    // write!(stdout, "{}", termion::cursor::Show).unwrap();
}
