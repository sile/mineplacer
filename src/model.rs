use pagurus::{
    failure::OrFail,
    random::StdRng,
    spatial::{Contains, Position, Region, Size},
    Result, System,
};
use rand::seq::SliceRandom;
use std::time::Duration;

const WIDTH: usize = 16;
const HEIGHT: usize = 30;

#[derive(Debug, Default, Clone, Copy)]
pub enum Level {
    Small,
    #[default]
    Large,
    LargeWithWormhole,
    Custom {
        width: usize,
        height: usize,
        mines: usize,
        wormholes: usize,
    },
}

impl Level {
    pub fn from_qs(qs: &str) -> Result<Option<Self>> {
        if !qs.starts_with('?') {
            return Ok(None);
        }

        let mut width: usize = 16;
        let mut height: usize = 30;
        let mut mines: usize = 99;
        let mut wormholes: usize = 99;
        let mut custom = false;

        for kv in qs[1..].split('&') {
            match kv.splitn(2, '=').collect::<Vec<_>>().as_slice() {
                ["width", v] => {
                    width = v
                        .parse::<usize>()
                        .ok()
                        .filter(|v| (16..=64).contains(v))
                        .or_fail()
                        .map_err(|f| {
                            f.message("'width' parameter should be a integer between 16 and 64")
                        })?;
                    custom = true;
                }
                ["height", v] => {
                    height = v
                        .parse::<usize>()
                        .ok()
                        .filter(|v| (16..=64).contains(v))
                        .or_fail()
                        .map_err(|f| {
                            f.message("'height' parameter should be a integer between 16 and 64")
                        })?;
                    custom = true;
                }
                ["mines", v] => {
                    mines = v
                        .parse::<usize>()
                        .ok()
                        .filter(|v| (1..=999).contains(v))
                        .or_fail()
                        .map_err(|f| {
                            f.message("'mines' parameter should be a integer between 1 and 1000")
                        })?;
                    custom = true;
                }
                ["wormholes", v] => {
                    wormholes = v
                        .parse::<usize>()
                        .ok()
                        .filter(|v| (0..=999).contains(v))
                        .or_fail()
                        .map_err(|f| {
                            f.message(
                                "'wormholes' parameter should be a integer between 0 and 1000",
                            )
                        })?;
                    custom = true;
                }
                _ => {}
            }
        }
        if !custom {
            return Ok(None);
        }

        let cells = width * height;
        (mines + wormholes <= cells)
            .or_fail()
            .map_err(|f| f.message("Too many mines and wormholes"))?;

        Ok(Some(Self::Custom {
            width,
            height,
            mines,
            wormholes,
        }))
    }

    fn mines(self) -> usize {
        match self {
            Level::Small => 15,
            Level::Large => 99,
            Level::LargeWithWormhole => 99,
            Level::Custom { mines, .. } => mines,
        }
    }

    fn wormholes(self) -> usize {
        match self {
            Level::Small => 0,
            Level::Large => 0,
            Level::LargeWithWormhole => 99,
            Level::Custom { wormholes, .. } => wormholes,
        }
    }

    fn width(self) -> usize {
        match self {
            Level::Small => 8,
            Level::Large => 16,
            Level::LargeWithWormhole => 16,
            Level::Custom { width, .. } => width,
        }
    }

    fn height(self) -> usize {
        match self {
            Level::Small => 15,
            Level::Large => 30,
            Level::LargeWithWormhole => 30,
            Level::Custom { height, .. } => height,
        }
    }

    fn offset(self) -> Position {
        match self {
            Level::Small => Position::from_xy(4, 7),
            Level::Large => Position::from_xy(0, 0),
            Level::LargeWithWormhole => Position::from_xy(0, 0),
            Level::Custom { .. } => Position::from_xy(0, 0),
        }
    }

    pub fn is_custom(self) -> bool {
        matches!(self, Level::Custom { .. })
    }

    pub fn board_size(self) -> Size {
        if let Level::Custom { width, height, .. } = self {
            Size::from_wh(width as u32, height as u32)
        } else {
            Size::from_wh(WIDTH as u32, HEIGHT as u32)
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum State {
    #[default]
    Initial,
    Playing,
    Won {
        elapsed_time: Duration,
    },
}

#[derive(Debug, Default, Clone)]
pub struct Model {
    rng: StdRng,
    board: Board,
    remaining_mines: usize,
    start_time: Duration,
    elapsed_time: Duration,
    level: Level,
    state: State,
}

impl Model {
    pub fn initialize<S: System>(&mut self, system: &mut S) -> Result<()> {
        self.rng = StdRng::from_clock_seed(system.clock_unix_time());
        self.board
            .set_board_size(Size::from_wh(WIDTH as u32, HEIGHT as u32));
        Ok(())
    }

    pub fn set_custom_level(&mut self, level: Level) {
        self.board.set_board_size(level.board_size());
        self.level = level;
    }

    pub fn start_game<S: System>(&mut self, system: &mut S, mut level: Level) -> Result<()> {
        if self.level.is_custom() {
            level = self.level;
        }
        self.level = level;
        self.board = Board::default();
        self.board.set_board_size(level.board_size());
        self.board.region = Region::new(
            level.offset(),
            Size::from_wh(level.width() as u32, level.height() as u32),
        );

        let mut mines = self.board.region.iter().collect::<Vec<_>>();
        mines.shuffle(&mut self.rng);
        for p in &mines[..level.wormholes()] {
            self.board.cells[p.y as usize][p.x as usize].wormhole = true;
        }
        for p in &mines[level.wormholes()..][..level.mines()] {
            self.board.cells[p.y as usize][p.x as usize].expected_mine = true;
        }

        self.start_time = system.clock_game_time();
        self.remaining_mines = level.mines();
        self.state = State::Playing;
        Ok(())
    }

    pub fn board_size(&self) -> Size {
        self.level.board_size()
    }

    pub fn wormholes(&self) -> usize {
        self.level.wormholes()
    }

    pub fn mines(&self) -> usize {
        self.level.mines()
    }

    pub fn is_custom_mode(&self) -> bool {
        self.level.is_custom()
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn has_wormhole(&self, p: Position) -> bool {
        self.board.cells[p.y as usize][p.x as usize].wormhole
    }

    pub fn remaining_mines(&self) -> usize {
        self.remaining_mines
    }

    pub fn update_elapsed_time<S: System>(&mut self, system: &S) {
        self.elapsed_time = system.clock_game_time() - self.start_time;
    }

    pub fn elapsed_time(&self) -> Duration {
        self.elapsed_time
    }

    pub fn surrounding_mines(&self) -> impl '_ + Iterator<Item = (Position, isize)> {
        self.board
            .region
            .iter()
            .map(|p| (p, self.board.surrounding_mines(p)))
    }

    pub fn handle_click(&mut self, position: Position) {
        if self.state != State::Playing {
            return;
        }
        if !self.board.region.contains(&position) {
            return;
        }
        if self.has_wormhole(position) {
            return;
        }

        let cell = &mut self.board.cells[position.y as usize][position.x as usize];

        if !cell.actual_mine && self.remaining_mines == 0 {
            return;
        }

        cell.actual_mine = !cell.actual_mine;
        if cell.actual_mine {
            self.remaining_mines -= 1;
        } else {
            self.remaining_mines += 1;
        }

        if self.remaining_mines == 0
            && self
                .surrounding_mines()
                .filter(|(p, _)| !self.has_wormhole(*p))
                .all(|(_, m)| m == 0)
        {
            self.state = State::Won {
                elapsed_time: self.elapsed_time(),
            }
        }
    }

    pub fn has_mine(&self, p: Position) -> bool {
        self.board.cells[p.y as usize][p.x as usize].actual_mine
    }
}

#[derive(Debug, Default, Clone)]
struct Board {
    cells: Vec<Vec<Cell>>,
    region: Region,
}

impl Board {
    fn surrounding_mines(&self, p: Position) -> isize {
        let mut expected: isize = 0;
        let mut actual: isize = 0;
        for y_delta in [-1, 0, 1] {
            for x_delta in [-1, 0, 1] {
                let p = p.move_y(y_delta).move_x(x_delta);
                if !self.region.contains(&p) {
                    continue;
                }

                let cell = self.cells[p.y as usize][p.x as usize];
                if cell.expected_mine {
                    expected += 1;
                }
                if cell.actual_mine {
                    actual += 1;
                }
            }
        }
        expected - actual
    }

    fn set_board_size(&mut self, size: Size) {
        self.cells = vec![vec![Cell::default(); size.width as usize]; size.height as usize];
        self.region.size = size;
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Cell {
    expected_mine: bool,
    actual_mine: bool,
    wormhole: bool,
}
