#[allow(dead_code)]


#[macro_use]
extern crate clap;
use clap::{App, AppSettings, Arg};
use std::cmp::{max, min};

#[derive(Clone, Copy)]
struct MyU128 {
    hi: u64,
    lo: u64,
}

impl MyU128 {
    pub fn new(lo: u64) -> Self {
        MyU128 { hi: 0, lo: lo }
    }

    pub fn add_one(&self) -> Self {
        if self.lo == u64::max_value() {
            MyU128 {
                hi: self.hi + 1,
                lo: 0,
            }
        } else {
            MyU128 {
                hi: self.hi,
                lo: self.lo + 1,
            }
        }
    }

    pub fn bit_count(&self) -> usize {
        let mut r: usize = 0;
        let mut tmp = self.lo;
        if self.hi != 0 {
            r += 64;
            tmp = self.hi;
        }
        while tmp != 0 {
            r += 1;
            tmp >>= 1;
        }
        r
    }
}

use std::ops::*;

impl BitAnd for MyU128 {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        MyU128 {
            hi: self.hi & rhs.hi,
            lo: self.lo & rhs.lo,
        }
    }
}

impl Not for MyU128 {
    type Output = MyU128;
    fn not(self) -> Self {
        MyU128 {
            hi: !self.hi,
            lo: !self.lo,
        }
    }
}
impl BitXorAssign for MyU128 {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.hi ^= rhs.hi;
        self.lo ^= rhs.lo;
    }
}

impl Shr<usize> for MyU128 {
    type Output = Self;
    fn shr(self, rhs: usize) -> Self {
        // Need to get the bits from the high side
        let hi = self.hi >> rhs;
        let mut lo = self.lo >> rhs;
        lo |= (self.hi & ((1 << rhs) - 1)) << 64 - rhs;
        Self { hi: hi, lo: lo }
    }
}

impl BitOr for MyU128 {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        MyU128 {
            hi: self.hi | rhs.hi,
            lo: self.lo | rhs.lo,
        }
    }
}

struct Kolakoski {
    x: MyU128,
    y: MyU128,
    idx: u64,
    count: u64,
    deviation: f64,
}

impl Kolakoski {
    pub fn new() -> Self {
        Kolakoski {
            x: MyU128::new(0),
            y: MyU128::new(0),
            idx: 1,
            count: 1,
            deviation: 0.0,
        }
    }

    pub fn next(&mut self) {
        self.idx += 1;
        self.count += self.x.lo & 1; // This is 0 if it's a 2, 1 if it's a 1
        let f: MyU128 = self.y & !self.y.add_one();
        self.x ^= f;
        self.y = self.y.add_one() | (f & (self.x >> 1));
        // Calculate deviation
        let diff = max(self.count, self.idx / 2) - min(self.count, self.idx / 2);
        let dev = diff as f64 / self.idx as f64;
        self.deviation = if self.deviation < dev {
            dev
        } else {
            self.deviation
        };
    }

    /// Batch update the state
    pub fn step(&mut self, steps: u64) {
        for _ in 0..steps {
            self.next();
        }
    }

    /// Update state to the given index
    pub fn goto(&mut self, index: u64) {
        if index < self.idx {
            panic!();
        }
        let steps = index - self.idx;
        self.step(steps);
    }

    pub fn reset_deviation(&mut self) {
        self.deviation = 0.0;
    }
}

fn main() {
    let matches = App::new("kolakoski")
        .global_settings(&[AppSettings::ColoredHelp])
        .version(crate_version!())
        .author("Patrick Jackson <PatrickSJackson@gmail.com>")
        .about("counts the distribution of 1's in the Kolakoski Sequence")
        .arg(
            Arg::with_name("VALUE")
                .help("Sets the value to find")
                .required(false)
                .default_value("1000000000")
                .index(1)
                .multiple(false),
        )
        .get_matches();

    let target: u64 = matches
        .value_of("VALUE")
        .unwrap()
        .parse::<u64>()
        .expect("Must be a number.");

    let mut k = Kolakoski::new();
    let mut next_print = 10 as u64;
    while k.idx < target {
        next_print = std::cmp::min(next_print, target);
        k.goto(next_print);
        println!(
            "{}, {}, {}, {}, {}",
            k.idx,
            k.count,
            k.deviation,
            k.x.bit_count(),
            k.y.bit_count()
        );
        k.reset_deviation();
        next_print *= 10;
    }
}
