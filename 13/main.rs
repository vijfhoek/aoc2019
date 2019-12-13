use std::collections::HashSet;
use std::convert::{From, TryFrom};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;
use text_io::{try_read, try_scan};

#[derive(Debug, Clone, Eq, PartialEq)]
enum Tile {
    Empty,
    Wall,
    Block,
    HorizontalPaddle,
    Ball,
}
impl Tile {
    fn new() -> Self {
        Tile::Empty
    }
}
impl From<i64> for Tile {
    fn from(value: i64) -> Self {
        match value {
            0 => Tile::Empty,
            1 => Tile::Wall,
            2 => Tile::Block,
            3 => Tile::HorizontalPaddle,
            4 => Tile::Ball,
            _ => panic!("unknown tile {}", value),
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
                        while rx.try_recv().is_ok() {}
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

fn part1(memory: &Vec<i64>) -> usize {
    let mut interpreter = Interpreter::new(memory);
    let (tx_input, rx_input) = channel();
    let (tx_output, rx_output) = channel();
    interpreter.rx = Some(rx_input);
    interpreter.tx = Some(tx_output);

    let mut map = vec![vec![Tile::Empty; 50]; 26];
    let mut score = 0;
    let mut x = None;
    let mut y = None;

    tx_input.send(1).unwrap();
    while interpreter.step() {
        if let Ok(out) = rx_output.try_recv() {
            if x.is_none() {
                x = Some(out);
            } else if y.is_none() {
                y = Some(out);
            } else if x == Some(-1) && y == Some(0) {
                score = out;
            } else {
                map[y.unwrap() as usize][x.unwrap() as usize] = Tile::from(out);
                x = None;
                y = None;
            }
        }
    }

    draw_map(&map, score);

    map.iter()
        .map(|row| row.iter().filter(|tile| **tile == Tile::Block).count())
        .sum()
}

fn draw_map(map: &Vec<Vec<Tile>>, score: i64) {
    print!("\x1B[1;1H");
    for row in map {
        for tile in row {
            print!(
                "{}",
                match tile {
                    Tile::Empty => "  ",
                    Tile::Ball => "⬤ ",
                    Tile::Block => "█▉",
                    Tile::HorizontalPaddle => "▀▀",
                    Tile::Wall => "██",
                }
            )
        }
        println!();
    }
    println!("score: {}", score);
}

fn part2(memory: &Vec<i64>) -> i64 {
    println!("\x1B[3J\x1Bc");
    let mut memory = memory.clone();
    memory[0] = 2;

    let mut interpreter = Interpreter::new(&memory);
    let (tx_input, rx_input) = channel();
    let (tx_output, rx_output) = channel();
    interpreter.rx = Some(rx_input);
    interpreter.tx = Some(tx_output);

    let mut map = vec![vec![Tile::Empty; 50]; 26];
    let mut paddle = 0;
    let mut ball = None;
    let mut score = 0;
    let mut x = None;
    let mut y = None;

    tx_input.send(1).unwrap();
    while interpreter.step() {
        if let Ok(out) = rx_output.try_recv() {
            if x.is_none() {
                x = Some(out);
            } else if y.is_none() {
                y = Some(out);
            } else if x == Some(-1) && y == Some(0) {
                score = out;
                x = None;
                y = None;
            } else {
                let x_ = x.unwrap() as usize;
                let y_ = y.unwrap() as usize;
                let tile = Tile::from(out);
                if tile == Tile::HorizontalPaddle {
                    paddle = x_;
                } else if tile == Tile::Ball {
                    ball = Some(x_);
                }

                map[y_][x_] = tile;

                x = None;
                y = None;
            }

            if let Some(ball_) = ball {
                draw_map(&map, score);

                if paddle > ball_ {
                    tx_input.send(-1).unwrap();
                } else if paddle == ball_ {
                    tx_input.send(0).unwrap();
                } else if paddle < ball_ {
                    tx_input.send(1).unwrap();
                }

                ball = None;
            }
        }
    }

    score
}

fn main() {
    let file = File::open(std::env::args().nth(1).expect("no filename provided")).unwrap();
    let mut input = String::new();
    BufReader::new(file).read_line(&mut input).unwrap();

    let memory: Vec<i64> = input
        .split(",")
        .map(|x| x.trim().parse().unwrap())
        .collect();

    let part1 = part1(&memory);
    let part2 = part2(&memory);
    dbg!(part1, part2);
}
