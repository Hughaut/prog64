use env::{cartpole::CartPole, Environment, pcgrng::PCGRng};

/// Bits par instruction : 4 champs de 2 bits
const BITS_PER_LINE: u8 = 8;
/// Nombre d'instructions du programme
const NB_LINES: usize = 8;
/// Bits par champ (4 valeurs possibles : R0-R3, I0-I3, 4 ops).
const FIELD_BITS: u8 = 2;

/// Positions de départ de chaque champ dans l'octet d'instruction.
/// bits 0-1 : registre destination
const DEST_START: u8 = 0;
/// bits 2-3 : registre source
const REG_START: u8 = 2;
/// bits 4-5 : index d'entrée
const INPUT_START: u8 = 4;
/// bits 6-7 : opération
const OP_START: u8 = 6;

/// 4 registres (codés sur 2 bits)
#[derive(Debug, Clone, Copy, PartialEq)]
enum Reg {
    R0,
    R1,
    R2,
    R3,
}

impl From<u8> for Reg {
    fn from(value: u8) -> Self{
        match value {
            0b00 => Reg::R0,
            0b01 => Reg::R1,
            0b10 => Reg::R2,
            0b11 => Reg::R3,
            _ => unreachable!("a 2-bit field can only be 0 to 3"),
        }
    }
}

#[cfg(test)]
mod tests_reg {
    use super::*;

    #[test]
    fn test_from_reg() {
        assert_eq!(Reg::R0, Reg::from(0));
        assert_eq!(Reg::R1, Reg::from(1));
        assert_eq!(Reg::R2, Reg::from(2));
        assert_eq!(Reg::R3, Reg::from(3));
    }
}

/// 4 entrées (codées sur 2 bits)
#[derive(Debug, Clone, Copy, PartialEq)]
enum Input {
    I0,
    I1,
    I2,
    I3,
}

impl From<u8> for Input {
    fn from(value: u8) -> Self{
        match value {
            0b00 => Input::I0,
            0b01 => Input::I1,
            0b10 => Input::I2,
            0b11 => Input::I3,
            _ => unreachable!("a 2-bit field can only be 0 to 3"),
        }
    }
}

#[cfg(test)]
mod tests_input {
    use super::*;

    #[test]
    fn test_from_input() {
        assert_eq!(Input::I0, Input::from(0));
        assert_eq!(Input::I1, Input::from(1));
        assert_eq!(Input::I2, Input::from(2));
        assert_eq!(Input::I3, Input::from(3));
    }
}

/// 4 opérations (codées sur 2 bits)
#[derive(Debug, Clone, Copy, PartialEq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl From<u8> for Op {
    fn from(value: u8) -> Self{
        match value {
            0b00 => Op::Add,
            0b01 => Op::Sub,   
            0b10 => Op::Mul,
            0b11 => Op::Div,
            _ => unreachable!("a 2-bit field can only be 0 to 3"),
        }
    }
}

#[cfg(test)]
mod tests_op {
    use super::*;

    #[test]
    fn test_from_op() {
        assert_eq!(Op::Add, Op::from(0));
        assert_eq!(Op::Sub, Op::from(1));
        assert_eq!(Op::Mul, Op::from(2));
        assert_eq!(Op::Div, Op::from(3));
    }
}

/// Fonction d'extraction de bits d'un entier non signé u64 à partir d'une position de départ et d'une longueur spécifiée.
fn extract_bits(value: u64, start: u8, length: u8) -> Option<u8> {
    if start >= 64 || length == 0 || length > 8 || length > 64 - start {
        return None;
    }
    Some(((value >> start) & ((1u64 << length) - 1)) as u8)
}

#[cfg(test)]
mod tests_extract_bits {
    use super::*;

    #[test]
    fn test_extract_bits() {
        assert_eq!(extract_bits(0b1011_0110, 0, 4), Some(0b0110));
        assert_eq!(extract_bits(0xDEAD_BEEF, 16, 8), Some(0xAD)); // octet 2
        assert_eq!(extract_bits(0xDEAD_BEEF, 24, 8), Some(0xDE)); // octet 3 (poids fort)
        assert_eq!(extract_bits(0xDEAD_BEEF, 0, 8),  Some(0xEF)); // octet 0 (poids faible)

        // cas u64 : octet de poids fort d'un grand nombre
        assert_eq!(extract_bits(0x0123_4567_89AB_CDEF, 56, 8), Some(0x01));
        assert_eq!(extract_bits(0x0123_4567_89AB_CDEF, 0, 8),  Some(0xEF));
        assert_eq!(extract_bits(0xFFFF_FFFF_FFFF_FFFF, 56, 8), Some(0xFF));

        // cas limites
        assert_eq!(extract_bits(0x1, 0, 9),   None); // > 8 bits, ne tient pas dans un u8
        assert_eq!(extract_bits(0x1, 64, 1),  None); // start hors bornes
        assert_eq!(extract_bits(0x1, 62, 3),  None); // déborde du u64
        assert_eq!(extract_bits(0x1, 0, 0),   None); // longueur nulle
        assert_eq!(extract_bits(0x1, 63, 1),  Some(0)); // fenêtre valide au bord
    }
}

/// Une instruction avec la logique suivante :
/// Reg[dest] = Reg[reg] op Input[input]
#[derive(Debug, Clone, Copy, PartialEq)]
struct Instruction {
    dest: Reg,
    reg: Reg,
    input: Input,
    op: Op,
}

/// Fonction de décodage d'une ligne d'instruction à partir d'un programme
/// Ex si line = 0, on décode les bits 0 à 7 du programme
/// Ex si line = 1, on décode les bits 8 à 15 du programme, etc.
fn decode_instruction(program: u64, line: usize) -> Instruction {
    debug_assert!(line < NB_LINES, "line doit être < NB_LINES");

    let base = (line * BITS_PER_LINE as usize) as u8;

    let dest: Reg = Reg::from(
        extract_bits(program, base + DEST_START, FIELD_BITS)
            .expect("champ dest toujours valide : line < NB_LINES"),
    );

    let reg: Reg = Reg::from(
        extract_bits(program, base + REG_START, FIELD_BITS)
            .expect("champ reg toujours valide : line < NB_LINES"),
    );
    
    let input: Input = Input::from(
        extract_bits(program, base + INPUT_START, FIELD_BITS)
            .expect("champ input toujours valide : line < NB_LINES"),
    );

    let op: Op = Op::from(
        extract_bits(program, base + OP_START, FIELD_BITS)
            .expect("champ op toujours valide : line < NB_LINES"),
    );

    Instruction { dest, reg, input, op }
}

#[cfg(test)]
mod tests_decode_instruction {
    use super::*;

    #[test]
    fn test_multiple_lines() {
        let program = 0b11_10_01_00u64 | (0b00_01_10_11u64 << 8) | (0b01_00_11_10u64 << 16) | (0b10_11_00_01u64 << 24) 
            | (0b00_00_00_00u64 << 32) | (0b11_11_11_11u64 << 40) | (0b01_01_01_01u64 << 48) | (0b10_10_10_10u64 << 56);

        assert_eq!(decode_instruction(program, 0), Instruction { dest: Reg::R0, reg: Reg::R1, input: Input::I2, op: Op::Div });

        assert_eq!(decode_instruction(program, 1), Instruction { dest: Reg::R3, reg: Reg::R2, input: Input::I1, op: Op::Add });

        assert_eq!(decode_instruction(program, 2), Instruction { dest: Reg::R2, reg: Reg::R3, input: Input::I0, op: Op::Sub });

        assert_eq!(decode_instruction(program, 3), Instruction { dest: Reg::R1, reg: Reg::R0, input: Input::I3, op: Op::Mul });

        assert_eq!(decode_instruction(program, 4), Instruction { dest: Reg::R0, reg: Reg::R0, input: Input::I0, op: Op::Add });

        assert_eq!(decode_instruction(program, 5), Instruction { dest: Reg::R3, reg: Reg::R3, input: Input::I3, op: Op::Div });

        assert_eq!(decode_instruction(program, 6), Instruction { dest: Reg::R1, reg: Reg::R1, input: Input::I1, op: Op::Sub });

        assert_eq!(decode_instruction(program, 7), Instruction { dest: Reg::R2, reg: Reg::R2, input: Input::I2, op: Op::Mul });
    }

    #[test]
    fn test_decode_given_zero_lines() {
        for line in 0..NB_LINES {
            let instr = decode_instruction(0, line);
            assert_eq!(instr.op, Op::Add);
            assert_eq!(instr.input, Input::I0);
            assert_eq!(instr.reg, Reg::R0);
            assert_eq!(instr.dest, Reg::R0);
        }
    }
}


/// Fonction d'application d'une operation en fonction de l'entrée et de 2 variables a et b. Retourne le résultat de l'opération.
/// Si l'opération est une division et que b est égal à 0, retourne 0 pour éviter la division par zéro.
/// Si le résultat de l'opération n'est pas un nombre fini (NaN ou infini), retourne 0.
fn apply_operation(op: Op, a: f32, b: f32) -> f32 {
    let result = match op {
        Op::Add => a + b,
        Op::Sub => a - b,
        Op::Mul => a * b,
        Op::Div => {
            if b == 0.0 {
                0.0
            } else {
                a / b
            }
        }
    };
    if result.is_finite() {
        result
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests_apply_operation {
    use super::*;

    #[test]
    fn test_apply_operation() {
        assert_eq!(apply_operation(Op::Add, 2.0, 3.0), 5.0);
        assert_eq!(apply_operation(Op::Sub, 5.0, 3.0), 2.0);
        assert_eq!(apply_operation(Op::Mul, 2.0, 3.0), 6.0);
        assert_eq!(apply_operation(Op::Div, 6.0, 3.0), 2.0);
        assert_eq!(apply_operation(Op::Div, 6.0, 0.0), 0.0); // division par zéro
    }
}

/// Structure représentant un programme composé de 8 instructions codées sur 64 bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Program (u64);

impl Program {

    /// Exécute le programme sur les entrées données et retourne la valeur du registre R0 après l'exécution.
    /// Initialisation des registres à 0.0.
    /// Chaque instruction est exécutée dans l'ordre
    /// On lit une ligne d'instruction, on applique l'opération sur le registre source et l'entrée correspondante, et on stocke le résultat dans le registre destination.
    /// R0 est retrouné à la fin de l'exécution du programme.
    fn run(&self, inputs: [f32; 4]) -> f32 {
        let mut registers: [f32; 4] = [0.0; 4];

        for line in 0..NB_LINES {
            let instruction = decode_instruction(self.0, line);
            let res = apply_operation(instruction.op, registers[instruction.reg as usize], inputs[instruction.input as usize]);
            registers[instruction.dest as usize] = res;
        }

        registers[0]
    }
}

#[cfg(test)]
mod tests_program {
    use super::*;

    #[test]
    fn test_run_program() {
        let program = Program(0b00_00_00_00u64 | (0b01_00_00_00u64 << 8) | (0b10_01_00_00u64 << 16) | (0b11_10_00_00u64 << 24)
            | (0b00_11_00_00u64 << 32) | (0b01_01_00_00u64 << 40) | (0b10_10_00_00u64 << 48) | (0b11_11_00_00u64 << 56));

        let inputs = [1.0, 2.0, 3.0, 4.0];
        let result = program.run(inputs);
        assert_eq!(result, 1.5);
    }
}

/// Fonction d'évaluation d'un programme
/// Elle joue un épisode complet et retourne le nombre de pas effectués
/// On utilise env.resest pour initialiser l'environnement aléatoirement
/// Puis on exectute le programme jusqu'à ce que donne = true
fn evaluate(program: &Program, env: &mut CartPole, rng: &mut PCGRng) -> env::Score {
    let mut total_reward = 0.0;
    let mut done = false;

    let mut state = env.reset(rng);

    while !done {
        let inputs = state;
        let action = program.run(inputs);
        let reward;
        (state, reward, done) = env.step(action, rng);
        total_reward += reward;
    }

    total_reward
}

fn main() {
    let seed = 1;
    let mut rng = PCGRng::seed(seed);
    let mut env = CartPole::default();

    let program = Program(0);

    let score = evaluate(&program, &mut env, &mut rng);
    println!("Score du programme : {}", score);
    /*let mut total_reward = 0.0;

    loop {
        let action = (rng.next_u32() % 4) as f32;
        let (_, reward, done) = env.step(action, &mut rng);

        total_reward += reward;
        if done {
            break;
        }
    }

    println!("Total reward: {}", total_reward)*/
}
