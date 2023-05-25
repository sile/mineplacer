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
}

impl Model {
    pub fn initialize<S: System>(&mut self, system: &mut S) -> Result<()> {
        self.rng = StdRng::from_clock_seed(system.clock_unix_time());
        Ok(())
    }

    pub fn generate_board(&mut self) -> Result<()> {
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
        Ok(())
    }

    pub fn board(&self) -> impl '_ + Iterator<Item = (Position, CellType)> {
        self.positions().map(|p| (p, self.board.cell_type(p)))
    }

    pub fn handle_click(&mut self, position: Position) -> Result<()> {
        Size::from_wh(WIDTH as u32, HEIGHT as u32)
            .contains(&position)
            .or_fail()?;

        let cell = &mut self.board.cells[position.y as usize][position.x as usize];
        cell.actual_mine = !cell.actual_mine;

        Ok(())
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
    fn cell_type(&self, p: Position) -> CellType {
        if self.cells[p.y as usize][p.x as usize].actual_mine {
            return CellType::Mine;
        }

        let mut expected: usize = 0;
        let mut actual: usize = 0;
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

        CellType::Number(expected.saturating_sub(actual))
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Cell {
    expected_mine: bool,
    actual_mine: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    Mine,
    Number(usize),
}
