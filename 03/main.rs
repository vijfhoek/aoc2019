use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::io::BufRead;

enum Direction {
    X,
    Y,
}

fn run(items: &[Vec<(Direction, i64)>]) -> (i64, i64) {
    let mut positions = HashMap::new();
    let mut crossings_manhattan = BinaryHeap::new();
    let mut crossings_length = BinaryHeap::new();

    for wire in items {
        let mut x = 0i64;
        let mut y = 0i64;
        let mut self_positions = HashSet::new();
        let mut wire_length = 0i64;

        for (direction, amount) in wire {
            let amount = *amount;
            for _ in if amount < 0 { amount..=-1 } else { 1..=amount } {
                match direction {
                    Direction::X => x += amount.signum(),
                    Direction::Y => y += amount.signum(),
                };
                wire_length += 1;

                if self_positions.contains(&(x, y)) {
                    continue;
                }

                let lengths = positions.entry((x, y)).or_insert_with(HashSet::new);
                lengths.insert(wire_length);

                let manhattan = x.abs() + y.abs();
                if manhattan != 0 && lengths.len() > 1 && !self_positions.contains(&(x, y)) {
                    crossings_manhattan.push(Reverse(manhattan));
                }

                if lengths.len() > 1 && !self_positions.contains(&(x, y)) {
                    crossings_length.push(Reverse(lengths.iter().sum()));
                }

                self_positions.insert((x, y));
            }
        }
    }

    let Reverse(manhattan) = crossings_manhattan.pop().unwrap();
    let Reverse(length) = crossings_length.pop().unwrap();
    (manhattan, length)
}

fn main() {
    let items: Vec<Vec<(Direction, i64)>> = std::io::stdin()
        .lock()
        .lines()
        .map(|wire| {
            wire.unwrap()
                .split(',')
                .map(|command| {
                    let (direction, amount) = command.trim().split_at(1);
                    let amount: i64 = amount.parse().unwrap();
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
    dbg!(part1, part2);
}
