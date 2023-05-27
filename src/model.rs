use std::time::Duration;

use pagurus::{
    failure::OrFail,
    random::StdRng,
    spatial::{Contains, Position, Size},
    Result, System,
};
use rand::seq::SliceRandom;

const MINES: usize = 99;
const WIDTH: usize = 16;
const HEIGHT: usize = 30;
const BOARD_SIZE: Size = Size::from_wh(WIDTH as u32, HEIGHT as u32);

#[derive(Debug, Default, Clone)]
pub struct Model {
    rng: StdRng,
    board: Board,
    start_time: Duration,
    elapsed_time: Duration,
}

impl Model {
    pub fn initialize<S: System>(&mut self, system: &mut S) -> Result<()> {
        self.rng = StdRng::from_clock_seed(system.clock_unix_time());
        Ok(())
    }

    pub fn generate_board<S: System>(&mut self, system: &mut S) -> Result<()> {
        self.board = Board::default();
        let mut mines = [0; WIDTH * HEIGHT];
        for p in self.positions() {
            let i = p.y as usize * WIDTH as usize + p.x as usize;
            mines[i] = i;
        }
        mines.shuffle(&mut self.rng);
        for i in &mines[0..MINES] {
            let y = i / WIDTH;
            let x = i % WIDTH;
            self.board.cells[y][x].expected_mine = true;
        }

        self.start_time = system.clock_game_time();
        Ok(())
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

    pub fn handle_click(&mut self, position: Position) -> Result<()> {
        Size::from_wh(WIDTH as u32, HEIGHT as u32)
            .contains(&position)
            .or_fail()?;

        let cell = &mut self.board.cells[position.y as usize][position.x as usize];
        cell.actual_mine = !cell.actual_mine;

        Ok(())
    }

    pub fn has_mine(&self, p: Position) -> bool {
        self.board.cells[p.y as usize][p.x as usize].actual_mine
    }

    fn positions(&self) -> impl '_ + Iterator<Item = Position> {
        (0..HEIGHT).flat_map(|y| (0..WIDTH).map(move |x| Position::from_xy(x as i32, y as i32)))
    }
}

#[derive(Debug, Default, Clone)]
struct Board {
    cells: [[Cell; WIDTH]; HEIGHT],
}

impl Board {
    fn surrounding_mines(&self, p: Position) -> isize {
        let mut expected: isize = 0;
        let mut actual: isize = 0;
        for y_delta in [-1, 0, 1] {
            for x_delta in [-1, 0, 1] {
                let p = p.move_y(y_delta).move_x(x_delta);
                if !BOARD_SIZE.to_region().contains(&p) {
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
