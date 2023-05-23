use pagurus::{image::Sprite, Result};

#[derive(Debug, Default)]
pub struct Assets {
    sprite: Option<Sprite>,
}

impl Assets {
    pub fn load(&mut self) -> Result<()> {
        Ok(())
    }
}
