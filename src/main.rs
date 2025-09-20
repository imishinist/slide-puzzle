use ::rand::Rng;
use macroquad::prelude::*;

#[derive(Debug, Copy, Clone)]
struct Cell {
    x: usize,
    y: usize,
}

impl Cell {
    fn new(x: usize, y: usize) -> Self {
        Cell { x, y }
    }

    #[inline]
    fn as_index(&self, _rows: usize, cols: usize) -> usize {
        self.y * cols + self.x
    }
}

macro_rules! cell {
    ($x:expr, $y:expr) => {
        Cell::new($x, $y)
    };
    ($idx:expr, $_rows:expr, $cols:expr) => {
        Cell::new($idx % $cols, $idx / $cols)
    };
}

#[derive(Debug)]
struct Board {
    size: Vec2,
    border_width: f32,

    rows: usize,
    cols: usize,

    cell_size: Vec2,

    pieces: Vec<Option<Piece>>,
}

impl Board {
    fn new(size: Vec2, border_width: f32, cell: (usize, usize)) -> Self {
        let cell_size = size / vec2(cell.1 as f32, cell.0 as f32);
        let pieces = vec![None; cell.0 * cell.1];
        Board {
            size,
            border_width,
            rows: cell.0,
            cols: cell.1,
            cell_size,
            pieces,
        }
    }

    #[inline]
    fn put_piece(&mut self, cell: Cell, piece: Option<Piece>) {
        let idx = cell.as_index(self.rows, self.cols);
        assert!(idx < self.rows * self.cols);
        self.pieces[idx] = piece;
    }

    fn get_cell(&mut self, pos: Vec2) -> Option<Cell> {
        if pos.x < 0.0 || pos.y < 0.0 || pos.x >= self.size.x || pos.y >= self.size.y {
            return None;
        }
        let cell_x = (pos.x / self.cell_size.x) as usize;
        let cell_y = (pos.y / self.cell_size.y) as usize;

        Some(cell!(cell_x, cell_y))
    }

    fn draw(&self, pos: Vec2) {
        let cell_width = self.cell_size.x;
        let cell_height = self.cell_size.y;

        for i in 1..=self.cols {
            let x = i as f32 * cell_width;
            draw_relative_line(pos, x, 0.0, x, self.size.y, self.border_width / 2.0, GRAY);
        }
        for j in 1..=self.rows {
            let y = j as f32 * cell_height;
            draw_relative_line(pos, 0.0, y, self.size.x, y, self.border_width / 2.0, GRAY);
        }

        let box_width = cell_width * 0.90;
        let box_height = cell_height * 0.90;

        let font_size = (cell_width.min(cell_height) * 0.75) as u16;
        for (i, piece) in self.pieces.iter().enumerate() {
            if let Some(piece) = piece {
                let cell_x = (i % self.cols) as f32;
                let cell_y = (i / self.cols) as f32;

                let cell_center_x = cell_x * cell_width + cell_width / 2.0;
                let cell_center_y = cell_y * cell_height + cell_height / 2.0;

                draw_rectangle(
                    pos.x + cell_center_x - box_width / 2.0,
                    pos.y + cell_center_y - box_height / 2.0,
                    box_width,
                    box_height,
                    BROWN,
                );

                let text = &format!("{}", piece.num);
                let center = get_text_center(text, Option::None, font_size, 1.0, 0.0);
                draw_text_ex(
                    text,
                    pos.x + cell_center_x - center.x,
                    pos.y + cell_center_y - center.y,
                    TextParams {
                        font_size,
                        color: BLACK,
                        ..Default::default()
                    },
                );
            }
        }

        draw_relative_rectangle_lines(
            pos,
            0.0,
            0.0,
            self.size.x,
            self.size.y,
            self.border_width,
            BLACK,
        );
    }
}

#[inline]
fn draw_relative_line(pos: Vec2, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
    draw_line(
        pos.x + x1,
        pos.y + y1,
        pos.x + x2,
        pos.y + y2,
        thickness,
        color,
    )
}

#[inline]
fn draw_relative_rectangle_lines(
    pos: Vec2,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    thickness: f32,
    color: Color,
) {
    draw_rectangle_lines(pos.x + x, pos.y + y, w, h, thickness, color);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Piece {
    num: usize,
}

impl Piece {
    fn new(num: usize) -> Self {
        Piece { num }
    }
}

struct State {
    pieces: Vec<Option<Piece>>,

    rows: usize,
    cols: usize,

    blank_index: usize,
}

impl State {
    fn new(rows: usize, cols: usize) -> Self {
        let num = rows * cols;
        let mut pieces = (1..num).map(|n| Some(Piece::new(n))).collect::<Vec<_>>();
        pieces.push(None);

        State {
            rows,
            cols,
            pieces,
            blank_index: num - 1,
        }
    }

    #[inline]
    fn swap(&mut self, cell1: Cell, cell2: Cell) {
        let idx1 = cell1.as_index(self.rows, self.cols);
        let idx2 = cell2.as_index(self.rows, self.cols);

        assert!(
            self.blank_index == idx1 || self.blank_index == idx2,
            "swap target should equal blank_index"
        );

        self.pieces.swap(idx1, idx2);
        if idx1 == self.blank_index {
            self.blank_index = idx2;
        } else if idx2 == self.blank_index {
            self.blank_index = idx1;
        }
    }

    fn shuffle(&mut self) {
        let mut rng = ::rand::rng();
        let n = rng.random_range(500..=1000);

        let dx = [1, 0, -1, 0];
        let dy = [0, 1, 0, -1];

        for _ in 0..n {
            let cell = cell!(self.blank_index, self.rows, self.cols);
            let k = rng.random_range(0..4);
            let nx = dx[k] + cell.x as isize;
            let ny = dy[k] + cell.y as isize;

            if nx < 0 || ny < 0 || nx >= self.cols as isize || ny >= self.rows as isize {
                continue;
            }

            let next = cell!(nx as usize, ny as usize);
            self.swap(cell, next);
        }
    }

    fn is_finished(&mut self) -> bool {
        for (i, piece) in self.pieces.iter().enumerate() {
            if let Some(piece) = piece {
                if i != piece.num - 1 {
                    return false;
                }
            } else if i != self.rows * self.cols - 1 {
                return false;
            }
        }

        self.blank_index == self.rows * self.cols - 1
    }

    fn movable_position(&self, cell: Cell) -> Option<Cell> {
        let rows = self.rows;
        let cols = self.cols;

        let dx: Vec<isize> = vec![1, 0, -1, 0];
        let dy: Vec<isize> = vec![0, 1, 0, -1];
        for k in 0..4 {
            let nx = cell.x as isize + dx[k];
            let ny = cell.y as isize + dy[k];

            if nx < 0 || ny < 0 || nx >= cols as isize || ny >= rows as isize {
                continue;
            }

            let next = cell!(nx as usize, ny as usize);
            let idx = next.as_index(rows, cols);

            if self.blank_index == idx {
                return Some(next);
            }
        }
        None
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
