pub type Index = i8;

pub type AgentID = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Coordinate {
    pub x: Index,
    pub y: Index,
}

impl Coordinate {
    pub fn move_direction(&self, direction: Direction) -> Coordinate {
        match direction {
            Direction::Up => Coordinate {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Down => Coordinate {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Left => Coordinate {
                x: self.x - 1,
                y: self.y,
            },
            Direction::Right => Coordinate {
                x: self.x + 1,
                y: self.y,
            },
            Direction::None => self.clone(),
        }
    }
}

pub const OUT_OF_BOUND: Coordinate = Coordinate { x: -1, y: -1 };

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlideType {
    NoSlide,
    FastSlide,
    #[allow(dead_code)]
    SlowSlide(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitResult {
    Stop,
    NoResistance,
    #[allow(dead_code)]
    MoveTo(Coordinate),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GoalType {
    #[allow(dead_code)]
    AtLeast(u8),
    #[allow(dead_code)]
    AtMost(u8),
    Exactly(u8),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProgressUpdates {
    IncreaseStat(String, u8),
    DecreaseStat(String, u8),
    #[allow(dead_code)]
    SetStat(String, u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameSignal {}

pub struct StatusUpdate {
    pub progress_updates: Vec<ProgressUpdates>,
    #[allow(dead_code)]
    pub signals: Vec<GameSignal>,
}

impl StatusUpdate {
    pub fn nothing() -> Self {
        StatusUpdate {
            progress_updates: vec![],
            signals: vec![],
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DestructionResult {
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureType {
    None,
    BasicBlock,
    BasicImpassable,
    Goal(u8),
}
