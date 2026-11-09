use env::{cartpole::CartPole, Environment, pcgrng::PCGRng};

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
mod tests {
    use super::*;

    #[test]
    fn test_from() {
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
            0b02 => Input::I2,
            0b03 => Input::I3,
            _ => unreachable!("a 2-bit field can only be 0 to 3"),
        }
    }
}

/// 4 Opérations (codées sur 2 bits)
#[derive(Debug, Clone, Copy, PartialEq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

fn main() {
    let seed = 1;
    let mut rng = PCGRng::seed(seed);
    let mut env = CartPole::default();
    let mut total_reward = 0.0;

    loop {
        let action = (rng.next_u32() % 4) as f32;
        let (_, reward, done) = env.step(action, &mut rng);

        total_reward += reward;
        if done {
            break;
        }
    }

    println!("Total reward: {}", total_reward)
}
