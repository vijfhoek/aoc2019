use std::fs::File;
use std::io::{BufRead, BufReader};
use std::fmt::{Display, Formatter};
use text_io::{try_read, try_scan};

fn get(mem: &Vec<i64>, (mode, value): (i64, i64)) -> Option<i64> {
    match mode {
        0 => mem.get(value as usize).map(|i| *i),
        1 => Some(value),
        _ => panic!("unexpected mode {}", mode),
    }
}

fn get_mut(mem: &mut Vec<i64>, (mode, value): (i64, i64)) -> Option<&mut i64> {
    match mode {
        0 => mem.get_mut(value as usize),
        1 => panic!("can't get immediate as mut"),
        _ => panic!("unexpected mode {}", mode),
    }
}

#[derive(Debug)]
enum Opcode {
    Add,
    Multiply,
    Halt,
}

impl From<i64> for Opcode {
    fn from(item: i64) -> Self {
        match item {
            1 => Opcode::Add,
            2 => Opcode::Multiply,
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

    fn value(&self, memory: &Vec<i64>) -> Option<i64> {
        match self.mode {
            ParameterMode::Position => memory.get(self.value as usize).map(|i| *i),
            ParameterMode::Immediate => Some(self.value),
        }
    }

    fn value_mut<'a>(&self, memory: &'a mut Vec<i64>) -> Option<&'a mut i64> {
        match self.mode {
            ParameterMode::Position => memory.get_mut(self.value as usize),
            ParameterMode::Immediate => panic!("can't get immediate as mut"),
        }
    }
}

impl Display for Parameter {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{:?}:{}", self.mode, self.value)
    }
}

#[derive(Debug)]
struct Instruction {
    pub opcode: Opcode,
    pub parameters: (Parameter, Parameter, Parameter),
}

impl Instruction {
    pub fn fetch(ip: usize, memory: &mut Vec<i64>) -> Option<Self> {
        let instruction = memory.get(ip)?;

        let opcode = Opcode::from(instruction % 100);
        let parameters = (
            Parameter::new(
                ParameterMode::from(instruction / 100 % 10),
                *memory.get(ip + 1).unwrap_or(&0),
            ),
            Parameter::new(
                ParameterMode::from(instruction / 100 % 10),
                *memory.get(ip + 2).unwrap_or(&0),
            ),
            Parameter::new(
                ParameterMode::from(instruction / 100 % 10),
                *memory.get(ip + 3).unwrap_or(&0),
            ),
        );

        Some(Self { opcode, parameters })
    }
}

struct Interpreter {
    pub memory: Vec<i64>,
}

impl Interpreter {
    fn fetch_instruction(&self, ip: usize) -> Option<Instruction> {
        None
    }

    pub fn run(&mut self) {}
}

fn run(mem: &mut Vec<i64>) -> Option<()> {
    let mut ip = 0;
    println!(
        " {:<5}  {:<6}  {:<5}  {:<5}  {:<5}",
        "ip", "opcode", "a", "b", "c"
    );
    println!("========================================");
    loop {
        let instruction = Instruction::fetch(ip, mem)?;
        let (a, b, c) = &instruction.parameters;

        println!(
            " {:<5}  {:?}\t  {}  {}  {}",
            ip,
            instruction.opcode,
            a, b, c,
        );

        ip = match instruction.opcode {
            Opcode::Add => {
                *c.value_mut(mem)? = a.value(mem)? + b.value(mem)?;
                ip + 4
            }

            Opcode::Multiply => {
                *c.value_mut(mem)? = a.value(mem)? * b.value(mem)?;
                ip + 4
            }

            Opcode::Halt => {
                break;
            }

            // 3 => {
            //     // read
            //     *get_mut(mem, a)? = try_read!().ok()?;
            //     ip + 2
            // }

            // 4 => {
            //     // write
            //     println!(">> {}", get(mem, a)?);
            //     ip + 2
            // }

            // 5 => {
            //     // jump-if-true
            //     if get(mem, a)? != 0 {
            //         get(mem, b)? as usize
            //     } else {
            //         ip + 3
            //     }
            // }

            // 6 => {
            //     // jump-if-false
            //     if get(mem, a)? == 0 {
            //         get(mem, b)? as usize
            //     } else {
            //         ip + 3
            //     }
            // }

            // 7 => {
            //     // less than
            //     let result = get(mem, a)? < get(mem, b)?;
            //     *get_mut(mem, c)? = if result { 1 } else { 0 };
            //     ip + 4
            // }

            // 8 => {
            //     // equals
            //     let result = get(mem, a)? == get(mem, b)?;
            //     *get_mut(mem, c)? = if result { 1 } else { 0 };
            //     ip + 4
            // }

            // 99 => {
            //     // halt
            //     break;
            // }

            // _ => {
            //     // unknown opcode
            //     panic!("unknown opcode (instruction={})", instruction)
            // }
        }
    }
    Some(())
}

fn main() -> Result<(), ()> {
    let file = File::open(std::env::args().nth(1).expect("no filename provided")).unwrap();
    let mut input = String::new();
    BufReader::new(file).read_line(&mut input).unwrap();

    let mut mem: Vec<i64> = input
        .split(",")
        .map(|x| x.trim().parse().unwrap())
        .collect();

    run(&mut mem);

    Ok(())
}
