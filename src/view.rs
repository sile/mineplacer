use crate::{asset::Assets, model::Model};
use pagurus::{
    event::{Event, MouseEvent},
    failure::OrFail,
    image::{Canvas, Color},
    spatial::{Contains, Position, Region, Size},
    Result,
};

const BACKGROUND_COLOR: Color = Color::rgb(133, 131, 131);

#[derive(Debug, Default)]
pub struct Window {
    assets: Assets,
    focus_cell: Option<Position>,
    pressing: usize,
    last_pixel_position: Position, // TODO: rename
}

impl Window {
    pub const MARGIN_SIZE: u32 = 4;
    pub const CELL_SIZE: u32 = 16;
    pub const HEADER_SIZE: Size = Size::from_wh(Self::CELL_SIZE * 16, 24);
    pub const BOARD_SIZE: Size = Size::from_wh(Self::CELL_SIZE * 16, Self::CELL_SIZE * 30);
    pub const WINDOW_SIZE: Size = Size::from_wh(
        Self::BOARD_SIZE.width + Self::MARGIN_SIZE * 2,
        Self::BOARD_SIZE.height + Self::MARGIN_SIZE * 3 + Self::HEADER_SIZE.height,
    );

    pub fn load_assets(&mut self) -> Result<()> {
        self.assets.load().or_fail()?;
        Ok(())
    }

    fn header_region(&self) -> Region {
        Self::HEADER_SIZE
            .to_region()
            .move_x(Self::MARGIN_SIZE as i32)
            .move_y(Self::MARGIN_SIZE as i32)
    }

    fn board_region(&self) -> Region {
        Self::BOARD_SIZE
            .to_region()
            .move_x(Self::MARGIN_SIZE as i32)
            .move_y(self.header_region().end().y + Self::MARGIN_SIZE as i32)
    }

    pub fn render(&self, canvas: &mut Canvas, model: &Model) -> Result<()> {
        canvas.fill_color(BACKGROUND_COLOR);

        let header_region = self.header_region();
        self.render_header(&mut canvas.subregion(header_region))
            .or_fail()?;

        let board_region = self.board_region();
        self.render_board(&mut canvas.subregion(board_region), model)
            .or_fail()?;

        Ok(())
    }

    fn render_header(&self, canvas: &mut Canvas) -> Result<()> {
        let sprite = self.assets.header_sprite().or_fail()?;
        canvas.draw_sprite(&sprite);
        Ok(())
    }

    fn render_board(&self, canvas: &mut Canvas, model: &Model) -> Result<()> {
        canvas.fill_color(Color::WHITE);

        let cell_region = Size::square(Self::CELL_SIZE).to_region();
        let sprite = self.assets.cell_sprites().or_fail()?;
        for (position, mines) in model.surrounding_mines() {
            let cell_region = cell_region.shift_x(position.x).shift_y(position.y);
            let mut canvas = canvas.offset(cell_region.position);

            if mines >= 0 {
                canvas.draw_sprite(&sprite.over);
            } else {
                canvas.draw_sprite(&sprite.under);
            }

            if model.has_mine(position) {
                canvas.draw_sprite(&sprite.mine);
            } else {
                canvas.draw_sprite(&sprite.numbers[mines.abs() as usize]);
                if self.focus_cell == Some(position) && mines > 0 {
                    canvas.draw_sprite(&sprite.focus);
                }
            }
        }

        Ok(())
    }

    pub fn handle_event(&mut self, event: Event, model: &mut Model) -> Result<()> {
        match event {
            Event::Mouse(event) => {
                self.handle_mouse_event(event, model).or_fail()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_mouse_event(&mut self, event: MouseEvent, model: &mut Model) -> Result<()> {
        let pixel_position = event.position();
        self.focus_cell = None;

        if matches!(event, MouseEvent::Down { .. }) {
            self.pressing += 1;
        }

        if self.board_region().contains(&pixel_position) {
            if self.pressing == 0 {
                let cell_pixel_position = pixel_position - self.board_region().start();
                let cell_position = cell_pixel_position / Self::CELL_SIZE;
                self.focus_cell = Some(cell_position);
            }
        }

        if matches!(event, MouseEvent::Down { .. }) {
            self.last_pixel_position = pixel_position;
        }

        if matches!(event, MouseEvent::Up { .. }) {
            self.pressing = self.pressing.saturating_sub(1);

            if self.pressing == 0 && self.last_pixel_position == pixel_position {
                if self.board_region().contains(&pixel_position) {
                    let cell_pixel_position = pixel_position - self.board_region().start();
                    let cell_position = cell_pixel_position / Self::CELL_SIZE;
                    model.handle_click(cell_position).or_fail()?;
                }
            }
        }

        Ok(())
    }
}
