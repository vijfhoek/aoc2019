use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::io::BufRead;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Component {
    count: usize,
    name: usize,
}

impl Component {
    fn from_str(string: &str) -> Self {
        let mut split = string.split(' ');
        let count = split.next().unwrap().parse().unwrap();
        let name = usize::from_str_radix(split.next().unwrap(), 36).unwrap();

        Self { count, name }
    }
}

#[derive(Debug)]
struct Reaction {
    components: Vec<Component>,
    result: Component,
}

impl Reaction {
    fn from_str(string: &str) -> Self {
        let line = string.trim();

        let mut sides = line.split(" => ");
        let components: Vec<_> = sides
            .next()
            .unwrap()
            .split(", ")
            .map(|component| Component::from_str(component))
            .collect();
        let result = Component::from_str(sides.next().unwrap());

        Self { components, result }
    }
}

impl Debug for Component {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        let mut value = self.name;
        let mut name = Vec::new();
        while value > 0 {
            name.push(value % 36);
            value /= 36;
        }
        let name = name
            .iter()
            .rev()
            .map(|x| {
                "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"
                    .chars()
                    .nth(*x as usize)
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("");

        write!(formatter, "{} {}", self.count, name)
    }
}

fn read_input() -> HashMap<usize, Reaction> {
    let stdin = std::io::stdin();

    stdin
        .lock()
        .lines()
        .map(|line| {
            let reaction = Reaction::from_str(&line.unwrap());
            (reaction.result.name, reaction)
        })
        .collect()
}

fn part1(reactions: &HashMap<usize, Reaction>) -> usize {
    let ore = usize::from_str_radix("ORE", 36).unwrap();
    let mut required_ore = 0;
    let mut produced = 0;

    let mut buffer: HashMap<usize, usize> = HashMap::new();
    buffer.insert(ore, 1_000_000_000_000);
    let mut required = vec![Component::from_str("1 FUEL")];
    loop {
        if buffer.get(&ore).unwrap() % 1_000_000 == 0 {
        println!("{:?}", buffer.get(&ore));
        }
        if required.is_empty() {
            produced += 1;
            required.push(Component::from_str("1 FUEL"));
        }

        let mut component = required.pop().unwrap();

        // dbg!(component);
        if let Some(buffered) = buffer.remove(&component.name) {
            // dbg!(buffered);
            component.count = match buffered.cmp(&component.count) {
                Ordering::Greater => {
                    buffer.insert(component.name, buffered - component.count);
                    0
                }
                Ordering::Equal => 0,
                Ordering::Less => component.count - buffered,
            }
        }

        if component.count == 0 {
            continue;
        } else if component.name == ore {
            break;
        }

        let reaction = reactions.get(&component.name).unwrap();
        let count = (component.count as f64 / reaction.result.count as f64).ceil() as usize;
        for _ in 0..count {
            for requirement in &reaction.components {
                required.push(*requirement);
            }
        }

        buffer.insert(
            reaction.result.name,
            reaction.result.count * count - component.count,
        );
    }

    produced
}

fn main() {
    let reactions = read_input();
    dbg!(part1(&reactions));
}
