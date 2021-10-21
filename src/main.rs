use std::io::{stdin, stdout, Write};
use std::sync::mpsc::channel;
use std::sync::{Arc, RwLock};
use std::{io::Stdout, thread, time};
use termion::{
    clear, color, cursor, cursor::Goto, event, input::TermRead, raw::IntoRawMode, raw::RawTerminal,
    style, terminal_size,
};

fn main() {
    let mut stdout = stdout().into_raw_mode().unwrap();

    let size = terminal_size().unwrap();
    let border_buffer = 100;
    let grid_width = size.0 / 2;
    let grid_height = size.1 - 2;
    let mut grid = vec![
        vec![0; (grid_width + border_buffer * 2).into()];
        (grid_height + border_buffer * 2).into()
    ];
    let mut step = 0;

    let center_x = grid.len() / 2;
    let center_y = grid[0].len() / 2;

    grid[center_x][center_y] = 1;
    grid[center_x + 1][center_y] = 1;
    grid[center_x - 1][center_y] = 1;
    grid[center_x + 1][center_y + 1] = 1;
    grid[center_x][center_y + 2] = 1;

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
    write!(stdout, "{}{}{}", clear::All, cursor::Hide, Goto(1, 1)).unwrap();
    while *running.read().unwrap() {
        for event in re.try_iter() {
            match event {
                event::Event::Key(event::Key::Ctrl('c')) => {
                    *running.write().unwrap() = false;
                }
                _ => {}
            }
        }
        //display table
        display(
            &mut stdout,
            &grid,
            border_buffer,
            grid_width,
            grid_height,
            step,
        );
        //iteration
        let prev = grid.clone();
        for x in 0..grid.len().into() {
            for y in 0..grid[x].len() {
                let live_neighbors = count_live_neighbors(&prev, x, y);
                //rules of conway's game of life
                if prev[x][y] == 1 {
                    if live_neighbors < 2 || live_neighbors > 3 {
                        grid[x][y] = 0;
                    }
                } else {
                    if live_neighbors == 3 {
                        grid[x][y] = 1;
                    }
                }
            }
        }
        step += 1;
        //delay
        thread::sleep(time::Duration::from_millis(500));
    }
    event_loop.join().unwrap();
    write!(
        stdout,
        "{}{}{}",
        clear::All,
        style::Reset,
        cursor::Goto(1, 1)
    )
    .unwrap();
    return;
}

fn count_live_neighbors(grid: &Vec<Vec<i32>>, row: usize, col: usize) -> i32 {
    let mut count = 0;
    for i in -1..2 {
        for j in -1..2 {
            if i == 0 && j == 0 {
                continue;
            }
            let r = (row as i32 + i) as usize;
            let c = (col as i32 + j) as usize;
            if r < grid.len() && c < grid[r].len() {
                if grid[r][c] > 0 {
                    count += 1;
                }
            }
        }
    }
    return count;
}

fn display(
    stdout: &mut RawTerminal<Stdout>,
    grid: &Vec<Vec<i32>>,
    border_buffer: u16,
    grid_width: u16,
    grid_height: u16,
    step: usize,
) {
    write!(stdout, "{}{}", cursor::Goto(1, 1), color::Bg(color::Reset)).unwrap();
    //display info
    write!(stdout, "Number of steps: {}\n\r", step).unwrap();
    for row in &grid[border_buffer.into()..(grid_height + border_buffer).into()] {
        for cell in &row[border_buffer.into()..(grid_width + border_buffer).into()] {
            write!(
                stdout,
                "{}  ",
                if *cell > 0 {
                    color::Bg(color::White).to_string()
                } else {
                    color::Bg(color::Black).to_string()
                }
            )
            .unwrap();
        }
        write!(stdout, "\n\r").unwrap();
        stdout.flush().unwrap();
    }
}
