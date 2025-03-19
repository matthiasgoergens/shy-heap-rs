/*

Simple soft heaps

We offer:

insert
delete-min (but not pop)
meld

convert-to-vector

We use fixed size ordered vectors as the underlying data storage.

---

Update:

Use a tree representation for simplicity.  Don't give a 'convert-to-vector' function, but let people register a callback to get notified about corruption / deletion.

(I we could track non-corrupt deletion?)

Hmm, I think we need to 'box' our iterators?  Just so that the type stays the same, no matter the 'level' of our iterator in the tree.

We may need `peekable` to put stuff back?

Or perhaps use Python?  Easier for more people to read?

*/

pub mod lazy_pairing;
pub mod pairing;
pub mod schubert;
pub mod trees;

use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    if a == 1 {
        panic!("I don't like 1");
    }
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn softheap(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}
