use std::time::Instant;

fn filter(mut p: u32) -> bool {
    let mut double = 0;
    let mut prev_double = 0;

    for i in 0..5 {
        let b = p % 10;
        let a = p / 10 % 10;
        if a > b {
            return false;
        }

        if a == b {
            if double == a {
                prev_double = double;
                double = 0;
            } else if prev_double != a && double == 0 {
                double = a
            }
        }
        p = p / 10;
    }

    double != 0
}

fn main() {
    let now = Instant::now();

    let from = 100000;
    let to = 999999;
    let mut count = 0;

    for i in from..to {
        if filter(i) {
            count += 1
        }
    }

    let elapsed = now.elapsed();
    println!("{} {:?}", count, elapsed);
}
