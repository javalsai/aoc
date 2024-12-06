use std::{fs::File, io::Read};

fn main() -> std::io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let filename = &args[1];

    let mut contents = String::new();
    let mut file = File::open(filename)?;
    file.read_to_string(&mut contents)?;

    let lines: Vec<_> = contents.lines().collect();
    // let height = lines.len();
    // let width = lines[0].len();
    // let res_buf: Vec<Vec<_>> = (0..height)
    //     .map(|_| (0..width).map(|_| '.').collect())
    //     .collect();
    // _ = &res_buf;

    print_res(
        &lines
            .iter()
            .map(|ln| ln.chars().collect::<Vec<_>>())
            .collect::<Vec<_>>(),
    );

    // let r = eq_in_dir(
    //     &lines
    //         .iter()
    //         .map(|ln| ln.chars().collect::<Vec<_>>())
    //         .collect::<Vec<_>>(),
    //     (4, 0),
    //     (1, 1),
    //     "MAS".chars(),
    // );
    // println!("{r:?}");
    // return Ok(());

    let mut t = 0;
    for (i, ln) in lines.iter().enumerate() {
        for (j, ch) in ln.chars().enumerate() {
            if ch == 'X' {
                // println!("X found at ({j}, {i})");
                let map = [-1, 0, 1].map(|i| [-1, 0, 1].map(|j| (i, j)));
                for dir in map.as_flattened() {
                    let r = eq_in_dir(
                        &lines
                            .iter()
                            .map(|ln| ln.chars().collect::<Vec<_>>())
                            .collect::<Vec<_>>(),
                        (j, i),
                        *dir,
                        "MAS".chars(),
                    );
                    if r { t += 1; }
                    // println!(" {dir:?}: {r:?}");
                }
            }
        }
    }

    println!("total {t}");
    Ok(())
}

fn eq_in_dir(
    buf: &[Vec<char>],
    pos: (usize, usize),
    dir: (isize, isize),
    matches: impl Iterator<Item = char>,
) -> bool {
    opt_eq_in_dir(buf, pos, dir, matches).is_some()
}

fn opt_eq_in_dir(
    buf: &[Vec<char>],
    pos: (usize, usize),
    dir: (isize, isize),
    mut matches: impl Iterator<Item = char>,
) -> Option<()> {

    // println!(" {pos:?} {dir:?}");
    let Some(must_match) = matches.next() else {
        return Some(());
    };
    // println!("  {must_match:?}");

    let new_1 = do_the_thing(dir.1, pos.1)?;
    let new_0 = do_the_thing(dir.0, pos.0)?;
    let ln = buf.get(new_1)?;
    let ch = ln.get(new_0)?;
    // println!("   {ch:?}");

    if must_match == *ch {
        opt_eq_in_dir(buf, (new_0, new_1), dir, matches)
    } else {
        None
    }
}

fn do_the_thing(a: isize, b: usize) -> Option<usize> {
    let r = b as isize + a;
    if r < 0 {
        return None;
    }
    Some(r as usize)
}

fn print_res(buf: &[Vec<char>]) {
    for ln in buf.iter() {
        println!("{}", ln.iter().collect::<String>())
    }
}
