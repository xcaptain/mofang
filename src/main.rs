use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Text, Widget},
    Terminal,
};
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // Setup event handlers
    let events = Events::new();

    let mofang_size = 5_usize;
    let colors: Vec<Color> = vec![
        Color::Red,
        Color::Yellow,
        Color::Blue,
        Color::Green,
        Color::Cyan,
    ];
    let mut color_matrix: Vec<Vec<Color>> = vec![];
    for i in 0..mofang_size {
        color_matrix.push(vec![colors[i]; mofang_size]);
    }
    let mut constraints = vec![];
    for _i in 0..mofang_size {
        constraints.push(Constraint::Ratio(1, mofang_size as u32));
    }

    let mut cur_row = 0;
    let mut cur_col = 0;

    let start_time = Instant::now();

    loop {
        terminal.draw(|mut f| {
            let size = f.size();
            let block = Block::default()
                .borders(Borders::ALL)
                .title("2D Mofang game")
                .border_type(BorderType::Rounded);
            f.render_widget(block, size);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(80),
                        Constraint::Percentage(10),
                    ]
                    .as_ref(),
                )
                .split(f.size());
            // render header
            {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Percentage(10),
                            Constraint::Percentage(80),
                            Constraint::Percentage(10),
                        ]
                        .as_ref(),
                    )
                    .split(chunks[0]);
                
                let cur_time = Instant::now();
                let time_passed = cur_time.duration_since(start_time).as_secs();
                let time_str = format!("{} seconds passed!", time_passed);
                let s1 = &time_str;
                let t1 = [Text::raw(s1)];
                
                let title = Paragraph::new(t1.iter())
                    .block(Block::default())
                    .alignment(Alignment::Center);

                f.render_widget(title, chunks[1]);
    
            }
            {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Percentage(10),
                            Constraint::Percentage(80),
                            Constraint::Percentage(10),
                        ]
                        .as_ref(),
                    )
                    .split(chunks[1]);

                {
                    let row_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(constraints.as_ref())
                        .split(chunks[1]);
                    for (i, row_chunk) in row_chunks.into_iter().enumerate() {
                        let col_chunks = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints(constraints.as_ref())
                            .split(row_chunk);
                        for (j, col_chunk) in col_chunks.into_iter().enumerate() {
                            let mut block = Block::default()
                                .borders(Borders::ALL)
                                .style(Style::default().bg(color_matrix[i][j]));
                            if cur_row == i && cur_col == j {
                                block = block.title("|--");
                            } else if cur_row == i {
                                block = block.title("--");
                            } else if cur_col == j {
                                block = block.title("|");
                            }

                            f.render_widget(block, col_chunk);
                        }
                    }
                }
            }
        })?;

        if let Event::Input(key) = events.next()? {
            if key == Key::Char('q') {
                break;
            } else if key == Key::Down {
                if cur_row < mofang_size - 1 {
                    cur_row += 1;
                }
            } else if key == Key::Up {
                if cur_row > 0 {
                    cur_row -= 1;
                }
            } else if key == Key::Right {
                if cur_col < mofang_size - 1 {
                    cur_col += 1;
                }
            } else if key == Key::Left {
                if cur_col > 0 {
                    cur_col -= 1;
                }
            } else if key == Key::Char('a') {
                row_cycle(&mut color_matrix, cur_row);
            } else if key == Key::Char('s') {
                col_cycle(&mut color_matrix, cur_col);
            }
        }
    }
    Ok(())
}

fn row_cycle(color_matrix: &mut Vec<Vec<Color>>, row_idx: usize) {
    // 按行轮换一次颜色（左右轮换）
    let mut row = color_matrix[row_idx].clone();
    let l = row.len();
    let first = row[0];
    for i in 0..l - 1 {
        row[i] = row[i + 1];
    }
    row[l - 1] = first;
    color_matrix[row_idx] = row;
}

fn col_cycle(color_matrix: &mut Vec<Vec<Color>>, col_idx: usize) {
    // 按列轮换一次颜色（上下轮换）
    let l = color_matrix[0].len();
    let first = color_matrix[0][col_idx];
    for i in 0..l - 1 {
        color_matrix[i][col_idx] = color_matrix[i + 1][col_idx];
    }
    color_matrix[l - 1][col_idx] = first;
}

use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;

use termion::input::TermRead;

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    ignore_exit_key: Arc<AtomicBool>,
    input_handle: thread::JoinHandle<()>,
    tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let ignore_exit_key = Arc::new(AtomicBool::new(false));
        let input_handle = {
            let tx = tx.clone();
            let ignore_exit_key = ignore_exit_key.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        if let Err(err) = tx.send(Event::Input(key)) {
                            eprintln!("{}", err);
                            return;
                        }
                        if !ignore_exit_key.load(Ordering::Relaxed) && key == config.exit_key {
                            return;
                        }
                    }
                }
            })
        };
        let tick_handle = {
            thread::spawn(move || loop {
                tx.send(Event::Tick).unwrap();
                thread::sleep(config.tick_rate);
            })
        };
        Events {
            rx,
            ignore_exit_key,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn disable_exit_key(&mut self) {
        self.ignore_exit_key.store(true, Ordering::Relaxed);
    }

    pub fn enable_exit_key(&mut self) {
        self.ignore_exit_key.store(false, Ordering::Relaxed);
    }
}
