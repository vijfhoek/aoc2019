use regex::Regex;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::BufRead;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct Point {
    x: i64,
    y: i64,
    z: i64,
}

impl Point {
    fn energy(&self) -> i64 {
        self.x.abs() + self.y.abs() + self.z.abs()
    }
}

impl Display for Point {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(
            formatter,
            "<x={:>3}, y={:>3}, z={:>3}>",
            self.x, self.y, self.z
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct Moon {
    pos: Point,
    vel: Point,
}

impl Moon {
    fn update_vel(&self, other: &Self) -> Point {
        let mut vel = self.vel.clone();
        if self.pos.x < other.pos.x {
            vel.x += 1;
        } else if self.pos.x > other.pos.x {
            vel.x -= 1;
        }

        if self.pos.y < other.pos.y {
            vel.y += 1;
        } else if self.pos.y > other.pos.y {
            vel.y -= 1;
        }

        if self.pos.z < other.pos.z {
            vel.z += 1;
        } else if self.pos.z > other.pos.z {
            vel.z -= 1;
        }
        vel
    }

    fn update_pos(&mut self) {
        self.pos.x += self.vel.x;
        self.pos.y += self.vel.y;
        self.pos.z += self.vel.z;
    }
}

impl Display for Moon {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "pos={}, vel={}", self.pos, self.vel)
    }
}

fn read_input() -> Vec<Moon> {
    let stdin = std::io::stdin();
    let re = Regex::new(r"<x=([\d\-]+), y=([\d\-]+), z=([\d\-]+)>").unwrap();

    stdin
        .lock()
        .lines()
        .map(|line| {
            let line = line.unwrap();
            let captures = re.captures(&line).unwrap();
            Moon {
                pos: Point {
                    x: captures[1].parse().unwrap(),
                    y: captures[2].parse().unwrap(),
                    z: captures[3].parse().unwrap(),
                },
                vel: Point { x: 0, y: 0, z: 0 },
            }
        })
        .collect()
}

fn part1(moons: &mut Vec<Moon>) -> i64 {
    const steps: usize = 1000;
    for ts in 0..steps {
        println!("\nAfter {} steps:", ts);
        for moon in moons.iter() {
            println!("{}", moon);
        }

        for b in 0..moons.len() {
            for a in 0..moons.len() {
                if a == b {
                    continue;
                }

                moons[a].vel = moons[a].update_vel(&moons[b]);
            }
        }

        for moon in moons.iter_mut() {
            moon.update_pos();
        }
    }

    println!("\nAfter {} steps:", steps);
    for moon in moons.iter() {
        println!("{}", moon);
    }

    println!("\nEnergy:");
    moons
        .iter()
        .map(|moon| {
            let pot = moon.pos.energy();
            let kin = moon.vel.energy();
            let total = pot * kin;
            println!("pot: {:>5}; kin: {:>5}; total: {:>5}", pot, kin, total);
            total
        })
        .sum()
}

fn part2(moons: &mut Vec<Moon>) -> i64 {
    let mut sets: Vec<_> = (0..moons.len()).map(|_| HashMap::new()).collect();
    let mut cycle_lengths: Vec<_> = (0..moons.len()).map(|_| None).collect();

    let mut ts = 0usize;
    loop {
        for b in 0..moons.len() {
            for a in 0..moons.len() {
                if a == b {
                    continue;
                }
                moons[a].vel = moons[a].update_vel(&moons[b]);
            }
        }
        for moon in moons.iter_mut() {
            moon.update_pos();
        }

        let mut has_length = 0;
        for (i, moon) in moons.iter().enumerate() {
            if cycle_lengths[i].is_some() {
                has_length += 1;
                continue;
            }

            if let Some(old_ts) = sets[i].insert(moon.clone(), ts) {
                cycle_lengths[i] = Some((old_ts, ts - old_ts));
            }
        }

        if ts % 1000000 == 0 {
            println!("{} {}     \r", ts, has_length);
        }
        if has_length == moons.len() {
            break;
        }

        ts += 1;
    }

    dbg!(cycle_lengths);

    0
}

fn main() {
    let moons = read_input();
    dbg!(part1(&mut moons.clone()));
    dbg!(part2(&mut moons.clone()));

    // let cycle_lengths = [
    //     Some((23446604, 2010370)),
    //     Some((3247283, 16599261)),
    //     Some((5033188, 10043143)),
    //     Some((834120, 11533736)),
    // ];
    // for (start, length) in cycle_lengths {

    // }
}
