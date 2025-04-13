use std::collections::HashSet;

/// Basic blocks used by the sliding game
use crate::utils_backend::{
    AgentID, DestructionResult, Direction, HitResult, ProgressUpdates, SlideType, StatusUpdate,
    TextureType,
};

pub trait Block {
    fn can_enter(&self, _direction: Direction) -> bool {
        false
    }

    fn can_exit(&self, _direction: Direction) -> bool {
        false
    }

    #[must_use]
    fn enter_agent(&mut self, _agent: AgentID) -> StatusUpdate {
        panic!("An agent tried to enter a block that does hat not implemented entering agents");
    }

    #[must_use]
    fn remove_agent(&mut self, _agent: AgentID) -> StatusUpdate {
        panic!("Tried to remove an agent from a block that does not implement removing agents");
    }

    #[must_use]
    fn get_agents(&self) -> HashSet<AgentID> {
        HashSet::new()
    }

    #[must_use]
    fn on_hit(&mut self, direction: Direction) -> (HitResult, StatusUpdate);

    #[must_use]
    fn on_destruction(&self) -> (DestructionResult, StatusUpdate) {
        (DestructionResult::None, StatusUpdate::nothing())
    }

    #[must_use]
    fn start_slide(&mut self, _direction: Direction) -> (SlideType, StatusUpdate) {
        panic!("Tried to slide on a block that does not implement sliding");
    }

    #[must_use]
    fn get_texture(&self) -> TextureType {
        panic!("Tried to render on a block that does not implement rendering");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Air {}

impl Block for Air {
    fn on_hit(&mut self, _direction: Direction) -> (HitResult, StatusUpdate) {
        (HitResult::NoResistance, StatusUpdate::nothing())
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
    fn can_enter(&self, _direction: Direction) -> bool {
        self.passable
    }

    fn enter_agent(&mut self, agent: AgentID) -> StatusUpdate {
        assert!(!self.agents.contains(&agent));

        self.agents.insert(agent);

        if self.num_goal_agents > 0 && self.num_goal_agents == self.agents.len() as u8 {
            StatusUpdate {
                progress_updates: vec![ProgressUpdates::IncreaseStat(
                    String::from("BlocksSatisfied"),
                    1,
                )],
                signals: vec![],
            }
        } else {
            StatusUpdate::nothing()
        }
    }

    fn get_agents(&self) -> HashSet<AgentID> {
        self.agents.clone()
    }

    fn remove_agent(&mut self, agent: AgentID) -> StatusUpdate {
        self.agents.remove(&agent);

        if self.num_goal_agents > 0 && self.num_goal_agents - 1 == self.agents.len() as u8 {
            StatusUpdate {
                progress_updates: vec![ProgressUpdates::DecreaseStat(
                    String::from("BlocksSatisfied"),
                    1,
                )],
                signals: vec![],
            }
        } else {
            StatusUpdate::nothing()
        }
    }

    fn on_hit(&mut self, _direction: Direction) -> (HitResult, StatusUpdate) {
        //TODO Maybe add more functionality here later
        (HitResult::Stop, StatusUpdate::nothing())
    }

    fn start_slide(&mut self, _direction: Direction) -> (SlideType, StatusUpdate) {
        (self.default_slide, StatusUpdate::nothing())
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
