use std::collections::VecDeque;
use std::io::{self, Result, Write, stdout};
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, poll, read};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode, size,
};
use crossterm::{cursor, execute, queue, terminal};
use rand::RngExt; // Assuming rand v0.9 based on your original snippet

#[derive(Debug, PartialEq, Clone, Copy)]
struct Point {
    x: u16,
    y: u16,
}

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Right,
    Left,
}

#[derive(PartialEq)]
enum GamePhase {
    Menu,
    Playing,
    GameOver,
}

struct State {
    snake: VecDeque<Point>,
    direction: Direction,
    food: Point,
    size: (u16, u16),
    last_tail: Option<Point>, // Track the tail to erase it without clearing the screen
}

impl State {
    pub fn new(size: (u16, u16)) -> Self {
        let mut snake = VecDeque::new();
        snake.push_front(Point { x: 0, y: 0 });
        snake.push_front(Point { x: 1, y: 0 });
        snake.push_front(Point { x: 2, y: 0 });
        snake.push_front(Point { x: 3, y: 0 });
        snake.push_front(Point { x: 4, y: 0 });
        snake.push_front(Point { x: 5, y: 0 });
        snake.push_front(Point { x: 6, y: 0 });
        snake.push_front(Point { x: 7, y: 0 });
        snake.push_front(Point { x: 8, y: 0 });

        let direction = Direction::Right;
        let food = Point {
            x: size.0 / 2,
            y: size.1 / 2,
        };

        State {
            snake,
            direction,
            food,
            size,
            last_tail: None,
        }
    }

    fn set_random_food_location(&mut self) {
        let mut rng = rand::rng();
        loop {
            let new_food = Point {
                x: rng.random_range(0..self.size.0),
                y: rng.random_range(0..self.size.1),
            };
            if !self.snake.contains(&new_food) {
                self.food = new_food;
                break;
            }
        }
    }

    fn move_forward(&mut self) -> bool {
        let snake_front = self.snake.front().unwrap();

        let new_front = match self.direction {
            Direction::Up => Point {
                x: snake_front.x,
                y: if snake_front.y == 0 {
                    self.size.1 - 1
                } else {
                    snake_front.y - 1
                },
            },
            Direction::Down => Point {
                x: snake_front.x,
                y: if snake_front.y == self.size.1 - 1 {
                    0
                } else {
                    snake_front.y + 1
                },
            },
            Direction::Right => Point {
                x: if snake_front.x == self.size.0 - 1 {
                    0
                } else {
                    snake_front.x + 1
                },
                y: snake_front.y,
            },
            Direction::Left => Point {
                x: if snake_front.x == 0 {
                    self.size.0 - 1
                } else {
                    snake_front.x - 1
                },
                y: snake_front.y,
            },
        };

        // Check for collision
        if self.snake.contains(&new_front) {
            return false; // Game Over
        }

        // Update snake head
        self.snake.push_front(new_front);

        // Check if food eaten
        if new_front == self.food {
            self.set_random_food_location();
            self.last_tail = None; // Grew, so we don't erase the tail this frame
        } else {
            self.last_tail = self.snake.pop_back(); // Move forward, save tail to erase
        }

        true
    }

    fn render(&self) -> Result<()> {
        let mut stdout = stdout();

        // 1. Erase the old tail using two spaces
        if let Some(tail) = self.last_tail {
            queue!(stdout, cursor::MoveTo(tail.x * 2, tail.y), Print("  "))?;
        }

        // 2. Draw the Food (Red block)
        queue!(
            stdout,
            cursor::MoveTo(self.food.x * 2, self.food.y),
            SetForegroundColor(Color::Red),
            Print("🦀"),
            ResetColor
        )?;

        // 3. Draw the Snake
        for (index, point) in self.snake.iter().enumerate() {
            if index == 0 {
                // Head
                queue!(
                    stdout,
                    cursor::MoveTo(point.x * 2, point.y),
                    SetForegroundColor(Color::DarkGreen),
                    Print("00")
                )?;
            } else {
                // Body
                queue!(
                    stdout,
                    cursor::MoveTo(point.x * 2, point.y),
                    SetForegroundColor(Color::Green),
                    Print("██")
                )?;
            }
        }

        // 4. Draw the Score HUD
        queue!(
            stdout,
            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::Yellow),
            Print(format!(" Score: {} ", self.snake.len() - 2)),
            ResetColor,
            cursor::Hide
        )?;

        stdout.flush()?;
        Ok(())
    }
}

fn draw_centered_text(stdout: &mut io::Stdout, text: &str, row: u16, cols: u16) -> Result<()> {
    let padding = (cols.saturating_sub(text.len() as u16)) / 2;
    queue!(stdout, cursor::MoveTo(padding, row), Print(text))?;
    Ok(())
}

fn run() -> io::Result<()> {
    let (cols, rows) = size()?;
    let logical_width = cols / 2; // Divide by 2 for the aspect ratio fix

    let mut state = State::new((logical_width, rows));
    let mut phase = GamePhase::Menu;
    let mut last_tick = Instant::now();

    let mut stdout = stdout();
    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide
    )?;

    loop {
        match phase {
            GamePhase::Menu => {
                draw_centered_text(&mut stdout, " RUSTY-SNAKE ", rows / 2 - 2, cols)?;
                draw_centered_text(&mut stdout, "Press ENTER to Start", rows / 2, cols)?;
                draw_centered_text(&mut stdout, "Press ESC to Quit", rows / 2 + 1, cols)?;
                stdout.flush()?;

                if poll(Duration::from_millis(100))? {
                    if let Event::Key(key_event) = read()? {
                        match key_event.code {
                            KeyCode::Enter => {
                                state = State::new((logical_width, rows));
                                execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
                                phase = GamePhase::Playing;
                                last_tick = Instant::now();
                            }
                            KeyCode::Esc => break,
                            _ => {}
                        }
                    }
                }
            }

            GamePhase::Playing => {
                // Dynamic tick rate for visual consistency
                let tick_rate = match state.direction {
                    Direction::Up | Direction::Down => Duration::from_millis(120),
                    Direction::Left | Direction::Right => Duration::from_millis(120), // Can adjust if still feels off
                };

                // Decoupled Input Polling
                let elapsed = last_tick.elapsed();
                let timeout = tick_rate.saturating_sub(elapsed);

                if poll(timeout)? {
                    if let Event::Key(key_event) = read()? {
                        match key_event.code {
                            KeyCode::Up if state.direction != Direction::Down => {
                                state.direction = Direction::Up
                            }
                            KeyCode::Down if state.direction != Direction::Up => {
                                state.direction = Direction::Down
                            }
                            KeyCode::Left if state.direction != Direction::Right => {
                                state.direction = Direction::Left
                            }
                            KeyCode::Right if state.direction != Direction::Left => {
                                state.direction = Direction::Right
                            }
                            KeyCode::Esc => phase = GamePhase::Menu,
                            _ => {}
                        }
                    }
                }

                // Tick the game
                if last_tick.elapsed() >= tick_rate {
                    if !state.move_forward() {
                        phase = GamePhase::GameOver;
                        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
                    } else {
                        state.render()?;
                    }
                    last_tick = Instant::now();
                }
            }

            GamePhase::GameOver => {
                let score = state.snake.len() - 2;
                draw_centered_text(&mut stdout, " GAME OVER ", rows / 2 - 2, cols)?;
                draw_centered_text(
                    &mut stdout,
                    &format!(" Final Score: {} ", score),
                    rows / 2,
                    cols,
                )?;
                draw_centered_text(&mut stdout, "Press R to Restart", rows / 2 + 2, cols)?;
                draw_centered_text(&mut stdout, "Press ESC to Quit", rows / 2 + 3, cols)?;
                stdout.flush()?;

                if poll(Duration::from_millis(100))? {
                    if let Event::Key(key_event) = read()? {
                        match key_event.code {
                            KeyCode::Char('r') | KeyCode::Char('R') | KeyCode::Enter => {
                                state = State::new((logical_width, rows));
                                execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
                                phase = GamePhase::Playing;
                                last_tick = Instant::now();
                            }
                            KeyCode::Esc => break,
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    if let Err(e) = run() {
        println!("Error: {e:?}\r");
    }

    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    disable_raw_mode()
}

// use std::collections::VecDeque;
// use std::io::{Result, Write, stdout};
// use std::thread;
// use std::{io, time::Duration};

// use crossterm::style::{Print, ResetColor, SetForegroundColor};
// use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
// use crossterm::{cursor, queue, terminal};
// use crossterm::{
//     event::{Event, KeyCode, poll, read},
//     execute,
//     terminal::{disable_raw_mode, enable_raw_mode, size},
// };
// use rand::RngExt;

// #[derive(Debug, PartialEq, Clone, Copy)]
// struct Point {
//     x: u16,
//     y: u16,
// }

// #[derive(PartialEq)]
// enum Direction {
//     Up,
//     Down,
//     Right,
//     Left,
// }

// struct State {
//     snake: VecDeque<Point>,
//     direction: Direction,
//     food: Point,
//     size: (u16, u16),
//     last_tail: Option<Point>,
// }

// impl State {
//     pub fn new(size: (u16, u16)) -> Self {
//         let mut snake = VecDeque::new();
//         snake.push_front(Point { x: 0, y: 0 });
//         snake.push_front(Point { x: 1, y: 0 });
//         let direction = Direction::Right;
//         let food = Point {
//             x: size.0 / 2,
//             y: size.1 / 2,
//         };

//         State {
//             snake,
//             direction: direction,
//             food,
//             size,
//             last_tail: Some(Point { x: 0, y: 0 }),
//         }
//     }

//     fn set_random_food_location(&mut self) {
//         while self.snake.contains(&self.food) {
//             let mut rng = rand::rng();
//             self.food.x = rng.random_range(0..self.size.0);
//             self.food.y = rng.random_range(0..self.size.1);
//         }
//     }

//     fn move_forward(&mut self) -> Option<Point> {
//         let snake_front = self.snake.front()?;

//         let new_front;
//         match self.direction {
//             Direction::Up => {
//                 new_front = Point {
//                     x: snake_front.x,
//                     y: if snake_front.y == 0 {
//                         self.size.1 - 1
//                     } else {
//                         snake_front.y - 1
//                     }, //wrap the snake to bottom
//                 };
//             }
//             Direction::Down => {
//                 new_front = Point {
//                     x: snake_front.x,
//                     y: if snake_front.y == self.size.1 - 1 {
//                         0
//                     } else {
//                         snake_front.y + 1
//                     }, //wrap the snake to top
//                 };
//             }
//             Direction::Right => {
//                 new_front = Point {
//                     x: if snake_front.x == self.size.0 - 1 {
//                         0
//                     } else {
//                         snake_front.x + 1
//                     },
//                     y: snake_front.y,
//                 };
//             }
//             Direction::Left => {
//                 new_front = Point {
//                     x: if snake_front.x == 0 {
//                         self.size.0 - 1
//                     } else {
//                         snake_front.x - 1
//                     },
//                     y: snake_front.y,
//                 };
//             }
//         }

//         // Check for collision
//         if self.snake.contains(&new_front) {
//             return None;
//         }

//         // Update snake
//         self.snake.push_front(new_front);

//         // Check if food eaten
//         if new_front == self.food {
//             self.set_random_food_location();
//             self.last_tail = None
//         } else {
//             self.snake.pop_back();
//             self.last_tail = self.snake.pop_back();
//         }

//         Some(new_front)
//     }

//     fn tick(&mut self) -> bool {
//         let res = self.move_forward();
//         match res {
//             Some(_) => return true,
//             None => return false,
//         }
//     }

//     fn render(&self) -> Result<()> {
//         let mut stdout = stdout();
//         // if let Some(tail) = self.last_tail {
//         // queue!(stdout, cursor::MoveTo(tail.x * 2, tail.y), Print("  "))?;
//         // }
//         queue!(stdout, terminal::Clear(terminal::ClearType::All))?;
//         // stdout.flush()?;
//         queue!(
//             stdout,
//             cursor::MoveTo(self.food.x * 2, self.food.y),
//             SetForegroundColor(crossterm::style::Color::Red),
//             Print("🦀"),
//             ResetColor
//         )?;
//         queue!(stdout, SetForegroundColor(crossterm::style::Color::Green))?;
//         for (index, point) in self.snake.iter().enumerate() {
//             if index == 0 {
//                 // The Head
//                 queue!(
//                     stdout,
//                     cursor::MoveTo(point.x * 2, point.y),
//                     SetForegroundColor(crossterm::style::Color::DarkGreen), // Darker green head
//                     Print("██") // A textured block for the head
//                 )?;
//             } else {
//                 // The Body
//                 queue!(
//                     stdout,
//                     cursor::MoveTo(point.x * 2, point.y),
//                     SetForegroundColor(crossterm::style::Color::Green),
//                     Print("██")
//                 )?;
//             }
//         }
//         queue!(stdout, ResetColor, cursor::Hide)?;
//         stdout.flush()?;
//         Ok(())
//     }
// }

// const HELP: &str = r#"Blocking poll() & non-blocking read()
//  - Keyboard, mouse and terminal resize events enabled
//  - Prints "." every second if there's no event
//  - Hit "c" to print current cursor position
//  - Use Esc to quit
// "#;

// fn run() -> io::Result<()> {
//     let (cols, rows) = size()?;
//     let mut state = State::new((cols / 2, rows));
//     loop {
//         // Wait up to 1s for another event
//         if poll(Duration::from_millis(1_00))? {
//             // It's guaranteed that read() won't block if `poll` returns `Ok(true)`

//             if let Event::Key(key_event) = read()? {
//                 match key_event.code {
//                     KeyCode::Up | KeyCode::Down => {
//                         if (state.direction != Direction::Down)
//                             && (state.direction != Direction::Up)
//                         {
//                             state.direction = if key_event.code == KeyCode::Up {
//                                 Direction::Up
//                             } else {
//                                 Direction::Down
//                             }
//                         }
//                     }
//                     KeyCode::Left | KeyCode::Right => {
//                         if (state.direction != Direction::Left)
//                             && (state.direction != Direction::Right)
//                         {
//                             state.direction = if key_event.code == KeyCode::Left {
//                                 Direction::Left
//                             } else {
//                                 Direction::Right
//                             }
//                         }
//                     }
//                     KeyCode::Esc => {
//                         break;
//                     }
//                     KeyCode::Enter => {
//                         thread::sleep(Duration::from_secs(10));
//                     }
//                     _ => {}
//                 }
//             }
//         }
//         if state.tick() {
//             state.render()?
//         } else {
//             break;
//         }
//     }

//     Ok(())
// }

// fn main() -> io::Result<()> {
//     println!("{HELP}");

//     enable_raw_mode()?;

//     let mut stdout = io::stdout();

//     execute!(stdout, EnterAlternateScreen)?;

//     if let Err(e) = run() {
//         println!("Error: {e:?}\r");
//     }

//     execute!(stdout, LeaveAlternateScreen)?;
//     // execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

//     disable_raw_mode()
// }
