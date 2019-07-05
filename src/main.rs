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

use troika_rust::troika::Troika;
use troika_rust::ftroika::Ftroika;
use troika_rust::stroika::{Stroika, Strit};
use std::time::SystemTime;
//use std::cmp::min;
//use std::arch;
//#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))]
#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(unused_variables)]
#[allow(unused_assignments)]
fn main(){
    // testin REAL SIMD - seems it doesn't work properly ?!
    let x; let y; let z:__m256i;
    unsafe {
        y = _mm256_setr_epi64x(8888, 777, 66, 5);
    }
    println!("SIMD_TEST {:?}",y);
    unsafe{
        //x = _mm256_setr_epi64x(100,200,3000,4000);
        x = y.clone();
        println!("SIMD_TEST {:?}",x);
        z = _mm256_add_epi64(x, y);
        //z = x | y;
    }
    //let x = arch::x86_64::_mm256_add_epi64([1,2,3,4] arch::x86_64::__m2 ,[5,6,7,8]); */
    println!("REAL SIMD HAS TO WAIT, FOR NOW ONLY 64/32 BIT\n");

    const IN_SIZE: usize = 8019;
    const OUT_SIZE: usize = 243;
    const FEEDBACK: usize = 243;

    let mut troika = Troika::default();
    let mut input = [0u8; IN_SIZE];
    let mut output = [0u8; 243];
    let t1 = SystemTime::now();
    let loop_max = 225;
    for _ in 0..loop_max {
        troika.absorb(&input[..]);
        troika.squeeze(&mut output);
        troika.reset();
        input[0..FEEDBACK].clone_from_slice(&output[0..FEEDBACK]);
    }
    let t2 = SystemTime::now().duration_since(t1);
    println!(" Troika took {}", t2.unwrap().as_millis());

    let mut ftroika = Ftroika::default();
    let mut input2 = [0u8; IN_SIZE];
    let mut output2 = [0u8; 243];
    let t1 = SystemTime::now();
    for _ in 0..loop_max {
        ftroika.absorb(&input2[..IN_SIZE/2]);
        ftroika.absorb(&input2[IN_SIZE/2..]);
        ftroika.finalize();
        ftroika.squeeze(&mut output2[0..OUT_SIZE/3]);
        ftroika.squeeze(&mut output2[OUT_SIZE/3..OUT_SIZE/3*2]);
        ftroika.squeeze(&mut output2[OUT_SIZE/3*2..]);
        ftroika.reset();
        input2[0..FEEDBACK].clone_from_slice(&output2[0..FEEDBACK]);
    }
    let t2 = SystemTime::now().duration_since(t1);
    println!("fTroika took {}", t2.unwrap().as_millis());

    let mut stroika = Stroika::default();
    let mut input3 = [Strit::ZERO(); IN_SIZE];
    let mut output3 = [Strit::ZERO(); 243];
    let t1 = SystemTime::now();
    for _ in 0..loop_max {
        stroika.absorb(&input3[..IN_SIZE/4]);
        stroika.absorb(&input3[IN_SIZE/4..]);
        stroika.finalize();
        stroika.squeeze(&mut output3[..OUT_SIZE/4*3]);
        stroika.squeeze(&mut output3[OUT_SIZE/4*3..]);
        stroika.reset();
        input3[0..FEEDBACK].clone_from_slice(&output3[0..FEEDBACK]);
    }
    let t2 = SystemTime::now().duration_since(t1);
    println!("sTroika took {}", t2.unwrap().as_millis());

    for i in 0..OUT_SIZE {
        print!("{}{}{:?},", output[i], output2[i], output3[i]);
    }
    assert!(
        output.iter().zip(output2.iter().zip(output3.iter())).all(|(a, (b, c))|
            (a == b) && *b == c.to_u8()),
        "Arrays are not equal"
    );
    let _bla = 1;
}