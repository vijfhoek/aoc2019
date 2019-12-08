use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
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
            99 => Opcode::Halt,
            _ => panic!("unknown instruction {}", item),
        }
    }
}

#[derive(Debug)]
enum ParameterMode {
    Position = 0,
    Immediate = 1,
}

impl From<i64> for ParameterMode {
    fn from(item: i64) -> Self {
        match item {
            0 => ParameterMode::Position,
            1 => ParameterMode::Immediate,
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

    fn value(&self, memory: &Vec<i64>) -> i64 {
        match self.mode {
            ParameterMode::Position => memory[self.value as usize],
            ParameterMode::Immediate => self.value,
        }
    }

    fn value_mut<'a>(&self, memory: &'a mut Vec<i64>) -> &'a mut i64 {
        match self.mode {
            ParameterMode::Position => memory.get_mut(self.value as usize).unwrap(),
            ParameterMode::Immediate => panic!("can't get immediate as mut"),
        }
    }
}

impl Display for Parameter {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self.mode {
            ParameterMode::Immediate => write!(formatter, "{}", self.value),
            ParameterMode::Position => write!(formatter, "[{}]", self.value),
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
    pub inputs: VecDeque<i64>,
    pub outputs: VecDeque<i64>,
    pub ip: i64,
}

impl Interpreter {
    fn new(memory: &Vec<i64>) -> Self {
        Self {
            memory: memory.clone(),
            inputs: VecDeque::new(),
            outputs: VecDeque::new(),
            ip: 0,
        }
    }

    fn fetch_instruction(&self, ip: usize) -> Option<Instruction> {
        None
    }

    pub fn step(&mut self) -> bool {
        let instruction = Instruction::fetch(self.ip, &&self.memory).unwrap();
        let (a, b, c) = &instruction.parameters;

        println!(
            " {:<5}  {:?} {}, {}, {}",
            self.ip, instruction.opcode, a, b, c,
        );

        self.ip = match instruction.opcode {
            Opcode::Add => {
                *c.value_mut(&mut self.memory) = a.value(&self.memory) + b.value(&self.memory);
                self.ip + 4
            }

            Opcode::Multiply => {
                *c.value_mut(&mut self.memory) = a.value(&self.memory) * b.value(&self.memory);
                self.ip + 4
            }

            Opcode::Read => {
                *a.value_mut(&mut self.memory) = match self.inputs.pop_front() {
                    Some(input) => input,
                    None => try_read!().unwrap(),
                };

                self.ip + 2
            }

            Opcode::Write => {
                let value = a.value(&self.memory);
                println!(">> {}", value);
                self.outputs.push_back(value);
                self.ip + 2
            }

            Opcode::JumpIfTrue => {
                if a.value(&self.memory) != 0 {
                    b.value(&self.memory)
                } else {
                    self.ip + 3
                }
            }

            Opcode::JumpIfFalse => {
                if a.value(&self.memory) == 0 {
                    b.value(&self.memory)
                } else {
                    self.ip + 3
                }
            }

            Opcode::LessThan => {
                let result = a.value(&self.memory) < b.value(&self.memory);
                *c.value_mut(&mut self.memory) = if result { 1 } else { 0 };
                self.ip + 4
            }

            Opcode::Equals => {
                let result = a.value(&self.memory) == b.value(&self.memory);
                *c.value_mut(&mut self.memory) = if result { 1 } else { 0 };
                self.ip + 4
            }

            Opcode::Halt => {
                return false;
            }
        };

        true
    }

    pub fn run(&mut self) {
        while self.step() {}
    }
}

fn run_cached(phase: i64, value: i64, mem: &Vec<i64>, cache: &mut HashMap<(i64, i64), i64>) -> i64 {
    if let Some(output) = cache.get(&(phase, value)) {
        return *output;
    }

    let mut interpreter = Interpreter::new(mem);
    interpreter.inputs = vec![phase, value].into();
    interpreter.run();

    let output = interpreter.outputs.pop_front().unwrap();
    cache.insert((phase, value), output);
    output
}

fn part1(memory: &Vec<i64>) -> i64 {
    let mut cache: HashMap<(i64, i64), i64> = HashMap::new();
    let mut signals = BinaryHeap::new();
    let from = 0;
    let to = 5;

    for a in from..to {
        let a_out = run_cached(a, 0, &memory, &mut cache);
        for b in from..to {
            if a == b {
                continue;
            }
            let b_out = run_cached(b, a_out, &memory, &mut cache);
            for c in from..to {
                if a == c || b == c {
                    continue;
                }
                let c_out = run_cached(c, b_out, &memory, &mut cache);
                for d in from..to {
                    if d == a || d == b || d == c {
                        continue;
                    }
                    let d_out = run_cached(d, c_out, &memory, &mut cache);
                    for e in from..to {
                        if e == a || e == b || e == c || e == d {
                            continue;
                        }
                        let e_out = run_cached(e, d_out, &memory, &mut cache);
                        signals.push(e_out);
                    }
                }
            }
        }
    }

    *signals.peek().unwrap()
}

fn part2(memory: &Vec<i64>) -> i64 {
    0
}

fn main() {
    let file = File::open(std::env::args().nth(1).expect("no filename provided")).unwrap();
    let mut input = String::new();
    BufReader::new(file).read_line(&mut input).unwrap();

    let mem: Vec<i64> = input
        .split(",")
        .map(|x| x.trim().parse().unwrap())
        .collect();

    dbg!(part1(&mem));
    dbg!(part2(&mem));
}
