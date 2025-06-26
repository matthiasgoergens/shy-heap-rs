- [X] Introduce the witness set idea from soft sequence heaps:
  Done, but only seems to be helping with constant factors, at the cost of implementation complexity.

- [ ] Try corruption only at the top.  Use soft heap selection paper idea to select the top k items in about O(k) and corrupt all of them.

***

# Corruption at the top

When processing more than some threshold `B` of children:

Finish multi-pass merge as usual, initialise a counter of work `c := number_of_children_just_processed.saturating_sub(B)` (saturating at 0).

Then go through the heap like https://arxiv.org/abs/1802.07041 **Selection from heaps, row-sorted matrices and X+Y using soft heaps**.  Go and prepare for selecting `c/B` items to corrupt.  But whenever we hit a node with more than X children (eg X=2), first we run a (recursive) merge-children on them and add the work done to the counter `c` (like in the original, though perhaps without the `saturating_sub`).

Funny enough, to run the whole thing we need a soft heap itself.  But it doesn't need to be configurable.  So we can use one with the previous rule (or we can even try using the same rule, if work gets smaller quickly enough?)
