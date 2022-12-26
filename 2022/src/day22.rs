use std::ops::{Add, Sub};

use anyhow::Context;
use nom::{
    branch::alt,
    bytes::complete::take,
    combinator::{map, map_res},
    multi::many0,
    IResult,
};
use rustc_hash::FxHashMap;

use crate::{parsers, Day};

pub static RUN: Day = Day { part1, part2 };

// TODO: investigate more efficient representation of tilemap than a hash map

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let (tiles, start, instructions) = parse_input(input)?;

    let mut heading = Heading::Right;
    let mut position = start;

    fn next_coord(
        map: &FxHashMap<(i32, i32), Tile>,
        (x, y): (i32, i32),
        heading: Heading,
    ) -> (i32, i32) {
        let (dx, dy) = heading_delta(heading);
        let unwrapped = (x + dx, y + dy);
        if map.contains_key(&unwrapped) {
            unwrapped
        } else {
            // find edge in opposite direction
            let mut edge = (x, y);
            while map.contains_key(&(edge.0 - dx, edge.1 - dy)) {
                edge = (edge.0 - dx, edge.1 - dy);
            }
            edge
        }
    }

    for instr in instructions {
        match instr {
            Instr::Walk(len) => {
                for _ in 0..len {
                    let next = next_coord(&tiles, position, heading);
                    match tiles[&next] {
                        Tile::Open => position = next,
                        Tile::Solid => break,
                    }
                }
            }
            Instr::Turn(dir) => heading = heading.turn(dir),
        }
    }

    let password = 1000 * position.1 + 4 * position.0 + (heading as i32);
    Ok(password.to_string())
}

fn heading_delta(heading: Heading) -> (i32, i32) {
    match heading {
        Heading::Up => (0, -1),
        Heading::Left => (-1, 0),
        Heading::Down => (0, 1),
        Heading::Right => (1, 0),
    }
}

fn parse_input(
    input: &[u8],
) -> anyhow::Result<(FxHashMap<(i32, i32), Tile>, (i32, i32), Vec<Instr>)> {
    let mut tiles: FxHashMap<(i32, i32), Tile> = FxHashMap::default();
    let mut lines = input.split(|ch| *ch == b'\n');
    let mut row = 1;
    let mut start = None;
    for line in lines.by_ref() {
        if line.is_empty() {
            // separator between map and instructions
            break;
        }
        for (col, ch) in line.iter().enumerate() {
            if let Ok(tile) = Tile::try_from(*ch) {
                tiles.insert((col as i32 + 1, row), tile);
                if matches!(tile, Tile::Open) && start.is_none() && row == 1 {
                    start = Some((col as i32 + 1, row));
                }
            }
        }

        row += 1;
    }
    let instructions_string = lines.next().context("missing instructions")?;
    let instructions = parsers::parse(many0(parse_instruction), instructions_string)?;
    let start = start.context("no starting location")?;
    Ok((tiles, start, instructions))
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let (tiles, start, instructions) = parse_input(input)?;

    let (max_col, max_row) = tiles
        .keys()
        .fold((0, 0), |(mc, mr), (c, r)| (mc.max(*c), mr.max(*r)));

    // detect example
    let width = if max_col % 50 == 0 && max_row % 50 == 0 {
        50
    } else {
        4
    };

    // find faces
    let mut faces = FxHashMap::default();
    let mut faces_by_id = Vec::new();

    let mut face = 0usize;
    for frow in 0..max_row / width {
        let row = frow * width + 1;
        for fcol in 0..max_col / width {
            let col = fcol * width + 1;
            if tiles.contains_key(&(col, row)) {
                faces.insert((fcol, frow), face);
                faces_by_id.push((fcol, frow));
                face += 1;
            }
        }
    }

    anyhow::ensure!(faces.len() == 6, "must have 6 faces");

    // align faces
    let mut cube = Cube::new();

    fn cube_dfs(
        faces: &FxHashMap<(i32, i32), usize>,
        faces_by_id: &[(i32, i32)],
        width: i32,
        cube: &mut Cube,
        face_id: usize,
    ) {
        if let Some(front) = cube.front_face().face_id {
            assert_eq!(front, face_id);
            return;
        }
        let (facex, facey) = faces_by_id[face_id];
        cube.set_front(
            face_id,
            facex * width + 1,
            facey * width + 1,
            (facex + 1) * width,
            (facey + 1) * width,
        );

        for h in [Heading::Up, Heading::Left, Heading::Right, Heading::Down] {
            let (dx, dy) = heading_delta(h);
            let nx = facex + dx;
            let ny = facey + dy;

            if let Some(n) = faces.get(&(nx, ny)) {
                match h {
                    Heading::Right => cube.rot_right(),
                    Heading::Down => cube.rot_down(),
                    Heading::Left => cube.rot_left(),
                    Heading::Up => cube.rot_up(),
                }
                cube_dfs(faces, faces_by_id, width, cube, *n);
                match h {
                    Heading::Right => cube.rot_left(),
                    Heading::Down => cube.rot_up(),
                    Heading::Left => cube.rot_right(),
                    Heading::Up => cube.rot_down(),
                }
            }
        }
    }

    cube_dfs(&faces, &faces_by_id, width, &mut cube, 0);

    fn next_coord(
        map: &FxHashMap<(i32, i32), Tile>,
        width: i32,
        cube: &mut Cube,
        (x, y): (i32, i32),
        heading: Heading,
    ) -> Option<((i32, i32), Heading)> {
        let (dx, dy) = heading_delta(heading);
        let unwrapped = (x + dx, y + dy);
        if div_floor(x + dx - 1, width) == div_floor(x - 1, width)
            && div_floor(y + dy - 1, width) == div_floor(y - 1, width)
        {
            match map[&unwrapped] {
                Tile::Open => Some((unwrapped, heading)),
                Tile::Solid => None,
            }
        } else {
            // // position alongside edge
            let (left_top, right_bottom) = cube.front_box();
            assert!(left_top.0 < right_bottom.0 && left_top.1 < right_bottom.1);
            let face_pos = (x - left_top.0, y - left_top.1);
            // println!(
            //     "leaving cube at {:?} towards {:?} - face pos {:?} ({:?}, {:?})",
            //     (x, y),
            //     heading,
            //     face_pos,
            //     left_top,
            //     right_bottom,
            // );
            // leaving cube face in direction `heading`
            match heading {
                Heading::Right => cube.rot_right(),
                Heading::Down => cube.rot_down(),
                Heading::Left => cube.rot_left(),
                Heading::Up => cube.rot_up(),
            }

            // align with map again by turning the cube until up is up
            let mut new_face_pos = if matches!(heading, Heading::Right | Heading::Left) {
                (width - 1 - face_pos.0, face_pos.1)
            } else {
                (face_pos.0, width - 1 - face_pos.1)
            };
            let mut new_heading = heading;
            let mut num_turns = 0;
            loop {
                let (left_top, right_bottom) = cube.front_box();
                if !(left_top.0 < right_bottom.0 && left_top.1 < right_bottom.1) {
                    new_heading = new_heading.turn(Turn::Cw);
                    cube.turn_right();
                    new_face_pos = (width - 1 - new_face_pos.1, new_face_pos.0);
                    num_turns += 1;
                } else {
                    break;
                }
            }
            let (left_top, _) = cube.front_box();
            let (newx, newy) = (left_top.0 + new_face_pos.0, left_top.1 + new_face_pos.1);
            // println!(
            //     "entering cube at {:?} towards {:?} - face pos {:?} ({:?}, {:?})",
            //     (newx, newy),
            //     new_heading,
            //     new_face_pos,
            //     left_top,
            //     right_bottom,
            // );

            match map[&(newx, newy)] {
                Tile::Open => Some(((newx, newy), new_heading)),
                Tile::Solid => {
                    // undo cube manipulation if we hit a wall
                    for _ in 0..num_turns {
                        cube.turn_left();
                    }
                    match heading.inverse() {
                        Heading::Right => cube.rot_right(),
                        Heading::Down => cube.rot_down(),
                        Heading::Left => cube.rot_left(),
                        Heading::Up => cube.rot_up(),
                    }
                    None
                }
            }
        }
    }

    let mut heading = Heading::Right;
    let mut position = start;
    for instr in instructions {
        match instr {
            Instr::Walk(len) => {
                for _ in 0..len {
                    if let Some((next_pos, next_heading)) =
                        next_coord(&tiles, width, &mut cube, position, heading)
                    {
                        // println!("{:?}", next_pos);
                        position = next_pos;
                        heading = next_heading;
                    } else {
                        // println!("blocked");
                        break;
                    }
                }
            }
            Instr::Turn(dir) => heading = heading.turn(dir),
        }
    }

    let password = 1000 * position.1 + 4 * position.0 + (heading as i32);
    Ok(password.to_string())
}

fn parse_instruction(input: &[u8]) -> IResult<&[u8], Instr> {
    alt((
        map(nom::character::complete::u32, Instr::Walk),
        map_res(take(1usize), |dir: &[u8]| {
            Turn::try_from(dir[0]).map(Instr::Turn)
        }),
    ))(input)
}

/// Edge transitions of each cube face
#[derive(Clone, PartialEq, Eq, Debug)]
struct FaceTransition([Option<(usize, Heading)>; 4]);

/// A literal cube with data attached to each of its faces. The front face (Z coordinate -1) has a
/// special meaning.
///
/// When crossing cube face boundaries, we just "rotate" this cube by applying a rotation matrix to
/// all coordinates stored within. That is highly inefficient, but works quite well in terms of not
/// having to think too hard about this problem.
#[derive(Clone, PartialEq, Eq, Debug)]
struct Cube {
    faces: [CubeFace; 6],
}

/// A face of the cube, holding it's center coordinate, as well as the coordinates for each of the
/// corners, together with some associated data.
#[derive(Clone, PartialEq, Eq, Debug)]
struct CubeFace {
    center: Vec3<i32>,
    face_id: Option<usize>,
    corners: [(Vec3<i32>, Option<(i32, i32)>); 4],
}

impl CubeFace {
    fn new(center: Vec3<i32>, dx: Vec3<i32>, dy: Vec3<i32>) -> Self {
        Self {
            center,
            face_id: None,
            corners: [
                (center - dx - dy, None),
                (center + dx - dy, None),
                (center - dx + dy, None),
                (center + dx + dy, None),
            ],
        }
    }

    fn map_coords(&mut self, f: impl Fn(Vec3<i32>) -> Vec3<i32>) {
        self.center = f(self.center);
        for (corner, _) in self.corners.iter_mut() {
            *corner = f(*corner);
        }
    }
}

impl Cube {
    fn new() -> Self {
        Cube {
            faces: [
                CubeFace::new(Vec3::new(1, 0, 0), Vec3::new(0, 1, 0), Vec3::new(0, 0, 1)),
                CubeFace::new(Vec3::new(-1, 0, 0), Vec3::new(0, 1, 0), Vec3::new(0, 0, 1)),
                CubeFace::new(Vec3::new(0, 1, 0), Vec3::new(1, 0, 0), Vec3::new(0, 0, 1)),
                CubeFace::new(Vec3::new(0, -1, 0), Vec3::new(1, 0, 0), Vec3::new(0, 0, 1)),
                CubeFace::new(Vec3::new(0, 0, 1), Vec3::new(1, 0, 0), Vec3::new(0, 1, 0)),
                CubeFace::new(Vec3::new(0, 0, -1), Vec3::new(1, 0, 0), Vec3::new(0, 1, 0)),
            ],
        }
    }

    fn set_front(&mut self, face_id: usize, left: i32, top: i32, right: i32, bottom: i32) {
        if let Some(f) = self.faces.iter_mut().find(|f| f.center.z == -1) {
            assert!(f.face_id.is_none());
            f.face_id = Some(face_id);
            for (corner, value) in f.corners.iter_mut() {
                assert_eq!(corner.z, -1);
                *value = Some((
                    ((corner.x == -1) as i32) * left + ((corner.x == 1) as i32) * right,
                    ((corner.y == -1) as i32) * top + ((corner.y == 1) as i32) * bottom,
                ));
            }
        }
    }

    fn front_face(&self) -> &CubeFace {
        self.faces
            .iter()
            .find(|f| f.center.z == -1)
            .expect("invariant")
    }

    /// Get the bounding box in map coordinates of the front face.
    fn front_box(&self) -> ((i32, i32), (i32, i32)) {
        let front = self.front_face();
        let left_top = front
            .corners
            .iter()
            .find(|c| c.0 == Vec3::new(-1, -1, -1))
            .expect("invariant")
            .1
            .expect("must be initialized");
        let right_bottom = front
            .corners
            .iter()
            .find(|c| c.0 == Vec3::new(1, 1, -1))
            .expect("invariant")
            .1
            .expect("must be initialized");
        (left_top, right_bottom)
    }

    fn rot_up(&mut self) {
        self.faces.iter_mut().for_each(|f| f.map_coords(rot_up));
    }

    fn rot_down(&mut self) {
        self.faces.iter_mut().for_each(|f| f.map_coords(rot_down));
    }

    fn rot_left(&mut self) {
        self.faces.iter_mut().for_each(|f| f.map_coords(rot_left));
    }

    fn rot_right(&mut self) {
        self.faces.iter_mut().for_each(|f| f.map_coords(rot_right));
    }

    fn turn_left(&mut self) {
        self.faces.iter_mut().for_each(|f| f.map_coords(turn_left));
    }

    fn turn_right(&mut self) {
        self.faces.iter_mut().for_each(|f| f.map_coords(turn_right));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Instr {
    Walk(u32),
    Turn(Turn),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Turn {
    Ccw,
    Cw,
}

impl TryFrom<u8> for Turn {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'L' => Ok(Turn::Ccw),
            b'R' => Ok(Turn::Cw),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Heading {
    Right = 0,
    Down = 1,
    Left = 2,
    Up = 3,
}

impl Heading {
    fn turn(self, turn: Turn) -> Self {
        match (self, turn) {
            (Heading::Up, Turn::Ccw) => Heading::Left,
            (Heading::Up, Turn::Cw) => Heading::Right,
            (Heading::Left, Turn::Ccw) => Heading::Down,
            (Heading::Left, Turn::Cw) => Heading::Up,
            (Heading::Down, Turn::Ccw) => Heading::Right,
            (Heading::Down, Turn::Cw) => Heading::Left,
            (Heading::Right, Turn::Ccw) => Heading::Up,
            (Heading::Right, Turn::Cw) => Heading::Down,
        }
    }

    fn inverse(self) -> Self {
        match self {
            Heading::Right => Heading::Left,
            Heading::Down => Heading::Up,
            Heading::Left => Heading::Right,
            Heading::Up => Heading::Down,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tile {
    Open,
    Solid,
}

impl TryFrom<u8> for Tile {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'.' => Ok(Tile::Open),
            b'#' => Ok(Tile::Solid),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vec3<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    pub fn zip_with<S, R>(self, other: Vec3<S>, mut f: impl FnMut(T, S) -> R) -> Vec3<R> {
        Vec3 {
            x: f(self.x, other.x),
            y: f(self.y, other.y),
            z: f(self.z, other.z),
        }
    }

    pub fn all(self, mut f: impl FnMut(&T) -> bool) -> bool {
        f(&self.x) && f(&self.y) && f(&self.z)
    }

    pub fn map<R>(self, mut f: impl FnMut(T) -> R) -> Vec3<R> {
        Vec3 {
            x: f(self.x),
            y: f(self.y),
            z: f(self.z),
        }
    }
}

impl Vec3<bool> {
    pub fn and(self) -> bool {
        self.x && self.y && self.z
    }
}

impl<T: Default> Default for Vec3<T> {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            z: Default::default(),
        }
    }
}

impl<T: Add<T>> Add<Vec3<T>> for Vec3<T> {
    type Output = Vec3<T::Output>;

    fn add(self, rhs: Vec3<T>) -> Self::Output {
        self.zip_with(rhs, Add::add)
    }
}

impl<T: Sub<T>> Sub<Vec3<T>> for Vec3<T> {
    type Output = Vec3<T::Output>;

    fn sub(self, rhs: Vec3<T>) -> Self::Output {
        self.zip_with(rhs, Sub::sub)
    }
}

fn rot_up(vec: Vec3<i32>) -> Vec3<i32> {
    Vec3::new(vec.x, vec.z, -vec.y)
}

fn rot_down(vec: Vec3<i32>) -> Vec3<i32> {
    Vec3::new(vec.x, -vec.z, vec.y)
}

fn rot_left(vec: Vec3<i32>) -> Vec3<i32> {
    Vec3::new(vec.z, vec.y, -vec.x)
}

fn rot_right(vec: Vec3<i32>) -> Vec3<i32> {
    Vec3::new(-vec.z, vec.y, vec.x)
}

fn turn_left(vec: Vec3<i32>) -> Vec3<i32> {
    Vec3::new(vec.y, -vec.x, vec.z)
}

fn turn_right(vec: Vec3<i32>) -> Vec3<i32> {
    Vec3::new(-vec.y, vec.x, vec.z)
}

fn div_floor(a: i32, b: i32) -> i32 {
    if a < 0 {
        (a - b + 1) / b
    } else {
        a / b
    }
}

crate::test_day!(RUN, "day22", "93226", "37415");
