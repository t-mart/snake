use std::fmt;

const WALL_STR: &str = "█";
const SNAKE_STR: &str = "●";
const FOOD_STR: &str = "⋆";
const AIR_STR: &str = " ";

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

pub struct Board {
    tiles: Vec<Vec<Tile>>,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //  (0,2) (1,2) (2,2)
        //  (0,1) (1,1) (2,1)
        //  (0,0) (1,0) (2,0)
        // outer vec is column
        let height = self.tiles[0].len();
        write!(f, "{}\n", WALL_STR.repeat(height + 2))?;
        for col_idx in 0..self.tiles.len() {
            write!(f, "{}", WALL_STR)?;
            for row_idx in 0..height {
                write!(f, "{}", self.tiles[col_idx][row_idx])?;
            }
            write!(f, "{}\n", WALL_STR)?;
        }
        write!(f, "{}\n", WALL_STR.repeat(height + 2))?;
        Ok(())
    }
}

impl Board {
    pub fn from_game(game: &Game) -> Board {
        let mut board = Board { tiles: Vec::new() };
        for y in 0..game.height {
            let mut col = Vec::new();
            for x in 0..game.width {
                col.push(Tile::AIR)
            }
            board.tiles.push(col);
        }
        for snake_part in &game.snake {
            board.tiles[snake_part.0][snake_part.1] = Tile::SNAKE;
        }
        if let Some(food) = game.food {
            board.tiles[food.0][food.1] = Tile::FOOD;
        }
        board
    }
}

#[derive(PartialEq)]
pub enum Input {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

impl Input {
    fn rev(&self) -> Input {
        match self {
            Input::DOWN => Input::UP,
            Input::UP => Input::DOWN,
            Input::LEFT => Input::RIGHT,
            Input::RIGHT => Input::LEFT,
        }
    }

    fn offset(&self) -> (i8, i8) {
        match self {
            Input::DOWN => (0, -1),
            Input::UP => (0, 1),
            Input::LEFT => (-1, 0),
            Input::RIGHT => (1, 0),
        }
    }

    fn offset_xy(&self, xy: (usize, usize), max_xy: (usize, usize)) -> Option<(usize, usize)> {
        let (x_offset, y_offset) = self.offset();
        let x_opt = x_offset.checked_add(xy.0.try_into().unwrap());
        let y_opt = x_offset.checked_add(xy.1.try_into().unwrap());
        if x_opt.is_none() || y_opt.is_none() {
            return None;
        } else {
            let x = x_opt.unwrap() as usize;
            let y = y_opt.unwrap() as usize;
            let foo = 0..5;
            if !(0..max_xy.0).contains(&x) || !(0..max_xy.1).contains(&y) {
                return None;
            }
            return Some((x, y));
        }
    }
}

#[derive(PartialEq)]
enum GameState {
    RUNNING,
    DEAD,
    WON,
}

pub struct Game {
    // board: Board,
    snake: Vec<(usize, usize)>,
    food: Option<(usize, usize)>,
    width: usize,
    height: usize,
    state: GameState,
    // last_input: Input,
}

// use std::{sync::Mutex, collections::HashMap};
// use once_cell::sync::Lazy;
// use rand::prelude::*;

// static RNG: Lazy<ThreadRng> = Lazy::new(|| {
//     thread_rng()
// });

impl Game {
    pub fn start(height: usize, width: usize) {
        if height < 2 || width < 2 {
            panic!("Board too small. Must have minimum dimension of 2.")
        }
    }

    pub fn tick(&self, input: Input) -> GameState {
        let old_head = self.snake[0];
        let mut new_head_opt = input.offset_xy(old_head, (self.width, self.height));
        match new_head_opt {
            None => return GameState::DEAD,
            Some(new_head) => {}
        }

        // if turning back on self, rever direction
        // HH
        // if self.last_input == input.rev() {
        // turning back on itself not allowed
        // input = input.rev();
        // }
        // match input {
        //     Input::DOWN => n
        // }
    }

    fn board(&self) -> Board {
        Board::from_game(self)
    }
}
