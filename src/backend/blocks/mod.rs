/// Basic blocks used by the sliding game
use crate::utils::{
    AgentID, Coordinate, DestructionResult, Direction, HitResult, OUT_OF_BOUND, SlideType,
};

pub trait block {
    fn can_enter(&self) -> bool {
        false
    }

    fn enter_agent(&mut self, agent_id: AgentID) -> () {
        panic!("An agent tried to enter a block that does hat not implemented entering agents");
    }

    fn remove_agent(&mut self, agent_id: AgentID) -> () {
        panic!("Tried to remove an agent from a block that does not implement removing agents");
    }

    fn get_agents(&self) -> Vec<AgentID> {
        vec![]
    }

    fn on_hit(&mut self, direction: Direction) -> HitResult;

    fn on_destruction(&self) -> DestructionResult {
        DestructionResult::None
    }

    fn start_slide(&mut self, direction: Direction) -> SlideType {
        panic!("Tried to slide on a block that does not implement sliding");
    }
}

pub struct Air {}

impl block for Air {
    fn on_hit(&mut self, direction: Direction) -> HitResult {
        HitResult::NoResistance
    }
}

impl Air {
    pub fn new() -> Self {
        Air {}
    }
}
