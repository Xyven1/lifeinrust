use std::io::stdin;
use termion::{
    clear, color, cursor, cursor::Goto, event, input::TermRead, raw::IntoRawMode, raw::RawTerminal,
    style, terminal_size,
};
use tokio;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

#[tokio::main]
async fn main() {
    //channels
    let (stdout, mut stdout_rx) = unbounded_channel();
    let stdout_iterator = stdout.clone();
    let (input, mut input_rx) = unbounded_channel();
    let events = stdin().events();
    let mut std = std::io::stdout().into_raw_mode().unwrap();

    //tasks

    //setup
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
    grid[center_x - 1][center_y] = 1;
    grid[center_x + 1][center_y] = 1;
    grid[center_x][center_y - 1] = 1;
    grid[center_x + 1][center_y + 1] = 1;
    stdout
        .clone()
        .send(format!("{}{}{}", clear::All, cursor::Hide, Goto(1, 1)))
        .unwrap();
    let iterator = tokio::spawn(async move {
        loop {
            while let Ok(event) = input_rx.try_recv() {
                stdout_iterator.send("1".to_string()).unwrap();
            }
            display(
                stdout_iterator.clone(),
                &grid,
                border_buffer,
                grid_width,
                grid_height,
                step,
            );
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
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    });
    let output_pump = tokio::spawn(async move {
        // let mut stdout = std::io::stdout().into_raw_mode().unwrap();
        let mut stdout = tokio::io::stdout();
        while let Some(message) = stdout_rx.recv().await {
            stdout.write_all(message.as_bytes()).await.unwrap();
            stdout.flush().await.unwrap();
        }
    });
    let input_task = tokio::task::spawn_blocking(move || {
        for event in events {
            let e = event.unwrap();
            input.send(e.clone()).unwrap();
            match e {
                event::Event::Key(event::Key::Ctrl('c')) => {
                    break;
                }
                event::Event::Key(event::Key::Char('s')) => {}
                _ => {}
            }
        }
    });
    input_task.await.unwrap();
    // stdout
    //     .clone()
    //     .send(format!(
    //         "{}{}{}",
    //         clear::All,
    //         style::Reset,
    //         cursor::Goto(1, 1)
    //     ))
    //     .unwrap();
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
    stdout: UnboundedSender<String>,
    grid: &Vec<Vec<i32>>,
    border_buffer: u16,
    grid_width: u16,
    grid_height: u16,
    step: usize,
) {
    let mut strng;
    strng = format!("{}{}", cursor::Goto(1, 1), color::Bg(color::Reset));
    //display info
    strng = format!("{}{}{}", strng, cursor::Goto(1, 1), color::Bg(color::Reset));
    strng = format!("{}Number of steps: {}\n\r", strng, step);
    for row in &grid[border_buffer.into()..(grid_height + border_buffer).into()] {
        for cell in &row[border_buffer.into()..(grid_width + border_buffer).into()] {
            strng = format!(
                "{}{}  ",
                strng,
                if *cell > 0 {
                    color::Bg(color::White).to_string()
                } else {
                    color::Bg(color::Black).to_string()
                }
            );
        }
        strng = format!("{}\n\r", strng);
    }
    stdout.send(strng).unwrap();
}
