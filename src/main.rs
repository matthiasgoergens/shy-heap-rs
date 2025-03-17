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

pub mod pairing;
pub mod schubert;
pub mod trees;

fn main() {
    println!("Hello, world!");
}
