use softheap::pairing::SoftHeap;

pub fn main() {
    let n = 1_000_000;
    let mut pairing: SoftHeap<3, _> = SoftHeap::default();
    for i in 0..n {
        pairing = pairing.insert(i);
    }
    let mut all_corrupted = 0;

    let mut c = 0;
    while !pairing.is_empty() {
        c += 1;
        let (new_pairing, _item, newly_corrupted) = pairing.delete_min();
        all_corrupted += newly_corrupted.len();
        pairing = new_pairing;
        if !newly_corrupted.is_empty() {
            print!(
                "{c}\tTotal corrupted: {all_corrupted}\tNewly corrupted: {}\t",
                newly_corrupted.len()
            );
            let corrupted_fraction = all_corrupted as f64 / n as f64;
            println!("Corrupted fraction: {:.2}%", corrupted_fraction * 100.0);
        }
        // println!(
        //     "Total corrupted: {all_corrupted}\tUncorrupted: {}\tCorrupted: {}",
        //     pairing.count_uncorrupted(),
        //     pairing.count_corrupted()
        // );
    }
    // println!("Corrupted: {all_corrupted}");
}
