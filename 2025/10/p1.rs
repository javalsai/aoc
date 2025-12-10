#![feature(slice_split_once)]

use std::fmt;

/// The max indicator is like 10 long, eyeballing it, makes sense so theres only a digit.
///
/// So use a u16 as a bitfield for that, saves a lot of memory.
#[derive(Clone)]
struct Machine {
    indicators: u16,
    buttons: Box<[u16]>,
    joltages: Box<[u16]>, // This u16 is unrelated (for now) btw
    len: u8,
}

impl Machine {
    fn from_slice(s: &[u8]) -> Self {
        let (left, right) = s.split_once(|&b| b == b']').unwrap();

        let len = left[1..].len() as u8;
        let indicators = left[1..]
            .iter()
            .rev()
            .fold(0u16, |acc, &b| (acc << 1) | (b == b'#') as u16);

        let (left, right) = right.split_once(|&b| b == b'{').unwrap();

        let joltages = right[..(right.len() - 1)]
            .split(|&b| b == b',')
            .map(|s| s.iter().fold(0, |acc, b| acc * 10 + (b - b'0') as u16))
            .collect::<Vec<_>>()
            .into_boxed_slice();

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

        Self {
            indicators,
            buttons,
            joltages,
            len,
        }
    }

    fn try_solution(&self, max: &mut u8, mut solution: u32) {
        let params = count_bit_population(solution);
        if params >= *max {
            return;
        }

        let mut i = 0;
        let mut acc = self.indicators;
        while solution != 0 {
            let this_vec = (solution & 1) == 1;
            solution >>= 1;

            if this_vec {
                acc ^= self.buttons[i];

                // found a solution
                if acc == 0 {
                    let rem_params = count_bit_population(solution);
                    *max = params - rem_params;
                }
            }

            i += 1;
        }
    }
}

fn count_bit_population(mut n: u32) -> u8 {
    let mut count = 0;
    while n != 0 {
        count += 1;
        n = n & (n - 1);
    }

    count
}

impl fmt::Debug for Machine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Machine([{:0width$b}] ",
            self.indicators,
            width = self.len as usize
        )?;

        for button in &self.buttons {
            write!(f, "{button:0width$b} ", width = self.len as usize)?;
        }

        write!(f, "{{{:?}}})", self.joltages)
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn challenge_usize(buf: &[u8]) -> usize {
    // Istg this is just solving a ℤ₂ vector system to find the smallest sum of coords that get you 0⃗
    //
    // The parameters are also ℤ₂ because applying the same button twice would just undo it. It's
    // either brute forceable or fancy math O(1), I just know how to do systems by hand, not
    // computer. But ummmmmmm.
    //
    // I = a b₀ + b b₁ + ...
    //  where I: Machine::indicators
    //
    // ⎧ a b₀₀ + b b₀₁ + ... = I₀
    // ⎨ a b₁₀ + b b₁₁ + ... = I₁
    // ⎩ ...
    //
    // ⎛ b₀  ⎞ ⎛ a ⎞   ⎛ I₀  ⎞
    // ⎜ b₁  ⎟·⎜ b ⎟ = ⎜ I₁  ⎟
    // ⎝ ... ⎠ ⎝...⎠   ⎝ ... ⎠
    //    A   ·  C   =   I
    //
    // Ofc that'd just be I · A⁻¹, but A is not square and will have multiple solutions.

    let mut presses_count = 0;

    let machines = buf[..(buf.len() - 1)]
        .split(|&b| b == b'\n')
        .map(Machine::from_slice);

    for machine in machines {
        let mut max = machine.buttons.len() as u8;
        let max_comb_bitf = 1u32 << max;

        let mut vec_bitfield = 1u32; // represents the vectors to try
        while vec_bitfield < max_comb_bitf {
            machine.try_solution(&mut max, vec_bitfield);
            vec_bitfield += 1;
        }

        presses_count += max as usize;
    }

    presses_count
}
