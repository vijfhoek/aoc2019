use std::io::BufRead;
use std::collections::HashMap;

fn main() {
    let stdin = std::io::stdin();
    let lines = stdin.lock().lines();
    let mut tree = HashMap::new();
    for line in lines {
        let line = line.unwrap();
        let mut parts = line.trim().split(')');
        let from = String::from(parts.next().unwrap());
        let to = String::from(parts.next().unwrap());
        
        let entry = tree.entry(from).or_insert_with(Vec::new);
        entry.push(to);
    }

    dbg!(&tree);

    let mut orbits = 0;
    let mut santa_path = None;
    let mut you_path = None;

    let mut stack = vec![(String::from("COM"), vec![])];
    while !stack.is_empty() {
        let (node, path) = stack.pop().unwrap();
        orbits += path.len();

        if node == "SAN" {
            santa_path = Some(path.clone());
        }
        if node == "YOU" {
            you_path = Some(path.clone());
        }

        if let Some(children) = tree.get(&node) {
            for child in children {
                let mut new_path = path.clone();
                new_path.push(child);
                stack.push((String::from(child), new_path));
            }
        }
    }

    dbg!(orbits);

    let santa = santa_path.unwrap();
    let you = you_path.unwrap();
    for (santa_depth, santa_node) in santa.iter().rev().enumerate() {
        if let Some(you_depth) = you.iter().rev().position(|you_node| you_node == santa_node) {
            dbg!(santa_depth + you_depth - 2);
            break;
        }
    }
}
