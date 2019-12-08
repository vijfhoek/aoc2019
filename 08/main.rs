use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::io::{BufRead, Read};

const WIDTH: usize = 25;
const HEIGHT: usize = 6;

fn read_input() -> Vec<u32> {
    let mut stdin = std::io::stdin();
    let mut input = String::new();
    stdin.read_to_string(&mut input).unwrap();
    input
        .trim()
        .chars()
        .map(|c| c.to_digit(10).unwrap())
        .collect()
}

fn part1(input: &Vec<u32>) -> usize {
    let mut heap = BinaryHeap::new();
    for i in (0..input.len()).step_by(WIDTH * HEIGHT) {
        let layer = &input[i..i + WIDTH * HEIGHT];
        let zeros = layer.iter().filter(|i| **i == 0).count();
        let ones = layer.iter().filter(|i| **i == 1).count();
        let twos = layer.iter().filter(|i| **i == 2).count();
        heap.push((Reverse(zeros), ones * twos));
    }

    heap.pop().unwrap().1
}

fn part2(input: &Vec<u32>) {
    let mut image = [2; WIDTH * HEIGHT];
    for i in (0..input.len()).step_by(WIDTH * HEIGHT) {
        let layer = &input[i..i + WIDTH * HEIGHT];
        for (i, pixel) in layer.iter().enumerate() {
            if image[i] == 2 {
                image[i] = *pixel;
            }
        }
    }

    for y in (0..image.len()).step_by(WIDTH) {
        for x in 0..WIDTH {
            let pixel = if image[y + x] > 0 { "🦀" } else { "  " };
            print!("{}", pixel);
        }
        println!();
    }
}

fn main() {
    let input = read_input();
    dbg!(part1(&input));
    part2(&input);
}
