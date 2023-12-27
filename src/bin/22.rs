use advent_of_code::template::RunType;
use anyhow::{anyhow, Context, Result};
use std::collections::{BTreeMap, BTreeSet};

advent_of_code::solution!(22);

#[derive(Debug, Clone, Eq, PartialEq)]
struct Point {
    x: u64,
    y: u64,
    z: u64,
}

impl std::str::FromStr for Point {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
        let (x_str, input) = input.split_once(',').context("Expected at least 1 ,")?;
        let (y_str, z_str) = input.split_once(',').context("Expected at least 1 ,")?;

        Ok(Self {
            x: x_str.parse().context("Failed to parse x_str")?,
            y: y_str.parse().context("Failed to parse y_str")?,
            z: z_str.parse().context("Failed to parse z_str")?,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Brick {
    a: Point,
    b: Point,
}

impl Brick {
    fn range(&self, a: u64, b: u64) -> std::ops::Range<u64> {
        if a <= b {
            a..b + 1
        } else {
            b + 1..a
        }
    }

    fn lower(&mut self, lower_z: u64) -> usize {
        let delta = if self.a.z <= self.b.z {
            self.a.z - lower_z
        } else {
            self.b.z - lower_z
        };
        self.a.z -= delta;
        self.b.z -= delta;
        delta as usize
    }

    fn x_range(&self) -> std::ops::Range<u64> {
        self.range(self.a.x, self.b.x)
    }

    fn y_range(&self) -> std::ops::Range<u64> {
        self.range(self.a.y, self.b.y)
    }

    fn z_range(&self) -> std::ops::Range<u64> {
        self.range(self.a.z, self.b.z)
    }
}

#[derive(Debug)]
struct Bricks {
    bricks: Vec<Brick>,
    brick_map: Vec<Vec<BTreeMap<u64, usize>>>,
}

struct FallReport(Vec<(usize, usize)>);

impl Bricks {
    fn from_snapshot(mut bricks: Vec<Brick>, fall: bool) -> Result<(Self, FallReport)> {
        let mut min_x = u64::MAX;
        let mut min_y = u64::MAX;
        let mut max_x = 0;
        let mut max_y = 0;

        for brick in &bricks {
            min_x = std::cmp::min(min_x, brick.x_range().start);
            min_y = std::cmp::min(min_x, brick.y_range().start);
            max_x = std::cmp::max(max_x, brick.x_range().end);
            max_y = std::cmp::max(max_x, brick.y_range().end);
        }

        if min_x != 0 {
            return Err(anyhow!("Min x must be 0"));
        }
        if min_y != 0 {
            return Err(anyhow!("Min y must be 0"));
        }

        let mut map: Vec<Vec<BTreeMap<u64, usize>>> = Vec::with_capacity(max_x as usize);
        for _ in 0..max_x {
            let mut row = Vec::with_capacity(max_y as usize);
            for _ in 0..max_y {
                row.push(BTreeMap::new());
            }
            map.push(row);
        }

        // We want to go through bricks from the bottom of the snapshot upwards
        bricks.sort_by_key(|brick| brick.z_range().start);
        let mut falls = Vec::new();

        for (brick_id, brick) in bricks.iter_mut().enumerate() {
            // We can at least stay where we are
            let mut lowest_z = 0;
            for x in brick.x_range() {
                let row = &map[x as usize];
                for y in brick.y_range() {
                    if let Some(existing) = row[y as usize].iter().last() {
                        lowest_z = std::cmp::max(*existing.0, lowest_z);
                    }
                }
            }
            if fall {
                let fall = brick.lower(lowest_z + 1);
                falls.push((brick_id, fall));
            }
            for x in brick.x_range() {
                for y in brick.y_range() {
                    for z in brick.z_range() {
                        map[x as usize][y as usize].insert(z, brick_id);
                    }
                }
            }
        }

        Ok((
            Self {
                bricks,
                brick_map: map,
            },
            FallReport(falls),
        ))
    }

    fn supporting(&self, brick_id: usize) -> BTreeSet<usize> {
        let mut out = BTreeSet::new();
        let brick = &self.bricks[brick_id];
        let top = brick.z_range().end;
        // println!("Supporting: {} {:?}", brick_id, brick);
        for x in brick.x_range() {
            let row = &self.brick_map[x as usize];
            for y in brick.y_range() {
                // println!("x={} y={} and next={}", x, y, top);
                if let Some(other) = row[y as usize].get(&(top)) {
                    out.insert(*other);
                }
            }
        }
        // println!("out={:?}", out);
        out
    }

    fn supported_by(&self, brick_id: usize) -> BTreeSet<usize> {
        let mut out = BTreeSet::new();
        let brick = &self.bricks[brick_id];
        // println!("Start: {} {:?}", brick_id, brick);
        let bottom = brick.z_range().start;
        if bottom == 0 {
            return out;
        }
        for x in brick.x_range() {
            let row = &self.brick_map[x as usize];
            for y in brick.y_range() {
                // println!("HERE: {:?} {} {} {}", brick, x, y, bottom - 1);
                if let Some(other) = row[y as usize].get(&(bottom - 1)) {
                    out.insert(*other);
                }
            }
        }
        out
    }

    fn would_fall(&self, brick_id: usize) -> Result<BTreeSet<usize>> {
        let mut out = BTreeSet::new();
        let supporting = self.supporting(brick_id);
        for supporting_id in supporting {
            let supporting = self.supported_by(supporting_id);
            if !supporting.contains(&brick_id) {
                return Err(anyhow!(
                    "Somehow brick {} is not supported by {} even though it's supporting it",
                    brick_id,
                    supporting_id
                ));
            }
            // If this brick is supported only by the current one it
            // will fall if removed
            if supporting.len() == 1 {
                out.insert(supporting_id);
            }
        }
        Ok(out)
    }

    fn can_disintegrate(&self) -> Result<BTreeSet<usize>> {
        let mut out = BTreeSet::new();
        for (brick_id, _) in self.bricks.iter().enumerate() {
            if self.would_fall(brick_id)?.is_empty() {
                out.insert(brick_id);
            }
        }

        Ok(out)
    }
}

fn parse_input(input: &str) -> Result<Vec<Brick>> {
    let mut out = Vec::new();
    for line in input.lines() {
        let (a_str, b_str) = line.split_once('~').context("Expected ~ dividing points")?;
        out.push(Brick {
            a: a_str.parse().context("failed to parse point a")?,
            b: b_str.parse().context("failed to parse point b")?,
        });
    }

    Ok(out)
}

pub fn part_one(input: &str, _run_type: RunType) -> Result<Option<usize>, anyhow::Error> {
    let snapshot = parse_input(input).context("Failed to parse input")?;
    let (bricks, _) = Bricks::from_snapshot(snapshot, true).context("Failed to build bricks")?;

    // println!("After falling:");
    // print_bricks(&bricks);

    let can_destroy = bricks
        .can_disintegrate()
        .context("Failed to find which bricks we can remove")?;

    Ok(Some(can_destroy.len()))
}

pub fn part_two(input: &str, _run_type: RunType) -> Result<Option<usize>, anyhow::Error> {
    let snapshot = parse_input(input).context("Failed to parse input")?;
    let (bricks, _) = Bricks::from_snapshot(snapshot, true).context("Failed to build bricks")?;

    let mut out = 0;
    for (brick_id, _) in bricks.bricks.iter().enumerate() {
        let mut to_test = Vec::with_capacity(bricks.bricks.len());
        for (other_brick_id, other_brick) in bricks.bricks.iter().enumerate() {
            if brick_id != other_brick_id {
                to_test.push(other_brick.clone());
            }
        }

        let (_, fall_report) = Bricks::from_snapshot(to_test, true)?;
        let num_fallen = fall_report
            .0
            .iter()
            .filter(|(_, fall_dist)| *fall_dist != 0)
            .count();

        println!("Removing {} makes {} fall", brick_id, num_fallen);
        out += num_fallen;
    }
    Ok(Some(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input, RunType::Example)?;
        assert_eq!(result, Some(5));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input, RunType::Example)?;
        assert_eq!(result, Some(7));
        Ok(())
    }
}
