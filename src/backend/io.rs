use super::Board;
use super::GameState;
use super::blocks::{Air, block_factory};
use super::utils_backend::{
    AgentID, Coordinate, Direction, GoalType, Index, OUT_OF_BOUND, StatusUpdate,
};

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use toml::{self, Table, Value};

pub enum BoardLoadingError {
    FileNotFound,
    FileReadingError,
    TOMLParsingError,
    BoardDescriptionError(String),
}

impl super::Board {
    pub fn from_file(path: &str) -> Result<Self, BoardLoadingError> {
        let mut file: File;
        match File::open(path) {
            Err(_) => return Err(BoardLoadingError::FileNotFound),
            Ok(f) => file = f,
        }

        let mut board: Board = Board {
            board: vec![],
            board_start_configuration: vec![],
            x_size: 0,
            y_size: 0,
            num_agents: 0,
            num_agents_alive: 0,
            num_agents_must_finish: 0,
            game_progress: HashMap::new(),
            game_progress_start: HashMap::new(),
            game_goal: HashMap::new(),
            agent_positions: vec![],
            agent_start_positions: vec![],
            game_state: GameState::Running,
            action_stack: vec![],
        };

        let mut contend: String = String::new();
        match file.read_to_string(&mut contend) {
            Err(_) => return Err(BoardLoadingError::FileReadingError),
            _ => (),
        }
        let table: Table;
        match contend.parse::<Table>() {
            Err(_) => return Err(BoardLoadingError::TOMLParsingError),
            Ok(t) => table = t,
        }

        match board.load_victory_conditions(&table) {
            Err(msg) => return Err(BoardLoadingError::BoardDescriptionError(msg)),
            _ => (),
        }

        match board.load_blocks(&table) {
            Err(msg) => return Err(BoardLoadingError::BoardDescriptionError(msg)),
            _ => (),
        }

        match board.load_agents(&table) {
            Err(msg) => return Err(BoardLoadingError::BoardDescriptionError(msg)),
            _ => (),
        }

        Ok(board)
    }

    fn load_blocks(&mut self, level_table: &Table) -> Result<(), String> {
        match level_table.get("x_size") {
            None => return Err(String::from("Missing x dimension\n")),
            Some(val) => match val {
                Value::Integer(val_int) => self.x_size = *val_int as Index,
                _ => return Err(String::from("x dimension was not an integer")),
            },
        }
        match level_table.get("y_size") {
            None => return Err(String::from("Missing y dimension\n")),
            Some(val) => match val {
                Value::Integer(val_int) => self.y_size = *val_int as Index,
                _ => return Err(String::from("y dimension was not an integer")),
            },
        }

        for _ in 0..self.x_size * self.y_size {
            self.board.push(Box::new(Air::new()));
        }

        match level_table.get("block") {
            None => return Err(String::from("No blocks found")),
            Some(blocks) => match blocks {
                Value::Table(block_table) => {
                    let x_coords: Vec<String> = block_table.keys().cloned().collect();
                    for x in x_coords {
                        match block_table.get(&x) {
                            None => return Err(String::from("Block coordinate error")),
                            Some(row_wrapped) => match row_wrapped {
                                Value::Table(row) => {
                                    let y_coords: Vec<String> = row.keys().cloned().collect();
                                    for y in y_coords {
                                        let block: &Table;
                                        match row.get(&y) {
                                            None => {
                                                return Err(String::from("Block has no entries"));
                                            }
                                            Some(block_wrapped) => match block_wrapped {
                                                Value::Table(t) => block = t,
                                                _ => {
                                                    return Err(String::from(
                                                        "Block has no entries",
                                                    ));
                                                }
                                            },
                                        }
                                        let x_as_int: Index;
                                        let y_as_int: Index;

                                        match x.parse::<Index>() {
                                            Ok(num) => x_as_int = num,
                                            Err(_) => {
                                                return Err(String::from(
                                                    "Coordinate not specified correctly",
                                                ));
                                            }
                                        }
                                        match y.parse::<Index>() {
                                            Ok(num) => y_as_int = num,
                                            Err(_) => {
                                                return Err(String::from(
                                                    "Coordinate not specified correctly",
                                                ));
                                            }
                                        }

                                        if self.out_of_bounds(Coordinate {
                                            x: x_as_int,
                                            y: y_as_int,
                                        }) {
                                            return Err(String::from("Block out of bounds"));
                                        }
                                        match block_factory(block) {
                                            Err(msg) => return Err(msg),
                                            Ok(b) => {
                                                self.set_block(
                                                    Coordinate {
                                                        x: x_as_int,
                                                        y: y_as_int,
                                                    },
                                                    b,
                                                );
                                            }
                                        }
                                    }
                                }
                                _ => return Err(String::from("Blocks not properly specified")),
                            },
                        }
                    }
                }
                _ => return Err(String::from("Blocks not properly specified")),
            },
        }

        // Store the initial state of the board so it can be reconstructed for undo later
        for x in 0..(self.x_size) {
            for y in 0..(self.y_size) {
                let table: Table = self
                    .get_block(Coordinate {
                        x: x as Index,
                        y: y as Index,
                    })
                    .to_table();
                self.board_start_configuration.push(table);
            }
        }

        Ok(())
    }

    fn load_agents(&mut self, table: &Table) -> Result<(), String> {
        match table.get("num_agents") {
            None => {
                return Err(String::from("Missing number of agents dimension"));
            }
            Some(num_unwrapped) => match num_unwrapped {
                Value::Integer(num) => self.num_agents = *num as u8,
                _ => {
                    return Err(String::from("number of agents must be an integer"));
                }
            },
        }

        self.num_agents_alive = self.num_agents;
        self.agent_positions
            .resize(self.num_agents as usize, OUT_OF_BOUND);
        self.agent_start_positions
            .resize(self.num_agents as usize, OUT_OF_BOUND);

        match table.get("num_agents_must_finish") {
            None => self.num_agents_must_finish = self.num_agents,
            Some(num_unwrapped) => match num_unwrapped {
                Value::Integer(num) => self.num_agents_must_finish = *num as u8,
                _ => {
                    return Err(String::from("number of agents must be an integer"));
                }
            },
        }

        let agents: &Table;

        match table.get("agent") {
            None => return Err(String::from("No agents in level")),
            Some(agents_unwrapped) => match agents_unwrapped {
                Value::Table(t) => agents = t,
                _ => return Err(String::from("Error in loading agents")),
            },
        }

        let agent_ids_as_strings = agents.keys();

        if agent_ids_as_strings.len() != self.num_agents as usize {
            return Err(String::from(
                "Number of agents specified does not match number of agents declared",
            ));
        }
        for agent in agent_ids_as_strings {
            let agent_info: &Table;

            match agents.get(agent) {
                None => {
                    return Err(String::from(
                        "Some weird error happend when trying to read in an agent",
                    ));
                }
                Some(agent_info_unwrapped) => match agent_info_unwrapped {
                    Value::Table(t) => agent_info = t,
                    _ => return Err(String::from("Agent missing info tags")),
                },
            }

            let agent_id: AgentID;

            match agent.parse::<AgentID>() {
                Err(_) => return Err(String::from("Invalid Agent ID (not an int)")),
                Ok(num) => {
                    if num < self.num_agents {
                        agent_id = num;
                    } else {
                        return Err(String::from("Invalid Agent ID (too large)"));
                    }
                }
            }

            match agent_info.get("start") {
                None => return Err(String::from("Agent missing start position")),
                Some(position_unwrapped) => match position_unwrapped {
                    Value::Array(arr) => {
                        if arr.len() != 2 {
                            return Err(String::from(
                                "Agent start position must be array of two ints",
                            ));
                        }

                        if let Value::Integer(x) = arr[0] {
                            if let Value::Integer(y) = arr[1] {
                                if self.out_of_bounds(Coordinate {
                                    x: x as Index,
                                    y: y as Index,
                                }) {
                                    return Err(String::from(
                                        "Agent start position is out of bounds",
                                    ));
                                }
                                if self
                                    .get_block(Coordinate {
                                        x: x as Index,
                                        y: y as Index,
                                    })
                                    .can_enter(Direction::None)
                                {
                                    let update: StatusUpdate = self
                                        .get_block(Coordinate {
                                            x: x as Index,
                                            y: y as Index,
                                        })
                                        .enter_agent(agent_id);
                                    self.agent_positions[agent_id as usize] = Coordinate {
                                        x: x as Index,
                                        y: y as Index,
                                    };
                                    self.agent_start_positions[agent_id as usize] = Coordinate {
                                        x: x as Index,
                                        y: y as Index,
                                    };
                                    self.process_update(update);
                                } else {
                                    return Err(String::from(
                                        "Agent start placed on block that does not support agents",
                                    ));
                                }
                            } else {
                                return Err(String::from("Y coordinate of agent must be integer"));
                            }
                        } else {
                            return Err(String::from("X coordinate must be integer"));
                        }
                    }
                    _ => {
                        return Err(String::from(
                            "Agent start position must be array of two ints",
                        ));
                    }
                },
            }
        }

        Ok(())
    }

    fn load_victory_conditions(&mut self, table: &Table) -> Result<(), String> {
        let victory_conditions: &Table;

        match table.get("victory_conditions") {
            None => return Err(String::from("No victory conditions in level")),
            Some(victory_conditions_unwrapped) => match victory_conditions_unwrapped {
                Value::Table(t) => victory_conditions = t,
                _ => return Err(String::from("Error in loading victory conditions")),
            },
        }

        let condition_names = victory_conditions.keys();

        for condition in condition_names {
            let cond_value: u8;
            match victory_conditions.get(condition) {
                None => {
                    return Err(String::from(
                        "Some weird error in loading victory conditions",
                    ));
                }
                Some(num_unwrapped) => match num_unwrapped {
                    Value::Integer(num) => cond_value = *num as u8,
                    _ => {
                        return Err(String::from(
                            "Victory condition must be assigned integer value",
                        ));
                    }
                },
            }
            //TODO shpuld also read in min and max type conditions
            self.game_progress.insert(condition.clone(), 0);
            self.game_goal
                .insert(condition.clone(), GoalType::Exactly(cond_value));
        }

        self.game_progress_start = self.game_progress.clone();

        Ok(())
    }
}
