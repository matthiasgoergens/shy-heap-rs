// Run as
//  RUST_MIN_STACK=16777216 cargo run --release --example run

// const EVERY: usize = 2;
const EVERY: usize = 8;
// const ELOG: usize = EVERY.next_power_of_two().ilog2() as usize;

use std::cmp::max;

use itertools::enumerate;
use rand::{seq::SliceRandom, Rng};
use seq_macro::seq;
use softheap::{pairing::SoftHeap, tools::with_counter}; // Add import for seq_macro

pub fn one_batch() {
    // let n = 10_000_000;
    // const EVERY: usize = 6;
    // const ELOG: usize = EVERY.next_power_of_two().ilog2() as usize;
    // const EXPECTED_CORRUPTED_FRACTION: f64 = RAW / (1.0-RAW);
    // Wrong formula.
    // const EXPECTED_CORRUPTED_FRACTION: f64 = 1.0 / (EVERY as f64 - ELOG as f64);
    // const EXPECTED_CORRUPTED_FRACTION: f64 = 1.0 / (3.0 * EVERY as f64 - ELOG as f64 - 2.0);
    // const EXPECTED_CORRUPTED_FRACTION: f64 = 1.0 / (EVERY as f64);
    println!("EVERY: {EVERY} one_batch random");
    for e in 0..30 {
        let n = 1 << e;

        let mut pairing: SoftHeap<EVERY, _> = SoftHeap::default();
        let mut x = (0..n).collect::<Vec<_>>();

        x.shuffle(&mut rand::rng());
        let (counter, x) = with_counter(x);
        for (_index, item) in enumerate(x) {
            pairing = pairing.insert(item);
            // pairing = pairing.insert(j);
        }

        let prep_count = counter.get();
        let count_ration = prep_count as f64 / (n as f64 - 1.0);
        print!("cmp: {prep_count:10}\tcmp_ratio: {count_ration:10.6}\t");

        let mut all_corrupted = 0;
        let mut max_corrupted = 0;

        // let mut c = 0;
        while !pairing.is_empty() {
            // c += 1;
            let (new_pairing, _item, newly_corrupted) = pairing.heavy_delete_min();
            // let (new_pairing, _item, newly_corrupted) = pairing.delete_min();
            all_corrupted += newly_corrupted.len();
            pairing = new_pairing;
            max_corrupted = max(max_corrupted, pairing.count_corrupted());
            // if !newly_corrupted.is_empty() {
            //     print!(
            //         "{c}\tTotal corrupted: {all_corrupted}\tNewly corrupted: {}\t",
            //         newly_corrupted.len()
            //     );
            //     let corrupted_fraction = all_corrupted as f64 / n as f64;
            //     println!("Corrupted fraction: {:.2}%", corrupted_fraction * 100.0);
            // }
            // println!(
            //     "Total corrupted: {all_corrupted}\tUncorrupted: {}\tCorrupted: {}",
            //     pairing.count_uncorrupted(),
            //     pairing.count_corrupted()
            // );
        }
        let ever_corrupted_fraction = all_corrupted as f64 / n as f64;
        print!(
            "crp: {all_corrupted:10}\tEver crp ratio: {:8.5}%\texpo: {e:3}\tn: {n:10}\t",
            ever_corrupted_fraction * 100.0
        );
        // let work = counter.get() as f64 / n as f64;
        let max_corrupted_fraction = max_corrupted as f64 / n as f64;
        let remaining_work = counter.get() - prep_count;
        println!(
            "Max crp frac: {:8.5}%\trem work: {:10.6}\trem work/n: {:10.6}\tlog-factor: {:10.6}",
            max_corrupted_fraction * 100.0,
            remaining_work,
            remaining_work as f64 / n as f64,
            remaining_work as f64 / n as f64 / ((n as f64).log2()),
        );
    }
    // println!(
    //     "Total corrupted: {all_corrupted}\tUncorrupted: {}\tCorrupted: {}",
    //     pairing.count_uncorrupted(),
    //     pairing.count_corrupted()
    // );
}

pub fn one_batch_meld() {
    // let n = 10_000_000;
    // const EXPECTED_CORRUPTED_FRACTION: f64 = RAW / (1.0-RAW);
    // Wrong formula.
    // const EXPECTED_CORRUPTED_FRACTION: f64 = 1.0 / (EVERY as f64 - ELOG as f64);
    println!("EVERY: {EVERY} one_batch random meld");
    for e in 0..28 {
        let n = 1 << e;

        let (counter, x) = with_counter((0..n).collect::<Vec<_>>());
        let mut x: Vec<SoftHeap<EVERY, _>> =
            x.into_iter().map(SoftHeap::singleton).collect::<Vec<_>>();

        while x.len() > 1 {
            let a = sample_swap_pop(&mut x);
            let b = sample_swap_pop(&mut x);
            x.push(a.meld(b));
        }
        let mut pairing = x.pop().unwrap();
        let prep_count = counter.get();
        assert!(x.is_empty());

        let count = counter.get();
        let count_ration = count as f64 / n as f64;
        print!("cmp: {count:10}\tcmp_ratio: {count_ration:10.6}\t");

        // let mut all_corrupted = 0;
        let mut max_corrupted = 0;

        // let mut c = 0;
        while !pairing.is_empty() {
            // c += 1;
            let (new_pairing, _item, _newly_corrupted) = pairing.heavy_delete_min();
            // let (new_pairing, _item, newly_corrupted) = pairing.delete_min();
            // all_corrupted += newly_corrupted.len();
            pairing = new_pairing;
            max_corrupted = max(max_corrupted, pairing.count_corrupted());
            // if !newly_corrupted.is_empty() {
            //     print!(
            //         "{c}\tTotal corrupted: {all_corrupted}\tNewly corrupted: {}\t",
            //         newly_corrupted.len()
            //     );
            //     let corrupted_fraction = all_corrupted as f64 / n as f64;
            //     println!("Corrupted fraction: {:.2}%", corrupted_fraction * 100.0);
            // }
            // println!(
            //     "Total corrupted: {all_corrupted}\tUncorrupted: {}\tCorrupted: {}",
            //     pairing.count_uncorrupted(),
            //     pairing.count_corrupted()
            // );
        }
        // let ever_corrupted_fraction = all_corrupted as f64 / n as f64;
        // print!(
        //     "Corrupted fraction: {:.2}%\texponent: {e}\tn: {n:10}\t",
        //     ever_corrupted_fraction * 100.0
        // );
        // println!(
        //     "Max corrupted fraction: {:6.5}%\t?< {:6.5}%",
        //     max_corrupted as f64 / n as f64 * 100.0,
        //     EXPECTED_CORRUPTED_FRACTION * 100.0
        // );
        let remaining_work = counter.get() - prep_count;
        println!(
            "\trem: {:10}\trem_ratio: {:11.6}\tlog-factor: {:10.6}",
            remaining_work,
            remaining_work as f64 / n as f64,
            remaining_work as f64 / n as f64 / ((n as f64).log2()),
        );
    }
    // println!(
    //     "Total corrupted: {all_corrupted}\tUncorrupted: {}\tCorrupted: {}",
    //     pairing.count_uncorrupted(),
    //     pairing.count_corrupted()
    // );
}

fn sample_swap_pop<T>(x: &mut Vec<T>) -> T {
    x.swap_remove(rand::rng().random_range(..x.len()))
}

pub fn interleave() {
    // let n = 10_000_000;
    const EVERY: usize = 4;
    println!("EVERY: {EVERY}");
    for e in 0..30 {
        let n = 1 << e;

        let mut pairing: SoftHeap<EVERY, _> = SoftHeap::default();

        let mut all_corrupted = 0;
        let mut non_corrupted_pops = 0;
        let mut c = 0;
        for _i in 0..n {
            c += 1;
            pairing = pairing.insert(c);
            // if pairing.root.as_ref().map(|r| r.children.len()).unwrap_or(0) >= EVERY {
            // if c % (EVERY-2) == 0 {
            if c % 2 == 0 {
                // while pairing.count_children() > 0 && pairing.count_children() % EVERY == 0 {
                let (new_pairing, item, newly_corrupted) = pairing.delete_min();
                all_corrupted += newly_corrupted.len();
                non_corrupted_pops += usize::from(item.is_some());
                pairing = new_pairing;
            }
            // }
        }
        let ever_corrupted_fraction = all_corrupted as f64 / c as f64;
        print!(
            "e: {e} n: {c}\tcurrent_corrupted: {:.2}\tsize: {}\tsize prop: {:.2}%\tCorrupted fraction (intermediate): {:.2}%\t\t\t",
            pairing.count_corrupted() as f64 / c as f64,
            pairing.size,
            pairing.size as f64 / c as f64 * 100.0,
            ever_corrupted_fraction * 100.0,
        );
        while !pairing.is_empty() {
            // c += 1;
            let (new_pairing, item, newly_corrupted) = pairing.delete_min();
            all_corrupted += newly_corrupted.len();
            non_corrupted_pops += usize::from(item.is_some());
            pairing = new_pairing;
            // if !newly_corrupted.is_empty() {
            //     print!(
            //         "{c}\tTotal corrupted: {all_corrupted}\tNewly corrupted: {}\t",
            //         newly_corrupted.len()
            //     );
            //     let corrupted_fraction = all_corrupted as f64 / n as f64;
            //     println!("Corrupted fraction: {:.2}%", corrupted_fraction * 100.0);
            // }
            // println!(
            //     "Total corrupted: {all_corrupted}\tUncorrupted: {}\tCorrupted: {}",
            //     pairing.count_uncorrupted(),
            //     pairing.count_corrupted()
            // );
        }
        let ever_corrupted_fraction = all_corrupted as f64 / c as f64;
        println!(
            "Corrupted fraction: {:.2}%\tn: {c}\tcheck_sum: {}",
            ever_corrupted_fraction * 100.0,
            (all_corrupted as i64) + (non_corrupted_pops as i64) - (c as i64)
        );
    }
    // println!(
    //     "Total corrupted: {all_corrupted}\tUncorrupted: {}\tCorrupted: {}",
    //     pairing.count_uncorrupted(),
    //     pairing.count_corrupted()
    // );
}

pub fn interleave1<const EVERY: usize>() -> f64 {
    // let n = 10_000_000;
    // const EVERY: usize = 15;
    // println!("EVERY: {EVERY}");
    let e: usize = 22;

    let n = 1 << e;

    let mut pairing: SoftHeap<EVERY, _> = SoftHeap::default();

    let mut all_corrupted = 0;
    let mut _non_corrupted_pops = 0; // Changed to _non_corrupted_pops
    let mut c = 0;
    for _i in 0..n {
        c += 1;
        pairing = pairing.insert(c);
        // if pairing.root.as_ref().map(|r| r.children.len()).unwrap_or(0) >= EVERY {
        // if c % (EVERY-2) == 0 {
        if c % 2 == 0 {
            // while pairing.count_children() > EVERY && pairing.count_children() % EVERY == 0 {
            // while pairing.count_children() > EVERY {
            let (new_pairing, item, newly_corrupted) = pairing.delete_min();
            all_corrupted += newly_corrupted.len();
            _non_corrupted_pops += usize::from(item.is_some()); // Changed to _non_corrupted_pops
            pairing = new_pairing;
        }
    }
    while !pairing.is_empty() {
        let (new_pairing, item, newly_corrupted) = pairing.delete_min();
        all_corrupted += newly_corrupted.len();
        _non_corrupted_pops += usize::from(item.is_some()); // Changed to _non_corrupted_pops
        pairing = new_pairing;
    }
    all_corrupted as f64 / c as f64
}

// The run_for_range macro is no longer needed and can be removed.

pub fn sort1<const EVERY: usize>() -> (f64, f64) {
    // let n = 10_000_000;
    // const EVERY: usize = 15;
    // println!("EVERY: {EVERY}");
    let e: usize = 23;

    let n = 1 << e;
    let mut max_corrupted = 0;

    let mut pairing: SoftHeap<EVERY, _> = SoftHeap::default();

    let mut all_corrupted = 0;
    let mut _non_corrupted_pops = 0; // Changed to _non_corrupted_pops
    let mut x = (0..n).collect::<Vec<_>>();
    x.shuffle(&mut rand::rng());
    for i in x {
        pairing = pairing.insert(i);
        max_corrupted = max(max_corrupted, pairing.count_corrupted());
    }
    while !pairing.is_empty() {
        let (new_pairing, item, newly_corrupted) = pairing.delete_min();
        all_corrupted += newly_corrupted.len();
        _non_corrupted_pops += usize::from(item.is_some()); // Changed to _non_corrupted_pops
        pairing = new_pairing;
        max_corrupted = max(max_corrupted, pairing.count_corrupted());
    }
    (
        all_corrupted as f64 / n as f64,
        max_corrupted as f64 / n as f64,
    )
}

pub fn interleave_n() {
    println!(" N\tCorrupted fraction\tlog2(1/N)\tlog2(N)\tN*result");
    seq!(N in 2..=256 {
        let result = interleave1::<N>();
        let lresult = -result.log2();
        let l_n = (N as f64).log2();
        let x = N as f64 * result;
        println!("{:2}\t\t{:6.2}%\t{lresult:6.2}\t{l_n:6.2}\t{x:6.2}", N, result * 100.0);
    });
}

pub fn sort_n() {
    println!("Sort N");
    println!(" N\tCorrupted fraction\tlog2(1/N)\tlog2(N)\tN*result\tmax_frac\tmax_frac*N");
    seq!(N in 2..=256 {
        let (result, max_frac) = sort1::<N>();
        let lresult = -result.log2();
        let l_n = (N as f64).log2();
        let x = N as f64 * result;
        let xx = max_frac * N as f64;
        println!("{:2}\t\t{:6.2}%\t{lresult:6.2}\t{l_n:6.2}\t{x:6.2}\t{:6.2}\t{xx:6.2}", N, result * 100.0, max_frac * 100.0);
    });
}

pub fn main() {
    one_batch();
    // interleave();
    // interleave_n();
    // sort_n();
    // one_batch_meld();
}
