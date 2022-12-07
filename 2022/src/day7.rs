use std::{fmt::Debug, str::Utf8Error};

use anyhow::anyhow;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    combinator::{map, map_res},
    multi::fold_many0,
    sequence::{preceded, separated_pair, terminated},
    IResult,
};

use crate::{parsers, Day};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let fs = parsers::parse(parse_tree, input)?;
    let dirsizes = compute_dir_size(&fs);

    let result = dirsizes
        .iter()
        .copied()
        .filter(|size| *size <= 100000)
        .sum::<u64>();

    Ok(result.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let fs = parsers::parse(parse_tree, input)?;
    let dirsizes = compute_dir_size(&fs);

    let total = 70000000;
    let needed = 30000000;
    let used = dirsizes[0]; // size of root
    let unused = total - used;

    let size = dirsizes
        .iter()
        .copied()
        .filter(|size| unused + *size >= needed)
        .min_by_key(|size| *size)
        .ok_or_else(|| anyhow!("no deletion candidate found"))?;

    Ok(size.to_string())
}

fn compute_dir_size(fs: &Fs) -> Vec<u64> {
    let mut dirsizes = vec![0; fs.dirs.len()];

    // count files
    for file in fs.files.iter() {
        dirsizes[file.parent.0] += file.size;
    }
    // count dirs (we know child dirs are always added after their parents and thus have a higher
    // id) - but skip the self-rerential root
    for (index, dir) in fs.dirs.iter().enumerate().skip(1).rev() {
        dirsizes[dir.parent.0] += dirsizes[index];
    }

    dirsizes
}

fn parse_tree(input: &[u8]) -> IResult<&[u8], Fs> {
    let mut fs = Fs::new();
    let mut walker = Walker::new(&mut fs);
    let (rest, _) = fold_many0(
        alt((
            map(preceded(tag("$ "), parse_cmd), CmdOrLs::Cmd),
            map(parse_ls, CmdOrLs::Ls),
        )),
        || (),
        |_, cmd_or_ls| match cmd_or_ls {
            CmdOrLs::Cmd(cmd) => match cmd {
                Command::Cd { name } => match name {
                    ".." => walker.leave(),
                    "/" => walker.goto_root(),
                    other => walker.enter(other),
                },
                Command::Ls => (),
            },
            CmdOrLs::Ls(row) => match row.typ {
                LsType::Dir => walker.insert_dir(row.name),
                LsType::File { size } => walker.insert_file(row.name, size),
            },
        },
    )(input)?;
    Ok((rest, fs))
}

fn parse_cmd(input: &[u8]) -> IResult<&[u8], Command> {
    terminated(
        alt((
            map(tag("ls"), |_| Command::Ls),
            map_res(preceded(tag("cd "), take_until("\n")), |name: &[u8]| {
                Ok::<_, Utf8Error>(Command::Cd {
                    name: std::str::from_utf8(name)?,
                })
            }),
        )),
        parsers::newline,
    )(input)
}

fn parse_ls(input: &[u8]) -> IResult<&[u8], LsRow> {
    terminated(
        map(
            separated_pair(
                alt((
                    map(tag("dir"), |_| LsType::Dir),
                    map(parsers::u64, |size| LsType::File { size }),
                )),
                tag(" "),
                map_res(take_until("\n"), std::str::from_utf8),
            ),
            |(typ, name)| LsRow { typ, name },
        ),
        parsers::newline,
    )(input)
}

enum CmdOrLs<'a> {
    Cmd(Command<'a>),
    Ls(LsRow<'a>),
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Command<'a> {
    Cd { name: &'a str },
    Ls,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum LsType {
    Dir,
    File { size: u64 },
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct LsRow<'a> {
    typ: LsType,
    name: &'a str,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
struct DirId(usize);

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
struct FileId(usize);

struct Fs<'a> {
    files: Vec<FileEntry<'a>>,
    dirs: Vec<DirEntry<'a>>,
}

impl<'a> Fs<'a> {
    pub fn new() -> Self {
        Fs {
            files: vec![],
            dirs: vec![DirEntry {
                parent: DirId(0), // make `/` self-referential
                name: "/",
                files: vec![],
                dirs: vec![],
            }],
        }
    }
}

struct DirEntry<'a> {
    parent: DirId,
    name: &'a str,
    files: Vec<FileId>,
    dirs: Vec<DirId>,
}

struct FileEntry<'a> {
    parent: DirId,
    name: &'a str,
    size: u64,
}

struct Walker<'a, 'b> {
    cur_dir: Vec<DirId>,
    fs: &'a mut Fs<'b>,
}

impl<'a, 'b> Walker<'a, 'b> {
    fn new(fs: &'a mut Fs<'b>) -> Self {
        Walker {
            cur_dir: vec![DirId(0)],
            fs,
        }
    }

    fn enter(&mut self, name: &str) {
        let cur = self.cur_dir.last().copied().unwrap();
        let fs = &*self.fs;
        let next = *fs.dirs[cur.0]
            .dirs
            .iter()
            .find(|idx| fs.dirs[idx.0].name == name)
            .expect("no such directory");
        self.cur_dir.push(next);
    }

    fn leave(&mut self) {
        assert!(self.cur_dir.len() > 1, "cannot leave root");
        self.cur_dir.pop();
    }

    fn insert_dir(&mut self, name: &'b str) {
        let cur = self.cur_dir.last().copied().unwrap();

        let new_dir = DirEntry {
            name,
            parent: cur,
            files: vec![],
            dirs: vec![],
        };
        let new_dir_id = DirId(self.fs.dirs.len());
        self.fs.dirs.push(new_dir);
        self.fs.dirs[cur.0].dirs.push(new_dir_id);
    }

    fn insert_file(&mut self, name: &'b str, size: u64) {
        let cur = self.cur_dir.last().copied().unwrap();

        let new_file = FileEntry {
            name,
            size,
            parent: cur,
        };
        let new_file_id = FileId(self.fs.files.len());
        self.fs.files.push(new_file);
        self.fs.dirs[cur.0].files.push(new_file_id);
    }

    fn goto_root(&mut self) {
        self.cur_dir = vec![DirId(0)];
    }
}

impl<'a> Debug for Fs<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn print_level(
            fs: &Fs,
            parent: DirId,
            indent: &mut String,
            f: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            let entry = &fs.dirs[parent.0];
            writeln!(f, "{}- {} (dir)", indent, entry.name)?;
            let indent_before = indent.len();
            indent.push_str("  ");
            for dir in entry.dirs.iter().copied() {
                print_level(fs, dir, indent, f)?;
            }
            for file in entry.files.iter().copied() {
                let entry = &fs.files[file.0];
                writeln!(f, "{}- {} (file, size={})", indent, entry.name, entry.size)?;
            }
            indent.truncate(indent_before);
            Ok(())
        }
        print_level(self, DirId(0), &mut String::new(), f)
    }
}

crate::test_day!(RUN, "day7", "1306611", "13210366");
