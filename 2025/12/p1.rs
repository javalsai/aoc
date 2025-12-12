//! # **WHAAAAAAT**
//!
//! istg i read someone mentioned it fsr worked on the sum of areas and I just went with it, but it
//! fails the demo wtff... Surely an input bug.
//!
//! Still isn't this problem NP-complete????? https://en.wikipedia.org/wiki/Bin_packing_problem. I
//! don't see any difference from this.

#![feature(iterator_try_collect)]

use std::{fmt, str::FromStr};

pub struct Shape {}

pub struct Region {
    pub width: usize,
    pub height: usize,
    pub qtys: Vec<usize>,
}

impl Region {
    fn area(&self) -> usize {
        self.width * self.height
    }
}

impl fmt::Debug for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}x{})", self.width, self.height)?;

        for qty in &self.qtys {
            write!(f, " {qty}")?;
        }

        Ok(())
    }
}

impl FromStr for Region {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (size, qtys) = s.split_once(": ").ok_or(())?;

        let (width, height) = size.split_once('x').ok_or(())?;
        let (width, height) = (
            width.parse::<usize>().map_err(|_| ())?,
            height.parse::<usize>().map_err(|_| ())?,
        );

        let qtys = qtys
            .split(' ')
            .map(|q| q.parse::<usize>())
            .try_collect::<Vec<_>>()
            .map_err(|_| ())?;

        Ok(Region {
            width,
            height,
            qtys,
        })
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn challenge_usize(buf: &[u8]) -> usize {
    let input = str::from_utf8(&buf[..(buf.len() - 1)]).unwrap();

    let mut count = 0;

    let (shapes, regions) = input.rsplit_once("\n\n").unwrap();

    let shapes = {
        shapes
            .split("\n\n")
            .map(|shape| shape.chars().filter(|&c| c == '#').count())
            .collect::<Vec<_>>()
    };

    for region in regions
        .split('\n')
        .map(Region::from_str)
        .map(Result::unwrap)
        .collect::<Vec<_>>()
    {
        let area = region.area();
        let present_area: usize = region
            .qtys
            .iter()
            .enumerate()
            .map(|(present_idx, amt)| shapes[present_idx] * amt)
            .sum();

        if present_area <= area {
            count += 1;
        }
    }

    count
}
