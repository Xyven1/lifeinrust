use rand::Rng;
use std::io::{stdin, stdout, Write};
use std::sync::mpsc::channel;
use std::sync::{Arc, RwLock};
use std::{thread, time};
use termion::cursor::Goto;
use termion::{clear, cursor, event, input::TermRead, raw::IntoRawMode, terminal_size};

fn main() {
    let mut rng = rand::thread_rng();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let size = terminal_size().unwrap();

    let mut table = vec![vec![0; size.0.into()]; size.1.into()];

    let (te, re) = channel();
    let running = Arc::new(RwLock::new(true));
    let events = stdin().events();
    let running_ = running.clone();
    let event_loop = thread::spawn(move || {
        for event in events {
            let e = event.unwrap();
            te.send(e.clone()).unwrap();
            match e {
                event::Event::Key(event::Key::Ctrl('c')) => break,
                _ => {}
            }
            if !*running_.read().unwrap() {
                break;
            }
        }
    });
    write!(stdout, "{}{}{}", cursor::Hide, Goto(1, 1), clear::All).unwrap();
    while *running.read().unwrap() {
        for event in re.try_iter() {
            match event {
                event::Event::Key(event::Key::Ctrl('c')) => {
                    *running.write().unwrap() = false;
                }
                _ => {}
            }
        }
        write!(stdout, "{}{}", cursor::Goto(1, 1), clear::UntilNewline).unwrap();
        //iteration
        let prev = table.clone();
        for row in 0..table.len() {
            for col in 0..table[row].len() {
                let rand: bool = rng.gen();
                table[row][col] = rand.into();
            }
        }
        //display
        for row in table.iter() {
            for cell in row.iter() {
                write!(stdout, "{}", if *cell > 0 { "⬜" } else { "⬛" }).unwrap();
            }
            write!(stdout, "\n\r").unwrap();
        }
        //delay
        thread::sleep(time::Duration::from_millis(500));
    }
    event_loop.join().unwrap();
    write!(stdout, "{}{}{}", clear::All, Goto(1, 1), cursor::Show).unwrap();
    return;
}
