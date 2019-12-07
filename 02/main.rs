use std::io::{self, BufRead};
use std::time::Instant;

fn run(mem: &mut Vec<i64>) {
    let mut ip = 0;
    loop {
        let opcode = mem[ip];
        let left = mem[ip + 1] as usize;
        let right = mem[ip + 2] as usize;
        let to = mem[ip + 3] as usize;
        match opcode {
            1 => {
                mem[to] = mem[left] + mem[right];
                ip += 4;
            }

            2 => {
                mem[to] = mem[left] * mem[right];
                ip += 4;
            }

            _ => break,
        }
    }
}

fn part1(mem: &Vec<i64>) -> i64 {
    let mut mem = (*mem).clone();
    mem[1] = 12;
    mem[2] = 2;

    run(&mut mem);
    mem[0]
}

fn part2(mem: &Vec<i64>) -> i64 {
    for noun in 0..100 {
        for verb in 0..100 {
            let mut mem = (*mem).clone();
            mem[1] = noun;
            mem[2] = verb;
            run(&mut mem);

            if mem[0] == 19690720 {
                return 100 * noun + verb;
            }
        }
    }
    panic!("no solution found");
}

fn main() -> Result<(), ()> {
    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input).unwrap();

    let items: Vec<i64> = input
        .split(",")
        .map(|x| x.trim().parse().unwrap())
        .collect();

    dbg!(part1(&items));
    dbg!(part2(&items));

    Ok(())
}
