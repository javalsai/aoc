use std::{fs::File, io::Read, time::Instant};

fn main() -> std::io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let filename = &args[1];

    let mut contents = vec![];
    let mut file = File::open(filename)?;
    file.read_to_end(&mut contents)?;

    let directions: &[(isize, isize)] = &[
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ];

    let a = Instant::now();

    let lines: Vec<_> = contents.split(|&b| b == b'\n').collect();
    let mut t = 0;
    for (i, ln) in lines.iter().enumerate() {
        for (j, &ch) in ln.iter().enumerate() {
            if ch == b'X' {
                for dir in directions.iter() {
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

macro_rules! pstve_try_bool {
    ($a:expr, $b:expr) => {{
        let r = $a + $b;
        if r < 0 { return false; }
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
    let Some(ln) = buf.get(new_1) else { return false; };
    let Some(ch) = ln.get(new_0) else { return false; };

    if must_match == *ch {
        eq_in_dir(buf, (new_0, new_1), dir, matches)
    } else {
        false
    }
}
