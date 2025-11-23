use enum_iterator::Sequence;
use macroquad::{
    color::{Color, colors::*},
    rand::ChooseRandom
};

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
}

impl Block {
    /// returns if this block is static and not updated
    fn is_static(&self) -> bool {
        return matches!(self, Block::Air | Block::Stone);
    }

    /// returns if this block is solid
    fn is_solid(&self) -> bool {
        return matches!(self, Block::Stone | Block::Sand);
    }

    /// returns if moving into the other block is a valid operation
    fn can_move_to(&self, other: Self) -> bool {
        !other.is_solid() && other != *self && (!matches!(*self, Block::Water(_)) || !matches!(other, Block::Water(_)))
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
            _ => unimplemented!("Block Type: {self:?} does not have a color")
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

                match block {
                    Block::Sand => {
                        self.apply_gravity(x, y);
                    }
                    Block::Water(flow_dir) => {
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
                                self.blocks[y][position] = Block::Water(flowing);

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
        let mut fell = false;
        let block = self.blocks[y][x];
        // dont bother checking if on floor
        if y < self.height - 1 {
            let below = y + 1;

            if block.can_move_to(self.blocks[below][x]) {
                // Fall straight
                self.blocks[y][x] = self.blocks[below][x];
                self.blocks[below][x] = block;

                fell = true;
            } else {
                // Fall to the side
                let mut positions = Vec::new();

                if x > 0 && block.can_move_to(self.blocks[below][x-1]) && !self.blocks[y][x-1].is_solid() {
                    positions.push(x-1);
                }
                if x < self.width-1 && block.can_move_to(self.blocks[below][x+1]) && !self.blocks[y][x+1].is_solid() {
                    positions.push(x+1);
                }

                if let Some(&position) = positions.choose() {
                    self.blocks[y][x] = self.blocks[below][position];
                    self.blocks[below][position] = block;

                    fell = true;
                }
            }
        }

        return fell;
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
