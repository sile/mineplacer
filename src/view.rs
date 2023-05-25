use pagurus::spatial::Size;

#[derive(Debug, Default)]
pub struct Window {}

impl Window {
    pub const MARGIN_SIZE: u32 = 8;
    pub const CELL_SIZE: u32 = 16;
    pub const HEADER_SIZE: Size = Size::from_wh(Self::CELL_SIZE * 16, 48);
    pub const WINDOW_SIZE: Size = Size::from_wh(
        Self::CELL_SIZE * 16 + Self::MARGIN_SIZE * 2,
        Self::CELL_SIZE * 30 + Self::MARGIN_SIZE * 3 + Self::HEADER_SIZE.height,
    );
}
