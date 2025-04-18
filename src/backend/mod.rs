/// The backend of the sliding game
mod blocks;
use blocks::{Air, BasicBlock, Block, block_factory};

use crate::utils_backend::{
    AgentID, Coordinate, Direction, GoalType, HitResult, Index, OUT_OF_BOUND, ProgressUpdates,
    SlideType, StatusUpdate,
};

use std::collections::{HashMap, HashSet};

use std::fs::File;
use std::io::Read;
use toml::{self, Table, Value};

const MAXIMUM_STEP_NUMBER: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Running,
    Won,
    Lost,
}

pub struct Board {
    board: Vec<Box<dyn Block>>,
    x_size: Index,
    y_size: Index,
    num_agents: u8,
    num_agents_alive: u8,
    num_agents_must_finish: u8,
    game_progress: HashMap<String, u8>,
    game_goal: HashMap<String, GoalType>,
    agent_positions: Vec<Coordinate>,
    game_state: GameState,
}

pub struct ActionLog {}

impl Board {
    pub fn get_game_state(&self) -> GameState {
        self.game_state
    }

    pub fn read_block(&self, coordinate: Coordinate) -> &Box<dyn Block> {
        assert!(!self.out_of_bounds(coordinate));
        let index = self.coordinate_to_index(coordinate);
        &self.board[index]
    }

    pub fn get_dimensions(&self) -> (Index, Index) {
        (self.x_size, self.y_size)
    }

    pub fn can_move_agent(&self, agent: AgentID, direction: Direction) -> bool {
        assert!(agent < self.num_agents as AgentID);
        assert!(self.game_state == GameState::Running);

        let target_coordinate = self.agent_positions[agent as usize].move_direction(direction);

        if self.out_of_bounds(target_coordinate) {
            false
        } else {
            self.read_block(target_coordinate).can_enter(direction)
        }
    }

    pub fn move_agent(&mut self, agent: AgentID, direction: Direction) -> ActionLog {
        assert!(agent < self.num_agents as AgentID);
        assert!(self.game_state == GameState::Running);

        let current_coordinate: Coordinate = self.agent_positions[agent as usize];
        let target_coordinate: Coordinate =
            self.agent_positions[agent as usize].move_direction(direction);

        if self.out_of_bounds(target_coordinate) {
            //TODO This should return a proper action log
        }

        if self.get_block(target_coordinate).can_enter(direction) {
            let mut update: StatusUpdate = self.get_block(current_coordinate).remove_agent(agent);
            self.process_update(update);
            update = self.get_block(target_coordinate).enter_agent(agent);
            self.process_update(update);
            self.agent_positions[agent as usize] = target_coordinate;
        }

        self.check_victory();

        ActionLog {}
    }

    pub fn slide_agent(&mut self, start_agent: AgentID, direction: Direction) -> ActionLog {
        assert!(start_agent < self.num_agents as AgentID);
        assert!(self.game_state == GameState::Running);

        let mut current_coordinate: Coordinate = self.agent_positions[start_agent as usize];

        let slide_output: (SlideType, StatusUpdate) =
            self.get_block(current_coordinate).start_slide(direction);

        self.process_update(slide_output.1);
        let mut current_sliding: SlideType = slide_output.0;
        let current_direction = direction;

        let mut steps_so_far: usize = 0;

        while current_sliding != SlideType::NoSlide {
            let target_coordinate = current_coordinate.move_direction(current_direction);

            if self.out_of_bounds(target_coordinate) {
                self.move_block(current_coordinate, OUT_OF_BOUND);
                current_coordinate = OUT_OF_BOUND;
                current_sliding = SlideType::NoSlide;
            } else {
                let hit_result = self.get_block(target_coordinate).on_hit(current_direction);
                self.process_update(hit_result.1);
                match hit_result.0 {
                    HitResult::Stop => {
                        current_sliding = SlideType::NoSlide;
                    }
                    HitResult::NoResistance => {
                        //TODO Check if the block does something special when destroyed
                        self.move_block(current_coordinate, target_coordinate);
                        current_coordinate = target_coordinate;
                    }
                    HitResult::MoveTo(new_position) => {
                        //TODO Write code to move block to new position
                        self.move_block(current_coordinate, new_position);
                        current_coordinate = new_position;
                    }
                }
            }
            steps_so_far += 1;

            if steps_so_far > MAXIMUM_STEP_NUMBER {
                self.game_state = GameState::Lost;
                return ActionLog {};
            }

            self.check_victory();
        }

        ActionLog {}
    }

    fn move_block(&mut self, start: Coordinate, mut end: Coordinate) -> () {
        if self.out_of_bounds(end) {
            end = OUT_OF_BOUND;
        }

        let agents_start: HashSet<AgentID> = self.get_block(start).get_agents();

        for agent in &agents_start {
            self.agent_positions[*agent as usize] = end;
        }

        if end == OUT_OF_BOUND {
            self.num_agents_alive -= agents_start.len() as u8;

            if self.num_agents_alive < self.num_agents_must_finish {
                //TODO write logic for loosing the game
                self.game_state = GameState::Lost;
            }

            //TODO check if destroying block sliding out has effect

            self.set_block(start, Box::new(Air::new()));
        } else {
            //TODO check if destroxing block that was at new position has an effect

            let agents_end: HashSet<AgentID> = self.get_block(end).get_agents();

            let index_start: usize = self.coordinate_to_index(start);
            let index_end: usize = self.coordinate_to_index(end);

            self.board.swap(index_start, index_end);
            self.set_block(start, Box::new(Air::new()));

            for agent in agents_end {
                let update: StatusUpdate = self.get_block(end).enter_agent(agent);
                self.process_update(update);
            }
        }
    }

    fn process_update(&mut self, update: StatusUpdate) {
        // Handle all updates to the game stats
        for progress_update in update.progress_updates {
            match progress_update {
                ProgressUpdates::IncreaseStat(name, val) => {
                    match self.game_progress.get_mut(&name) {
                        None => panic!("Tried to alter game stat that is not tracked"),
                        Some(num) => *num += val,
                    }
                }
                ProgressUpdates::DecreaseStat(name, val) => {
                    match self.game_progress.get_mut(&name) {
                        None => panic!("Tried to alter game stat that is not tracked"),
                        Some(num) => *num -= val,
                    }
                }
                ProgressUpdates::SetStat(name, val) => match self.game_progress.get_mut(&name) {
                    None => panic!("Tried to alter game stat that is not tracked"),
                    Some(num) => *num = val,
                },
            }
        }

        //TODO write signal handeling
    }

    fn check_victory(&mut self) -> () {
        let mut all_satisfied: bool = true;

        for key in self.game_goal.keys() {
            let current: u8;

            match self.game_progress.get(key) {
                None => panic!("Game has goal that is not tracked"),
                Some(val) => current = *val,
            };

            match self.game_goal.get(key) {
                None => panic!("Something went very wrong with the goal map"),
                Some(goal) => match goal {
                    GoalType::AtLeast(num) => {
                        if current < *num {
                            all_satisfied = false;
                            break;
                        }
                    }
                    GoalType::AtMost(num) => {
                        if current > *num {
                            all_satisfied = false;
                            break;
                        }
                    }
                    GoalType::Exactly(num) => {
                        if current != *num {
                            all_satisfied = false;
                            break;
                        }
                    }
                },
            }
        }

        if all_satisfied && self.game_state == GameState::Running {
            self.game_state = GameState::Won;
        } else if self.game_state != GameState::Lost {
            self.game_state = GameState::Running;
        }
    }

    fn set_block(&mut self, coordinate: Coordinate, block: Box<dyn Block>) {
        assert!(!self.out_of_bounds(coordinate));

        let index: usize = self.coordinate_to_index(coordinate);
        self.board[index] = block;
    }

    fn get_block(&mut self, coordinate: Coordinate) -> &mut Box<dyn Block> {
        assert!(!self.out_of_bounds(coordinate));

        let index: usize = self.coordinate_to_index(coordinate);
        &mut self.board[index]
    }

    fn coordinate_to_index(&self, coordinate: Coordinate) -> usize {
        coordinate.x as usize * self.y_size as usize + coordinate.y as usize
    }

    fn out_of_bounds(&self, coordinate: Coordinate) -> bool {
        coordinate.x >= self.x_size || coordinate.y >= self.y_size
    }

    #[allow(dead_code)]
    pub fn new_test() -> Self {
        let mut ret: Board = Board {
            board: vec![],
            x_size: 5,
            y_size: 5,
            num_agents: 2,
            num_agents_alive: 2,
            num_agents_must_finish: 2,
            game_progress: HashMap::new(),
            game_goal: HashMap::new(),
            agent_positions: vec![],
            game_state: GameState::Running,
        };

        ret.game_progress.insert(String::from("BlocksSatisfied"), 0);
        ret.game_goal
            .insert(String::from("BlocksSatisfied"), GoalType::Exactly(1));

        for _ in 0..25 {
            ret.board.push(Box::new(Air::new()));
        }

        ret.set_block(
            Coordinate { x: 4, y: 0 },
            Box::new(BasicBlock::new(true, &vec![0], SlideType::FastSlide, 0)),
        );
        ret.set_block(
            Coordinate { x: 0, y: 1 },
            Box::new(BasicBlock::new(true, &vec![], SlideType::FastSlide, 0)),
        );
        ret.set_block(
            Coordinate { x: 3, y: 1 },
            Box::new(BasicBlock::new(true, &vec![], SlideType::FastSlide, 0)),
        );
        ret.set_block(
            Coordinate { x: 2, y: 2 },
            Box::new(BasicBlock::new(true, &vec![], SlideType::FastSlide, 2)),
        );
        ret.set_block(
            Coordinate { x: 4, y: 4 },
            Box::new(BasicBlock::new(true, &vec![1], SlideType::FastSlide, 0)),
        );

        ret.agent_positions = vec![Coordinate { x: 4, y: 0 }, Coordinate { x: 4, y: 4 }];

        ret
    }
}

pub enum BoardLoadingError {
    FileNotFound,
    FileReadingError,
    TOMLParsingError,
    BoardDescriptionError(String),
}

impl Board {
    pub fn from_file(path: &str) -> Result<Self, BoardLoadingError> {
        let mut file: File;
        match File::open(path) {
            Err(_) => return Err(BoardLoadingError::FileNotFound),
            Ok(f) => file = f,
        }

        let mut board: Board = Board {
            board: vec![],
            x_size: 0,
            y_size: 0,
            num_agents: 0,
            num_agents_alive: 0,
            num_agents_must_finish: 0,
            game_progress: HashMap::new(),
            game_goal: HashMap::new(),
            agent_positions: vec![],
            game_state: GameState::Running,
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

        match table.get("x_size") {
            None => {
                return Err(BoardLoadingError::BoardDescriptionError(String::from(
                    "Missing x dimension",
                )));
            }
            Some(num_unwrapped) => match num_unwrapped {
                Value::Integer(num) => board.x_size = *num as u8,
                _ => {
                    return Err(BoardLoadingError::BoardDescriptionError(String::from(
                        "x_size must be an integer",
                    )));
                }
            },
        }
        match table.get("y_size") {
            None => {
                return Err(BoardLoadingError::BoardDescriptionError(String::from(
                    "Missing y dimension",
                )));
            }
            Some(num_unwrapped) => match num_unwrapped {
                Value::Integer(num) => board.y_size = *num as u8,
                _ => {
                    return Err(BoardLoadingError::BoardDescriptionError(String::from(
                        "y_size must be an integer",
                    )));
                }
            },
        }
        match table.get("num_agents") {
            None => {
                return Err(BoardLoadingError::BoardDescriptionError(String::from(
                    "Missing number of agents dimension",
                )));
            }
            Some(num_unwrapped) => match num_unwrapped {
                Value::Integer(num) => board.num_agents = *num as u8,
                _ => {
                    return Err(BoardLoadingError::BoardDescriptionError(String::from(
                        "number of agents must be an integer",
                    )));
                }
            },
        }
        match table.get("num_agents_must_finish") {
            None => board.num_agents_must_finish = board.num_agents,
            Some(num_unwrapped) => match num_unwrapped {
                Value::Integer(num) => board.num_agents_must_finish = *num as u8,
                _ => {
                    return Err(BoardLoadingError::BoardDescriptionError(String::from(
                        "number of agents must be an integer",
                    )));
                }
            },
        }

        board.num_agents_alive = board.num_agents;
        board
            .agent_positions
            .resize(board.num_agents as usize, OUT_OF_BOUND);

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
                Value::Integer(val_int) => self.x_size = *val_int as u8,
                _ => return Err(String::from("x dimension was not an integer")),
            },
        }
        match level_table.get("y_size") {
            None => return Err(String::from("Missing y dimension\n")),
            Some(val) => match val {
                Value::Integer(val_int) => self.y_size = *val_int as u8,
                _ => return Err(String::from("y dimension was not an integer")),
            },
        }

        for _ in 0..(self.x_size + 1) * (self.y_size + 1) {
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

        Ok(())
    }

    fn load_agents(&mut self, table: &Table) -> Result<(), String> {
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
                                    x: x as u8,
                                    y: y as u8,
                                }) {
                                    return Err(String::from(
                                        "Agent start position is out of bounds",
                                    ));
                                }
                                if self
                                    .get_block(Coordinate {
                                        x: x as u8,
                                        y: y as u8,
                                    })
                                    .can_enter(Direction::None)
                                {
                                    let update: StatusUpdate = self
                                        .get_block(Coordinate {
                                            x: x as u8,
                                            y: y as u8,
                                        })
                                        .enter_agent(agent_id);
                                    self.agent_positions[agent_id as usize] = Coordinate {
                                        x: x as u8,
                                        y: y as u8,
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
        Ok(())
    }
}
