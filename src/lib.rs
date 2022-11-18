use core::time;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent};
use rand::{thread_rng, Rng};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;
use std::{fmt, ops::Add};

const WALL_STR: &str = "█";
const SNAKE_STR: &str = "●";
const FOOD_STR: &str = "*";
const AIR_STR: &str = " ";

#[derive(Clone)]
pub enum Tile {
    SNAKE,
    FOOD,
    AIR,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tile_str = match self {
            Tile::SNAKE => SNAKE_STR,
            Tile::FOOD => FOOD_STR,
            Tile::AIR => AIR_STR,
        };
        write!(f, "{}", tile_str)
    }
}

#[derive(PartialEq, Debug)]
pub enum Input {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

impl Input {
    pub fn from_key(s: &str) -> Option<Input> {
        match s {
            "w" => Some(Input::UP),
            "a" => Some(Input::LEFT),
            "s" => Some(Input::DOWN),
            "d" => Some(Input::RIGHT),
            _ => None,
        }
    }

    fn rev(&self) -> Input {
        match self {
            Input::DOWN => Input::UP,
            Input::UP => Input::DOWN,
            Input::LEFT => Input::RIGHT,
            Input::RIGHT => Input::LEFT,
        }
    }

    fn offset(&self) -> Coord {
        match self {
            Input::DOWN => Coord { x: 0, y: 1 },
            Input::UP => Coord { x: 0, y: -1 },
            Input::LEFT => Coord { x: -1, y: 0 },
            Input::RIGHT => Coord { x: 1, y: 0 },
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Coord {
    x: isize, // these must be larger than the types of the height/width of the board and must be signed
    y: isize,
}

impl Add for Coord {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Coord {
    fn move_by(&self, input: &Input) -> Coord {
        let offset = input.offset();
        self.clone() + offset
    }
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)?;
        Ok(())
    }
}

#[derive(PartialEq, Debug)]
pub enum GameState {
    RUNNING,
    DEAD,
    WON,
}

pub struct Game {
    snake: Vec<Coord>,
    food: Option<Coord>, // food may not be present if board is completely filled with snake
    width: u8,
    height: u8,
    pub state: GameState,
    pub cur_input: Input,
}

impl Game {
    pub fn create(height: u8, width: u8) -> Game {
        if height < 2 || width < 2 {
            panic!("Board too small. Must have minimum dimension of 2.")
        }
        let mut game = Game {
            snake: vec![Coord { x: 0, y: 0 }],
            food: None,
            width,
            height,
            state: GameState::RUNNING,
            cur_input: Input::DOWN,
        };
        game.place_food();
        game
    }

    fn coord_is_in_bounds(&self, coord: &Coord) -> bool {
        coord.x >= 0 && coord.x < self.width.into() && coord.y >= 0 && coord.y < self.height.into()
    }

    fn get_head(&self) -> &Coord {
        &self.snake[0]
    }

    pub fn get_new_head(&self) -> Coord {
        let new_head = self.get_head().move_by(&self.cur_input);
        if self.snake.len() >= 2 && self.snake[1] == new_head {
            return self.snake[0].move_by(&self.cur_input.rev());
        }
        new_head
    }

    fn place_food(&mut self) -> () {
        let mut free_coords = HashSet::new();
        for y in 0..isize::from(self.height) {
            for x in 0..isize::from(self.width) {
                free_coords.insert(Coord { x, y });
            }
        }
        for snake_part in &self.snake {
            free_coords.remove(&snake_part);
        }

        if free_coords.len() == 0 {
            self.food = None;
            return;
        }

        let free_cords = free_coords.into_iter().collect::<Vec<Coord>>();
        let food_coord = &free_cords[thread_rng().gen_range(0..free_cords.len())];
        self.food = Some(food_coord.clone())
    }

    pub fn tick(&mut self) -> () {
        let new_head = self.get_new_head();
        if self.snake.contains(&new_head) {
            self.state = GameState::DEAD;
            return;
        }
        self.snake.insert(0, new_head);
        if !self.coord_is_in_bounds(self.get_head()) {
            self.state = GameState::DEAD;
            return;
        }
        let got_food = match &self.food {
            Some(food) if self.get_head() == food => {
                self.place_food();
                if self.food.is_none() {
                    // tried to place food, but no spots available, meaning all the board is a snake: you win
                    self.state = GameState::WON;
                    return;
                }
                true
            }
            _ => false,
        };
        if !got_food {
            self.snake.pop();
        }
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut tiles = vec![];
        for _ in 0..self.height {
            tiles.push(vec![Tile::AIR; self.width.into()])
        }

        let mut update_coord_tile = |coord: &Coord, tile: Tile| -> () {
            let x = usize::try_from(coord.x).unwrap();
            let y = usize::try_from(coord.y).unwrap();
            tiles[y][x] = tile
        };

        for snake_part in &self.snake {
            update_coord_tile(snake_part, Tile::SNAKE);
        }
        if let Some(food) = &self.food {
            update_coord_tile(food, Tile::FOOD);
        }

        write!(f, "{}\n", WALL_STR.repeat(usize::from(self.width) + 2))?;
        for y in 0..usize::from(self.height) {
            write!(f, "{}", WALL_STR)?;
            for x in 0..usize::from(self.width) {
                write!(f, "{}", tiles[y][x])?;
            }
            write!(f, "{}\n", WALL_STR)?;
        }
        write!(f, "{}\n", WALL_STR.repeat(usize::from(self.width) + 2))?;
        Ok(())
    }
}

pub struct InteractiveGame {
    game_mut: Arc<Mutex<Game>>,
    tick_wait: time::Duration,
}
impl InteractiveGame {
    pub fn play(height: u8, width: u8, tick_wait: time::Duration) -> () {
        let ig = InteractiveGame {
            game_mut: Arc::new(Mutex::new(Game::create(height, width))),
            tick_wait,
        };

        let ticker_mut = Arc::clone(&ig.game_mut);
        let ticker = thread::spawn(move || {
            // - print the board
            // - wait
            // - tick
            loop {
                {
                    let game = ticker_mut.lock().unwrap();
                    print!("{}", game);
                }
                thread::sleep(ig.tick_wait);
                {
                    let mut game = ticker_mut.lock().unwrap();
                    game.tick();
                    if game.state != GameState::RUNNING {
                        println!("{:?}", game.state);
                        break;
                    }
                }
            }
        });

        let input_handler_mut = Arc::clone(&ig.game_mut);
        let input_handler = thread::spawn(move || loop {
            if poll(ig.tick_wait).unwrap() {
                let event = read().unwrap();
                let input = match event {
                    Event::Key(KeyEvent {
                        modifiers: _,
                        code: KeyCode::Char('w'),
                    }) => Some(Input::UP),
                    Event::Key(KeyEvent {
                        modifiers: _,
                        code: KeyCode::Char('a'),
                    }) => Some(Input::LEFT),
                    Event::Key(KeyEvent {
                        modifiers: _,
                        code: KeyCode::Char('s'),
                    }) => Some(Input::DOWN),
                    Event::Key(KeyEvent {
                        modifiers: _,
                        code: KeyCode::Char('d'),
                    }) => Some(Input::RIGHT),
                    _ => None,
                };
                if let Some(i) = input {
                    let mut game = input_handler_mut.lock().unwrap();
                    game.cur_input = i;
                }
                // println!("Event::{:?}\r", event);
            } else if input_handler_mut.lock().unwrap().state != GameState::RUNNING {
                break;
            }
        });

        ticker.join().unwrap();
        input_handler.join().unwrap();
    }
}
