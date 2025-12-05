#[unsafe(no_mangle)]
pub static mut TIMERS: [(&str, Duration); TIMERS_LEN] = [
    ("insert", Duration::ZERO),
    ("count", Duration::ZERO),
];

#[unsafe(no_mangle)]
pub static TIMERS_LEN: usize = 2;
use std::{ops::RangeInclusive, time::{Duration, Instant}};

pub type RangeU = u128;

pub struct BIntervalNode<T: Ord + Copy> {
    value: RangeInclusive<T>,
    len: usize,
    lt: Option<Box<BIntervalNode<T>>>,
    gt: Option<Box<BIntervalNode<T>>>,
}

impl<T: Ord + Copy> BIntervalNode<T> {
    pub fn new(value: RangeInclusive<T>) -> Self {
        Self {
            value,
            len: 1,
            lt: None,
            gt: None,
        }
    }

    fn _is_gt_non_overlapping(lhs: &RangeInclusive<T>, rhs: &RangeInclusive<T>) -> bool {
        rhs.start() > lhs.start()
    }

    pub fn _overlap_with(&self, value: &RangeInclusive<T>) -> RangeInclusive<T> {
        (*self.value.start().min(value.start()))..=(*self.value.end().max(value.end()))
    }

    pub fn _overlap(&self, value: &RangeInclusive<T>) -> bool {
        (value.start() <= self.value.start() && value.end() >= self.value.start())
            || (value.end() >= self.value.end() && value.start() <= self.value.end())
    }

    pub fn _get_which(&mut self, which: bool) -> &mut Option<Box<Self>> {
        if which { &mut self.gt } else { &mut self.lt }
    }

    /// See [`Self::_swap_ptrs_with()`] for the ptr swap, now lenths.
    ///
    ///     self        self
    ///     (A)         (B)
    ///   (B)  C  -->  D  (A)
    ///  D   E           E   C
    ///
    /// stays the same: C, D, E
    /// A = B + C; B = D + E (C = A - B)
    /// A = E + C; B = D + A
    ///
    /// B' = B - E + A' = A
    /// A' = A - B + E
    pub fn _swap_with(self: &mut Box<Self>, which: bool) {
        unsafe fn copy_mut<'a, T>(mutref: *mut T) -> &'a mut T {
            unsafe { &mut *mutref }
        }

        let total_len = self.len;
        let e_len = self._swap_ptrs_with(which);
        // SAFETY: its different fields
        let sub = unsafe { copy_mut(self._get_which(which).as_mut().unwrap()) };

        self.len -= sub.len + e_len;
        sub.len = total_len;
    }

    /// True if the gt one, false if lt
    ///
    ///     self        self
    ///     (A)         (B)
    ///     a b         x y
    ///   (B)  C  -->  D  (A)
    ///   x y             a b
    ///  D   E           E   C
    ///
    /// `self` here is the **pointer to the pointer** to the root element
    ///  a = &E (y)
    ///  y = &A (self)
    ///  self = B (a)
    ///
    ///  also returns `E`s length if existent or 0
    pub fn _swap_ptrs_with(self: &mut Box<Self>, which: bool) -> usize {
        unsafe fn copy<T>(optbox: &T) -> T {
            unsafe { std::mem::transmute_copy::<_, T>(optbox) }
        }
        unsafe fn copy_mut<'a, T>(mutref: *mut T) -> &'a mut T {
            unsafe { &mut *mutref }
        }

        // SAFETY: we clone the mut ptr to separate their lifetime relation, its fine as long as we
        // dont walk into the `&mut Option<...>`s (`lt` and `gt` field ptrs) and treat it only as
        // the node field's ptr to be updated
        let my_sub_field_ptr = unsafe { copy_mut(self._get_which(which)) }; // a
        let my_sub_ptr = unsafe { copy_mut(my_sub_field_ptr.as_mut().unwrap()) }; // B
        let my_subs_other_sub_field_ptr = unsafe { copy_mut(my_sub_ptr._get_which(!which)) }; // y

        // SAFETY: Copied because it can't move out as the original location must remain valid.
        // However this is a triangular rotation and the final place will end up being replaced
        // too, so this is safe.
        *my_sub_field_ptr = unsafe { copy(my_subs_other_sub_field_ptr) };
        *my_subs_other_sub_field_ptr = Some(unsafe { copy(self) });
        *self = unsafe { copy(my_sub_ptr) };

        my_sub_field_ptr.as_ref().map(|n| n.len).unwrap_or(0)
    }

    /// Returns `true` if the len count has grown somewhere down the line
    pub fn _insert(self: &mut Box<Self>, value: RangeInclusive<T>) -> bool {
        if self._overlap(&value) {
            self.value = self._overlap_with(&value);
            return false;
        }

        let is_gt = Self::_is_gt_non_overlapping(&self.value, &value);
        let next_node: &mut _ = if is_gt { &mut self.gt } else { &mut self.lt };

        if let Some(next_node) = next_node {
            let inserted = next_node._insert(value);
            if inserted {
                self.len += 1;
            }

            // first of all, do we balance?
            // maybe >= 2 will balance more but itd be too unstable, TODO play with constant
            if (self.len + 1) / (next_node.len + 1) > 2 {
                // self._swap_with(is_gt);
            }

            inserted
        } else {
            *next_node = Some(Box::new(Self::new(value)));
            self.len += 1;
            true
        }
    }

    pub fn insert(self: &mut Box<Self>, value: RangeInclusive<T>) {
        self._insert(value);
    }

    pub fn contains(&self, value: &T) -> bool {
        if self.value.start() > value {
            self.lt.as_ref().is_some_and(|lt| lt.contains(value))
        } else if self.value.end() < value {
            self.gt.as_ref().is_some_and(|gt| gt.contains(value))
        } else {
            true
        }
    }
}

struct BIntervalTree<T: Ord + Copy> {
    first_node: Box<BIntervalNode<T>>,
}

impl<T: Ord + Copy> BIntervalTree<T> {
    fn insert(&mut self, value: RangeInclusive<T>) {
        self.first_node.insert(value);
    }

    fn contains(&self, value: &T) -> bool {
        self.first_node.contains(value)
    }
}

impl<A: Ord + Copy> FromIterator<RangeInclusive<A>> for BIntervalTree<A> {
    /// # Panic
    ///
    /// Must not be an empty iterator (this is not design I just want perf on this)
    fn from_iter<T: IntoIterator<Item = RangeInclusive<A>>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        let mut myself = Self {
            first_node: Box::new(BIntervalNode::new(iter.next().unwrap())),
        };

        for range in iter {
            myself.insert(range);
        }
        myself
    }
}

#[unsafe(no_mangle)]
extern "Rust" fn challenge_t_usize(buf: &[u8], t: &Instant) -> usize {
    let s = unsafe { str::from_utf8_unchecked(buf) };
    let mut lines = s.lines();

    let mut count = 0;

    let ran_iter = (&mut lines).take_while(|ln| !ln.is_empty()).map(|range| {
        let (l, r) = range.split_once('-').unwrap();
        let (l, r) = (l.parse().unwrap(), r.parse().unwrap());
        l..=r
    });

    let ranges = BIntervalTree::<RangeU>::from_iter(ran_iter);

    unsafe { TIMERS[0].1 = t.elapsed() };

    for id in lines {
        let id: RangeU = id.parse().unwrap();

        if ranges.contains(&id) {
            count += 1;
        }
    }

    unsafe { TIMERS[1].1 = t.elapsed() };

    count
}
