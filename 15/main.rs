use std::collections::{HashSet, VecDeque};
use std::convert::{From, TryFrom};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc::{channel, Receiver, Sender};
use text_io::{try_read, try_scan};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Movement {
    North,
    South,
    West,
    East,
}
impl Into<i64> for &Movement {
    fn into(self) -> i64 {
        match self {
            Movement::North => 1,
            Movement::South => 2,
            Movement::West => 3,
            Movement::East => 4,
        }
    }
}
impl Movement {
    fn reverse(&self) -> Self {
        match self {
            Movement::North => Movement::South,
            Movement::South => Movement::North,
            Movement::West => Movement::East,
            Movement::East => Movement::West,
        }
    }

    fn coords(&self) -> (i64, i64) {
        match self {
            Movement::North => (0, 1),
            Movement::South => (0, -1),
            Movement::West => (-1, 0),
            Movement::East => (1, 0),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Status {
    HitWall,
    Moved,
    Found,
}
impl From<i64> for Status {
    fn from(value: i64) -> Self {
        match value {
            0 => Status::HitWall,
            1 => Status::Moved,
            2 => Status::Found,
            _ => panic!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Opcode {
    Add,
    Multiply,
    Read,
    Write,
    JumpIfTrue,
    JumpIfFalse,
    LessThan,
    Equals,
    RelativeBase,
    Halt,
}

impl From<i64> for Opcode {
    fn from(item: i64) -> Self {
        match item {
            1 => Opcode::Add,
            2 => Opcode::Multiply,
            3 => Opcode::Read,
            4 => Opcode::Write,
            5 => Opcode::JumpIfTrue,
            6 => Opcode::JumpIfFalse,
            7 => Opcode::LessThan,
            8 => Opcode::Equals,
            9 => Opcode::RelativeBase,
            99 => Opcode::Halt,
            _ => panic!("unknown instruction {}", item),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ParameterMode {
    Position,
    Immediate,
    Relative,
}

impl From<i64> for ParameterMode {
    fn from(item: i64) -> Self {
        match item {
            0 => ParameterMode::Position,
            1 => ParameterMode::Immediate,
            2 => ParameterMode::Relative,
            _ => panic!("unknown parameter mode {}", item),
        }
    }
}

#[derive(Debug)]
struct Parameter {
    pub mode: ParameterMode,
    pub value: i64,
}

impl Parameter {
    fn new(mode: ParameterMode, value: i64) -> Self {
        Self { mode, value }
    }
}

impl Display for Parameter {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self.mode {
            ParameterMode::Immediate => write!(formatter, "{}", self.value),
            ParameterMode::Position => write!(formatter, "[{}]", self.value),
            ParameterMode::Relative => write!(formatter, "rel[{}]", self.value),
        }
    }
}

#[derive(Debug)]
struct Instruction {
    pub opcode: Opcode,
    pub parameters: (Parameter, Parameter, Parameter),
}

impl Instruction {
    pub fn fetch(ip: i64, memory: &Vec<i64>) -> Option<Self> {
        let ip = ip as usize;
        let instruction = memory.get(ip)?;

        let opcode = Opcode::from(instruction % 100);
        let parameters = (
            Parameter::new(
                ParameterMode::from(instruction / 100 % 10),
                *memory.get(ip + 1).unwrap_or(&0),
            ),
            Parameter::new(
                ParameterMode::from(instruction / 1000 % 10),
                *memory.get(ip + 2).unwrap_or(&0),
            ),
            Parameter::new(
                ParameterMode::from(instruction / 10000 % 10),
                *memory.get(ip + 3).unwrap_or(&0),
            ),
        );

        Some(Self { opcode, parameters })
    }
}

struct Interpreter {
    pub memory: Vec<i64>,
    pub rx: Option<Receiver<i64>>,
    pub tx: Option<Sender<i64>>,
    pub last_output: Option<i64>,
    pub ip: i64,
    pub relative_base: i64,
    pub debug: bool,
}

impl Interpreter {
    fn new(memory: &Vec<i64>) -> Self {
        Self {
            memory: memory.clone(),
            rx: None,
            tx: None,
            last_output: None,
            ip: 0,
            relative_base: 0,
            debug: false,
        }
    }

    fn reset(&mut self, memory: &Vec<i64>) {
        self.memory = memory.clone();
        self.ip = 0;
        self.relative_base = 0;
    }

    pub fn step(&mut self) -> bool {
        let instruction = Instruction::fetch(self.ip, &&self.memory).unwrap();
        let (a, b, c) = &instruction.parameters;

        if self.debug {
            let args = format!("{:?} {}, {}, {}", instruction.opcode, a, b, c);
            print!(
                "ip={:<5} rb={:<5} | {:<30} | {:>5} -> ?        ",
                self.ip,
                self.relative_base,
                args,
                self.memory.len()
            );
            std::io::stdout().flush().unwrap();
        }

        let (ip, arg_count) = match instruction.opcode {
            Opcode::Add => {
                *self.value_mut(&c) = self.value(&a) + self.value(&b);
                (self.ip + 4, 3)
            }

            Opcode::Multiply => {
                *self.value_mut(&c) = self.value(&a) * self.value(&b);
                (self.ip + 4, 3)
            }

            Opcode::Read => {
                *self.value_mut(&a) = match &self.rx {
                    Some(rx) => {
                        let input = rx.recv().unwrap();
                        if self.debug {
                            print!(">> {}", input);
                        }
                        input
                    }
                    None => {
                        print!(">> ");
                        std::io::stdout().flush().unwrap();
                        try_read!().unwrap()
                    }
                };
                (self.ip + 2, 1)
            }

            Opcode::Write => {
                let value = self.value(&a);
                self.last_output = Some(value);
                match &self.tx {
                    Some(tx) => {
                        let _ = tx.send(value);
                    }
                    None => print!("<< {}", value),
                }
                (self.ip + 2, 1)
            }

            Opcode::JumpIfTrue => (
                if self.value(&a) != 0 {
                    self.value(&b)
                } else {
                    self.ip + 3
                },
                2,
            ),

            Opcode::JumpIfFalse => (
                if self.value(&a) == 0 {
                    self.value(&b)
                } else {
                    self.ip + 3
                },
                2,
            ),

            Opcode::LessThan => {
                let result = self.value(&a) < self.value(&b);
                *self.value_mut(&c) = if result { 1 } else { 0 };
                (self.ip + 4, 3)
            }

            Opcode::Equals => {
                let result = self.value(&a) == self.value(&b);
                *self.value_mut(&c) = if result { 1 } else { 0 };
                (self.ip + 4, 3)
            }

            Opcode::RelativeBase => {
                self.relative_base += self.value(&a);
                (self.ip + 2, 1)
            }

            Opcode::Halt => {
                return false;
            }
        };

        if self.debug {
            let args = match arg_count {
                1 => format!("{:?} {}", instruction.opcode, a),
                2 => format!("{:?} {}, {}", instruction.opcode, a, b),
                3 => format!("{:?} {}, {}, {}", instruction.opcode, a, b, c),
                _ => panic!(),
            };

            if instruction.opcode == Opcode::Read {
                print!("\x1B[1A");
            }

            println!(
                "\rip=\x1B[5C rb=\x1B[5C | {:<30} | \x1B[5C -> {:?}",
                args,
                self.memory.len()
            );
        }

        self.ip = ip;

        true
    }

    pub fn run(&mut self) {
        while self.step() {}
    }

    fn value(&self, parameter: &Parameter) -> i64 {
        let index = match parameter.mode {
            ParameterMode::Position => parameter.value as usize,
            ParameterMode::Relative => (parameter.value + self.relative_base) as usize,
            ParameterMode::Immediate => {
                return parameter.value;
            }
        };

        *self.memory.get(index).unwrap_or(&0)
    }

    fn value_mut<'a>(&'a mut self, parameter: &Parameter) -> &'a mut i64 {
        let index = usize::try_from(match parameter.mode {
            ParameterMode::Position => parameter.value,
            ParameterMode::Relative => (parameter.value + self.relative_base),
            ParameterMode::Immediate => panic!("can't get immediate as mut"),
        })
        .unwrap();

        if index >= self.memory.len() {
            if self.debug {
                print!("resizing memory to {}\r", index + 1);
                std::io::stdout().flush().unwrap();
            }
            self.memory.resize(index + 1, 0);
        }

        self.memory.get_mut(index).unwrap()
    }
}

fn draw_map(
    map: &[Vec<bool>],
    seen: &HashSet<(i64, i64)>,
    dx: i64,
    dy: i64,
    oxygen: Option<(i64, i64)>,
) {
    print!("\x1B[1;1H");
    for (y, row) in map.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            let x = x as i64 - 25;
            let y = y as i64 - 25;
            if *tile {
                print!("██")
            } else if oxygen == Some((x, y)) {
                print!("⛳")
            } else if dx == x && dy == y {
                print!("🦀")
            } else if x == 0 && y == 0 {
                print!("🚦")
            } else if seen.contains(&(x, y)) {
                print!("  ")
            } else {
                print!("▒▒")
            }
        }
        println!();
    }
}

fn part1(memory: &Vec<i64>) -> (usize, Vec<Vec<bool>>, (i64, i64)) {
    let mut interpreter = Interpreter::new(memory);
    let (tx_input, rx_input) = channel();
    let (tx_output, rx_output) = channel();
    interpreter.rx = Some(rx_input);
    interpreter.tx = Some(tx_output);

    let mut map = vec![vec![false; 50]; 50];

    let mut stack: VecDeque<(_, (i64, i64))> = VecDeque::from(vec![
        (vec![Movement::North], (0, -1)),
        (vec![Movement::South], (0, 1)),
        (vec![Movement::West], (-1, 0)),
        (vec![Movement::East], (1, 0)),
    ]);

    let mut seen = HashSet::new();
    let mut queue = VecDeque::from(vec![Movement::North]);
    let (mut path, (mut x, mut y)) = stack.pop_front().unwrap();

    let mut oxygen = None;
    let mut distance = None;

    let movement = &queue.pop_front().unwrap();
    tx_input.send(movement.into()).unwrap();
    while interpreter.step() {
        if let Ok(out) = rx_output.try_recv() {
            let status = Status::from(out);
            if queue.is_empty() {
                // draw_map(&map, &seen, x, y, oxygen);
                match &status {
                    Status::HitWall => {
                        let y_ = (y + 25) as usize;
                        let x_ = (x + 25) as usize;
                        map[y_][x_] = true;
                    }
                    Status::Moved => {
                        let new = [path.clone(), vec![Movement::North]].concat();
                        stack.push_back((new, (x, y - 1)));

                        let new = [path.clone(), vec![Movement::South]].concat();
                        stack.push_back((new, (x, y + 1)));

                        let new = [path.clone(), vec![Movement::West]].concat();
                        stack.push_back((new, (x - 1, y)));

                        let new = [path.clone(), vec![Movement::East]].concat();
                        stack.push_back((new, (x + 1, y)));
                    }
                    Status::Found => {
                        if distance.is_none() {
                            oxygen = Some((x, y));
                            distance = Some(path.len());
                        }
                    }
                }

                while queue.is_empty() {
                    if let Some((path_, (x_, y_))) = stack.pop_front() {
                        path = path_;
                        x = x_;
                        y = y_;

                        if seen.insert((x, y)) {
                            interpreter.reset(memory);
                            queue.extend(&path);
                        }
                    } else {
                        return (distance.unwrap(), map, oxygen.unwrap());
                    }
                }
            }

            let movement = &queue.pop_front().unwrap();
            tx_input.send(movement.into()).unwrap();
        }
    }

    panic!();
}

fn part2(map: &Vec<Vec<bool>>, (x, y): (i64, i64)) -> i64 {
    let x = (x + 25) as usize;
    let y = (y + 25) as usize;
    let mut stack = vec![((x, y), 0)];
    let mut seen = HashSet::new();
    let mut max = 0;
    while !stack.is_empty() {
        let ((x, y), length) = stack.pop().unwrap();
        if !seen.insert((x, y)) || map[y][x] {
            continue;
        }
        if length > max {
            max = length;
        }

        stack.push(((x, y - 1), length + 1));
        stack.push(((x, y + 1), length + 1));
        stack.push(((x - 1, y), length + 1));
        stack.push(((x + 1, y), length + 1));
    }

    max
}

fn main() {
    let file = File::open(std::env::args().nth(1).expect("no filename provided")).unwrap();
    let mut input = String::new();
    BufReader::new(file).read_line(&mut input).unwrap();

    let memory: Vec<i64> = input
        .split(',')
        .map(|x| x.trim().parse().unwrap())
        .collect();

    let (part1, map, oxygen) = part1(&memory);
    dbg!(part1);
    let part2 = part2(&map, oxygen);
    dbg!(part2);
}
