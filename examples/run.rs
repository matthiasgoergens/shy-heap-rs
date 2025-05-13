use softheap::pairing::SoftHeap;

pub fn main() {
    let n = 1_000_000;
    let mut pairing: SoftHeap<100, _> = SoftHeap::default();
    for i in 0..n {
        pairing = pairing.insert(i);
    }
    let mut all_corrupted = 0;

    let mut c = 0;
    while pairing.count_uncorrupted() > 0 {
        c += 1;
        let (new_pairing, newly_corrupted) = pairing.delete_min();
        all_corrupted += newly_corrupted.len();
        pairing = new_pairing;
        println!(
            "{c}\tTotal corrupted: {all_corrupted}",
            // pairing.count_uncorrupted(),
            // pairing.count_corrupted()
        );
        // println!(
        //     "Total corrupted: {all_corrupted}\tUncorrupted: {}\tCorrupted: {}",
        //     pairing.count_uncorrupted(),
        //     pairing.count_corrupted()
        // );
    }
    // println!("Corrupted: {all_corrupted}");
}
