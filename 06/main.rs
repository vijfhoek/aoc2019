use std::collections::HashMap;
use std::io::BufRead;
use std::time::Instant;

fn main() {
    let san: usize = usize::from_str_radix("san", 36).unwrap();
    let you: usize = usize::from_str_radix("you", 36).unwrap();
    let com: usize = usize::from_str_radix("com", 36).unwrap();

    let stdin = std::io::stdin();
    let lines = stdin.lock().lines();
    let mut tree = HashMap::new();
    for line in lines {
        let line = line.unwrap();
        let mut parts = line.trim().split(')');
        let from = usize::from_str_radix(parts.next().unwrap(), 36).unwrap();
        let to = usize::from_str_radix(parts.next().unwrap(), 36).unwrap();

        let entry = tree.entry(from).or_insert_with(Vec::new);
        entry.push(to);
    }
    let now = Instant::now();

    let mut orbits = 0;
    let mut santa_path = None;
    let mut you_path = None;

    let mut stack = vec![(com, vec![])];
    while !stack.is_empty() {
        let (node, path) = stack.pop().unwrap();
        orbits += path.len();

        if node == san {
            santa_path = Some(path.clone());
        } else if node == you {
            you_path = Some(path.clone());
        }

        if let Some(children) = tree.get(&node) {
            for child in children {
                let mut new_path = path.clone();
                new_path.push(child);
                stack.push((*child, new_path));
            }
        }
    }

    let santa_path = santa_path.unwrap();
    let you_path = you_path.unwrap();
    let mut lca = 0;
    for i in 0..santa_path.len() {
        if you_path[i] != santa_path[i] {
            lca = i;
            break;
        }
    }

    let transfers = santa_path.len() + you_path.len() - lca * 2 - 2;

    let elapsed = now.elapsed();
    println!("{} {} {:?}", orbits, transfers, elapsed);
}
