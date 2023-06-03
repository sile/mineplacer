use crate::{asset::Assets, model::Model};
use pagurus::{
    event::{Event, MouseEvent},
    failure::OrFail,
    image::{Canvas, Color, Sprite},
    spatial::{Contains, Position, Region, Size},
    Result,
};

const BACKGROUND_COLOR: Color = Color::rgb(133, 131, 131);

#[derive(Debug, Default)]
pub struct Window {
    assets: Assets,
    help_button: Button,
    start_8x15_button: Button,
    start_16x30_button: Button,
    pressing: bool,
}

impl Window {
    pub const MARGIN_SIZE: u32 = 3;
    pub const CELL_SIZE: u32 = 16;
    pub const HEADER_SIZE: Size = Size::from_wh(Self::CELL_SIZE * 16, 24);
    pub const BOARD_SIZE: Size = Size::from_wh(Self::CELL_SIZE * 16, Self::CELL_SIZE * 30);
    pub const WINDOW_SIZE: Size = Size::from_wh(
        Self::BOARD_SIZE.width + Self::MARGIN_SIZE * 2,
        Self::BOARD_SIZE.height + Self::MARGIN_SIZE * 3 + Self::HEADER_SIZE.height,
    );

    pub fn load_assets(&mut self) -> Result<()> {
        self.assets.load().or_fail()?;

        let button_region =
            Region::new(self.header_region().position, Size::from_wh(20, 21)).move_y(1);
        let [start_8x16, start_16x30, help] = self.assets.button_sprites().or_fail()?;
        self.start_8x15_button = Button::new(button_region.move_x(165), start_8x16);
        self.start_16x30_button = Button::new(button_region.move_x(191), start_16x30);
        self.help_button = Button::new(button_region.move_x(232), help);

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
        self.render_header(&mut canvas.subregion(header_region), model)
            .or_fail()?;

        self.help_button.render(canvas).or_fail()?;
        self.start_8x15_button.render(canvas).or_fail()?;
        self.start_16x30_button.render(canvas).or_fail()?;

        let board_region = self.board_region();
        self.render_board(&mut canvas.subregion(board_region), model)
            .or_fail()?;

        Ok(())
    }

    fn render_header(&self, canvas: &mut Canvas, model: &Model) -> Result<()> {
        let sprite = self.assets.header_sprite().or_fail()?;
        canvas.draw_sprite(&sprite);

        let elapsed = std::cmp::min(999, model.elapsed_time().as_secs()) as usize;
        let offset = Position::from_xy(24 + 10 * 2, 5);
        self.render_number(canvas, offset, elapsed).or_fail()?;

        let offset = Position::from_xy(84 + 10, 5);
        self.render_number(canvas, offset, model.remaining_mines())
            .or_fail()?;

        Ok(())
    }

    fn render_number(
        &self,
        canvas: &mut Canvas,
        mut offset: Position,
        mut number: usize,
    ) -> Result<()> {
        let sprites = self.assets.digit_sprites().or_fail()?;
        let mut first = true;
        while number > 0 || first {
            let digit = number % 10;
            let sprite = &sprites[digit];
            canvas.offset(offset).draw_sprite(sprite);
            offset.x -= 10;
            number /= 10;
            first = false;
        }
        Ok(())
    }

    fn render_board(&self, canvas: &mut Canvas, model: &Model) -> Result<()> {
        canvas.fill_color(Color::WHITE);

        let cell_region = Size::square(Self::CELL_SIZE).to_region();
        let sprite = self.assets.cell_sprites().or_fail()?;
        for (position, mines) in model.surrounding_mines() {
            let cell_region = cell_region.shift_x(position.x).shift_y(position.y);
            let mut canvas = canvas.offset(cell_region.position);

            if mines <= 0 {
                canvas.draw_sprite(&sprite.just);
            } else {
                canvas.draw_sprite(&sprite.over);
            }

            if model.has_mine(position) {
                canvas.draw_sprite(&sprite.mine);

                let mut canvas = canvas.offset(Position::from_xy(8, 8));
                if mines > 0 {
                    canvas.draw_sprite(&sprite.mini_numbers[mines as usize - 1]);
                } else if mines < 0 {
                    canvas.draw_sprite(&sprite.mini_warning);
                }
            } else {
                if mines > 0 {
                    canvas.draw_sprite(&sprite.numbers[mines as usize - 1]);
                } else if mines < 0 {
                    canvas.draw_sprite(&sprite.warning);
                }
            }
        }

        Ok(())
    }

    pub fn handle_event(&mut self, event: Event, model: &mut Model) -> Result<()> {
        match &event {
            Event::Mouse(event) => {
                self.handle_mouse_event(event, model).or_fail()?;
            }
            _ => {}
        }
        self.start_8x15_button.handle_event(&event).or_fail()?;
        self.start_16x30_button.handle_event(&event).or_fail()?;
        self.help_button.handle_event(&event).or_fail()?;

        Ok(())
    }

    pub fn take_help_button_clicked(&mut self) -> bool {
        self.help_button.take_clicked()
    }

    pub fn take_start_8x15_button_clicked(&mut self) -> bool {
        self.start_8x15_button.take_clicked()
    }

    pub fn take_start_16x30_button_clicked(&mut self) -> bool {
        self.start_16x30_button.take_clicked()
    }

    fn handle_mouse_event(&mut self, event: &MouseEvent, model: &mut Model) -> Result<()> {
        let pixel_position = event.position();

        if matches!(event, MouseEvent::Down { .. }) {
            self.pressing = true;
        }

        if matches!(event, MouseEvent::Up { .. }) && self.pressing {
            self.pressing = false;

            if self.board_region().contains(&pixel_position) {
                let cell_pixel_position = pixel_position - self.board_region().start();
                let cell_position = cell_pixel_position / Self::CELL_SIZE;
                model.handle_click(cell_position).or_fail()?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Button {
    region: Region,
    sprite: Sprite,
    state: ButtonState,
}

impl Button {
    pub fn new(region: Region, sprite: Sprite) -> Self {
        Self {
            region,
            sprite,
            state: ButtonState::Normal,
        }
    }

    pub fn render(&self, canvas: &mut Canvas) -> Result<()> {
        let mut region = self.region;
        match self.state {
            ButtonState::Normal => {}
            ButtonState::Hover => {
                region.position.y += 1;
                region.size.height -= 1;
            }
            ButtonState::Pressed | ButtonState::Clicked => {
                region.position.y += 2;
                region.size.height -= 2;
            }
        };
        canvas.subregion(region).draw_sprite(&self.sprite);
        Ok(())
    }

    pub fn handle_event(&mut self, event: &Event) -> Result<()> {
        let Event::Mouse(event) = event else { return Ok(()) };
        let position = event.position();
        if !self.region.contains(&position) {
            self.state = ButtonState::Normal;
            return Ok(());
        }

        match (self.state, event) {
            (ButtonState::Normal, MouseEvent::Move { .. }) => {
                self.state = ButtonState::Hover;
            }
            (_, MouseEvent::Down { .. }) => {
                self.state = ButtonState::Pressed;
            }
            (ButtonState::Pressed, MouseEvent::Up { .. }) => {
                self.state = ButtonState::Clicked;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn take_clicked(&mut self) -> bool {
        if self.state == ButtonState::Clicked {
            self.state = ButtonState::Normal;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    #[default]
    Normal,
    Hover,
    Pressed,
    Clicked,
}
