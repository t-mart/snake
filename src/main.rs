use core::time;
use snake::{Game, GameState, Input, InteractiveGame};
use std::io;

fn get_input() -> Option<Input> {
    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .expect("couldn't get line");
    Input::from_key(buffer.trim_end())
}

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
    InteractiveGame::play(10, 10, time::Duration::from_millis(500))
}
