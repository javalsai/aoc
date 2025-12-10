#![feature(slice_split_once, exact_div)]

use std::fmt;

use crate::eq::Equation;

pub mod eq {
    use std::{
        fmt,
        // ops::{AddAssign, MulAssign},
    };

    #[derive(Clone)]
    pub struct Equation {
        parameters: Box<[i16]>,
        term: i16,
    }

    // impl MulAssign<i16> for Equation {
    //     fn mul_assign(&mut self, rhs: i16) {
    //         self.parameters.iter_mut().for_each(|param| *param *= rhs);
    //         self.term *= rhs;
    //     }
    // }

    // impl AddAssign for Equation {
    //     fn add_assign(&mut self, rhs: Self) {
    //         assert_eq!(self.degree(), rhs.degree());

    //         self.parameters
    //             .iter_mut()
    //             .zip(rhs.parameters.iter())
    //             .for_each(|(me, other)| *me += other);
    //         self.term += rhs.term;
    //     }
    // }

    impl From<(Box<[i16]>, i16)> for Equation {
        fn from((parameters, term): (Box<[i16]>, i16)) -> Self {
            Self { parameters, term }
        }
    }

    impl fmt::Debug for Equation {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut prev_was_zero = true;

            for (param, c) in self.parameters.iter().zip(('a'..='z').cycle()) {
                if !prev_was_zero {
                    write!(f, " + ")?;
                } else {
                    write!(f, "   ")?;
                }

                prev_was_zero = *param == 0;
                if !prev_was_zero {
                    write!(f, "{param:>4}{c}")?;
                } else {
                    write!(f, "     ")?;
                }
            }

            write!(f, " = {:>3}", self.term)
        }
    }

    impl Equation {
        pub const fn new(parameters: Box<[i16]>, term: i16) -> Self {
            Self { parameters, term }
        }

        pub fn set_n_term(&mut self, n: usize, value: i16) {
            self.parameters[n] = value;
        }

        pub fn set_term(&mut self, value: i16) {
            self.term = value;
        }

        pub fn zeroed(len: usize) -> Self {
            Self {
                parameters: vec![0; len].into_boxed_slice(),
                term: 0,
            }
        }

        pub fn left_padded(&self) -> usize {
            self.parameters
                .iter()
                .position(|&item| item != 0)
                .unwrap_or(self.degree())
        }

        pub fn degree(&self) -> usize {
            self.parameters.len()
        }

        pub fn eliminate_by(&mut self, other: &Self) -> bool {
            let term_idx = self.left_padded();
            let Some(&my_term) = self.parameters.get(term_idx) else {
                return false;
            };

            let Some(&other_term) = other.parameters.get(term_idx) else {
                return false;
            };

            // self * other_term - other * self_term
            self.parameters
                .iter_mut()
                .zip(other.parameters.iter())
                .for_each(|(a, &b)| *a = *a * other_term - b * my_term);

            self.term = self.term * other_term - other.term * my_term;
            true
        }

        pub fn is_empty(&self) -> bool {
            self.parameters.iter().all(|&v| v == 0)
        }

        pub fn has_known(&self) -> Option<(usize, i16)> {
            let (only_idx, &only_val) =
                self.parameters.iter().enumerate().find(|&(_, &v)| v != 0)?;
            if self.parameters.iter().enumerate().all(|(i, &v)| {
                if i == only_idx {
                    return true;
                }

                v == 0
            }) {
                Some((
                    only_idx,
                    self.term
                        .div_exact(only_val)
                        .expect("Equation found a fractional solution"),
                ))
            } else {
                None
            }
        }
    }

    // pub mod var {
    //     pub enum VarDependency

    //     pub enum VarKnowledgeType {
    //         DependantOn()
    //         Known(i16),
    //         Unknown,
    //     }

    //     pub struct VarKnowledge {
    //         var_idx: usize,
    //         type: VarKnowledgeType,
    //     }
    // }
}

mod __hide {
    use std::{cmp::Ordering, iter::Sum, ops::Add};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum EqEvalRes {
        Correct,
        Undershoot,
        Overshoot,
    }

    impl Add for EqEvalRes {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            use EqEvalRes::*;

            match (self, rhs) {
                (Correct, x) => x,
                (_, Overshoot) | (Overshoot, _) => Overshoot,
                (Undershoot, _) => Undershoot,
            }
        }
    }

    impl Sum<Self> for EqEvalRes {
        fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
            iter.reduce(|a, b| a + b).unwrap_or(Self::Correct)
        }
    }

    impl From<Ordering> for EqEvalRes {
        fn from(value: Ordering) -> Self {
            use EqEvalRes::*;

            match value {
                Ordering::Less => Undershoot,
                Ordering::Equal => Correct,
                Ordering::Greater => Overshoot,
            }
        }
    }
}

/// The max indicator is like 10 long, eyeballing it, makes sense so theres only a digit.
///
/// So use a u16 as a bitfield for that, saves a lot of memory.
#[derive(Clone)]
struct Machine {
    indicators: u16,
    buttons: Box<[u16]>,
    equations: Box<[Equation]>,
    len: u8,
}

impl Machine {
    fn gauss(&mut self) {
        let mut do_smth = true;
        while do_smth {
            do_smth = false;
            self.equations.sort_by_key(|eq| eq.left_padded());

            for i in 1..self.equations.len() {
                for j in 0..i {
                    if self.equations[i].left_padded() == self.equations[j].left_padded() {
                        // SAFETY: i != j (0..!=i), so its not the same things we're borrowing
                        do_smth |= unsafe { &mut *std::ptr::from_mut(&mut self.equations[i]) }
                            .eliminate_by(&self.equations[j]);
                    }
                }
            }
        }
    }

    fn from_slice(s: &[u8]) -> Self {
        let (left, right) = s.split_once(|&b| b == b']').unwrap();

        let len = left[1..].len() as u8;
        let indicators = left[1..]
            .iter()
            .rev()
            .fold(0u16, |acc, &b| (acc << 1) | (b == b'#') as u16);

        let (left, right) = right.split_once(|&b| b == b'{').unwrap();

        let buttons = left
            .trim_ascii()
            .split(|&b| b == b' ')
            .map(|button| {
                button[1..(button.len() - 1)]
                    .split(|&b| b == b',')
                    .fold(0, |bitfield, n| {
                        debug_assert_eq!(n.len(), 1);

                        let idx = n[0] - b'0';
                        bitfield | (1 << idx)
                    })
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let joltages = right[..(right.len() - 1)]
            .split(|&b| b == b',')
            .map(|s| s.iter().fold(0, |acc, b| acc * 10 + (b - b'0') as i16));

        let mut eqs = Vec::with_capacity(len as usize);
        for (pos, jolts) in joltages.enumerate() {
            let mut eq = Equation::zeroed(buttons.len());
            eq.set_term(jolts);

            for (i, b) in buttons.iter().enumerate() {
                // println!("{pos} {b:b}");
                if b & (1 << pos) != 0 {
                    eq.set_n_term(i, 1);
                }
            }

            eqs.push(eq);
        }

        Self {
            indicators,
            buttons,
            equations: eqs.into_boxed_slice(),
            len,
        }
    }
}

impl fmt::Debug for Machine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Machine([{:0width$b}]",
            self.indicators,
            width = self.len as usize
        )?;

        for button in &self.buttons {
            write!(f, " {button:0width$b}", width = self.len as usize)?;
        }

        write!(f, ")")
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn challenge_usize(buf: &[u8]) -> usize {
    let machines = buf[..(buf.len() - 1)]
        .split(|&b| b == b'\n')
        .map(Machine::from_slice);

    for (i, mut machine) in machines.enumerate() {
        println!("{i}: {machine:?}");

        for eq in &machine.equations {
            println!(" {eq:?} {:?}", eq.has_known());
        }
        println!();

        machine.gauss();

        for eq in &machine.equations {
            println!(" {eq:?} {:?}", eq.has_known());
        }
        println!();
    }

    42
}

/// Just an implementation of euler's algorithm
fn gcd(mut a: u16, mut b: u16) -> u16 {
    while b != 0 {
        (a, b) = (b, a % b)
    }
    a
}

fn lcm(a: u16, b: u16) -> u16 {
    a * b / gcd(a, b)
}
