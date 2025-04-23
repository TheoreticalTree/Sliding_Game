/// The backend of the sliding game
mod blocks;
use blocks::{Air, BasicBlock, Block, block_factory};

pub mod utils_backend;
use utils_backend::{
    AgentID, Coordinate, Direction, GoalType, HitResult, Index, OUT_OF_BOUND, PlayerInput,
    ProgressUpdates, SlideType, StatusUpdate,
};
pub mod io;

use std::collections::{HashMap, HashSet};
use toml::{self, Table};

const MAXIMUM_STEP_NUMBER: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Running,
    Won,
    Lost,
}

pub struct Board {
    board: Vec<Box<dyn Block>>,
    /// Stores tables from whch the board original position can be restored
    board_start_configuration: Vec<Table>,
    x_size: Index,
    y_size: Index,
    num_agents: u8,
    num_agents_alive: u8,
    num_agents_must_finish: u8,
    game_progress: HashMap<String, u8>,
    game_progress_start: HashMap<String, u8>,
    game_goal: HashMap<String, GoalType>,
    agent_positions: Vec<Coordinate>,
    agent_start_positions: Vec<Coordinate>,
    game_state: GameState,
    action_stack: Vec<PlayerInput>,
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

    pub fn undo(&mut self) -> () {
        if self.action_stack.is_empty() {
            return;
        }

        // TODO consider maybe doing something more pretty than just copying the entire stack
        self.action_stack.pop();
        let action_stack_frozen: Vec<PlayerInput> = self.action_stack.clone();

        self.reset_game();

        print!("Num agents alive: {}\n", self.num_agents_alive);

        self.action_stack.clear();

        for action in action_stack_frozen {
            print!("Num agents alive: {}\n", self.num_agents_alive);
            match action {
                PlayerInput::Move(agent, direction) => self.move_agent(agent, direction),
                PlayerInput::Slide(agent, direction) => self.slide_agent(agent, direction),
            };
        }
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

        self.action_stack.push(PlayerInput::Move(agent, direction));

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

        self.action_stack
            .push(PlayerInput::Slide(start_agent, direction));

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
                print!("Lost due to too many steps\n");
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
                print!(
                    "Lost due to killing too many agents. Agents alive {} and {} agents must finish\n",
                    self.num_agents_alive, self.num_agents
                );
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
            print!("Won due to satisfying all conditions\n");
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
        coordinate.x >= self.x_size
            || coordinate.x < 0
            || coordinate.y >= self.y_size
            || coordinate.y < 0
    }

    #[allow(dead_code)]
    pub fn new_test() -> Self {
        let mut ret: Board = Board {
            board: vec![],
            board_start_configuration: vec![],
            x_size: 5,
            y_size: 5,
            num_agents: 2,
            num_agents_alive: 2,
            num_agents_must_finish: 2,
            game_progress: HashMap::new(),
            game_progress_start: HashMap::new(),
            game_goal: HashMap::new(),
            agent_positions: vec![],
            agent_start_positions: vec![],
            game_state: GameState::Running,
            action_stack: vec![],
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

    /// Resets the entire game except for the action stack back to the start of the level
    fn reset_game(&mut self) -> () {
        self.game_progress = self.game_progress_start.clone();

        for x in 0..self.x_size {
            for y in 0..self.y_size {
                self.set_block(
                    Coordinate {
                        x: x as Index,
                        y: y as Index,
                    },
                    block_factory(
                        &self.board_start_configuration[self.coordinate_to_index(Coordinate {
                            x: x as Index,
                            y: y as Index,
                        })],
                    )
                    .unwrap(),
                );
            }
        }

        self.num_agents_alive = self.num_agents;
        self.agent_positions = self.agent_start_positions.clone();
        for agent in 0..self.num_agents {
            let update: StatusUpdate = self
                .get_block(self.agent_positions[agent as usize])
                .enter_agent(agent);
            self.process_update(update);
        }

        self.game_state = GameState::Running;
    }
}
