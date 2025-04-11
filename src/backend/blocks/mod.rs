use std::collections::HashSet;

/// Basic blocks used by the sliding game
use crate::utils::{
    AgentID, Coordinate, DestructionResult, Direction, HitResult, OUT_OF_BOUND, SlideType,
    TextureType,
};

pub trait Block {
    fn can_enter(&self) -> bool {
        false
    }

    fn enter_agent(&mut self, agent: AgentID) -> () {
        panic!("An agent tried to enter a block that does hat not implemented entering agents");
    }

    fn remove_agent(&mut self, agent: AgentID) -> () {
        panic!("Tried to remove an agent from a block that does not implement removing agents");
    }

    fn get_agents(&self) -> HashSet<AgentID> {
        HashSet::new()
    }

    fn on_hit(&mut self, direction: Direction) -> HitResult;

    fn on_destruction(&self) -> DestructionResult {
        DestructionResult::None
    }

    fn start_slide(&mut self, direction: Direction) -> SlideType {
        panic!("Tried to slide on a block that does not implement sliding");
    }

    fn get_texture(&self) -> TextureType {
        panic!("Tried to render on a block that does not implement rendering");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Air {}

impl Block for Air {
    fn on_hit(&mut self, direction: Direction) -> HitResult {
        HitResult::NoResistance
    }

    fn get_texture(&self) -> TextureType {
        TextureType::None
    }
}

impl Air {
    pub fn new() -> Self {
        Air {}
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicBlock {
    passable: bool,
    agents: HashSet<AgentID>,
    default_slide: SlideType,
    num_goal_agents: u8,
}

impl Block for BasicBlock {
    fn can_enter(&self) -> bool {
        self.passable
    }

    fn enter_agent(&mut self, agent: AgentID) -> () {
        assert!(!self.agents.contains(&agent));

        self.agents.insert(agent);
    }

    fn get_agents(&self) -> HashSet<AgentID> {
        self.agents.clone()
    }

    fn remove_agent(&mut self, agent: AgentID) -> () {
        self.agents.remove(&agent);
    }

    fn on_hit(&mut self, direction: Direction) -> HitResult {
        //TODO Maybe add more functionality here later
        HitResult::Stop
    }

    fn start_slide(&mut self, direction: Direction) -> SlideType {
        self.default_slide
    }

    fn get_texture(&self) -> TextureType {
        //TODO detail this more later
        if self.num_goal_agents > 0 {
            return TextureType::Goal(self.num_goal_agents);
        }

        if self.passable {
            return TextureType::BasicBlock;
        } else {
            return TextureType::BasicImpassable;
        }
    }
}

impl BasicBlock {
    pub fn new(
        passable: bool,
        starting_agents: &Vec<AgentID>,
        default_slide: SlideType,
        num_goal_agents: u8,
    ) -> Self {
        BasicBlock {
            passable,
            agents: HashSet::from_iter(starting_agents.iter().cloned()),
            default_slide,
            num_goal_agents,
        }
    }
}
