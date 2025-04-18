mod backend;
use backend::{Board, BoardLoadingError, GameState};
mod utils_backend;
use utils_backend::{AgentID, Coordinate, Direction, Index, TextureType};

use std::collections::HashSet;
use std::io;

//use colored::Colorize;

fn print_board(board: &Board) {
    let dimensions: (Index, Index) = board.get_dimensions();

    for y in 0..dimensions.0 {
        for x in 0..dimensions.0 {
            match board.read_block(Coordinate { x, y }).get_texture() {
                TextureType::BasicImpassable => {
                    print!("█████");
                }
                TextureType::BasicBlock => {
                    print!("▒▒▒▒▒");
                }
                TextureType::Goal(_) => {
                    print!("▚▚▚▚▚");
                }
                TextureType::None => {
                    print!("     ");
                }
            }
        }
        print!("\n");
        for x in 0..dimensions.0 {
            match board.read_block(Coordinate { x, y }).get_texture() {
                TextureType::BasicImpassable => {
                    print!("█   █");
                }
                TextureType::BasicBlock => {
                    let agents: HashSet<AgentID> =
                        board.read_block(Coordinate { x, y }).get_agents();
                    print!(
                        "▒{} {}▒",
                        if agents.contains(&0) { "0" } else { " " },
                        if agents.contains(&1) { "1" } else { " " }
                    );
                }
                TextureType::Goal(_) => {
                    let agents: HashSet<AgentID> =
                        board.read_block(Coordinate { x, y }).get_agents();
                    print!(
                        "▚{} {}▚",
                        if agents.contains(&0) { "0" } else { " " },
                        if agents.contains(&1) { "1" } else { " " }
                    );
                }
                TextureType::None => {
                    print!("     ");
                }
            }
        }
        print!("\n");
        for x in 0..dimensions.0 {
            match board.read_block(Coordinate { x, y }).get_texture() {
                TextureType::BasicImpassable => {
                    print!("█   █");
                }
                TextureType::BasicBlock => {
                    print!("▒   ▒");
                }
                TextureType::Goal(num) => {
                    print!(
                        "▚{}/{}▚",
                        num,
                        board.read_block(Coordinate { x, y }).get_agents().len()
                    );
                }
                TextureType::None => {
                    print!("     ");
                }
            }
        }
        print!("\n");
        for x in 0..dimensions.0 {
            match board.read_block(Coordinate { x, y }).get_texture() {
                TextureType::BasicImpassable => {
                    print!("█   █");
                }
                TextureType::BasicBlock => {
                    let agents: HashSet<AgentID> =
                        board.read_block(Coordinate { x, y }).get_agents();
                    print!(
                        "▒{} {}▒",
                        if agents.contains(&2) { "2" } else { " " },
                        if agents.contains(&3) { "3" } else { " " }
                    );
                }
                TextureType::Goal(_) => {
                    let agents: HashSet<AgentID> =
                        board.read_block(Coordinate { x, y }).get_agents();
                    print!(
                        "▚{} {}▚",
                        if agents.contains(&2) { "2" } else { " " },
                        if agents.contains(&3) { "3" } else { " " }
                    );
                }
                TextureType::None => {
                    print!("     ");
                }
            }
        }
        print!("\n");
        for x in 0..dimensions.0 {
            match board.read_block(Coordinate { x, y }).get_texture() {
                TextureType::BasicImpassable => {
                    print!("█████");
                }
                TextureType::BasicBlock => {
                    print!("▒▒▒▒▒");
                }
                TextureType::Goal(_) => {
                    print!("▚▚▚▚▚");
                }
                TextureType::None => {
                    print!("     ");
                }
            }
        }
        print!("\n");
    }
}

fn char_to_direction(input: &String) -> Result<Direction, String> {
    if input.trim().eq(&String::from("u")) {
        return Ok(Direction::Up);
    }
    if input.trim().eq(&String::from("d")) {
        return Ok(Direction::Down);
    }
    if input.trim().eq(&String::from("l")) {
        return Ok(Direction::Left);
    }
    if input.trim().eq(&String::from("r")) {
        return Ok(Direction::Right);
    }

    Err(String::from("Not a direction"))
}

fn run_game_console(file: &str) -> () {
    let mut board: Board;
    match Board::from_file(file) {
        Err(msg) => {
            match msg {
                BoardLoadingError::FileNotFound => print!("File not found"),
                BoardLoadingError::FileReadingError => print!("File could not be read"),
                BoardLoadingError::TOMLParsingError => print!("TOML parsing failed"),
                BoardLoadingError::BoardDescriptionError(text) => print!("{}", text),
            }
            return;
        }
        Ok(b) => board = b,
    };
    print_board(&board);

    let stdin = io::stdin();
    let input = &mut String::new();

    while board.get_game_state() == GameState::Running {
        input.clear();
        print!("Which Agent do you want to move?\n");
        match stdin.read_line(input) {
            Err(_) => {
                print!("Somehow that input failed, please try again.\n");
                continue;
            }
            Ok(_) => match input.trim().parse::<AgentID>() {
                Ok(agent) => {
                    input.clear();
                    print!("Which direction do you want to move in (u, d, l, r)?\n");

                    match stdin.read_line(input) {
                        Err(_) => {
                            print!("Reading the direction somehow failed\n");
                        }
                        Ok(_) => (),
                    }

                    match char_to_direction(&input) {
                        Err(_) => {
                            print!(
                                "That was not a direction, please use a direction(u, d, l, r)\n"
                            );
                            continue;
                        }
                        Ok(direction) => {
                            if board.can_move_agent(agent, direction) {
                                board.move_agent(agent, direction);
                            } else {
                                print!("Sliding agent {} in direction {}", agent, input);
                                board.slide_agent(agent, direction);
                            }
                        }
                    }
                }
                Err(_) => {
                    print!("That was not a number.\n");
                }
            },
        }
        print_board(&board);
    }

    match board.get_game_state() {
        GameState::Won => print!("\n\nCONGRATULATIONS! YOU ARE A WINNER!\n"),
        GameState::Lost => print!("\n\nWomp womp, you lost.\n"),
        GameState::Running => {
            print!("\n\nSome kind of gub relating to the game state occured.\nMost curious.")
        }
    }
}

fn main() {
    //run_game_console();
    match Board::from_file("levels/testing_levels/example0.toml") {
        Err(msg) => match msg {
            BoardLoadingError::FileNotFound => print!("File not found"),
            BoardLoadingError::FileReadingError => print!("File could not be read"),
            BoardLoadingError::TOMLParsingError => print!("TOML parsing failed"),
            BoardLoadingError::BoardDescriptionError(text) => print!("{}", text),
        },
        _ => (),
    }
    print!("\n\n\n");

    run_game_console("levels/testing_levels/example0.toml");
}
