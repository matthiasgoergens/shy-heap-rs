// Run as
//  RUST_MIN_STACK=16777216 cargo run --release --example run

// const EVERY: usize = 2;
const EVERY: usize = 3;
// const ELOG: usize = EVERY.next_power_of_two().ilog2() as usize;
const MAX_THREADS: usize = 32; // Configurable number of processors for parallel execution

use std::cmp::max;

use itertools::enumerate;
use rand::{seq::SliceRandom, Rng};
use rayon::prelude::*;
use seq_macro::seq;
use softheap::{
    pairing::{Pairing, SoftHeap, UnboundWitnessed},
    tools::with_counter,
    witness_set::{Witnessed, WitnessedSet},
}; // Add import for seq_macro

pub fn dbg() {
    let a: UnboundWitnessed<i32> = {
        let mut to_be_witnessed = WitnessedSet::<i32>::default();
        to_be_witnessed.add_child(Witnessed::singleton(1));
        UnboundWitnessed {
            to_be_witnessed,
            pairing: Pairing::new(2),
        }
    };

    let b: UnboundWitnessed<i32> = {
        let mut to_be_witnessed = WitnessedSet::<i32>::default();
        to_be_witnessed.add_child(Witnessed::singleton(3));
        UnboundWitnessed {
            to_be_witnessed,
            pairing: Pairing::new(4),
        }
    };
    println!("a: {a:#?}",);
    println!("b: {b:#?}",);
    let c = a.meld(b);
    println!("a.meld(b): {c:#?}",);
}

pub fn one_batch_db() {
    println!("EVERY: {EVERY} one_batch debug");
    let e = 15;
    let n = 1 << e;
    let mut pairing: SoftHeap<_> = SoftHeap::new(EVERY);
    let mut x = (0..n).collect::<Vec<_>>();
    x.shuffle(&mut rand::rng());
    let (_counter, x) = with_counter(x);
    for (_index, item) in enumerate(x) {
        pairing = pairing.insert(item);
    }
    while !pairing.is_empty() {
        let (new_pairing, _pool, corrupted) = pairing.heavy_pop_min();
        pairing = new_pairing;
        let delayed = pairing.count_delayed_corruption();
        if delayed > 0 {
            println!("left: {}, delayed: {}", pairing.size, delayed);
        }
        if !corrupted.is_empty() {
            println!("Newly corrupted: {}\t", corrupted.len());
        }
        // println!("delayed: {}", pairing.count_delayed_corruption());
    }

    // let prep_count = counter.get();
    // println!("{:#?}", pairing);
}

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

        let mut pairing: SoftHeap<_> = SoftHeap::new(EVERY);
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
            let (new_pairing, _item, newly_corrupted) = pairing.heavy_pop_min();
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
        let mut x: Vec<SoftHeap<_>> = x
            .into_iter()
            .map(|item| SoftHeap::singleton(EVERY, item))
            .collect();

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
            let (new_pairing, _item, _newly_corrupted) = pairing.heavy_pop_min();
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

        let mut pairing: SoftHeap<_> = SoftHeap::new(EVERY);

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
                let (new_pairing, item, newly_corrupted) = pairing.pop_min();
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
            let (new_pairing, item, newly_corrupted) = pairing.pop_min();
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

    let mut pairing: SoftHeap<_> = SoftHeap::new(EVERY);

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
            let (new_pairing, item, newly_corrupted) = pairing.pop_min();
            all_corrupted += newly_corrupted.len();
            _non_corrupted_pops += usize::from(item.is_some()); // Changed to _non_corrupted_pops
            pairing = new_pairing;
        }
    }
    while !pairing.is_empty() {
        let (new_pairing, item, newly_corrupted) = pairing.pop_min();
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

    let mut pairing: SoftHeap<_> = SoftHeap::new(EVERY);

    let mut all_corrupted = 0;
    let mut _non_corrupted_pops = 0; // Changed to _non_corrupted_pops
    let mut x = (0..n).collect::<Vec<_>>();
    x.shuffle(&mut rand::rng());
    for i in x {
        pairing = pairing.insert(i);
        max_corrupted = max(max_corrupted, pairing.count_corrupted());
    }
    while !pairing.is_empty() {
        let (new_pairing, item, newly_corrupted) = pairing.pop_min();
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

/// Generic parallel wrapper that runs a function over a range in parallel
/// and prints results in order as they complete
///
/// # Arguments
/// * `range` - The range of values to process (must implement IntoParallelIterator)
/// * `f` - A function that takes a usize and returns a String to be printed
///
/// # Example
/// ```
/// run_parallel(0..10, |i| format!("Processed item {}\n", i));
/// ```
///
/// This will process items 0-9 in parallel using MAX_THREADS threads,
/// and print the results in order (0, 1, 2, ...) as they complete.
pub fn run_parallel<F, R>(range: R, f: F)
where
    F: Fn(usize) -> String + Send + Sync,
    R: IntoParallelIterator<Item = usize> + Send,
    R::Iter: IndexedParallelIterator,
{
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Shared state for tracking completed results and next expected result
    let completed_results: Arc<Mutex<HashMap<usize, String>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let next_to_print: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

    // Create a thread pool with the specified number of threads
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(MAX_THREADS)
        .build()
        .unwrap();

    // Process the range in parallel
    pool.install(|| {
        range.into_par_iter().for_each(|e| {
            let result = f(e);

            // Store the result and try to print in order
            {
                let mut completed = completed_results.lock().unwrap();
                completed.insert(e, result);

                // Try to print as many results as possible in order
                let mut next = next_to_print.lock().unwrap();
                while let Some(result) = completed.get(&*next) {
                    print!("{result}");
                    completed.remove(&*next);
                    *next += 1;
                }
            }
        });
    });
}

pub fn one_batch_parallel() {
    println!("EVERY: {EVERY} one_batch random (parallel with {MAX_THREADS} threads)");

    run_parallel(0..25, |e| {
        let n = 1 << e;

        let mut pairing: SoftHeap<_> = SoftHeap::new(EVERY);
        let mut x = (0..n).collect::<Vec<_>>();

        x.shuffle(&mut rand::rng());
        let (counter, x) = with_counter(x);
        for (_index, item) in enumerate(x) {
            pairing = pairing.insert(item);
        }

        let prep_count = counter.get();
        let _count_ratio = prep_count as f64 / (n as f64 - 1.0);

        let mut all_corrupted = 0;
        let mut max_corrupted = 0;

        while !pairing.is_empty() {
            let (new_pairing, _item, newly_corrupted) = pairing.heavy_pop_min();
            all_corrupted += newly_corrupted.len();
            pairing = new_pairing;
            max_corrupted = max(max_corrupted, pairing.count_corrupted());
        }

        let ever_corrupted_fraction = all_corrupted as f64 / n as f64;
        let max_corrupted_fraction = max_corrupted as f64 / n as f64;
        let remaining_work = counter.get() - prep_count;
        let remaining_work_per_n = remaining_work as f64 / n as f64;
        let log_factor = remaining_work_per_n / ((n as f64).log2());

        let eps_work = -max_corrupted_fraction.log2();
        let eps_work_factor = remaining_work_per_n / eps_work;

        format!(
            "{e:3}:  Ever crp: {:8.5}%  Max crp: {:8.5}%  rem work/n: {:8.6}  log-factor: {:8.6}  eps_work_f: {:8.6}\n",
            ever_corrupted_fraction * 100.0,
            max_corrupted_fraction * 100.0,
            remaining_work_per_n,
            log_factor,
            eps_work_factor,
        )
    });
}

/// Helper function that contains the one_batch logic for a single exponent
fn one_batch_single(e: usize) -> String {
    let n = 1 << e;

    let mut pairing: SoftHeap<_> = SoftHeap::new(EVERY);
    let mut x = (0..n).collect::<Vec<_>>();

    x.shuffle(&mut rand::rng());
    let (counter, x) = with_counter(x);
    for (_index, item) in enumerate(x) {
        pairing = pairing.insert(item);
    }

    let prep_count = counter.get();
    let count_ratio = prep_count as f64 / (n as f64 - 1.0);

    let mut all_corrupted = 0;
    let mut max_corrupted = 0;

    while !pairing.is_empty() {
        let (new_pairing, _item, newly_corrupted) = pairing.heavy_pop_min();
        all_corrupted += newly_corrupted.len();
        pairing = new_pairing;
        max_corrupted = max(max_corrupted, pairing.count_corrupted());
    }

    let ever_corrupted_fraction = all_corrupted as f64 / n as f64;
    let max_corrupted_fraction = max_corrupted as f64 / n as f64;
    let remaining_work = counter.get() - prep_count;
    let remaining_work_per_n = remaining_work as f64 / n as f64;
    let log_factor = remaining_work_per_n / ((n as f64).log2());

    format!(
        "cmp: {prep_count:10}\tcmp_ratio: {count_ratio:10.6}\tcrp: {all_corrupted:10}\tEver crp ratio: {:8.5}%\texpo: {e:3}\tn: {n:10}\tMax crp frac: {:8.5}%\trem work: {:10.6}\trem work/n: {:10.6}\tlog-factor: {:10.6}\n",
        ever_corrupted_fraction * 100.0,
        max_corrupted_fraction * 100.0,
        remaining_work,
        remaining_work_per_n,
        log_factor,
    )
}

/// More flexible version that can work with any range
pub fn one_batch_parallel_range(start: usize, end: usize) {
    println!(
        "EVERY: {EVERY} one_batch random (parallel with {MAX_THREADS} threads, range {start}-{end})"
    );

    run_parallel(start..end, one_batch_single);
}

/// Generic parallel function that can work with any function
pub fn run_parallel_generic<F>(range: std::ops::Range<usize>, f: F, description: &str)
where
    F: Fn(usize) -> String + Send + Sync,
{
    println!(
        "{} (parallel with {} threads, range {}-{})",
        description, MAX_THREADS, range.start, range.end
    );

    run_parallel(range, f);
}

/// Simple test function to demonstrate the parallel wrapper
pub fn test_parallel() {
    println!("Testing parallel wrapper with simple computation...");

    run_parallel(0..10, |i| {
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(100 * (10 - i) as u64));
        format!("Processed item {} (took {}ms)\n", i, 100 * (10 - i))
    });
}

pub fn main() {
    one_batch_parallel();
    // test_parallel();  // Uncomment to test the parallel wrapper with simple computation
    // one_batch_parallel_range(0, 30);  // Same as above but with explicit range
    // run_parallel_generic(0..30, one_batch_single, "EVERY: {EVERY} one_batch random");  // Most generic version
    // one_batch();  // Original sequential version
    // interleave();
    // interleave_n();
    // sort_n();
    // one_batch_meld();
    // one_batch_db();
    // dbg();
}
