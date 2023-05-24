use pagurus::spatial::Size;

pub mod asset;
pub mod game;
pub mod model;
pub mod tag;
pub mod view;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Level {
    pub size: Size,
    pub mines: usize,
}

impl Level {
    pub const EASY: Self = Self::new(Size::from_wh(9, 9), 10);
    pub const MEDIUM: Self = Self::new(Size::from_wh(16, 16), 40);
    pub const HARD: Self = Self::new(Size::from_wh(30, 16), 99);

    pub const fn new(size: Size, mines: usize) -> Self {
        Self { size, mines }
    }
}
