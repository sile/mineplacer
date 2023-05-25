use pagurus::{
    failure::OrFail,
    image::{Canvas, Color},
    spatial::Size,
    Result,
};

use crate::{asset::Assets, model::Model};

const BACKGROUND_COLOR: Color = Color::rgb(133, 131, 131);

#[derive(Debug, Default)]
pub struct Window {
    assets: Assets,
}

impl Window {
    pub const MARGIN_SIZE: u32 = 8;
    pub const CELL_SIZE: u32 = 16;
    pub const HEADER_SIZE: Size = Size::from_wh(Self::CELL_SIZE * 16, 48);
    pub const BOARD_SIZE: Size = Size::from_wh(Self::CELL_SIZE * 16, Self::CELL_SIZE * 30);
    pub const WINDOW_SIZE: Size = Size::from_wh(
        Self::BOARD_SIZE.width + Self::MARGIN_SIZE * 2,
        Self::BOARD_SIZE.height + Self::MARGIN_SIZE * 3 + Self::HEADER_SIZE.height,
    );

    pub fn load_assets(&mut self) -> Result<()> {
        self.assets.load().or_fail()?;
        Ok(())
    }

    pub fn render(&self, canvas: &mut Canvas, model: &Model) -> Result<()> {
        canvas.fill_color(BACKGROUND_COLOR);

        let header_region = Self::HEADER_SIZE
            .to_region()
            .move_x(Self::MARGIN_SIZE as i32)
            .move_y(Self::MARGIN_SIZE as i32);
        self.render_header(&mut canvas.subregion(header_region))
            .or_fail()?;

        let board_region = Self::BOARD_SIZE
            .to_region()
            .move_x(Self::MARGIN_SIZE as i32)
            .move_y(header_region.end().y + Self::MARGIN_SIZE as i32);
        self.render_board(&mut canvas.subregion(board_region), model)
            .or_fail()?;

        Ok(())
    }

    fn render_header(&self, canvas: &mut Canvas) -> Result<()> {
        canvas.fill_color(Color::WHITE);
        Ok(())
    }

    fn render_board(&self, canvas: &mut Canvas, model: &Model) -> Result<()> {
        canvas.fill_color(Color::WHITE);

        let cell_close_sprite = self.assets.sprite_cell_close().or_fail()?;
        let cell_region = Size::square(Self::CELL_SIZE).to_region();
        for y in 0..30 {
            for x in 0..16 {
                let cell_region = cell_region.shift_x(x).shift_y(y);
                canvas
                    .offset(cell_region.position)
                    .draw_sprite(&cell_close_sprite);
            }
        }

        let sprite = self.assets.cell_sprites().or_fail()?;
        for (position, cell_type) in model.board() {
            let cell_region = cell_region.shift_x(position.x).shift_y(position.y);
            let mut canvas = canvas.offset(cell_region.position);

            canvas.draw_sprite(&sprite.open);
            match cell_type {
                crate::model::CellType::Mine => {
                    canvas.draw_sprite(&sprite.mine);
                }
                crate::model::CellType::Number(n) => {
                    canvas.draw_sprite(&sprite.numbers[n]);
                }
            }
        }

        Ok(())
    }
}
