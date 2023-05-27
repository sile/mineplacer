use pagurus::{failure::OrFail, image::Sprite, spatial::Size, Result};

#[derive(Debug, Default)]
pub struct Assets {
    sprite: Option<Sprite>,
}

impl Assets {
    pub fn load(&mut self) -> Result<()> {
        self.sprite = Some(decode_sprite(include_bytes!("../assets/ui.png")).or_fail()?);

        Ok(())
    }

    pub fn cell_sprites(&self) -> Result<CellSprites> {
        let sprite = self.sprite.as_ref().or_fail()?;
        let region = Size::square(16).to_region();
        Ok(CellSprites {
            over: sprite.clip(region.shift_x(1)).or_fail()?,
            mine: sprite.clip(region.shift_x(2)).or_fail()?,
            focus: sprite.clip(region.shift_x(3)).or_fail()?,
            under: sprite.clip(region.shift_y(1).shift_x(3)).or_fail()?,
            numbers: [
                sprite.clip(region).or_fail()?,
                sprite.clip(region.shift_y(1).shift_x(0)).or_fail()?,
                sprite.clip(region.shift_y(1).shift_x(1)).or_fail()?,
                sprite.clip(region.shift_y(1).shift_x(2)).or_fail()?,
                sprite.clip(region.shift_y(2).shift_x(0)).or_fail()?,
                sprite.clip(region.shift_y(2).shift_x(1)).or_fail()?,
                sprite.clip(region.shift_y(2).shift_x(2)).or_fail()?,
                sprite.clip(region.shift_y(3).shift_x(0)).or_fail()?,
                sprite.clip(region.shift_y(3).shift_x(1)).or_fail()?,
                sprite.clip(region.shift_y(3).shift_x(2)).or_fail()?,
            ],
        })
    }

    pub fn header_sprite(&self) -> Result<Sprite> {
        let sprite = self.sprite.as_ref().or_fail()?;
        let region = Size::from_wh(16 * 16, 24).to_region().move_y(64);
        sprite.clip(region).or_fail()
    }

    pub fn digit_sprites(&self) -> Result<[Sprite; 10]> {
        let sprite = self.sprite.as_ref().or_fail()?;
        let region = Size::from_wh(8, 16).to_region().move_x(64);
        Ok([
            sprite.clip(region.shift_x(1)).or_fail()?,
            sprite.clip(region.shift_x(3)).or_fail()?,
            sprite.clip(region.shift_x(5)).or_fail()?,
            sprite.clip(region.shift_x(7)).or_fail()?,
            sprite.clip(region.shift_x(9)).or_fail()?,
            sprite.clip(region.shift_x(1).shift_y(1)).or_fail()?,
            sprite.clip(region.shift_x(3).shift_y(1)).or_fail()?,
            sprite.clip(region.shift_x(5).shift_y(1)).or_fail()?,
            sprite.clip(region.shift_x(7).shift_y(1)).or_fail()?,
            sprite.clip(region.shift_x(9).shift_y(1)).or_fail()?,
        ])
    }

    pub fn button_sprites(&self) -> Result<[Sprite; 3]> {
        let sprite = self.sprite.as_ref().or_fail()?;
        let region = Size::from_wh(24, 24).to_region().move_y(32).move_x(48);
        Ok([
            sprite.clip(region).or_fail()?,
            sprite.clip(region.shift_x(1)).or_fail()?,
            sprite.clip(region.shift_x(2)).or_fail()?,
        ])
    }
}

fn decode_sprite(png: &[u8]) -> Result<Sprite> {
    let decoder = png::Decoder::new(png);
    let mut reader = decoder.read_info().or_fail()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).or_fail()?;
    let bytes = &buf[..info.buffer_size()];
    let size = Size::from_wh(info.width, info.height);
    (info.bit_depth == png::BitDepth::Eight)
        .or_fail()
        .map_err(|e| e.message(format!("unsupported PNG bit depth: {:?}", info.bit_depth)))?;

    match info.color_type {
        png::ColorType::Rgb => Sprite::from_rgb24_bytes(bytes, size).or_fail(),
        png::ColorType::Rgba => Sprite::from_rgba32_bytes(bytes, size).or_fail(),
        png::ColorType::Grayscale => Sprite::from_grayscale8_bytes(bytes, size).or_fail(),
        png::ColorType::GrayscaleAlpha => {
            Sprite::from_grayscale_alpha16_bytes(bytes, size).or_fail()
        }
        png::ColorType::Indexed => {
            let palette = reader.info().palette.as_ref().or_fail()?;
            let mut rgb_bytes = Vec::with_capacity(size.len());
            for i in bytes.iter().copied().map(usize::from) {
                rgb_bytes.push(palette[i * 3]);
                rgb_bytes.push(palette[i * 3 + 1]);
                rgb_bytes.push(palette[i * 3 + 2]);
            }
            Sprite::from_rgb24_bytes(&rgb_bytes, size).or_fail()
        }
    }
}

#[derive(Debug)]
pub struct CellSprites {
    pub over: Sprite,
    pub under: Sprite,
    pub mine: Sprite,
    pub focus: Sprite,
    pub numbers: [Sprite; 10],
}
