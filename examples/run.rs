use softheap::pairing::SoftHeap;

pub fn one_batch() {
    // let n = 10_000_000;
    for e in 0..25 {
        let n = 1 << e;

        let mut pairing: SoftHeap<3, _> = SoftHeap::default();
        for i in 0..n {
            pairing = pairing.insert(i);
        }
        let mut all_corrupted = 0;

        // let mut c = 0;
        while !pairing.is_empty() {
            // c += 1;
            let (new_pairing, _item, newly_corrupted) = pairing.delete_min();
            all_corrupted += newly_corrupted.len();
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
        let ever_corrupted_fraction = all_corrupted as f64 / n as f64;
        println!(
            "Corrupted fraction: {:.2}%\tn: {n}",
            ever_corrupted_fraction * 100.0
        );
    }
    // println!(
    //     "Total corrupted: {all_corrupted}\tUncorrupted: {}\tCorrupted: {}",
    //     pairing.count_uncorrupted(),
    //     pairing.count_corrupted()
    // );
}

pub fn interleave() {
    // let n = 10_000_000;
    const EVERY: usize = 15;
    println!("EVERY: {EVERY}");
    for e in 0..27 {
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
            // if c % 2 == 0 {
            while pairing.count_children() > 0 && pairing.count_children() % EVERY == 0 {
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

pub fn main() {
    // one_batch();
    interleave();
}
