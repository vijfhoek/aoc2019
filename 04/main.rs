#![feature(const_fn)]

fn filter(p: &u32) -> bool {
    let string = p.to_string();
    let bytes = string.as_bytes();
    let mut double = 0;
    let mut prev_double = 0;

    for i in 0..5 {
        if bytes[i + 1] < bytes[i] {
            return false;
        }

        if bytes[i] == bytes[i + 1] {
            if double == bytes[i] {
                prev_double = double;
                double = 0;
            } else if prev_double != bytes[i] && double == 0 {
                double = bytes[i]
            }
        }
    }

    double != 0
}

fn main() {
    let from = 100000;
    let to = 999999;

    let possibilities = (from..to).filter(filter).count();
    dbg!(possibilities);
}
