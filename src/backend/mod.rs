/// The backend of the sliding game
mod blocks;
use std::collections::HashSet;

use blocks::{Air, block};

use crate::utils::{AgentID, Coordinate, Direction, Index, OUT_OF_BOUND, SlideType};

struct Board {
    board: Vec<Vec<Box<dyn block>>>,
    x_size: Index,
    y_size: Index,
    num_agents: u8,
    num_agents_alive: u8,
    num_agents_must_finish: u8,
    agent_positions: Vec<Coordinate>,
}

struct ActionLog {}

impl Board {
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

    pub fn slide_agent(&mut self, startAgent: AgentID, direction: Direction) -> ActionLog {
        assert!(startAgent < self.num_agents as AgentID);

        let mut current_coordinate: Coordinate = self.agent_positions[startAgent as usize];

        let mut current_sliding: SlideType =
            self.get_block(current_coordinate).start_slide(direction);
        let mut current_direction: Direction = direction;

        let mut steps_so_far = 0;

        while current_sliding == SlideType::NoSlide {
            let target_coordinate = current_coordinate.move_direction(current_direction);

            if self.out_of_bounds(target_coordinate) {
                if !self.get_block(current_coordinate).get_agents().is_empty() {
                    //TODO Some agents die now
                    self.num_agents_alive -=
                        self.get_block(current_coordinate).get_agents().len() as u8;
                    for agent in self.get_block(current_coordinate).get_agents() {
                        self.agent_positions[agent as usize] = OUT_OF_BOUND;
                    }

                    if self.num_agents_alive < self.num_agents_must_finish {
                        //TODO write logic for loosing the game
                        return ActionLog {};
                    }
                }

                self.set_block(current_coordinate, Box::new(Air::new()));
                current_sliding = SlideType::NoSlide;
                current_coordinate = OUT_OF_BOUND;
            } else {
                //
            }
            steps_so_far += 1;
        }

        ActionLog {}
    }

    fn set_block(&mut self, coordinate: Coordinate, block: Box<dyn block>) {
        assert!(!self.out_of_bounds(coordinate));

        self.board[coordinate.x as usize][coordinate.y as usize] = block;
    }

    fn get_block(&mut self, coordinate: Coordinate) -> &mut Box<dyn block> {
        assert!(!self.out_of_bounds(coordinate));

        &mut self.board[coordinate.x as usize][coordinate.y as usize]
    }

    fn out_of_bounds(&self, coordinate: Coordinate) -> bool {
        coordinate.x < 0
            || coordinate.y < 0
            || coordinate.x >= self.x_size
            || coordinate.y >= self.y_size
    }
}
