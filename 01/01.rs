use std::io::{self, BufRead};

fn part1(modules: &Vec<i64>) -> i64 {
    modules.iter().map(|mass| mass / 3 - 2).sum()
}

fn part2(mut modules: Vec<i64>) -> i64 {
    let mut total = 0;
    loop {
        let mass = match modules.pop() {
            None => break,
            Some(mass) => mass,
        };

        let fuel = mass / 3 - 2;
        if fuel > 0 {
            modules.push(fuel);
            total += fuel;
        }
    }

    total
}

fn main() -> Result<(), ()> {
    let modules: Vec<i64> = std::io::stdin()
        .lock()
        .lines()
        .map(|mass| mass.unwrap().parse::<i64>().unwrap())
        .collect();

    dbg!(part1(&modules));
    dbg!(part2(modules));

    Ok(())
}
