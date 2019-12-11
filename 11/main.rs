use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;
use text_io::{try_read, try_scan};

#[derive(Debug)]
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

impl From<i128> for Opcode {
    fn from(item: i128) -> Self {
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

impl From<i128> for ParameterMode {
    fn from(item: i128) -> Self {
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
    pub value: i128,
}

impl Parameter {
    fn new(mode: ParameterMode, value: i128) -> Self {
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
    pub fn fetch(ip: u128, memory: &HashMap<u128, i128>) -> Option<Self> {
        let instruction = memory.get(&ip)?;

        let opcode = Opcode::from(instruction % 100);
        let parameters = (
            Parameter::new(
                ParameterMode::from(instruction / 100 % 10),
                *memory.get(&(ip + 1)).unwrap_or(&0),
            ),
            Parameter::new(
                ParameterMode::from(instruction / 1000 % 10),
                *memory.get(&(ip + 2)).unwrap_or(&0),
            ),
            Parameter::new(
                ParameterMode::from(instruction / 10000 % 10),
                *memory.get(&(ip + 3)).unwrap_or(&0),
            ),
        );

        Some(Self { opcode, parameters })
    }
}

struct Interpreter {
    pub memory: HashMap<u128, i128>,
    pub rx: Option<Receiver<i128>>,
    pub tx: Option<Sender<i128>>,
    pub last_output: Option<i128>,
    pub ip: u128,
    pub relative_base: i128,
    pub debug: bool,
}

impl Interpreter {
    fn new(memory: &HashMap<u128, i128>) -> Self {
        Self {
            memory: memory.clone(),
            rx: None,
            tx: None,
            last_output: None,
            ip: 0,
            relative_base: 0,
            debug: true,
        }
    }

    pub fn step(&mut self) -> bool {
        let instruction = Instruction::fetch(self.ip, &&self.memory).unwrap();
        let (a, b, c) = &instruction.parameters;

        // if self.debug {
        //     print!("{:?} {}, {}, {}", instruction.opcode, a, b, c);
        //     std::io::stdout().flush().unwrap();
        // }

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
                *self.value_mut(&c) = match &self.rx {
                    Some(rx) => rx.recv().unwrap(),
                    None => try_read!().unwrap(),
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
                    None => println!("> {}", value),
                }
                (self.ip + 2, 1)
            }

            Opcode::JumpIfTrue => (
                if self.value(&a) != 0 {
                    self.value(&b) as u128
                } else {
                    self.ip + 3
                },
                2,
            ),

            Opcode::JumpIfFalse => (
                if self.value(&a) == 0 {
                    self.value(&b) as u128
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
        self.ip = ip;

        if self.debug {
            let args = match arg_count {
                1 => format!("{:?} {}", instruction.opcode, a),
                2 => format!("{:?} {}, {}", instruction.opcode, a, b),
                3 => format!("{:?} {}, {}, {}", instruction.opcode, a, b, c),
                _ => panic!(),
            };

            println!(
                "\rip={:<5} rb={:<5} | {:<30} | {:?}",
                ip,
                self.relative_base,
                args,
                self.memory.iter().max()
            );
        }

        true
    }

    pub fn run(&mut self) {
        while self.step() {}
    }

    fn value(&self, parameter: &Parameter) -> i128 {
        let index = match parameter.mode {
            ParameterMode::Position => parameter.value as u128,
            ParameterMode::Relative => (parameter.value + self.relative_base) as u128,
            ParameterMode::Immediate => {
                return parameter.value;
            }
        };

        *self.memory.get(&index).unwrap_or(&0)
    }

    fn value_mut<'a>(&'a mut self, parameter: &Parameter) -> &'a mut i128 {
        let index = match parameter.mode {
            ParameterMode::Position => parameter.value as u128,
            ParameterMode::Relative => (parameter.value + self.relative_base) as u128,
            ParameterMode::Immediate => panic!("can't get immediate as mut"),
        };

        self.memory.entry(index).or_insert(0)
    }
}

fn part1(memory: &HashMap<u128, i128>) -> i128 {
    let mut interpreter = Interpreter::new(memory);
    while interpreter.step() {}
    interpreter.last_output.unwrap()
}

fn main() {
    let file = File::open(std::env::args().nth(1).expect("no filename provided")).unwrap();
    let mut input = String::new();
    BufReader::new(file).read_line(&mut input).unwrap();

    let memory: HashMap<u128, i128> = input
        .split(",")
        .enumerate()
        .map(|(i, x)| (i as u128, x.trim().parse().unwrap()))
        .collect();

    let now = Instant::now();
    dbg!(part1(&memory));
    dbg!(now.elapsed());
}
