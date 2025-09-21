use ::rand::Rng;
use macroquad::prelude::*;
use macroquad::rand::ChooseRandom;
use slide_puzzle::{Board, Cell, Piece, cell};

pub struct State {
    pieces: Vec<Option<Piece>>,

    rows: usize,
    cols: usize,

    blank_cell: Cell,
}

impl State {
    pub fn new(rows: usize, cols: usize) -> Self {
        let blank_index = rows * cols - 1;
        let mut pieces = (0..=blank_index)
            .map(|n| Some(Piece::new(n + 1)))
            .collect::<Vec<_>>();
        pieces[blank_index] = None;

        State {
            rows,
            cols,
            pieces,
            blank_cell: cell!(blank_index, rows, cols),
        }
    }

    fn neighbors(&self, cell: Cell) -> Vec<Cell> {
        self.neighbors_at(cell.x, cell.y)
    }

    fn neighbors_at(&self, x: usize, y: usize) -> Vec<Cell> {
        let dx = [1, 0, -1, 0];
        let dy = [0, 1, 0, -1];
        (0..4)
            .filter_map(|k| {
                let nx = x as isize + dx[k];
                let ny = y as isize + dy[k];

                if nx < 0 || ny < 0 || nx >= self.cols as isize || ny >= self.rows as isize {
                    return None;
                }
                Some(cell!(nx as usize, ny as usize))
            })
            .collect::<Vec<_>>()
    }

    #[inline]
    pub fn swap(&mut self, cell1: Cell, cell2: Cell) {
        assert!(
            self.blank_cell == cell1 || self.blank_cell == cell2,
            "swap target should equal blank_index"
        );

        self.pieces.swap(
            cell1.as_index(self.rows, self.cols),
            cell2.as_index(self.rows, self.cols),
        );
        match self.blank_cell {
            c if c == cell1 => self.blank_cell = cell2,
            c if c == cell2 => self.blank_cell = cell1,
            _ => unreachable!("swap target should equal blank_index"),
        }
    }

    pub fn shuffle(&mut self) {
        let mut rng = ::rand::rng();
        let n = rng.random_range(500..=1000);

        for _ in 0..n {
            if let Some(next) = self.neighbors(self.blank_cell).choose() {
                self.swap(self.blank_cell, *next);
            }
        }
    }

    pub fn is_finished(&self) -> bool {
        self.blank_cell == cell!(self.cols - 1, self.rows - 1)
            && self.pieces.iter().enumerate().all(|(i, piece)| {
                if let Some(piece) = piece {
                    i == piece.num - 1
                } else {
                    i == self.rows * self.cols - 1
                }
            })
    }

    pub fn movable_position(&self, cell: Cell) -> Option<Cell> {
        let neighbors = self.neighbors(cell);
        neighbors.into_iter().find(|&n| n == self.blank_cell)
    }
}

fn apply_state(board: &mut Board, state: &State) {
    for (i, piece) in state.pieces.iter().enumerate() {
        board.put_piece(cell!(i, state.rows, state.cols), *piece);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    const DEFAULT_BOARD_ROWS: usize = 4;
    const DEFAULT_BOARD_COLS: usize = 4;

    let args: Vec<_> = std::env::args().collect();
    let rows: usize = args
        .get(1)
        .and_then(|a| a.parse::<usize>().ok())
        .unwrap_or(DEFAULT_BOARD_ROWS);
    let cols: usize = args
        .get(2)
        .and_then(|a| a.parse::<usize>().ok())
        .unwrap_or(DEFAULT_BOARD_COLS);

    println!("(rows, cols) = ({}, {})", rows, cols);

    const BOARD_WIDTH: usize = 700;
    const BOARD_HEIGHT: usize = 700;
    const BOARD_BORDER: usize = 10;

    let board_relative_path = vec2(50.0, 80.0);
    let mut board = Board::new(
        vec2(BOARD_WIDTH as f32, BOARD_HEIGHT as f32),
        BOARD_BORDER as f32,
        (rows, cols),
    );

    let mut state = State::new(rows, cols);
    while state.is_finished() {
        state.shuffle();
    }

    loop {
        clear_background(WHITE);

        if is_key_down(KeyCode::Q) {
            break;
        }

        if state.is_finished() {
            board.draw(board_relative_path);
            draw_text("GAME CLEAR", 50.0, 50.0, 40.0, RED);
            next_frame().await;
            continue;
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let (x, y) = mouse_position();
            if let Some(cell) = board.get_cell(vec2(x, y) - board_relative_path)
                && let Some(move_to) = state.movable_position(cell)
            {
                state.swap(cell, move_to);
            }
        }

        apply_state(&mut board, &state);
        board.draw(board_relative_path);

        next_frame().await
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Slide Puzzle".to_string(),
        window_width: 1920,
        window_height: 1080,
        ..Default::default()
    }
}
