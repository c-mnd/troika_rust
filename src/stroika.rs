/*
 * Copyright (c) 2019 c-mnd
 *
 * MIT License
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use super::macros::*;
use crate::Result;
use std::fmt;

/// The Stroika struct is a Sponge that uses the Stroika
/// hashing algorithm.
/// ```rust
/// extern crate troika_rust;
/// use troika_rust::stroika::{Stroika, Strit};
/// // Create an array of 243 1s
/// let input = [Strit::ZERO(); 243];
/// // Create an array of 243 0s
/// let mut out = [Strit::ZERO(); 243];
/// let mut stroika = Stroika::default();
/// stroika.absorb(&input[..]);
/// stroika.finalize();
/// stroika.squeeze(&mut out[..]);
/// ```

// uncomment Your target type
pub type Sbit = u64;
//pub type Sbit = u32;
//pub type Sbit = u16;
//pub type Sbit = u8;

#[derive(Clone, Copy)]
pub struct Strit {
    p: Sbit,
    n: Sbit,
}

impl fmt::Debug for Strit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /*write!(
            f,
            "Stroika: [ps: {}, n: {}",
            self.p,
            self.n,
        )*/
        if self.p != 0 {
            write!(f,"1",)
        } else if self.n != 0 { write!(f,"2",) }
        else { write!(f,"0",) }
    }
}

impl Default for Strit {
    fn default() -> Strit {
        Strit {
            p: 0,
            n: 0,
        }
    }
}

#[allow(dead_code)]
#[allow(non_snake_case)]
impl Strit {
    pub const fn ONE() -> Strit { Strit{p: !0, n: 0} }
    pub const fn TWO() -> Strit { Strit{p: 0, n: !0} }
    pub const fn ZERO() -> Strit { Strit{p: 0, n: 0} }
    pub const fn PADDING() -> Strit { Strit::ONE() }
    pub fn add(&self, other: &Strit) -> Strit {
        let self_zero = !self.p & !self.n;
        let p = !(self.n ^ other.n) & !(self_zero ^ other.p);
        let n = !(self.p ^ other.p) & !(self_zero ^ other.n);
        Strit {p, n}
    }
    pub fn mul(&self, other: &Strit) -> Strit {
        let p = (self.p & other.p) | (self.n & other.n);
        let n = (self.p & other.n) | (self.n & other.p);
        Strit {p, n}
    }
    pub fn dec(&self) -> Self {
        Self::TWO().add(&self)
    }
    pub fn inc(&self) -> Self {
        Self::ONE().add(&self)
    }
    pub fn to_u8(&self) -> u8 {
        if (self.p & 1) != 0 {
            return 1;
        } else if (self.n & 1) != 0 {
            return 2;
        }
        0
    }


}


pub struct Stroika {
    num_rounds: usize,
    idx: usize,
    state: [Strit; STATE_SIZE],
}

impl Default for Stroika {
    fn default() -> Stroika {
        Stroika {
            num_rounds: NUM_ROUNDS,
            idx: 0,
            state: [Strit::ZERO(); STATE_SIZE],
        }
    }
}

impl fmt::Debug for Stroika {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Stroika: [rounds: [{}], state: {:?}",
            self.num_rounds,
            self.state.to_vec(),
        )
    }
}

#[allow(dead_code)]
#[allow(non_snake_case)]
impl Stroika {
    pub fn new(num_rounds: usize) -> Result<Stroika> {
        let mut troika = Stroika::default();
        troika.num_rounds = num_rounds;
        Ok(troika)
    }

    pub fn state(&self) -> &[Strit] {
        &self.state[..]
    }

    pub fn reset(&mut self) {
        self.idx = 0;
        self.state = [Strit::ZERO(); STATE_SIZE];
    }

    pub fn absorb(&mut self, trits: &[Strit]) {
        let mut length = trits.len();
        let mut space: usize;
        let mut trit_idx = 0;
        while length > 0 {
            if self.idx == TROIKA_RATE{
                self.permutation();
                self.idx = 0;
                self.nullify_rate();
            }
            space = TROIKA_RATE - self.idx;
            if length < space {
                space = length;
            }
            for _ in 0..space {
                self.state[self.idx] = trits[trit_idx];
                self.idx += 1;
                trit_idx += 1;
            }
            length -= space;
        }
    }

    pub fn finalize(&mut self){
        let pad = [Strit::ONE(); 1];
        self.absorb(&pad);
        self.idx = TROIKA_RATE;
    }

    pub fn squeeze(&mut self, trits: &mut [Strit]) {
        let mut length = trits.len();
        let mut space: usize;
        let mut trit_idx = 0;
        while length > 0 {
            if self.idx == TROIKA_RATE{
                self.permutation();
                self.idx = 0;
            }
            space = TROIKA_RATE - self.idx;
            if length < space {
                space = length;
            }
            for _ in 0..space {
                trits[trit_idx] = self.state[self.idx];
                self.idx += 1;
                trit_idx += 1;
            }
            length -= space;
        }
    }

    pub fn permutation(&mut self) {
        assert!(self.num_rounds <= NUM_ROUNDS);

        for round in 0..self.num_rounds {
            self.sub_trytes();
            //self.shift_rows_lanes(); // YES it's not needed anymore
            self.add_column_parity();
            self.add_round_constant(round);
        }
    }

    fn sub_tryte(a: &mut [Strit]){
        let d = a[0].dec();
        let e = d.mul(&a[1]).add(&a[2]);
        let f = e.mul(&a[1]).add(&d);
        let g = e.mul(&f).add(&a[1]);
        a[2] = e;
        a[1] = f;
        a[0] = g;
    }

    fn sub_trytes(&mut self) {
        for i in (0..STATESIZE).step_by(3){
            Self::sub_tryte(&mut self.state[i..(i + 3)]);
        }
    }
    /*
    fn shift_rows_lanes(&mut self) {
        let mut new_state = [Strit::ZERO(); STATESIZE];
        for i in 0..STATESIZE {
            new_state[i] = self.state[STROIKA_SHIFT_ROWS_LANES[i] as usize];
        }
        self.state.clone_from_slice(&new_state);
    }

    fn add_column_parity_old(&mut self) {
        let mut parity = [Strit::ZERO(); SLICES * COLUMNS];
        // First compute parity for each column
        for slice in 0..SLICES {
            for col in 0..COLUMNS {
                let mut col_sum = Strit::ZERO();
                for row in 0..ROWS {
                    col_sum = col_sum.add(&self.state[SLICESIZE * slice + COLUMNS * row + col]);
                }
                parity[COLUMNS * slice + col] = parity[COLUMNS * slice + col].add(&col_sum);
            }
        }
        // Add parity
        for slice in 0..SLICES {
            for row in 0..ROWS {
                for col in 0..COLUMNS {
                    let idx = SLICESIZE * slice + COLUMNS * row + col;
                    let sum_to_add = parity[(col + 8) % 9 + COLUMNS * slice].add(
                        &parity[(col + 1) % 9 + COLUMNS * ((slice + 1) % SLICES)]);
                    self.state[idx] = self.state[idx].add(&sum_to_add);
                }
            }
        }
    }
    */
    fn add_column_parity(&mut self) {
        let mut new_state = [Strit::ZERO(); STATESIZE];
        let mut parity_next_slice = [Strit::ZERO(); COLUMNS];
        let mut parity_this_slice = [Strit::ZERO(); COLUMNS];
        let mut parity_left_neighbour: Strit;
        let mut parity_this: Strit;
        let mut sum_to_add: Strit;
        let slice: usize = 0;
        for col in 0..COLUMNS {
            parity_next_slice[col] = self.state[STROIKA_SHIFT_ROWS_LANES[SLICESIZE * slice + col] as usize]
                .add(&self.state[STROIKA_SHIFT_ROWS_LANES[SLICESIZE * slice + COLUMNS + col] as usize])
                .add(&self.state[STROIKA_SHIFT_ROWS_LANES[SLICESIZE * slice + COLUMNS * 2 + col] as usize]);
        }
        for slice in (0..SLICES).rev() {
            let col = COLUMNS - 1;
            parity_left_neighbour = self.state[STROIKA_SHIFT_ROWS_LANES[SLICESIZE * slice + col] as usize]
                .add(&self.state[STROIKA_SHIFT_ROWS_LANES[SLICESIZE * slice + COLUMNS + col] as usize])
                .add(&self.state[STROIKA_SHIFT_ROWS_LANES[SLICESIZE * slice + COLUMNS * 2 + col] as usize]);
            for col in 0..COLUMNS {
                parity_this = Strit::ZERO();
                for row in 0..ROWS {
                    let idx = SLICESIZE * slice + COLUMNS * row + col;
                    sum_to_add = parity_left_neighbour.add(&parity_next_slice[(col + 1) % 9]);
                    let x = self.state[STROIKA_SHIFT_ROWS_LANES[idx] as usize];
                    new_state[idx] = x.add(&sum_to_add);
                    parity_this = x.add(&parity_this);
                }
                parity_left_neighbour = parity_this;
                parity_this_slice[col] = parity_this;
            }
            parity_next_slice.clone_from_slice(&parity_this_slice[..]);
        }
        self.state.clone_from_slice(&new_state[..]);
    }

    fn add_round_constant(&mut self, round: usize) {
        for slice in 0..SLICES {
            for col in 0..COLUMNS {
                let idx = SLICESIZE * slice + col;
                if (FROUND_CONSTANTS[round][col][0] & (1u32 << slice)) != 0 {
                    self.state[idx] = self.state[idx].inc();
                } else if (FROUND_CONSTANTS[round][col][1] & (1u32 << slice)) != 0 {
                    self.state[idx] = self.state[idx].dec();
                }
            }
        }
    }

    fn nullify_rate(&mut self) {
        for i in 0..TROIKA_RATE {
            self.state[i] = Strit::ZERO();
        }
    }
}


#[cfg(test)]
mod test_troika {
    use super::*;

    const HASH: [u8; 243] = [
        0, 2, 2, 1, 2, 1, 0, 1, 2, 1, 1, 1, 1, 2, 2, 1, 1, 1, 0, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 2,
        1, 1, 1, 0, 1, 0, 2, 1, 0, 0, 0, 1, 2, 0, 2, 1, 0, 0, 2, 1, 1, 1, 1, 1, 2, 0, 1, 0, 2, 1,
        1, 2, 0, 1, 1, 1, 1, 1, 2, 2, 0, 0, 2, 2, 2, 2, 0, 0, 2, 2, 2, 1, 2, 2, 0, 2, 1, 1, 2, 1,
        1, 1, 2, 2, 1, 1, 0, 0, 0, 2, 2, 2, 0, 2, 1, 1, 1, 1, 0, 0, 1, 0, 2, 0, 2, 0, 2, 0, 0, 0,
        0, 1, 1, 1, 0, 2, 1, 1, 1, 0, 2, 0, 0, 1, 0, 1, 0, 2, 0, 2, 2, 0, 0, 2, 2, 0, 1, 2, 1, 0,
        0, 1, 2, 1, 1, 0, 0, 1, 1, 0, 2, 1, 1, 0, 1, 2, 0, 0, 0, 1, 2, 2, 1, 1, 1, 0, 0, 2, 0, 1,
        1, 2, 1, 1, 2, 1, 0, 1, 2, 2, 2, 2, 1, 2, 0, 2, 2, 1, 2, 1, 2, 1, 2, 2, 1, 1, 2, 0, 2, 1,
        0, 1, 1, 1, 0, 2, 2, 0, 0, 2, 0, 2, 0, 1, 2, 0, 0, 2, 2, 1, 1, 2, 0, 1, 0, 0, 0, 0, 2, 0,
        2, 2, 2,
    ];

    #[test]
    fn test_hash() {
        let mut troika = Stroika::default();
        let mut output = [Strit::ZERO(); 243];
        let input = [Strit::ZERO(); 243];
        troika.absorb(&input);
        troika.finalize();
        troika.squeeze(&mut output);

        assert!(
            output.iter().zip(HASH.iter()).all(|(a, b)| a.to_u8() == *b),
            "Arrays are not equal"
        );
    }
}
