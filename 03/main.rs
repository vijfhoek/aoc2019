use ordered_float::NotNan;
use std::collections::BinaryHeap;
use std::io::BufRead;
use std::time::Instant;

enum Direction {
    X,
    Y,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Point(f32, f32);
impl Point {
    fn manhattan(self) -> f32 {
        self.0.abs() + self.1.abs()
    }
}

fn line_intersection(a: &(Point, Point), b: &(Point, Point)) -> Option<Point> {
    let a1 = a.0;
    let a2 = a.1;
    let b1 = b.0;
    let b2 = b.1;

    let denom = (a2.0 - a1.0) * (b2.1 - b1.1) - (a2.1 - a1.1) * (b2.0 - b1.0);
    if denom == 0.0 {
        return None;
    }

    let t = ((a1.1 - b1.1) * (b2.0 - b1.0) - (a1.0 - b1.0) * (b2.1 - b1.1)) / denom;
    let u = ((a1.1 - b1.1) * (a2.0 - a1.0) - (a1.0 - b1.0) * (a2.1 - a1.1)) / denom;
    if t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0 {
        Some(Point(a1.0 + t * (a2.0 - a1.0), a1.1 + t * (a2.1 - a1.1)))
    } else {
        None
    }
}

fn line_length(a: Point, b: Point) -> f32 {
    // Manhattan distance is fine, they're straight lines anyway.
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

fn to_lines(wire: &[(Direction, f32)]) -> Vec<(Point, Point)> {
    let mut current_pos = Point(0.0, 0.0);
    let mut lines = Vec::new();

    for (direction, amount) in wire {
        let old_pos = current_pos;
        match direction {
            Direction::X => current_pos.0 += amount,
            Direction::Y => current_pos.1 += amount,
        };

        lines.push((old_pos, current_pos));
    }

    lines
}

fn run(items: &[Vec<(Direction, f32)>]) -> (f32, f32) {
    let wire1 = to_lines(&items[0]);
    let wire2 = to_lines(&items[1]);

    let mut manhattans = BinaryHeap::new();
    let mut lengths = BinaryHeap::new();

    let mut a_length = 0.0;
    for a in &wire1 {
        a_length += line_length(a.0, a.1);

        let mut b_length = 0.0;
        for b in &wire2 {
            b_length += line_length(b.0, b.1);

            let point = match line_intersection(&a, &b) {
                None => continue,
                Some(point) => point,
            };

            let manhattan = point.manhattan();
            if manhattan != 0.0 {
                manhattans.push(-NotNan::new(manhattan).unwrap());
            }

            let length =
                (a_length - line_length(a.1, point)) + (b_length - line_length(b.1, point));
            if length != 0.0 {
                lengths.push(-NotNan::new(length).unwrap())
            }
        }
    }

    (
        -manhattans.peek().unwrap().into_inner(),
        -lengths.peek().unwrap().into_inner(),
    )
}

fn main() {
    let now = Instant::now();
    let items: Vec<Vec<(Direction, f32)>> = std::io::stdin()
        .lock()
        .lines()
        .map(|wire| {
            wire.unwrap()
                .split(',')
                .map(|command| {
                    let (direction, amount) = command.trim().split_at(1);
                    let amount: f32 = amount.parse().unwrap();
                    match direction {
                        "U" => (Direction::Y, amount),
                        "D" => (Direction::Y, -amount),
                        "R" => (Direction::X, amount),
                        "L" => (Direction::X, -amount),
                        _ => panic!(),
                    }
                })
                .collect()
        })
        .collect();

    let (part1, part2) = run(&items);
    dbg!(part1, part2, now.elapsed());
}
