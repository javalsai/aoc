#![feature(vec_pop_if)]

use std::{
    cmp,
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    time::{Duration, Instant},
};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let filename = &args[1];
    let runs = args[2]
        .parse()
        .expect("expected a number on second argument");

    let mut contents = vec![];
    File::open(filename)
        .expect("Couldn't open file")
        .read_to_end(&mut contents)
        .expect("Error reading file");
    contents.pop_if(|b| *b == b'\n');
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
}

fn run(lines: &[&[u8]]) -> usize {
    // let mut t = 0;
    let height = lines.len() as isize - 1;
    let width = lines[0].len() as isize - 1;
    // println!("bounds: {:?}", (width, height));

    let mut antinodes: HashSet<(isize, isize)> = HashSet::new();
    let mut hashmap: HashMap<u8, HashSet<(isize, isize)>> = HashMap::new();
    for (y, ln) in lines.iter().enumerate() {
        let y = y as isize;
        for (x, &ch) in ln.iter().enumerate() {
            let x = x as isize;
            if ch.is_ascii_alphanumeric() {
                let freq_set = hashmap.entry(ch).or_default();

                // Calculate antinodes with previous points
                // println!(
                //     "ascii alphanum at {};{} ({}) || {:?}",
                //     x,
                //     y,
                //     (char::from_u32(ch as u32)).unwrap(),
                //     freq_set
                // );
                for point in freq_set.iter() {
                    let dr = (x - point.0, y - point.1);
                    let t1 = (point.0 - dr.0, point.1 - dr.1);
                    let t2 = (x + dr.0, y + dr.1);

                    // println!(" calc relative to {:?} | {:?} and {:?}", point, t1, t2);

                    for tgt in [t1, t2] {
                        if tgt.0 < 0 || tgt.0 > width || tgt.1 < 0 || tgt.1 > height {
                            // println!("   target {:?} overshoots", tgt);
                            continue;
                        }
                        if antinodes.insert(tgt) { /* t += 1; */ }
                    }
                }

                freq_set.insert((x, y));
            }
        }
    }

    antinodes.len()
}
