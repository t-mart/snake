use core::time;
use snake::InteractiveGame;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

fn main() {
    // let mut game = Game::create(10, 40);
    // while game.state == GameState::RUNNING {
    //     print!("{}", game);
    //     if let Some(input) = get_input() {
    //         game.cur_input = input
    //     }
    //     game.tick();
    // }
    // println!("{:?}", game.state)

    enable_raw_mode().unwrap();
    InteractiveGame::play(10, 10, time::Duration::from_millis(200));
    disable_raw_mode().unwrap();
}
