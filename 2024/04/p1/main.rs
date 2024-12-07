use std::{
    cmp,
    fs::File,
    io::Read,
    time::{Duration, Instant},
};

fn main() -> std::io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let filename = &args[1];
    let runs: u32 = args[2]
        .parse()
        .expect("did not receive a number as `runs` argument");

    let mut contents = vec![];
    let mut file = File::open(filename)?;
    file.read_to_end(&mut contents)?;
    let lines: Vec<_> = contents.split(|&b| b == b'\n').collect();

    let mut max = Duration::ZERO;
    let mut avg = Duration::ZERO;
    let mut min = Duration::MAX;
    let result = run(&lines);
    for i in 0..runs {
        let a = Instant::now();
        let run_result = run(&lines);
        let b = Instant::now();
        if result != run_result {
            panic!(
                "supposed result is {} but this run got {} after {} runs",
                result, run_result, i
            );
        }

        let d = b - a;
        max = cmp::max(d, max);
        min = cmp::min(d, min);
        avg += d / runs;
    }

    println!("\x1b[0;34mtotal \x1b[1;33m{result}\x1b[0;34m\n  runs  \x1b[1;33m{runs:?}\x1b[0;34m\n  avg.  \x1b[1;35m{avg:?}\x1b[0;34m\n  max.  \x1b[1;31m{max:?}\x1b[0;34m\n  min.  \x1b[1;32m{min:?}\x1b[0m");
    Ok(())
}

fn run(lines: &[&[u8]]) -> usize {
    let mut t = 0;
    for (i, ln) in lines.iter().enumerate() {
        for (j, &ch) in ln.iter().enumerate() {
            if ch == b'X' {
                for dir in [
                    (-1, -1),
                    (-1, 0),
                    (-1, 1),
                    (0, -1),
                    (0, 1),
                    (1, -1),
                    (1, 0),
                    (1, 1),
                ] {
                    let r = eq_in_dir(lines, (j, i), dir, "MAS".bytes());
                    if r {
                        t += 1;
                    }
                }
            }
        }
    }
    t
}

macro_rules! pstve_try_bool {
    ($a:expr, $b:expr) => {{
        let r = $a + $b;
        if r < 0 {
            return false;
        }
        r
    }};
}

fn eq_in_dir(
    buf: &[&[u8]],
    pos: (usize, usize),
    dir: (isize, isize),
    mut matches: impl Iterator<Item = u8>,
) -> bool {
    let Some(must_match) = matches.next() else {
        return true;
    };

    let new_1 = pstve_try_bool!(pos.1 as isize, dir.1) as usize;
    let new_0 = pstve_try_bool!(pos.0 as isize, dir.0) as usize;
    let Some(ln) = buf.get(new_1) else {
        return false;
    };
    let Some(ch) = ln.get(new_0) else {
        return false;
    };

    if must_match == *ch {
        eq_in_dir(buf, (new_0, new_1), dir, matches)
    } else {
        false
    }
}
