use std::{fs::File, io::Read, time::Instant};

fn main() -> std::io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let filename = &args[1];

    let mut contents = vec![];
    let mut file = File::open(filename)?;
    file.read_to_end(&mut contents)?;

    let a = Instant::now();

    let lines: Vec<_> = contents.split(|&b| b == b'\n').collect();
    let mut t = 0;
    for (i, ln) in lines.iter().enumerate() {
        for (j, &ch) in ln.iter().enumerate() {
            if ch == b'X' {
                let map = [-1, 0, 1].map(|i| [-1, 0, 1].map(|j| (i, j)));
                for dir in map.as_flattened() {
                    if dir.0 == 0 && dir.1 == 0 {
                        continue;
                    }
                    let r = eq_in_dir(&lines, (j, i), *dir, "MAS".bytes());
                    if r {
                        t += 1;
                    }
                }
            }
        }
    }

    let b = Instant::now();

    println!("total {t} in {:?}", b - a);
    Ok(())
}

fn eq_in_dir(
    buf: &[&[u8]],
    pos: (usize, usize),
    dir: (isize, isize),
    matches: impl Iterator<Item = u8>,
) -> bool {
    opt_eq_in_dir(buf, pos, dir, matches).is_some()
}

fn opt_eq_in_dir(
    buf: &[&[u8]],
    pos: (usize, usize),
    dir: (isize, isize),
    mut matches: impl Iterator<Item = u8>,
) -> Option<()> {
    let Some(must_match) = matches.next() else {
        return Some(());
    };

    let new_1 = do_the_thing(dir.1, pos.1)?;
    let new_0 = do_the_thing(dir.0, pos.0)?;
    let ln = buf.get(new_1)?;
    let ch = ln.get(new_0)?;

    if must_match == *ch {
        opt_eq_in_dir(buf, (new_0, new_1), dir, matches)
    } else {
        None
    }
}

#[inline]
fn do_the_thing(a: isize, b: usize) -> Option<usize> {
    let r = b as isize + a;
    if r < 0 {
        return None;
    }
    Some(r as usize)
}
