use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};

use ::rand::Rng;
use clap::Parser;
use macroquad::prelude::*;
use macroquad::rand::ChooseRandom;

use slide_puzzle::{Board, Cell, Piece, cell};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct State {
    pieces: Vec<Option<Piece>>,

    rows: usize,
    cols: usize,

    blank_cell: Cell,
}

impl Ord for State {
    fn cmp(&self, _other: &Self) -> Ordering {
        Ordering::Greater
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
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
        let dx = [0, -1, 1, 0];
        let dy = [-1, 0, 0, 1];
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
    pub fn get_index(&self, cell: Cell) -> usize {
        cell.as_index(self.rows, self.cols)
    }

    pub fn get_piece(&self, cell: Cell) -> Option<Piece> {
        self.pieces[self.get_index(cell)]
    }

    #[inline]
    pub fn swap(&mut self, cell1: Cell, cell2: Cell) {
        assert!(
            self.blank_cell == cell1 || self.blank_cell == cell2,
            "swap target should equal blank_index"
        );

        let idx1 = self.get_index(cell1);
        let idx2 = self.get_index(cell2);

        self.pieces.swap(idx1, idx2);
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

    #[allow(dead_code)]
    fn show_pieces(&self) {
        for (i, piece) in self.pieces.iter().enumerate() {
            if let Some(piece) = piece {
                print!("{:2} ", piece.num);
            } else {
                print!(" - ");
            }
            if (i + 1) % self.cols == 0 {
                println!();
            }
        }
        println!();
    }

    fn find_piece(&self, needle: Option<Piece>) -> Cell {
        for (i, piece) in self.pieces.iter().enumerate() {
            if needle == *piece {
                let cell = cell!(i, self.rows, self.cols);
                return cell;
            }
        }
        unreachable!("should has piece");
    }

    fn state_string(&self) -> String {
        self.pieces
            .iter()
            .map(|p| {
                if let Some(p) = p {
                    format!("{:02}", p.num)
                } else {
                    "00".to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(",")
    }
}

fn apply_state(board: &mut Board, state: &State) {
    for (i, piece) in state.pieces.iter().enumerate() {
        board.put_piece(cell!(i, state.rows, state.cols), *piece);
    }
}

trait Solver {
    fn solve(&mut self) -> Vec<Cell>;
}

struct MySolver {
    states: State,
}

impl MySolver {
    fn new(states: State) -> Self {
        states.show_pieces();
        MySolver { states }
    }

    fn solve(&mut self) -> Vec<Cell> {
        let mut ans = vec![];

        //  1  2  3  4
        //  5  6  7  8
        //  9 10 11 12
        // 13 14 15

        let mut moved = vec![false; self.states.cols * self.states.rows];

        let mut targets = vec![];
        let depth = self.states.rows.min(self.states.cols) - 1;
        for i in 0..depth {
            for x in i..self.states.cols {
                targets.push(cell!(x, i));
            }
            for y in i + 1..self.states.rows {
                targets.push(cell!(i, y));
            }
        }

        for target in targets.into_iter().take(4) {
            if self.is_correct_place(target) {
                moved[self.states.get_index(target)] = true;
                continue;
            }

            let target_num = self.states.get_index(target) + 1;
            let current_target_cell = self.find_target_num(target_num);

            let tmp = self.move_to(current_target_cell, target, &mut moved);
            ans.extend(tmp);
            moved[self.states.get_index(target)] = true;
        }

        ans
    }

    fn move_to(&mut self, start: Cell, end: Cell, constraints: &mut [bool]) -> Vec<Cell> {
        let mut ans = vec![];

        let mut current = start;
        let routes = self.find_routes(current, end, constraints);
        for &next in routes.iter() {
            constraints[self.states.get_index(current)] = true;
            let blank_routes = self.find_routes(self.states.blank_cell, next, constraints);
            for &b in blank_routes.iter() {
                self.states.swap(self.states.blank_cell, b);
                ans.push(b);
            }
            constraints[self.states.get_index(current)] = false;
            self.states.swap(self.states.blank_cell, current);
            ans.push(current);

            current = next;
        }
        ans
    }

    fn find_routes(&self, start: Cell, end: Cell, constraints: &[bool]) -> Vec<Cell> {
        let mut queues = VecDeque::new();
        queues.push_back((start, vec![]));

        let mut memo = vec![false; constraints.len()];
        while !queues.is_empty() {
            let (current, routes) = queues.pop_front().unwrap();

            if current == end {
                return routes;
            }

            let dx = [1, 0, -1, 0];
            let dy = [0, 1, 0, -1];
            for k in 0..4 {
                let nx = dx[k] + current.x as isize;
                let ny = dy[k] + current.y as isize;

                if nx < 0
                    || ny < 0
                    || nx >= self.states.cols as isize
                    || ny >= self.states.rows as isize
                {
                    continue;
                }
                let nx = nx as usize;
                let ny = ny as usize;

                let next = cell!(nx, ny);
                let next_idx = self.states.get_index(next);
                if memo[next_idx] {
                    continue;
                }
                memo[next_idx] = true;

                if constraints[next_idx] {
                    continue;
                }

                let mut tmp = routes.clone();
                tmp.push(next);
                queues.push_back((next, tmp));
            }
        }
        vec![]
    }

    fn find_target_num(&self, num: usize) -> Cell {
        assert!(num >= 1 && num < self.states.rows * self.states.cols);

        println!("finding {}", num);
        for (i, piece) in self.states.pieces.iter().enumerate() {
            if let Some(piece) = piece
                && piece.num == num
            {
                let cell = cell!(i, self.states.rows, self.states.cols);
                println!("found at {} ({:?})", i, cell);
                return cell;
            }
        }
        unreachable!("should has piece");
    }

    fn is_correct_place(&self, cell: Cell) -> bool {
        if let Some(piece) = self.states.get_piece(cell) {
            return piece.num - 1 == cell.as_index(self.states.rows, self.states.cols);
        }
        cell == cell!(self.states.cols - 1, self.states.rows - 1)
    }
}

impl Solver for MySolver {
    fn solve(&mut self) -> Vec<Cell> {
        self.solve()
    }
}

struct AStarSolver {
    states: State,
}

impl AStarSolver {
    fn new(states: State) -> Self {
        AStarSolver { states }
    }
    fn solve_with_expected(&self, expected: &State) -> Vec<Cell> {
        let max_cost = 10000000;
        let mut pq: BinaryHeap<(usize, State, Vec<Cell>)> = BinaryHeap::new();

        let state = self.states.clone();
        let cost = self.heuristic(expected, &state);
        pq.push((max_cost - cost, state, vec![]));

        let mut memo = HashMap::new();
        while let Some((cost, state, routes)) = pq.pop() {
            if cost == max_cost {
                return routes;
            }

            let s = state.state_string();
            if let Some(count) = memo.get(&s)
                && *count < routes.len()
            {
                continue;
            }
            memo.insert(s, routes.len());

            for next in state.neighbors(state.blank_cell).into_iter() {
                let mut next_state = state.clone();
                next_state.swap(state.blank_cell, next);

                let mut next_routes = routes.clone();
                next_routes.push(next);

                let next_cost = self.heuristic(expected, &next_state);
                pq.push((max_cost - next_cost, next_state, next_routes));
            }
        }

        vec![]
    }

    fn heuristic(&self, expected: &State, current: &State) -> usize {
        let mut dist = 0;

        for i in 0..self.states.rows * self.states.cols {
            let cell = cell!(i, self.states.rows, self.states.cols);

            let expected = expected.get_piece(cell);

            let current_pos = current.find_piece(expected);
            dist += cell.manhattan_distance(&current_pos);
        }
        dist
    }
}

impl Solver for AStarSolver {
    fn solve(&mut self) -> Vec<Cell> {
        let expected = State::new(self.states.rows, self.states.cols);
        self.solve_with_expected(&expected)
    }
}

struct BFSSolver {
    states: State,
}

impl BFSSolver {
    fn new(states: State) -> Self {
        BFSSolver { states }
    }
    fn solve_with_expected(&self, expected: &State) -> Vec<Cell> {
        let mut q: VecDeque<(State, Vec<Cell>)> = VecDeque::new();

        let state = self.states.clone();
        q.push_back((state, vec![]));

        let mut memo = HashMap::new();
        while let Some((state, routes)) = q.pop_front() {
            if state.pieces == expected.pieces {
                return routes;
            }

            let s = state.state_string();
            if let Some(count) = memo.get(&s)
                && *count < routes.len()
            {
                continue;
            }
            memo.insert(s, routes.len());

            for next in state.neighbors(state.blank_cell).into_iter() {
                let mut next_state = state.clone();
                next_state.swap(state.blank_cell, next);

                let mut next_routes = routes.clone();
                next_routes.push(next);

                q.push_back((next_state, next_routes));
            }
        }

        vec![]
    }
}

impl Solver for BFSSolver {
    fn solve(&mut self) -> Vec<Cell> {
        let expected = State::new(self.states.rows, self.states.cols);
        self.solve_with_expected(&expected)
    }
}

const DEFAULT_BOARD_ROWS: usize = 4;
const DEFAULT_BOARD_COLS: usize = 4;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(default_value_t = DEFAULT_BOARD_ROWS)]
    rows: usize,

    #[clap(default_value_t = DEFAULT_BOARD_COLS)]
    cols: usize,

    #[clap(long)]
    ai: bool,
}

async fn play_with_ai(mut board: Board, mut state: State) {
    let board_relative_path = vec2(50.0, 80.0);

    let action_per_frame = 10;
    let mut frames = 0;

    let num = state.rows * state.cols;

    let mut solver: Box<dyn Solver> = if num > 16 {
        Box::new(MySolver::new(state.clone()))
    } else if num > 9 {
        Box::new(AStarSolver::new(state.clone()))
    } else {
        Box::new(BFSSolver::new(state.clone()))
    };

    let moves = solver.solve();
    let move_count = moves.len();
    let mut moves = moves.into_iter();
    loop {
        frames += 1;
        clear_background(WHITE);

        let txt = format!("{} moves", move_count);
        draw_text(&txt, 200.0, 50.0, 40.0, BLUE);

        if is_key_down(KeyCode::Q) {
            break;
        }

        if frames == action_per_frame {
            frames = 0;
            if let Some(mv) = moves.next() {
                state.swap(state.blank_cell, mv);
            }
        }

        apply_state(&mut board, &state);
        board.draw(board_relative_path);

        next_frame().await;
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let cli = Cli::parse();

    let rows = cli.rows;
    let cols = cli.cols;

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

    if cli.ai {
        play_with_ai(board, state.clone()).await;
        return;
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
