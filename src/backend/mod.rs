/// The backend of the sliding game
mod blocks;
use blocks::{Air, BasicBlock, Block};

use crate::utils::{AgentID, Coordinate, Direction, HitResult, Index, OUT_OF_BOUND, SlideType};

use std::collections::HashSet;

const MAXIMUM_STEP_NUMBER: usize = 100;

pub struct Board {
    board: Vec<Box<dyn Block>>,
    x_size: Index,
    y_size: Index,
    num_agents: u8,
    num_agents_alive: u8,
    num_agents_must_finish: u8,
    agent_positions: Vec<Coordinate>,
}

pub struct ActionLog {}

impl Board {
    pub fn read_block(&self, coordinate: Coordinate) -> &Box<dyn Block> {
        let index = self.coordinate_to_index(coordinate);
        &self.board[index]
    }

    pub fn get_dimensions(&self) -> (Index, Index) {
        (self.x_size, self.y_size)
    }

    pub fn can_move_agent(&self, agent: AgentID, direction: Direction) -> bool {
        assert!(agent < self.num_agents as AgentID);

        let target_coordinate = self.agent_positions[agent as usize].move_direction(direction);

        if self.out_of_bounds(target_coordinate) {
            false
        } else {
            self.read_block(target_coordinate).can_enter()
        }
    }

    pub fn move_agent(&mut self, agent: AgentID, direction: Direction) -> ActionLog {
        assert!(agent < self.num_agents as AgentID);

        let current_coordinate: Coordinate = self.agent_positions[agent as usize];
        let target_coordinate: Coordinate =
            self.agent_positions[agent as usize].move_direction(direction);

        if self.out_of_bounds(target_coordinate) {
            //TODO This should return a proper action log
            return ActionLog {};
        }

        if self.get_block(target_coordinate).can_enter() {
            self.get_block(current_coordinate).remove_agent(agent);
            self.get_block(target_coordinate).enter_agent(agent);
            self.agent_positions[agent as usize] = target_coordinate;
            return ActionLog {};
        }

        ActionLog {}
    }

    pub fn slide_agent(&mut self, start_agent: AgentID, direction: Direction) -> ActionLog {
        assert!(start_agent < self.num_agents as AgentID);

        let mut current_coordinate: Coordinate = self.agent_positions[start_agent as usize];

        let mut current_sliding: SlideType =
            self.get_block(current_coordinate).start_slide(direction);
        let current_direction = direction;

        let mut steps_so_far: usize = 0;

        while current_sliding != SlideType::NoSlide {
            print!(
                "Current Position: ({}, {})\n",
                current_coordinate.x, current_coordinate.y
            );

            let target_coordinate = current_coordinate.move_direction(current_direction);

            if self.out_of_bounds(target_coordinate) {
                self.move_block(current_coordinate, OUT_OF_BOUND);
                current_coordinate = OUT_OF_BOUND;
            } else {
                match self.get_block(target_coordinate).on_hit(current_direction) {
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
                //TODO this should be nicer
                panic!("Reached definitely infinite loop, trust me bro");
            }
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
                panic!("You killed too many agents");
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
                self.get_block(end).enter_agent(agent);
            }
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

    pub fn new_test() -> Self {
        let mut ret: Board = Board {
            board: vec![],
            x_size: 5,
            y_size: 5,
            num_agents: 2,
            num_agents_alive: 2,
            num_agents_must_finish: 2,
            agent_positions: vec![],
        };

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
