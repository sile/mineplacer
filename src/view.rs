use crate::{
    asset::Assets,
    model::{Model, State},
};
use orfail::OrFail;
use pagurus::{
    event::{Event, MouseEvent},
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
    start_16x30_with_wormhole_button: Button,
    start_custom_button: Button,
    pressing: bool,
}

impl Window {
    const MARGIN_SIZE: u32 = 3;
    const CELL_SIZE: u32 = 16;

    fn header_size(&self, model: &Model) -> Size {
        Size::from_wh(Self::CELL_SIZE * model.board_size().width, 24)
    }

    fn board_size(&self, model: &Model) -> Size {
        Size::from_wh(
            Self::CELL_SIZE * model.board_size().width,
            Self::CELL_SIZE * model.board_size().height,
        )
    }

    pub fn window_size(&self, model: &Model) -> Size {
        Size::from_wh(
            Self::MARGIN_SIZE * 2 + self.board_size(model).width,
            Self::MARGIN_SIZE * 3 + self.header_size(model).height + self.board_size(model).height,
        )
    }

    pub fn load_assets(&mut self, model: &Model) -> Result<()> {
        self.assets.load().or_fail()?;

        let button_region =
            Region::new(self.header_region(model).position, Size::from_wh(20, 21)).move_y(1);
        let [start_8x16, start_16x30, start_16x30_with_wormhole, help] =
            self.assets.button_sprites().or_fail()?;
        self.start_8x15_button = Button::new(button_region.move_x(145), start_8x16);
        self.start_16x30_button = Button::new(button_region.move_x(169), start_16x30);
        self.start_16x30_with_wormhole_button =
            Button::new(button_region.move_x(193), start_16x30_with_wormhole);
        self.help_button = Button::new(button_region.move_x(232), help);

        let custom = self.assets.custom_button_sprite().or_fail()?;
        self.start_custom_button = Button::new(
            Region::new(
                self.help_button.region.move_x(-32).position,
                Size::from_wh(23, 21),
            ),
            custom,
        );

        Ok(())
    }

    fn header_region(&self, model: &Model) -> Region {
        self.header_size(model)
            .to_region()
            .move_x(Self::MARGIN_SIZE as i32)
            .move_y(Self::MARGIN_SIZE as i32)
    }

    fn board_region(&self, model: &Model) -> Region {
        self.board_size(model)
            .to_region()
            .move_x(Self::MARGIN_SIZE as i32)
            .move_y(self.header_region(model).end().y + Self::MARGIN_SIZE as i32)
    }

    pub fn render(&self, canvas: &mut Canvas, model: &Model) -> Result<()> {
        canvas.fill_color(BACKGROUND_COLOR);

        let header_region = self.header_region(model);
        self.render_header(&mut canvas.subregion(header_region), model)
            .or_fail()?;

        self.help_button.render(canvas).or_fail()?;
        if model.is_custom_mode() {
            self.start_custom_button.render(canvas).or_fail()?;
            let mut offset = self
                .start_custom_button
                .render_region()
                .position
                .move_x(5)
                .move_y(14);
            self.render_small_number(canvas, offset, model.board_size().width as usize)
                .or_fail()?;
            offset = offset.move_x(12);
            self.render_small_number(canvas, offset, model.board_size().height as usize)
                .or_fail()?;
        } else {
            self.start_8x15_button.render(canvas).or_fail()?;
            self.start_16x30_button.render(canvas).or_fail()?;
            self.start_16x30_with_wormhole_button
                .render(canvas)
                .or_fail()?;
        }

        let board_region = self.board_region(model);
        self.render_board(&mut canvas.subregion(board_region), model)
            .or_fail()?;

        Ok(())
    }

    fn render_header(&self, canvas: &mut Canvas, model: &Model) -> Result<()> {
        let sprite = self
            .assets
            .header_sprite(model.is_custom_mode())
            .or_fail()?;
        canvas.draw_sprite(&sprite);

        if model.is_custom_mode() {
            self.render_number(canvas, Position::from_xy(164, 5), model.wormholes())
                .or_fail()?;

            if matches!(model.state(), State::Initial) {
                self.render_number(canvas, Position::from_xy(84 + 20, 5), model.mines())
                    .or_fail()?;
            }
        }

        let elapsed_time = match model.state() {
            State::Initial => return Ok(()),
            State::Playing => model.elapsed_time(),
            State::Won { elapsed_time } => elapsed_time,
        };

        let elapsed = std::cmp::min(999, elapsed_time.as_secs()) as usize;
        let offset = Position::from_xy(24 + 10 * 2, 5);
        self.render_number(canvas, offset, elapsed).or_fail()?;

        let mut offset = Position::from_xy(84 + 10, 5);
        if model.is_custom_mode() {
            offset.x += 10;
        };
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

    fn render_small_number(
        &self,
        canvas: &mut Canvas,
        mut offset: Position,
        mut number: usize,
    ) -> Result<()> {
        let sprites = self.assets.small_digit_sprites().or_fail()?;
        let mut first = true;
        while number > 0 || first {
            let digit = number % 10;
            let sprite = &sprites[digit];
            canvas.offset(offset).draw_sprite(sprite);
            offset.x -= 4;
            number /= 10;
            first = false;
        }
        Ok(())
    }

    fn render_board(&self, canvas: &mut Canvas, model: &Model) -> Result<()> {
        let cell_region = Size::square(Self::CELL_SIZE).to_region();
        let sprite = self.assets.cell_sprites().or_fail()?;
        for (position, mines) in model.surrounding_mines() {
            if model.has_wormhole(position) {
                continue;
            }

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
                #[allow(clippy::comparison_chain)]
                if mines > 0 {
                    canvas.draw_sprite(&sprite.mini_numbers[mines as usize - 1]);
                } else if mines < 0 {
                    canvas.draw_sprite(&sprite.mini_warning);
                }
            } else if mines > 0 {
                canvas.draw_sprite(&sprite.numbers[mines as usize - 1]);
            } else if mines < 0 {
                canvas.draw_sprite(&sprite.warning);
            }
        }

        Ok(())
    }

    pub fn handle_event(&mut self, event: Event, model: &mut Model) -> Result<()> {
        if let Event::Mouse(event) = &event {
            self.handle_mouse_event(event, model).or_fail()?;
        }
        if model.is_custom_mode() {
            self.start_custom_button.handle_event(&event).or_fail()?;
        } else {
            self.start_8x15_button.handle_event(&event).or_fail()?;
            self.start_16x30_button.handle_event(&event).or_fail()?;
            self.start_16x30_with_wormhole_button
                .handle_event(&event)
                .or_fail()?;
        }
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

    pub fn take_start_16x30_with_wormhole_button_clicked(&mut self) -> bool {
        self.start_16x30_with_wormhole_button.take_clicked()
    }

    pub fn take_start_custom_button_clicked(&mut self) -> bool {
        self.start_custom_button.take_clicked()
    }

    fn handle_mouse_event(&mut self, event: &MouseEvent, model: &mut Model) -> Result<()> {
        let pixel_position = event.position();

        if matches!(event, MouseEvent::Down { .. }) {
            self.pressing = true;
        }

        if matches!(event, MouseEvent::Up { .. }) && self.pressing {
            self.pressing = false;

            if self.board_region(model).contains(&pixel_position) {
                let cell_pixel_position = pixel_position - self.board_region(model).start();
                let cell_position = cell_pixel_position / Self::CELL_SIZE;
                model.handle_click(cell_position);
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
        canvas
            .subregion(self.render_region())
            .draw_sprite(&self.sprite);
        Ok(())
    }

    pub fn render_region(&self) -> Region {
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
        region
    }

    pub fn handle_event(&mut self, event: &Event) -> Result<()> {
        let Event::Mouse(event) = event else {
            return Ok(());
        };
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
