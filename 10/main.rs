use std::collections::{BTreeMap, BinaryHeap, VecDeque};
use std::f64;
use std::io::BufRead;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Point {
    x: usize,
    y: usize,
}

fn read_input() -> (Vec<Point>, (usize, usize)) {
    let stdin = std::io::stdin();
    let mut asteroids = Vec::new();
    let mut width = 0;
    let mut height = 0;

    for (y, line) in stdin.lock().lines().enumerate() {
        let line = line.unwrap();
        let line = line.trim();

        height += 1;
        if width == 0 {
            width = line.len();
        }

        for (x, c) in line.trim().chars().enumerate() {
            if c == '#' {
                asteroids.push(Point { x, y });
            }
        }
    }

    (asteroids, (width, height))
}

fn get_sight<'a>(station: &Point, asteroids: &'a Vec<Point>) -> BTreeMap<i64, Vec<&'a Point>> {
    let mut angles = BTreeMap::new();
    for asteroid in asteroids {
        if asteroid.x == station.x && asteroid.y == station.y {
            continue;
        }

        let dy = station.y as f64 - asteroid.y as f64;
        let dx = asteroid.x as f64 - station.x as f64;
        let angle = (dx.atan2(dy) * 180.0 * 256.0 / f64::consts::PI).round() as i64;
        let angle = (angle + 360 * 256) % (360 * 256);
        angles.entry(angle).or_insert_with(Vec::new).push(asteroid);
    }

    angles
}

fn part1(input: &Vec<Point>) -> (usize, &Point) {
    let mut sights = BinaryHeap::new();
    for station in input {
        let sight = get_sight(&station, input);
        sights.push((sight.len(), station));
    }
    sights.pop().unwrap()
}

fn part2(input: &Vec<Point>, station: &Point) -> usize {
    let mut sight = get_sight(&station, input);
    for (_, asteroids) in sight.iter_mut() {
        asteroids.sort_by_key(|a| {
            -(station.x as i64 - a.x as i64).pow(2) - (station.y as i64 - a.y as i64).pow(2)
        });
    }

    let mut obliterated = 0;
    loop {
        let mut count = 0;
        for (angle, asteroids) in sight.iter_mut() {
            let angle = *angle as f64 / 256.0;
            let asteroid = match asteroids.pop() {
                Some(v) => v,
                None => continue,
            };

            count += 1;

            obliterated += 1;
            println!("{:>5}: {:.1}, {:?}", obliterated, angle, asteroid);
            if obliterated == 200 {
                return asteroid.x * 100 + asteroid.y;
            }
        }

        if count == 0 {
            break;
        }
    }

    0
}

fn main() {
    let (input, (_, _)) = read_input();

    let (part1, station) = part1(&input);
    dbg!(part1);
    dbg!(part2(&input, &station));
}
