#![feature(slice_split_once, exact_div, never_type)]

use std::{
    collections::{HashMap, HashSet, hash_set},
    fmt,
};

use crate::eq::{Equation, var::VarKnowledge};

pub mod eq {
    use std::{
        fmt,
        ops::DivAssign,
        // ops::{AddAssign, MulAssign},
    };

    use crate::gcd;

    #[derive(Clone)]
    pub struct Equation {
        parameters: Box<[i32]>,
        term: i32,
    }

    impl DivAssign<i32> for Equation {
        fn div_assign(&mut self, rhs: i32) {
            self.parameters
                .iter_mut()
                .for_each(|param| *param = param.div_exact(rhs).unwrap());
            self.term = self.term.div_exact(rhs).unwrap();
        }
    }

    // impl MulAssign<i32> for Equation {
    //     fn mul_assign(&mut self, rhs: i32) {
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

    impl From<(Box<[i32]>, i32)> for Equation {
        fn from((parameters, term): (Box<[i32]>, i32)) -> Self {
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
        pub const fn new(parameters: Box<[i32]>, term: i32) -> Self {
            Self { parameters, term }
        }

        pub fn set_n_term(&mut self, n: usize, value: i32) {
            self.parameters[n] = value;
        }

        pub fn set_term(&mut self, value: i32) {
            self.term = value;
        }

        pub fn zeroed(len: usize) -> Self {
            Self {
                parameters: vec![0; len].into_boxed_slice(),
                term: 0,
            }
        }

        pub fn parameters(&self) -> &[i32] {
            &self.parameters
        }

        pub fn has_param(&self, param: usize) -> bool {
            self.parameters.get(param).is_some_and(|&p| p != 0)
        }

        pub fn param_idxs(&self) -> impl Iterator<Item = usize> {
            self.parameters
                .iter()
                .enumerate()
                .filter(|&(_, &v)| v != 0)
                .map(|(idx, _)| idx)
        }

        pub fn left_padded(&self) -> usize {
            self.parameters
                .iter()
                .position(|&item| item != 0)
                .unwrap_or(self.degree())
        }

        pub fn right_padded(&self) -> Option<usize> {
            self.parameters
                .iter()
                .rev()
                .position(|&item| item != 0)
                .map(|v| self.parameters.len() - v - 1)
        }

        pub fn degree(&self) -> usize {
            self.parameters.len()
        }

        /// Call this once in a while to get the gcd of the terms and turn it down instead of
        /// spiraling up.
        pub fn normalize(&mut self) {
            let mut the_gcd = self
                .parameters
                .iter()
                .map(|&signed| signed.unsigned_abs())
                .reduce(gcd)
                .unwrap();

            the_gcd = gcd(the_gcd, self.term as u32);
            if the_gcd != 0 {
                self.div_assign(the_gcd as i32);
            }
        }

        pub fn eliminate_by(&mut self, other: &Self) -> bool {
            let term_idx = self.left_padded();
            let Some(mut my_term) = self.parameters.get(term_idx).cloned() else {
                return false;
            };

            let Some(mut other_term) = other.parameters.get(term_idx).cloned() else {
                return false;
            };

            let the_gcd = gcd(my_term.unsigned_abs(), other_term.unsigned_abs()) as i32;
            my_term /= the_gcd;
            other_term /= the_gcd;

            // self * other_term - other * self_term
            self.parameters
                .iter_mut()
                .zip(other.parameters.iter())
                .for_each(|(a, &b)| *a = *a * other_term - b * my_term);

            self.term = self.term * other_term - other.term * my_term;
            true
        }

        pub fn back_eliminate_by(&mut self, other: &Self) -> bool {
            let Some(term_idx) = self.right_padded() else {
                return false;
            };

            let Some(mut my_term) = self.parameters.get(term_idx).cloned() else {
                return false;
            };

            let Some(mut other_term) = other.parameters.get(term_idx).cloned() else {
                return false;
            };

            let the_gcd = gcd(my_term.unsigned_abs(), other_term.unsigned_abs()) as i32;
            my_term /= the_gcd;
            other_term /= the_gcd;

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

        // pub fn has_relation(&self) -> Option<(usize, (i32, usize))> {
        //     if self.term != 0 {
        //         return None;
        //     }

        //     let (fonly_idx, &fonly_val) =
        //         self.parameters.iter().enumerate().find(|&(_, &v)| v != 0)?;
        //     let (sonly_idx, &sonly_val) = self
        //         .parameters
        //         .iter()
        //         .enumerate()
        //         .skip(fonly_idx + 1)
        //         .find(|&(_, &v)| v != 0)?;

        //     if self.parameters.iter().enumerate().all(|(i, &v)| {
        //         if i == fonly_idx || i == sonly_idx {
        //             return true;
        //         }

        //         v == 0
        //     }) {
        //         // fonly_val * fonly_idx = -sonly_val * sonly_idx, either:
        //         //  fonly_idx = -(sonly_val/fonly_val) * sonly_idx
        //         //  sonly_idx = -(fonly_val/sonly_val) * fonly_idx

        //         if sonly_val > fonly_val {
        //             Some((
        //                 fonly_idx,
        //                 (-sonly_val.div_exact(fonly_val).unwrap(), sonly_idx),
        //             ))
        //         } else {
        //             Some((
        //                 sonly_idx,
        //                 (-fonly_val.div_exact(sonly_val).unwrap(), fonly_idx),
        //             ))
        //         }
        //     } else {
        //         None
        //     }
        // }

        // pub fn has_relation(&self) -> Option<(usize, (i32, i32, usize))> {
        //     let (fonly_idx, &fonly_val) =
        //         self.parameters.iter().enumerate().find(|&(_, &v)| v != 0)?;
        //     let (sonly_idx, &sonly_val) = self
        //         .parameters
        //         .iter()
        //         .enumerate()
        //         .skip(fonly_idx + 1)
        //         .find(|&(_, &v)| v != 0)?;

        //     if self.parameters.iter().enumerate().all(|(i, &v)| {
        //         if i == fonly_idx || i == sonly_idx {
        //             return true;
        //         }

        //         v == 0
        //     }) {
        //         // fonly_val * fonly_idx = term - sonly_val * sonly_idx, either:
        //         //  fonly_idx = (term/fonly_val) - (sonly_val/fonly_val) * sonly_idx
        //         //  sonly_idx = (term/sonly_val) - (fonly_val/sonly_val) * fonly_idx

        //         dbg!((fonly_val, sonly_val));
        //         if sonly_val.abs() > fonly_val.abs() || fonly_val.abs() == 1 {
        //             dbg!(self.term, fonly_val);
        //             Some((
        //                 fonly_idx,
        //                 (
        //                     self.term.div_exact(fonly_val).unwrap(),
        //                     -sonly_val.div_exact(fonly_val).unwrap(),
        //                     sonly_idx,
        //                 ),
        //             ))
        //         } else {
        //             Some((
        //                 sonly_idx,
        //                 (
        //                     self.term.div_exact(sonly_val).unwrap(),
        //                     -fonly_val.div_exact(sonly_val).unwrap(),
        //                     fonly_idx,
        //                 ),
        //             ))
        //         }
        //     } else {
        //         None
        //     }
        // }

        pub fn has_known(&self) -> Option<(usize, i32)> {
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

        pub fn test_params(&self, params: &[u16]) -> bool {
            self.parameters
                .iter()
                .enumerate()
                .map(|(i, factor)| params[i] as i32 * factor)
                .sum::<i32>()
                == self.term
        }
    }

    pub mod var {
        #[derive(Clone, Copy)]
        pub enum VarStrategy {
            BruteForce,
            DependantOn(!),
            Known(i32),
        }

        #[derive(Clone, Copy)]
        pub struct VarKnowledge {
            var_idx: usize,
            strategy: VarStrategy,
        }
    }
}

#[allow(dead_code)]
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

    fn gauss_back(&mut self) {
        let mut do_smth = true;
        while do_smth {
            do_smth = false;

            for i in (0..(self.equations.len()) - 1).rev() {
                for j in ((i + 1)..self.equations.len()).rev() {
                    if self.equations[i].right_padded() == self.equations[j].right_padded() {
                        // SAFETY: i != j (0..!=i), so its not the same things we're borrowing
                        do_smth |= unsafe { &mut *std::ptr::from_mut(&mut self.equations[i]) }
                            .back_eliminate_by(&self.equations[j]);
                    }
                }
            }
        }
    }

    fn is_known(&self) -> Result<Vec<(usize, i32)>, Vec<(usize, i32)>> {
        let knowns = self
            .equations
            .iter()
            .filter_map(|eq| eq.has_known())
            .collect::<Vec<_>>();

        if knowns.len() == self.buttons.len() {
            Ok(knowns)
        } else {
            Err(knowns)
        }
    }

    fn _find_other_single_eq_with_param(
        &self,
        my_i: usize,
        param: usize,
    ) -> Option<(usize, &Equation)> {
        let iter = self
            .equations
            .iter()
            .enumerate()
            .filter(|&(idx, _)| idx != my_i)
            .filter(|&(_, eq)| eq.has_param(param));
        if iter.count() == 1 {
            let mut iter = self
                .equations
                .iter()
                .enumerate()
                .filter(|&(idx, _)| idx != my_i)
                .filter(|&(_, eq)| eq.has_param(param));

            Some(iter.next().unwrap())
        } else {
            None
        }
    }

    fn _strategies(&self, hash_set: &mut HashSet<usize>, i: usize, eq1: &Equation, param: usize) {
        println!("param {param} is unique here (eq1) (so other things can be made from this)");

        let leading_param = param;
        hash_set.insert(leading_param);

        for other_param in eq1.param_idxs().filter(|param| !hash_set.contains(param)) {
            let mut yet_this_other_branch_hash_set = hash_set.clone();
            yet_this_other_branch_hash_set.insert(other_param);
            println!(" could make {other_param} from it");

            if let Some((i, other)) = self._find_other_single_eq_with_param(i, other_param) {
                println!("{other:?}");
                self._strategies(&mut yet_this_other_branch_hash_set, i, other, other_param);
            }
        }
    }

    fn strategies(&self) {
        let strategy = HashMap::<usize, VarKnowledge>::new();
        let known_ones = HashSet::<usize>::from_iter(strategy.keys().cloned());

        for (i, eq1) in self.equations.iter().enumerate() {
            for param in eq1.param_idxs().filter(|param| !known_ones.contains(param)) {
                if self
                    .equations
                    .iter()
                    .enumerate()
                    .filter(|&(j, _)| i != j)
                    .all(|(_, eq)| !eq.has_param(param))
                {
                    let mut branch_set = known_ones.clone();
                    branch_set.insert(param);
                    self._strategies(&mut branch_set, i, eq1, param);
                }
            }
        }
    }

    // fn relations(&self) -> Vec<(usize, (i32, i32, usize))> {
    //     self.equations
    //         .iter()
    //         .filter_map(|eq| eq.has_relation())
    //         .collect::<Vec<_>>()
    // }

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
            .map(|s| s.iter().fold(0, |acc, b| acc * 10 + (b - b'0') as i32));

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

    pub fn test_params(&self, solutions: &[u16]) -> bool {
        self.equations
            .iter()
            .rev()
            .all(|eq| eq.test_params(solutions))
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

    let mut total_count = 0;

    for (i, mut machine) in machines.enumerate() {
        machine.gauss();
        machine.gauss_back();

        if i != 143 {
            continue;
        }
        match machine.is_known() {
            Ok(knowns) => knowns
                .iter()
                .for_each(|&(_, count)| total_count += count as usize),
            Err(knowns) => {
                // if [6, 8, 11, 13, 16, 22].contains(&i) {
                //     continue;
                // }

                println!("{i}: {machine:?}");
                println!("{knowns:?} {}/{}", knowns.len(), machine.len);

                let mut skips = Vec::new();
                let mut solution_buffer = vec![0; machine.buttons.len()];
                for (known_idx, known_val) in knowns {
                    assert!(known_val.is_positive() || known_val == 0);
                    solution_buffer[known_idx] = known_val.unsigned_abs();
                    skips.push(known_idx);
                }

                for eq in &machine.equations {
                    println!(" {eq:?} {:?}", eq.has_known());
                }
                // let relations = machine.relations();
                // println!("relations: {:?}", relations);
                machine.strategies();

                // total_count += iter_until(&mut solution_buffer[..], &skips, |potential_sol| {
                //     // println!("{potential_sol:?} total {}", potential_sol.iter().sum::<u16>());
                //     if machine.test_params(potential_sol) {
                //         Some(potential_sol.iter().sum::<u16>())
                //     } else {
                //         None
                //     }
                // }) as usize;
            }
        }
        println!();
    }

    total_count
}

/// Just an implementation of euler's algorithm
fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        (a, b) = (b, a % b)
    }
    a
}

fn lcm(a: u32, b: u32) -> u32 {
    a * b / gcd(a, b)
}

fn iter_until<T>(s: &mut [u32], skip: &[usize], mut f: impl FnMut(&[u32]) -> Option<T>) -> T {
    fn iter_until_rec<T>(
        s: &mut [u32],
        usable_size: usize,
        skip: &[usize],
        l: u32,
        f: &mut impl FnMut(&[u32]) -> Option<T>,
    ) -> Option<T> {
        if usable_size == 0 {
            if l == 0 { f(s) } else { None }
        } else if skip.iter().any(|&pos| pos == usable_size - 1) {
            iter_until_rec(s, usable_size - 1, skip, l, f)
        } else {
            for i in 0..=l {
                s[usable_size - 1] = i;

                if let Some(ret) = iter_until_rec(s, usable_size - 1, skip, l - i, f) {
                    return Some(ret);
                }
            }

            None
        }
    }

    for len in 1.. {
        if let Some(ret) = iter_until_rec(s, s.len(), skip, len, &mut f) {
            return ret;
        }
    }

    unreachable!()
}
