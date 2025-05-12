#[must_use]
pub fn previous_full_multiple(n: usize, m: usize) -> usize {
    (n + 1).next_multiple_of(m) - m
}
