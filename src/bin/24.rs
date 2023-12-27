use anyhow::{Context, Result};
use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;
use num_traits::identities::Zero;

advent_of_code::solution!(24);

fn parse_input(input: &str) -> Result<Vec<Hail>> {
    let mut out = Vec::new();
    for line in input.lines() {
        let (location_str, velocity_str) = line
            .split_once('@')
            .context("Expected to find '@' separator")?;

        let (x, rest) = location_str
            .trim()
            .split_once(',')
            .context("Expected at least one ','")?;
        let (y, z) = rest.split_once(',').context("Expected at least two ','")?;
        let point = Point(V3 {
            x: x.trim().parse().context("Failed to parse point.x")?,
            y: y.trim().parse().context("Failed to parse point.y")?,
            z: z.trim().parse().context("Failed to parse point.z")?,
        });

        let (x, rest) = velocity_str
            .trim()
            .split_once(',')
            .context("Expected at least one ','")?;
        let (y, z) = rest.split_once(',').context("Expected at least two ','")?;
        let velocity = Velocity(V3 {
            x: x.trim().parse().context("Failed to parse velocity.x")?,
            y: y.trim().parse().context("Failed to parse velocity.y")?,
            z: z.trim().parse().context("Failed to parse velocity.z")?,
        });

        out.push(Hail { point, velocity });
    }

    Ok(out)
}

#[derive(Debug, Clone)]
struct V3 {
    x: BigInt,
    y: BigInt,
    z: BigInt,
}

impl V3 {
    fn cross_prod(&self, other: &Self) -> Self {
        Self {
            // a2 b3 - a3 * b2,
            x: &self.y * &other.z - &self.z * &other.y,
            // a3 b1 - a1 * b3,
            y: &self.z * &other.x - &self.x * &other.z,
            // a1 b2 - a2 * b1,
            z: &self.x * &other.y - &self.y * &other.x,
        }
    }

    fn dot_prod(&self, other: &Self) -> BigInt {
        &self.x * &other.x + &self.y * &other.y + &self.z * &other.z
    }

    fn sub(&self, other: &Self) -> Self {
        Self {
            x: &self.x - &other.x,
            y: &self.y - &other.y,
            z: &self.z - &other.z,
        }
    }

    fn independent(&self, other: &Self) -> bool {
        let cross = self.cross_prod(other);
        !cross.x.is_zero() || !cross.y.is_zero() || !cross.z.is_zero()
    }
}

#[derive(Debug, Clone)]
struct Point(V3);

#[derive(Debug, Clone)]
struct Velocity(V3);

#[derive(Debug, Clone)]
struct Hail {
    point: Point,
    velocity: Velocity,
}

impl Hail {
    fn time_until(&self, target_x: f64) -> f64 {
        (target_x - self.point.0.x.to_f64().unwrap()) / self.velocity.0.x.to_f64().unwrap()
    }

    fn line_x_for_y(&self) -> Line {
        let m = self.velocity.0.y.to_f64().unwrap() / self.velocity.0.x.to_f64().unwrap();
        Line {
            m,
            b: self.point.0.y.to_f64().unwrap() - (m * self.point.0.x.to_f64().unwrap()),
        }
    }
}

#[derive(Debug)]
struct Line {
    m: f64,
    b: f64,
}

impl Line {
    fn y(&self, x: f64) -> f64 {
        self.m * x + self.b
    }

    fn intersect_2d(&self, other: &Line) -> (f64, f64) {
        let x = (other.b - self.b) / (self.m - other.m);

        (x, self.y(x))
    }
}

fn test_in_range(lines: &[(Line, Hail)], lowest: u64, highest: u64) -> usize {
    let mut intersects = 0;
    for (i, (la, ha)) in lines.iter().enumerate() {
        for (lb, hb) in lines.iter().skip(i + 1) {
            let (x, y) = la.intersect_2d(lb);

            let within_x = (x.floor() as u64) > lowest && (x.ceil() as u64) < highest;
            let within_y = (y.ceil() as u64) > lowest && (y.floor() as u64) < highest;
            let within = within_x && within_y;

            let time_until_a = ha.time_until(x);
            let time_until_b = hb.time_until(x);
            let after_a = time_until_a >= 0.0;
            let after_b = time_until_b >= 0.0;
            let after = after_a && after_b;
            if within && after {
                intersects += 1;
            }
        }
    }
    intersects
}

pub fn part_one(input: &str) -> Result<Option<usize>, anyhow::Error> {
    let hail = parse_input(input).context("Failed to parse input")?;
    let lines: Vec<(Line, Hail)> = hail.into_iter().map(|h| (h.line_x_for_y(), h)).collect();

    let lowest: u64 = 200000000000000;
    let highest: u64 = 400000000000000;

    Ok(Some(test_in_range(&lines, lowest, highest)))
}

fn find_independent<'a>(hail: &'a [Hail], existing_stones: &[&Hail]) -> Option<&'a Hail> {
    for new_stone in hail.iter() {
        let mut collided = false;
        for existing in existing_stones {
            if !new_stone.velocity.0.independent(&existing.velocity.0) {
                collided = true;
                break;
            }
        }
        if !collided {
            return Some(new_stone);
        }
    }
    None
}

fn lin(a_s: &BigInt, a: &V3, b_s: &BigInt, b: &V3, c_s: &BigInt, c: &V3) -> V3 {
    V3 {
        x: (&a.x * a_s + &b.x * b_s + &c.x * c_s),
        y: (&a.y * a_s + &b.y * b_s + &c.y * c_s),
        z: (&a.z * a_s + &b.z * b_s + &c.z * c_s),
    }
}

fn find_plane(s1: &Hail, s2: &Hail) -> (V3, BigInt) {
    let p12 = s1.point.0.sub(&s2.point.0);
    let v12 = s1.velocity.0.sub(&s2.velocity.0);
    let vv = s1.velocity.0.cross_prod(&s2.velocity.0);

    (p12.cross_prod(&v12), p12.dot_prod(&vv))
}

// Most of the math logic here is adapted from:
// https://www.reddit.com/r/adventofcode/comments/18pnycy/comment/kersplf/?utm_source=share&utm_medium=web3x&utm_name=web3xcss&utm_term=1&utm_content=share_button
pub fn part_two(input: &str) -> Result<Option<i128>, anyhow::Error> {
    let hail = parse_input(input).context("Failed to parse input")?;

    let s1 = &hail[0];
    let s2 = find_independent(&hail, &[s1]).context("Failed to find S2")?;
    let s3 = find_independent(&hail, &[s1, s2]).context("Failed to find S2")?;

    let (a, a_s) = find_plane(s1, s2);
    let (b, b_s) = find_plane(s1, s3);
    let (c, c_s) = find_plane(s2, s3);

    let w = lin(
        &a_s,
        &b.cross_prod(&c),
        &b_s,
        &c.cross_prod(&a),
        &c_s,
        &a.cross_prod(&b),
    );
    let t = a.dot_prod(&b.cross_prod(&c));

    let w = V3 {
        x: w.x / &t,
        y: w.y / &t,
        z: w.z / &t,
    };

    let w1 = s1.velocity.0.sub(&w);
    let w2 = s2.velocity.0.sub(&w);

    let ww = w1.cross_prod(&w2);

    let e_s = ww.dot_prod(&s2.point.0.cross_prod(&w2));
    let f_s = ww.dot_prod(&s1.point.0.cross_prod(&w1));
    let g_s = s1.point.0.dot_prod(&ww);
    let s_s = ww.dot_prod(&ww);

    let rock = lin(&e_s, &w1, &(&f_s * -1), &w2, &g_s, &ww);

    let out = (rock.x + rock.y + rock.z) / s_s;
    Ok(out.to_i128())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_intesect() -> anyhow::Result<()> {
        let l1 = Line { m: 2.0, b: 3.0 };
        let l2 = Line { m: -0.5, b: 7.0 };

        let (x, y) = l1.intersect_2d(&l2);
        assert_eq!(x, 1.6);
        assert_eq!(y, 6.2);

        Ok(())
    }

    #[test]
    fn test_part_one_example() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);

        let hail = parse_input(input).context("Failed to parse input")?;
        let data: Vec<(Line, Hail)> = hail.into_iter().map(|h| (h.line_x_for_y(), h)).collect();

        let result = test_in_range(&data, 7, 27);
        assert_eq!(result, 2);
        Ok(())
    }

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input)?;
        assert_eq!(result, Some(12740));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input)?;
        assert_eq!(result, Some(47));
        Ok(())
    }
}
