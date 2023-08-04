use pagurus::event::TimeoutTag;

pub const RENDERING_TIMEOUT: TimeoutTag = TimeoutTag::new(0);
pub const START_8X15_TIMEOUT: TimeoutTag = TimeoutTag::new(1);
pub const START_16X30_TIMEOUT: TimeoutTag = TimeoutTag::new(2);
pub const START_16X30_WITH_WORMHOLE_TIMEOUT: TimeoutTag = TimeoutTag::new(3);
