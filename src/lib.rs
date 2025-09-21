use macroquad::color::Color;
use macroquad::math::{Vec2, vec2};
use macroquad::{color, shapes, text};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Cell {
    pub x: usize,
    pub y: usize,
}

impl Cell {
    pub fn new(x: usize, y: usize) -> Self {
        Cell { x, y }
    }

    #[inline]
    pub fn as_index(&self, _rows: usize, cols: usize) -> usize {
        self.y * cols + self.x
    }
}

#[macro_export]
macro_rules! cell {
    ($x:expr, $y:expr) => {
        Cell::new($x, $y)
    };
    ($idx:expr, $_rows:expr, $cols:expr) => {
        Cell::new($idx % $cols, $idx / $cols)
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub num: usize,
}

impl Piece {
    pub fn new(num: usize) -> Self {
        Piece { num }
    }
}

#[derive(Debug)]
pub struct Board {
    size: Vec2,
    border_width: f32,

    rows: usize,
    cols: usize,

    cell_size: Vec2,

    pieces: Vec<Option<Piece>>,
}

impl Board {
    pub fn new(size: Vec2, border_width: f32, cell: (usize, usize)) -> Self {
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
    pub fn put_piece(&mut self, cell: Cell, piece: Option<Piece>) {
        let idx = cell.as_index(self.rows, self.cols);
        debug_assert!(idx < self.rows * self.cols);
        self.pieces[idx] = piece;
    }

    pub fn get_cell(&mut self, pos: Vec2) -> Option<Cell> {
        if pos.x < 0.0 || pos.y < 0.0 || pos.x >= self.size.x || pos.y >= self.size.y {
            return None;
        }
        let cell_x = (pos.x / self.cell_size.x) as usize;
        let cell_y = (pos.y / self.cell_size.y) as usize;

        Some(cell!(cell_x, cell_y))
    }

    pub fn draw(&self, pos: Vec2) {
        let cell_width = self.cell_size.x;
        let cell_height = self.cell_size.y;

        for i in 1..=self.cols {
            let x = i as f32 * cell_width;
            draw_relative_line(
                pos,
                x,
                0.0,
                x,
                self.size.y,
                self.border_width / 2.0,
                color::GRAY,
            );
        }
        for j in 1..=self.rows {
            let y = j as f32 * cell_height;
            draw_relative_line(
                pos,
                0.0,
                y,
                self.size.x,
                y,
                self.border_width / 2.0,
                color::GRAY,
            );
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

                shapes::draw_rectangle(
                    pos.x + cell_center_x - box_width / 2.0,
                    pos.y + cell_center_y - box_height / 2.0,
                    box_width,
                    box_height,
                    color::BROWN,
                );

                let text = &format!("{}", piece.num);
                let center = text::get_text_center(text, Option::None, font_size, 1.0, 0.0);
                text::draw_text_ex(
                    text,
                    pos.x + cell_center_x - center.x,
                    pos.y + cell_center_y - center.y,
                    text::TextParams {
                        font_size,
                        color: color::BLACK,
                        ..Default::default()
                    },
                );
            }
        }

        shapes::draw_rectangle_lines(
            pos.x,
            pos.y,
            self.size.x,
            self.size.y,
            self.border_width,
            color::BLACK,
        );
    }
}

#[inline]
fn draw_relative_line(pos: Vec2, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
    shapes::draw_line(
        pos.x + x1,
        pos.y + y1,
        pos.x + x2,
        pos.y + y2,
        thickness,
        color,
    )
}
