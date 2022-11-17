use snake::{Game, GameState, Input};
use std::io;

fn get_input() -> Option<Input> {
    // Some(Input::LEFT)
    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .expect("couldn't get line");
    Input::from_key(buffer.trim_end())
}

fn main() {
    let mut game = Game::start(10, 10);
    while game.state == GameState::RUNNING {
        print!("{}", game);
        if let Some(input) = get_input() {
            game.cur_input = input
        }
        game.tick();
    }
    println!("{:?}", game.state)
}
