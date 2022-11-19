use core::time;
use crossterm::{
    cursor::{self, Hide},
    event::{
        poll, read, Event,
        KeyCode::{Char, Down, Left, Right, Up},
        KeyEvent,
    },
    style::{Print, Stylize},
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use rand::{thread_rng, Rng};
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::{collections::HashSet, io::Stdout};
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
    WALL,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tile_str = match self {
            Tile::SNAKE => SNAKE_STR.green(),
            Tile::FOOD => FOOD_STR.red(),
            Tile::AIR => AIR_STR.stylize(),
            Tile::WALL => WALL_STR.white(),
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

pub enum TermUpdateType {
    Clear,
    Snake,
    Food,
}

pub struct TermUpdate {
    type_: TermUpdateType,
    coord: Coord,
}

impl TermUpdate {
    pub fn queue(&self, stdout: &mut Stdout) -> crossterm::Result<()> {
        let tile = match self.type_ {
            TermUpdateType::Clear => Tile::AIR,
            TermUpdateType::Snake => Tile::SNAKE,
            TermUpdateType::Food => Tile::FOOD,
        };
        // offset by (+1, +1) for walls
        stdout
            .queue(cursor::MoveTo(
                u16::try_from(self.coord.x).unwrap() + 1,
                u16::try_from(self.coord.y).unwrap() + 1,
            ))?
            .queue(Print(tile))?;
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

    pub fn tick(&mut self) -> Vec<TermUpdate> {
        let mut term_updates = Vec::new();

        let new_head = self.get_new_head();

        if self.snake[..self.snake.len() - 1].contains(&new_head) {
            // don't check the last snake part. we want to be able to move into that spot and not die
            self.state = GameState::DEAD;
            return term_updates;
        }

        self.snake.insert(0, new_head.clone());
        term_updates.push(TermUpdate {
            type_: TermUpdateType::Snake,
            coord: new_head.clone(),
        });

        if !self.coord_is_in_bounds(self.get_head()) {
            self.state = GameState::DEAD;
            return term_updates;
        }

        let got_food = match &self.food {
            Some(food) if self.get_head() == food => {
                // term_updates.push(TermUpdate {
                //     type_: TermUpdateType::Clear,
                //     coord: food.clone(),
                // });
                self.place_food();
                match &self.food {
                    Some(coord) => term_updates.push(TermUpdate {
                        type_: TermUpdateType::Food,
                        coord: coord.clone(),
                    }),
                    None => {
                        // if food is None, that means we couldn't place any food because board is full
                        // in other words, you've won?
                        self.state = GameState::WON;
                        return term_updates;
                    }
                }
                true
            }
            _ => false,
        };

        if !got_food {
            term_updates.push(TermUpdate {
                type_: TermUpdateType::Clear,
                coord: self.snake.last().unwrap().clone(),
            });
            self.snake.pop();
        }

        term_updates
    }

    /// Draw the initial board to stdout. No clearing is performed.
    pub fn draw_initial(&self) -> crossterm::Result<()> {
        let mut stdout = stdout();

        stdout.queue(Clear(ClearType::All))?;

        // draw the walls
        for y in 0..self.height + 2 {
            for x in 0..self.width + 2 {
                if y == 0 || y == self.height + 1 || x == 0 || x == self.width + 1 {
                    stdout
                        .queue(cursor::MoveTo(x.into(), y.into()))?
                        .queue(Print(Tile::WALL))?;
                }
            }
        }

        // draw the snake, offsetting by (+1, +1) for walls
        for coord in &self.snake {
            stdout
                .queue(cursor::MoveTo(
                    u16::try_from(coord.x).unwrap() + 1,
                    u16::try_from(coord.y).unwrap() + 1,
                ))?
                .queue(Print(Tile::SNAKE))?;
        }

        // draw the food, offsetting by (+1, +1) for walls
        if let Some(food) = &self.food {
            stdout
                .queue(cursor::MoveTo(
                    u16::try_from(food.x).unwrap() + 1,
                    u16::try_from(food.y).unwrap() + 1,
                ))?
                .queue(Print(Tile::FOOD))?;
        }

        Ok(())
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
            let mut stdout = stdout();
            stdout.queue(Hide).unwrap();
            ticker_mut.lock().unwrap().draw_initial().unwrap();
            stdout.flush().unwrap();

            let mut term_updates: Vec<TermUpdate> = Vec::new();
            loop {
                {
                    // let game = ticker_mut.lock().unwrap();
                    for term_update in &term_updates {
                        term_update.queue(&mut stdout).unwrap();
                    }
                    stdout.flush().unwrap();
                }
                thread::sleep(ig.tick_wait);
                {
                    let mut game = ticker_mut.lock().unwrap();

                    term_updates = game.tick();

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
                    Event::Key(KeyEvent { modifiers: _, code }) => match code {
                        Up | Char('w') | Char('W') => Some(Input::UP),
                        Left | Char('a') | Char('A') => Some(Input::LEFT),
                        Down | Char('s') | Char('S') => Some(Input::DOWN),
                        Right | Char('d') | Char('D') => Some(Input::RIGHT),
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(i) = input {
                    let mut game = input_handler_mut.lock().unwrap();
                    game.cur_input = i;
                }
            } else if input_handler_mut.lock().unwrap().state != GameState::RUNNING {
                break;
            }
        });

        ticker.join().unwrap();
        input_handler.join().unwrap();
    }
}
