#![feature(slice_split_once, exact_div, never_type, iterator_try_collect)]

use std::{
    collections::{HashMap, HashSet},
    env::Vars,
    fmt,
};

use crate::eq::{
    Equation, Known,
    var::{Var, VarStrategy},
};

pub mod eq {
    use std::{
        fmt,
        ops::{DivAssign, Index, IndexMut},
        // ops::{AddAssign, MulAssign},
    };

    use crate::{
        eq::{param::Parametrization, var::Var},
        gcd,
    };

    pub type Known = (Var, i32);

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

    impl Index<Var> for Equation {
        type Output = i32;

        fn index(&self, index: Var) -> &Self::Output {
            &self.parameters[index.0]
        }
    }

    impl IndexMut<Var> for Equation {
        fn index_mut(&mut self, index: Var) -> &mut Self::Output {
            &mut self.parameters[index.0]
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

        pub fn vars_with_params(&self) -> impl Iterator<Item = (Var, i32)> {
            self.parameters
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, param)| (Var(i), param))
        }

        pub fn get(&self, param: Var) -> Option<&i32> {
            self.parameters.get(param.0)
        }

        pub fn has_param(&self, param: Var) -> bool {
            self.get(param).is_some_and(|&p| p != 0)
        }

        pub fn non_zero_vars(&self) -> impl Iterator<Item = Var> {
            self.parameters
                .iter()
                .enumerate()
                .filter(|&(_, &v)| v != 0)
                .map(|(idx, _)| Var(idx))
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

        pub fn range(&self) -> usize {
            self.parameters.iter().filter(|&&v| v != 0).count()
        }

        pub fn dimension(&self) -> usize {
            self.parameters.len()
        }

        pub fn has_known(&self) -> Option<Known> {
            let (only_idx, only_val) = self.vars_with_params().find(|&(_, v)| v != 0)?;
            if self.vars_with_params().all(|(i, v)| {
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

        pub fn place_param(&mut self, var: Var, val: i32) {
            let term_left = self[var] * val;
            self.term -= term_left;
            self[var] = 0;
        }

        pub fn try_parametrize(&self, var: Var) -> Option<Parametrization> {
            // instead of moving all other parameters to the other side, swap with one with the
            // term

            let div = -self[var];
            let offset = -self.term.div_exact(div)?;

            let others = self
                .vars_with_params()
                .filter(|&(i, param)| param != 0 && i != var)
                .map(|(idx, param)| Some((idx, param.div_exact(div)?)))
                .try_collect::<Vec<_>>()?;

            Some(Parametrization { others, offset })
        }
    }

    pub mod param {
        use std::fmt;

        use crate::eq::var::Var;

        #[derive(Clone)]
        pub struct Parametrization {
            pub others: Vec<(Var, i32)>,
            pub offset: i32,
        }

        impl fmt::Debug for Parametrization {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                for other in &self.others {
                    write!(f, "{}{:?} + ", other.1, other.0)?;
                }

                write!(f, "{}", self.offset)
            }
        }
    }

    pub mod var {
        use std::fmt;

        use crate::eq::param::Parametrization;

        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct Var(pub(crate) usize);

        impl fmt::Debug for Var {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", ('a'..='z').cycle().nth(self.0).unwrap())
            }
        }

        #[derive(Clone, Debug)]
        pub enum VarStrategy {
            BruteForce,
            DependantOn(Parametrization),
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

    fn range(&self) -> usize {
        self.equations.iter().filter(|eq| !eq.is_empty()).count()
    }

    fn raw_dimension(&self) -> usize {
        self.equations.first().map(|eq| eq.dimension()).unwrap_or(0)
    }

    fn reduce(mut self) -> Result<Vec<Known>, (Self, Vec<Known>)> {
        let knowns = self
            .equations
            .iter()
            .filter_map(|eq| eq.has_known())
            .collect::<Vec<_>>();

        if knowns.len() == self.buttons.len() {
            Ok(knowns)
        } else {
            for &(known_idx, known_val) in &knowns {
                for eq in &mut self.equations {
                    eq.place_param(known_idx, known_val);
                }
            }

            self.equations.sort_by_key(|eq| eq.left_padded());
            Err((self, knowns))
        }
    }

    fn _find_other_single_eq_with_param(
        &self,
        my_i: usize,
        var: Var,
    ) -> Option<(usize, &Equation)> {
        let iter = self
            .equations
            .iter()
            .enumerate()
            .filter(|&(idx, _)| idx != my_i)
            .filter(|&(_, eq)| eq.has_param(var));
        if iter.count() == 1 {
            let mut iter = self
                .equations
                .iter()
                .enumerate()
                .filter(|&(idx, _)| idx != my_i)
                .filter(|&(_, eq)| eq.has_param(var));

            Some(iter.next().unwrap())
        } else {
            None
        }
    }

    fn _strategies(&self, hash_set: &mut HashSet<Var>, i: usize, eq1: &Equation, var: Var) {
        println!("param {var:?} is unique here (eq1) (so other things can be made from this)");

        let leading_param = var;
        hash_set.insert(leading_param);

        for other_param in eq1
            .non_zero_vars()
            .filter(|param| !hash_set.contains(param))
        {
            let mut yet_this_other_branch_hash_set = hash_set.clone();
            yet_this_other_branch_hash_set.insert(other_param);
            println!(" could make {other_param:?} from it");

            if let Some((i, other)) = self._find_other_single_eq_with_param(i, other_param) {
                println!("{other:?}");
                self._strategies(&mut yet_this_other_branch_hash_set, i, other, other_param);
            }
        }
    }

    /// Will fail if the parametrization is fractional
    fn parametrize_system_based_on(
        &self,
        mut strategies: Vec<(Var, VarStrategy)>,
    ) -> Option<Vec<(Var, VarStrategy)>> {
        fn deducible_var(var: Var, strategies: &[(Var, VarStrategy)]) -> bool {
            strategies.iter().any(|strat| strat.0 == var)
        }

        let mut found_any = true;

        while found_any {
            println!("{strategies:?}");
            found_any = false;

            for eq in self.equations.iter() {
                let (relative_to, deducible_eq) = {
                    if let Some(first_unknown) = eq
                        .non_zero_vars()
                        .find(|&var| !deducible_var(var, &strategies))
                    {
                        println!(" {first_unknown:?} ({eq:?})");
                        if eq
                            .non_zero_vars()
                            .filter(|&var| var != first_unknown)
                            .all(|var| deducible_var(var, &strategies))
                        {
                            (first_unknown, eq)
                        } else {
                            continue;
                        }
                    } else {
                        // all are knowns, so filter out
                        continue;
                    }
                };

                // ughhh, gotta find all equations with vars < n_params, not the immediate one bcs
                // in this system we need to parametrize m which is in an eq with 3 vars and theres
                // 3 paramaters, i need to parametrize it among others...
                println!(
                    "   {:?}",
                    deducible_eq.try_parametrize(relative_to),
                );
                found_any = true;

                strategies.push((
                    relative_to,
                    VarStrategy::DependantOn(deducible_eq.try_parametrize(relative_to)?),
                ));
            }
        }

        if self.equations.iter().all(|eq| {
            eq.non_zero_vars()
                .all(|var| deducible_var(var, &strategies))
        }) {
            Some(strategies)
        } else {
            None
        }
    }

    /// This will parametrize ALL the system given the number of parameters. It will get a list of
    /// the only strategy that doesn't involve a fractional equation. Hence all strategies will be
    /// with and give integers.
    fn parametrize(&self, num_params: usize) -> Vec<Vec<(Var, VarStrategy)>> {
        self.equations
            .iter()
            .filter(|eq| eq.range() == num_params + 1)
            .flat_map(|eq1| {
                eq1.non_zero_vars()
                    .filter_map(|param| Some((param, eq1.try_parametrize(param)?)))
                    .filter_map(|initial_param| {
                        let mut knowns = initial_param
                            .1
                            .others
                            .iter()
                            .map(|e| (e.0, VarStrategy::BruteForce))
                            .collect::<Vec<_>>();

                        knowns.push((initial_param.0, VarStrategy::DependantOn(initial_param.1)));

                        self.parametrize_system_based_on(knowns)
                    })
            })
            .collect()
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
        match machine.reduce() {
            Ok(knowns) => knowns
                .iter()
                .for_each(|&(_, count)| total_count += count as usize),
            Err((machine, knowns)) => {
                let parameter_count = machine.raw_dimension() - knowns.len() - machine.range();

                println!("{i}: {machine:?}");
                println!("needed params: {}", parameter_count);

                for eq in &machine.equations {
                    println!("{eq:?}");
                    if eq.is_empty() {
                        continue;
                    }
                }
                // let relations = machine.relations();
                // println!("relations: {:?}", relations);
                let strategies = machine.parametrize(parameter_count);
                println!("strategies len: {}", strategies.len());
                for strategy in strategies {
                    println!("{strategy:?}");
                }

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

// fn iter_until<T>(s: &mut [u32], skip: &[usize], mut f: impl FnMut(&[u32]) -> Option<T>) -> T {
//     fn iter_until_rec<T>(
//         s: &mut [u32],
//         usable_size: usize,
//         skip: &[usize],
//         l: u32,
//         f: &mut impl FnMut(&[u32]) -> Option<T>,
//     ) -> Option<T> {
//         if usable_size == 0 {
//             if l == 0 { f(s) } else { None }
//         } else if skip.iter().any(|&pos| pos == usable_size - 1) {
//             iter_until_rec(s, usable_size - 1, skip, l, f)
//         } else {
//             for i in 0..=l {
//                 s[usable_size - 1] = i;

//                 if let Some(ret) = iter_until_rec(s, usable_size - 1, skip, l - i, f) {
//                     return Some(ret);
//                 }
//             }

//             None
//         }
//     }

//     for len in 1.. {
//         if let Some(ret) = iter_until_rec(s, s.len(), skip, len, &mut f) {
//             return ret;
//         }
//     }

//     unreachable!()
// }
