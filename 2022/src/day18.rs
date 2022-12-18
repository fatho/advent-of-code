use std::ops::{Add, Sub};

use nom::bytes::complete::tag;
use nom::multi::many0;
use nom::sequence::{terminated, tuple};
use nom::IResult;
use nom::{character::complete::u32 as parse_u32, combinator::map};

use crate::parsers::newline;
use crate::{parsers, Day};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let droplets = parsers::parse(many0(terminated(parse_pos, newline)), input)?;

    let (min, max) = if let Some(bounding_box) = aabb(&droplets) {
        bounding_box
    } else {
        // No droplets, no sides
        return Ok(0u32.to_string());
    };

    let size = max - min + Vec3::new(1, 1, 1);

    let mut voxels = ndarray::Array3::<bool>::from_elem(index(size), false);

    for droplet in droplets.iter() {
        voxels[index(*droplet - min)] = true;
    }

    let out: usize = droplets
        .iter()
        .map(|droplet| {
            SIDES
                .iter()
                .filter(|side| {
                    !voxels
                        .get(index(*droplet + **side - min))
                        .copied()
                        .unwrap_or(false)
                })
                .count()
        })
        .sum();

    Ok(out.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let droplets = parsers::parse(many0(terminated(parse_pos, newline)), input)?;

    let (min, max) = if let Some(bounding_box) = aabb(&droplets) {
        bounding_box
    } else {
        // No droplets, no sides
        return Ok(0u32.to_string());
    };

    let size = max - min + Vec3::new(1, 1, 1);

    // 0 -> unprocessed air, 1 -> droplet, 2 -> outside air
    let mut voxels = ndarray::Array3::from_elem(index(size), Voxel::Air);

    for droplet in droplets.iter() {
        voxels[index(*droplet - min)] = Voxel::Lava;
    }

    let mut todo = vec![];

    // add a cube on the outer edge of the bounding box to the todo set, if it is air
    let mut push_outside_edge = |cube: Vec3<i32>| {
        let voxel_index = index(cube - min);
        if matches!(voxels[voxel_index], Voxel::Air) {
            // On the outside
            voxels[voxel_index] = Voxel::OutsideAir;
            todo.push(cube);
        }
    };

    for x in min.x..=max.x {
        for y in min.y..=max.y {
            push_outside_edge(Vec3::new(x, y, min.z));
            push_outside_edge(Vec3::new(x, y, max.z));
        }
    }
    for x in min.x..=max.x {
        for z in min.z..=max.z {
            push_outside_edge(Vec3::new(x, min.y, z));
            push_outside_edge(Vec3::new(x, max.y, z));
        }
    }
    for y in min.y..=max.y {
        for z in min.z..=max.z {
            push_outside_edge(Vec3::new(min.x, y, z));
            push_outside_edge(Vec3::new(max.x, y, z));
        }
    }

    while let Some(cube) = todo.pop() {
        // cube was on the outside, process neighbors
        for side in SIDES {
            let neighbor = cube + side;
            let neighbor_index = index(neighbor - min);

            if let Some(Voxel::Air) = voxels.get(neighbor_index) {
                voxels[neighbor_index] = Voxel::OutsideAir;
                todo.push(neighbor);
            }
        }
    }

    let mut out = 0;

    for droplet in &droplets {
        for side in SIDES {
            let neighbor = *droplet + side;
            let is_outside = voxels
                .get(index(neighbor - min))
                .copied()
                .map_or(true, |vox| matches!(vox, Voxel::OutsideAir));
            if is_outside {
                out += 1;
            }
        }
    }

    Ok(out.to_string())
}

fn aabb<T: Ord + Copy>(points: &[Vec3<T>]) -> Option<(Vec3<T>, Vec3<T>)> {
    points.split_first().map(|(first, rest)| {
        rest.iter().fold((*first, *first), |(min, max), droplet| {
            (
                min.zip_with(*droplet, Ord::min),
                max.zip_with(*droplet, Ord::max),
            )
        })
    })
}

fn index(droplet: Vec3<i32>) -> (usize, usize, usize) {
    (droplet.x as usize, droplet.y as usize, droplet.z as usize)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Voxel {
    Lava,
    Air,
    OutsideAir,
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

static SIDES: [Vec3<i32>; 6] = [
    Vec3::new(1, 0, 0),
    Vec3::new(-1, 0, 0),
    Vec3::new(0, 1, 0),
    Vec3::new(0, -1, 0),
    Vec3::new(0, 0, 1),
    Vec3::new(0, 0, -1),
];

fn parse_pos(input: &[u8]) -> IResult<&[u8], Vec3<i32>> {
    map(
        tuple((parse_u32, tag(","), parse_u32, tag(","), parse_u32)),
        |(x, _, y, _, z)| Vec3 { x, y, z }.map(|elem| elem as i32),
    )(input)
}

crate::test_day!(RUN, "day18", "3494", "2062");
