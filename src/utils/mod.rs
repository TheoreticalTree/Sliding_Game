pub type Index = u8;

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
        }
    }
}

pub const OUT_OF_BOUND: Coordinate = Coordinate { x: 255, y: 255 };

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlideType {
    NoSlide,
    FastSlide,
    SlowSlide(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitResult {
    Stop,
    NoResistance,
    MoveTo(Coordinate),
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
