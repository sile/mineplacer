use pagurus::{
    failure::OrFail,
    random::StdRng,
    spatial::{Contains, Position, Size},
    Result, System,
};
use rand::seq::SliceRandom;
use std::time::Duration;

#[derive(Debug, Default, Clone, Copy)]
pub enum Level {
    Small,
    #[default]
    Large,
}

impl Level {
    fn mines(self) -> usize {
        match self {
            Level::Small => 15,
            Level::Large => 99,
        }
    }

    fn width(self) -> usize {
        match self {
            Level::Small => 8,
            Level::Large => 16,
        }
    }

    fn height(self) -> usize {
        match self {
            Level::Small => 15,
            Level::Large => 30,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Model {
    rng: StdRng,
    board: Board,
    remaining_mines: usize,
    start_time: Duration,
    elapsed_time: Duration,
    level: Level,
}

impl Model {
    pub fn initialize<S: System>(&mut self, system: &mut S) -> Result<()> {
        self.rng = StdRng::from_clock_seed(system.clock_unix_time());
        Ok(())
    }

    pub fn start_game<S: System>(&mut self, system: &mut S, level: Level) -> Result<()> {
        self.level = level;
        self.board = Board::default();

        self.board.cells = vec![vec![Cell::default(); level.width()]; level.height()];
        self.board.size = Size::from_wh(level.width() as u32, level.height() as u32);

        let mut mines = vec![0; level.width() * level.height()];
        for p in self.positions() {
            let i = p.y as usize * level.width() as usize + p.x as usize;
            mines[i] = i;
        }
        mines.shuffle(&mut self.rng);
        for i in &mines[0..level.mines()] {
            let y = i / level.width();
            let x = i % level.width();
            self.board.cells[y][x].expected_mine = true;
        }

        self.start_time = system.clock_game_time();
        self.remaining_mines = level.mines();
        Ok(())
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
        self.positions()
            .map(|p| (p, self.board.surrounding_mines(p)))
    }

    pub fn handle_click(&mut self, position: Position) -> Result<bool> {
        self.board.size.contains(&position).or_fail()?;

        let cell = &mut self.board.cells[position.y as usize][position.x as usize];

        if !cell.actual_mine && self.remaining_mines == 0 {
            return Ok(false);
        }

        cell.actual_mine = !cell.actual_mine;
        if cell.actual_mine {
            self.remaining_mines -= 1;
        } else {
            self.remaining_mines += 1;
        }

        Ok(true)
    }

    pub fn has_mine(&self, p: Position) -> bool {
        self.board.cells[p.y as usize][p.x as usize].actual_mine
    }

    fn positions(&self) -> impl '_ + Iterator<Item = Position> {
        self.board.size.to_region().iter()
    }
}

#[derive(Debug, Default, Clone)]
struct Board {
    cells: Vec<Vec<Cell>>,
    size: Size,
}

impl Board {
    fn surrounding_mines(&self, p: Position) -> isize {
        let mut expected: isize = 0;
        let mut actual: isize = 0;
        for y_delta in [-1, 0, 1] {
            for x_delta in [-1, 0, 1] {
                let p = p.move_y(y_delta).move_x(x_delta);
                if !self.size.to_region().contains(&p) {
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
}

#[derive(Debug, Default, Clone, Copy)]
struct Cell {
    expected_mine: bool,
    actual_mine: bool,
}
