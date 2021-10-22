use std::collections::HashMap;
use std::io::stdin;
use termion::{
    clear, color, cursor, cursor::Goto, event, input::TermRead, raw::IntoRawMode, style,
    terminal_size,
};
use tokio;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::unbounded_channel;

type Grid = HashMap<(isize, isize), bool>;

#[tokio::main]
async fn main() {
    //channels
    let (stdout, mut stdout_rx) = unbounded_channel();
    let stdout_iterator = stdout.clone();
    let (input, mut input_rx) = unbounded_channel();
    let std = std::io::stdout().into_raw_mode().unwrap();
    let events = stdin().events();

    //setup

    let mut grid: Grid = HashMap::new();
    let mut x = 0;
    let mut y = 0;

    let mut step = 0;
    grid.insert((0, 0), true);
    grid.insert((-1, 0), true);
    grid.insert((1, 0), true);
    grid.insert((0, -1), true);
    grid.insert((1, 1), true);

    stdout
        .clone()
        .send(format!("{}{}{}", clear::All, cursor::Hide, Goto(1, 1)))
        .unwrap();

    // tasks
    let iterator = tokio::spawn(async move {
        loop {
            while let Ok(event) = input_rx.try_recv() {
                match event {
                    "left" => {
                        y -= 1;
                    }
                    "right" => {
                        y += 1;
                    }
                    "up" => {
                        x -= 1;
                    }
                    "down" => {
                        x += 1;
                    }
                    _ => {}
                }
            }
            let size = terminal_size().unwrap();
            let grid_width = size.0 / 2;
            let grid_height = size.1 - 2;
            stdout_iterator
                .send(display(&grid, grid_width, grid_height, x, y, step))
                .unwrap_or(());
            let update = positions_to_update(grid.clone());
            for point in &update {
                let i = count_live_neighbors(&update, point.0);
                if *point.1 {
                    if i < 2 || i > 3 {
                        grid.remove(point.0);
                    }
                } else {
                    if i == 3 {
                        grid.insert(*point.0, true);
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
            // input.send(e.clone()).unwrap();
            match e {
                event::Event::Key(event::Key::Ctrl('c')) => {
                    iterator.abort();
                    stdout
                        .clone()
                        .send(format!(
                            "{}{}{}{}",
                            cursor::Goto(1, 1),
                            clear::All,
                            style::Reset,
                            cursor::Show
                        ))
                        .unwrap();
                    break;
                }
                event::Event::Key(event::Key::Left) => {
                    input.send("left").unwrap();
                }
                event::Event::Key(event::Key::Right) => {
                    input.send("right").unwrap();
                }
                event::Event::Key(event::Key::Up) => {
                    input.send("up").unwrap();
                }
                event::Event::Key(event::Key::Down) => {
                    input.send("down").unwrap();
                }
                _ => {}
            }
        }
    });
    input_task.await.unwrap();
    output_pump.await.unwrap();
}

fn count_live_neighbors(grid: &Grid, (x, y): &(isize, isize)) -> i8 {
    let mut count = 0;
    for i in -1..2 {
        for j in -1..2 {
            if i == 0 && j == 0 {
                continue;
            }
            if let Some(v) = grid.get(&(x + i, y + j)) {
                if *v {
                    count += 1;
                }
                if count > 3 {
                    return 4;
                }
            }
        }
    }
    count
}

fn positions_to_update(mut grid: Grid) -> Grid {
    for ((x, y), _) in grid.clone() {
        for i in -1..2 {
            for j in -1..2 {
                if i == 0 && j == 0 {
                    continue;
                }
                let point = &(x + i, y + j);
                if grid.get(point).is_none() {
                    grid.insert(*point, false);
                }
            }
        }
    }
    grid
}

fn display(
    grid: &Grid,
    grid_width: u16,
    grid_height: u16,
    x: isize,
    y: isize,
    step: usize,
) -> String {
    let mut output = String::with_capacity(
        grid_height as usize * grid_width as usize * 2 + 9 * grid.len() + 100,
    );
    output.push_str(&format!(
        "{}{}{}",
        cursor::Goto(1, 1),
        color::Bg(color::Reset),
        clear::UntilNewline
    ));
    output.push_str(&format!(
        "Number of steps: {}\n\r{}",
        step,
        color::Bg(color::Black)
    ));
    let mut white = false;
    for row in 0..(grid_height as isize) {
        for cell in 0..(grid_width as isize) {
            let coords = (
                row - grid_height as isize / 2 + x,
                cell - grid_width as isize / 2 + y,
            );
            let alive = *grid.get(&coords).unwrap_or(&false);
            output.push_str(&format!(
                "{}  ",
                if white != alive {
                    white = alive;
                    if white {
                        color::Bg(color::White).to_string()
                    } else {
                        color::Bg(color::Black).to_string()
                    }
                } else {
                    "".to_string()
                },
            ));
        }
        output.push_str("\n\r");
    }
    output
}
