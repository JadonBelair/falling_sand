use enum_iterator::Sequence;
use macroquad::{
    color::{Color, colors::*},
    rand::ChooseRandom
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    Solid,
    Liquid,
    Gas,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Sequence)]
/// flowing directions for liquids
pub enum FlowDir {
    None,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Sequence)]
/// blocks within a falling sand world
pub enum Block {
    Air,
    Stone,
    Sand,
    Water(FlowDir),
    Lava(FlowDir),
}

impl Block {
    /// returns if this block is static and not updated
    fn is_static(&self) -> bool {
        return matches!(self, Block::Air | Block::Stone);
    }

    /// returns if moving into the other block is a valid operation
    fn can_move_to(&self, other: Self) -> bool {
        // !other.is_solid() && other != *self && (!matches!(*self, Block::Water(_)) || !matches!(other, Block::Water(_)))
        self.density() > other.density()
    }

    /// returns the density of the block, can be negative
    fn density(&self) -> i32 {
        match self {
            Self::Air => 0,
            Self::Water(_) => 1,
            Self::Lava(_) => 2,
            Self::Sand => 3,
            Self::Stone => 100,
        }
    }

    /// returs which state of matter the block is
    fn state(&self) -> State {
        match self {
            Self::Air => State::Gas,
            Self::Water(_) => State::Liquid,
            Self::Lava(_) => State::Liquid,
            Self::Sand => State::Solid,
            Self::Stone => State::Solid,
        }
    }

    /// returns the flow direction of the liquid
    fn get_flow_dir(&self) -> FlowDir {
        match self {
            Block::Water(flow_dir) | Block::Lava(flow_dir) => *flow_dir,
            _ => FlowDir::None
        }
    }

    // returns the color of the block
    pub fn get_color(&self) -> Color {
        match self {
            Block::Stone => {
                GRAY
            }
            Block::Sand => {
                YELLOW
            }
            Block::Water(_) => {
                BLUE
            }
            Self::Lava(_) => {
                RED
            }
            _ => unimplemented!("Block Type: {self:?} does not have a color")
        }
    }

    /// creates a copy of the liquid with a different flow direction
    fn clone_with_flow(&self, flowing: FlowDir) -> Block {
        match self {
            Block::Water(_) => Block::Water(flowing),
            Block::Lava(_) => Block::Lava(flowing),
            _ => unreachable!()
        }
    }
}

/// falling sand world
pub struct World {
    blocks: Vec<Vec<Block>>,
    width: usize,
    height: usize,
}

impl World {
    /// returns new world with specified size
    pub fn new(width: usize, height: usize) -> Self {
        let blocks = vec![vec![Block::Air; width]; height];
        Self {
            blocks,
            width,
            height,
        }
    }

    /// updates the world state
    pub fn update(&mut self) {
        // list of block positions that have already been updated this state
        // mainly used for blocks that move side-to-side
        let mut updated = Vec::new();

        for y in (0..self.height).rev() {
            for x in 0..self.width {
                let block = self.blocks[y][x];
                if block.is_static() || updated.contains(&(x, y)) {
                    continue;
                }

                match block.state() {
                    State::Solid => {
                        self.apply_gravity(x, y);
                    }
                    State::Liquid => {
                        let flow_dir = block.get_flow_dir();

                        // Move Side-to-Side if the water didn't flow downwards
                        if !self.apply_gravity(x, y) {
                            let mut positions = Vec::new();

                            if x > 0 && block.can_move_to(self.blocks[y][x-1]) {
                                positions.push(x-1);
                            }
                            if x < self.width-1 && block.can_move_to(self.blocks[y][x+1]) {
                                positions.push(x+1);
                            }

                            // remove other direction if continuing flow direction is possible
                            if flow_dir == FlowDir::Left && positions.contains(&(x-1)) && positions.len() > 1 {
                                positions.remove(1);
                            } else if flow_dir == FlowDir::Right && positions.contains(&(x+1)) && positions.len() > 1 {
                                positions.remove(0);
                            }

                            if let Some(&position) = positions.choose() {
                                let flowing = if position < x {
                                    FlowDir::Left
                                } else {
                                    FlowDir::Right
                                };

                                self.blocks[y][x] = self.blocks[y][position];
                                self.blocks[y][position] = block.clone_with_flow(flowing);

                                updated.push((position, y));
                            }
                        }
                    }
                    _ => unimplemented!("Block Type: {block:?} is unimplemented!")
                }
            }
        }
    }

    /// applies gravity to the specified position and returns if the block there fell
    fn apply_gravity(&mut self, x: usize, y: usize) -> bool {
        // dont bother checking if on floor
        if y >= self.height-1 {
            return false
        }

        let mut fell = false;
        let block = self.blocks[y][x];

        let below = y + 1;

        if self.can_fall_to((x, y), (x, below)) {
            // Fall straight
            self.blocks[y][x] = self.blocks[below][x];
            self.blocks[below][x] = block;

            fell = true;
        } else {
            // Fall to the side
            let mut positions = Vec::new();

            if x > 0 && self.can_fall_to((x, y), (x-1, below)) {
                positions.push(x-1);
            }
            if x < self.width-1 && self.can_fall_to((x, y), (x+1, below)) {
                positions.push(x+1);
            }

            if let Some(&position) = positions.choose() {
                self.blocks[y][x] = self.blocks[below][position];
                self.blocks[below][position] = block;

                fell = true;
            }
        }

        return fell;
    }

    /// returns if the block at the given position
    /// can fall into the target position due to gravity
    fn can_fall_to(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        if to.0 >= self.width || to.1 >= self.height {
            return false
        }

        let from_block = self.blocks[from.1][from.0];
        let to_block = self.blocks[to.1][to.0];

        if to.0 != from.0 {
            let above_to = self.blocks[from.1][to.0];
            return from_block.can_move_to(to_block) && from_block.can_move_to(above_to);
        } else {
            return from_block.can_move_to(to_block);
        }
    }

    /// returns the block at the given position
    pub fn get_block(&self, x: usize, y: usize) -> Option<Block> {
        Some(*(self.blocks.get(y)?.get(x)?))
    }

    /// sets the block at the given position
    pub fn set_block(&mut self, x: usize, y: usize, block: Block) {
        if x < self.width && y < self.height {
            self.blocks[y][x] = block;
        }
    }

    /// returns the width of the world
    pub fn get_width(&self) -> usize {
        self.width
    }

    /// returns the height of the world
    pub fn get_height(&self) -> usize {
        self.height
    }
}
