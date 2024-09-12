// perfect labyrinth - https://en.wikipedia.org/wiki/Perfect_labyrinth


use std::collections::VecDeque;
use std::fmt::Debug;

use macroquad::rand::{ChooseRandom, srand};

pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    sides: u8,
}

#[allow(dead_code)]
impl Cell {
    const fn new() -> Self {
        Cell {
            sides: 0b0000_1111 // all sides closed (top, bottom, left, right)
        }
    }
    const fn new_empty() -> Self {
        Cell {
            sides: 0b0000_0000
        }
    }

    fn open(&mut self, side: Side) {
        match side {
            Side::Top => self.sides &= 0b1111_0111,
            Side::Bottom => self.sides &= 0b1111_1011,
            Side::Left => self.sides &= 0b1111_1101,
            Side::Right => self.sides &= 0b1111_1110
        }
    }

    const fn is_open(self, side: Side) -> bool {
        match side {
            Side::Top => self.sides & 0b0000_1000 == 0,
            Side::Bottom => self.sides & 0b0000_0100 == 0,
            Side::Left => self.sides & 0b0000_0010 == 0,
            Side::Right => self.sides & 0b0001_0001 == 0
        }
    }
    const fn is_closed(self, side: Side) -> bool {
        !self.is_open(side)
    }
    const fn get_sides(&self) -> u8 {
        self.sides
    }
}

impl Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cell {{ sides: {:b} }}", self.sides)
    }
}

pub struct Labyrinth {
    pub cell_size: f32,
    pub size: (usize, usize), // (width, height)
    cells: Vec<Vec<Cell>>,
}

impl Labyrinth {
    pub fn new(cell_size: f32, size: (usize, usize)) -> Self {
        Labyrinth {
            cell_size,
            size,
            cells: vec![vec![Cell::new(); size.1]; size.0],
        }
    }
    pub fn get_cells(&self) -> &Vec<Vec<Cell>> {
        &self.cells
    }

    pub fn get_as_lines_explicit(&self) -> Vec<((f32, f32), (f32, f32))> {
        let mut lines = Vec::new();
        let cell_size = self.cell_size;

        for y in 0..self.size.1 {
            for x in 0..self.size.0 {
                let cell = &self.cells[y][x];
                let x_pos = x as f32 * cell_size;
                let y_pos = y as f32 * cell_size;
                let x_next = (x as f32 + 1.0) * cell_size;
                let y_next = (y as f32 + 1.0) * cell_size;

                if !cell.is_open(Side::Top) {
                    lines.push(((x_pos, y_pos), (x_next, y_pos)));
                }
                if !cell.is_open(Side::Bottom) {
                    lines.push(((x_pos, y_next), (x_next, y_next)));
                }
                if !cell.is_open(Side::Left) {
                    lines.push(((x_pos, y_pos), (x_pos, y_next)));
                }
                if !cell.is_open(Side::Right) {
                    lines.push(((x_next, y_pos), (x_next, y_next)));
                }
            }
        }
        lines
    }
    pub fn get_as_lines(&self) -> Vec<((f32, f32), (f32, f32))> {
        let mut lines = Vec::new();
        let cell_size = self.cell_size;
        
        // top line
        let mut x_start = None;
        for x in 0..self.size.0 {
            let cell = &self.cells[0][x];
            let x_pos = x as f32 * cell_size;
            let y_pos = 0.0;
            let x_next = (x as f32 + 1.0) * cell_size;
            if cell.is_closed(Side::Top) {
                if x_start.is_none() {
                    x_start = Some(x_pos);
                }
                if !cell.is_open(Side::Right) {
                    lines.push(( (x_start.unwrap(), y_pos), (x_next, y_pos) ));
                    x_start = None;
                } else if x == self.size.0 - 1 {
                    lines.push(( (x_start.unwrap(), y_pos), (x_next, y_pos) ));
                }
            } else {
                if let Some(start) = x_start {
                    lines.push(( (start, y_pos), (x_pos, y_pos) ));
                    x_start = None;
                }
            }
        }
        

        // Horizontal lines
        for y in 0..(self.size.1) {
            let mut x_start = None;
            for x in 0..self.size.0 {
                let cell = &self.cells[y][x];
                let cell_below = if y < self.size.1 - 1 { self.cells[y + 1][x] } else { Cell::new_empty() };
                let x_pos = x as f32 * cell_size;
                let x_next = (x as f32 + 1.0) * cell_size;
                let y_next = (y as f32 + 1.0) * cell_size;
                
                if cell.is_closed(Side::Bottom) {
                    if x_start.is_none() {
                        x_start = Some(x_pos);
                    } 
                    if !cell_below.is_open(Side::Right) || !cell.is_open(Side::Right) {
                        lines.push(( (x_start.unwrap(), y_next), (x_next, y_next) ));
                        x_start = None;
                    } else if x == self.size.0 - 1 {
                        lines.push(( (x_start.unwrap(), y_next), (x_next, y_next) ));
                    }
                } else {
                    if let Some(start) = x_start {
                        lines.push(( (start, y_next), (x_pos, y_next) ));
                        x_start = None;
                    }
                }
            }
        }
        
        // left line
        let mut y_start = None;
        for y in 0..self.size.1 {
            let cell = &self.cells[y][0];
            let y_pos = y as f32 * cell_size;
            let x_pos = 0.0;
            let y_next = (y as f32 + 1.0) * cell_size;
            if cell.is_closed(Side::Left) {
                if y_start.is_none() {
                    y_start = Some(y_pos);
                }
                if !cell.is_open(Side::Bottom) {
                    lines.push(( (x_pos, y_start.unwrap()), (x_pos, y_next) ));
                    y_start = None;
                } else if y == self.size.1 - 1 {
                    lines.push(( (x_pos, y_start.unwrap()), (x_pos, y_next) ));
                }
            } else {
                if let Some(start) = y_start {
                    lines.push(( (x_pos, start), (x_pos, y_pos) ));
                    y_start = None;
                }
            }
        }
        // Vertical lines
        for x in 0..self.size.0 {
            let mut y_start = None;
            for y in 0..self.size.1 {
                let cell = &self.cells[y][x];
                let cell_right = if x < self.size.0 - 1 { self.cells[y][x + 1] } else { Cell::new_empty() };
                let _x_pos = x as f32 * cell_size;
                let y_pos = y as f32 * cell_size;
                let y_next = (y as f32 + 1.0) * cell_size;
                let x_next = (x as f32 + 1.0) * cell_size;

                if cell.is_closed(Side::Right) {
                    if y_start.is_none() {
                        y_start = Some(y_pos);
                    }
                    if !cell_right.is_open(Side::Bottom) || !cell.is_open(Side::Bottom) {
                        lines.push(( (x_next, y_start.unwrap()), (x_next, y_next) ));
                        y_start = None;
                    } else if y == self.size.1 - 1 {
                        lines.push(( (x_next, y_start.unwrap()), (x_next, y_next) ));
                    }
                } else {
                    if let Some(start) = y_start {
                        lines.push(( (x_next, start), (x_next, y_pos) ));
                        y_start = None;
                    }
                }
                
            }
        }

        lines
    }
    pub fn generate_depth_first2(&mut self) {
        let mut visited = vec![vec![false; self.size.0]; self.size.1];

        let mut stack = VecDeque::new();
        stack.push_back((0, 0));
        const DIRECTIONS: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        let mut directions = DIRECTIONS.to_vec();
        srand(12);
        while let Some((x, y)) = stack.pop_front() {
            directions.shuffle();
            for (d_x, d_y) in &directions {
                let next_x = x + d_x;
                let next_y = y + d_y;
                if next_x < 0 || next_y < 0 ||
                    next_x >= self.size.0 as i32 || next_y >= self.size.1 as i32 { continue; }
                let next_x = next_x as usize;
                let next_y = next_y as usize;
                if visited[next_y][next_x] { continue; }
                visited[next_y][next_x] = true;
                stack.push_front((x, y));
                stack.push_front((next_x as i32, next_y as i32));
                match (d_x, d_y) {
                    (0, -1) => {
                        self.cells[y as usize][x as usize].open(Side::Top);
                        self.cells[next_y][next_x].open(Side::Bottom)
                    }
                    (0, 1) => {
                        self.cells[y as usize][x as usize].open(Side::Bottom);
                        self.cells[next_y][next_x].open(Side::Top)
                    }
                    (1, 0) => {
                        self.cells[y as usize][x as usize].open(Side::Right);
                        self.cells[next_y][next_x].open(Side::Left)
                    }
                    (-1, 0) => {
                        self.cells[y as usize][x as usize].open(Side::Left);
                        self.cells[next_y][next_x].open(Side::Right)
                    }
                    _ => unreachable!()
                }
                break;
            }
        }
    }    
    pub fn generate_depth_first(&mut self) {
        let mut visited = vec![vec![false; self.size.0]; self.size.1];

        let mut stack = VecDeque::new();
        stack.push_back((0, 0));
        const DIRECTIONS: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        let mut directions = DIRECTIONS.to_vec();
        // srand(12);
        let mut last_dir = (0, -1);
        while let Some((x, y)) = stack.pop_front() {
            directions.shuffle();
            for (d_x, d_y) in &directions {
                if last_dir == (*d_x, *d_y) { continue; }
                let next_x = x + d_x;
                let next_y = y + d_y;
                if next_x < 0 || next_y < 0 ||
                    next_x >= self.size.0 as i32 || next_y >= self.size.1 as i32 { continue; }
                let next_x = next_x as usize;
                let next_y = next_y as usize;
                if visited[next_y][next_x] { continue; }
                visited[next_y][next_x] = true;
                last_dir = (*d_x, *d_y);
                stack.push_front((x, y));
                stack.push_front((next_x as i32, next_y as i32));
                // debug!("({}, {})->({}, {}) ({},{}) {:?}",x,y, next_x, next_y, d_x, d_y , last_dir);
                match (d_x, d_y) {
                    (0, -1) => {
                        self.cells[y as usize][x as usize].open(Side::Top);
                        self.cells[next_y][next_x].open(Side::Bottom)
                    }
                    (0, 1) => {
                        self.cells[y as usize][x as usize].open(Side::Bottom);
                        self.cells[next_y][next_x].open(Side::Top)
                    }
                    (1, 0) => {
                        self.cells[y as usize][x as usize].open(Side::Right);
                        self.cells[next_y][next_x].open(Side::Left)
                    }
                    (-1, 0) => {
                        self.cells[y as usize][x as usize].open(Side::Left);
                        self.cells[next_y][next_x].open(Side::Right)
                    }
                    _ => unreachable!()
                }
                break;
            }
        }
        // debug!("visited {:?}", visited);
    }
}