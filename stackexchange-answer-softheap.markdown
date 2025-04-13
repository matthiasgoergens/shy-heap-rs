Yes, this problem is solvable in linear time.

We will describe an algorithm and sketch a proof of its runtime.

First, let reduce our concrete problem to a more abstract one:

If there are $n$ items of homework, Alice can use bucket-sort to order them by their deadlines in $O(n)$ time.  That's because any item with a deadline further out than $n$ can be treated as having a deadline of $n$.  W.l.o.g. that's also the assumption we make from here on.

Now let's do the actual reduction to a sequence of heap operations in $O(n)$ time:
- Initialise an empty sequence of operations $s$.
- Count our days as $d$ down from $n$ to $1$:
  - For each task $t$ with $\text{deadline}(t) = d$, append $\text{Insert}(t)$ to $s$.
  - Afterwards append $\text{DeleteMax}$ to $s$, regardless of whether the previous step added any items for this day.

We could now run the sequence of operations $s$ on a binary heap, and the items left over in the heap at the end would be the items that Alice won't do before their deadlines.  The algorithm also implicitly matches each deleted item with a $\text{DeleteMax}$ operation, which a bit more book-keeping can turn into a schedule.  In general, if you know which items are deleted or kept, you can reconstruct a schedule in linear time, even if you don't know the matching. (We assume that $\text{DeleteMax}$ is ignored if the heap is empty.)

Our hope is that we can forecast the result of these $n$ heap operations in $O(n)$ instead.  This is less outlandish than it might seem at first, because executing them one by one is an online algorithm, but we actually know all the operations in advance and only need to learn the final state of the heap.  So an offline algorithm might be faster.

Here is where I have to pull the first rabbit out of a hat: [soft heaps](https://en.wikipedia.org/wiki/Soft_heap).

# Soft Heaps

## Introduction

A soft heap is a data structure that allows us to execute $\text{Insert}$ and $\text{DeleteMax}$ operations in amortised $O(1)$ time.  Alas, we don't get that speed for free: as part of the Faustian bargain, the soft heap is allowed to make a certain carefully bounded number of errors.

More specifically, the soft heap sometimes *corrupts* items.  Corruption means an item travels the heap as if it has a smaller apparent priority (for a max heap) than it originally had.  (And vice versa for a min heap, corruption always moves items away from the root.)  The soft heap respects the heap priority ordering only for apparent priorities.

The guarantee we get is that the number corrupted items is bounded by $O(\varepsilon n)$, where $\varepsilon$ is an error parameter for the soft heap and $n$ is the number of items ever inserted into the heap.  Crucially, $n$ is not the number of items currently in the soft heap.

The lower $\varepsilon$ the longer $\text{DeleteMax}$ takes.  Specifically, the one of the constant factor hiding in $\text{DeleteMax}$'s $O(1)$ time is $O(1/\log \varepsilon)$, assuming $\varepsilon$ is picked independent of $n$.  So the amortised runtime of $\text{DeleteMax}$ is $O(1/\log \varepsilon)$, and the amortised runtime of $\text{Insert}$ is still $O(1)$.

## Application

Assume our sequence $s$ of operations has $m$ inserts and $k$ max-deletes.  W.l.o.g. we can assume that our deletes never occur when the heap is empty.  (Otherwise, we can remove them in an $O(n)$ preprocessing pass that does not even look at priorities.)  So $k \leq m$ and there are $r := m - k$ items left in the heap at the end (whether that's a soft heap or a normal one).

At this point, the soft heap will have corrupted up to $\varepsilon m$ items.  Leaving $r - \varepsilon m = m - k - \varepsilon m = (1-\varepsilon)m - k$ items in the heap that are not corrupted.  Because soft heaps only ever corrupt items to move away from the root, ie 'better' at staying in the heap, one can show that the uncorrupted items left in the soft heap are a subset of the items that would be left in a normal heap.

We can't say anything about the items that got deleted by the soft heap nor about the items that got corrupted.

Overall after running the soft heap, we can produce a new smaller instance of the problem.  Its size is $m' := m - (1-\varepsilon)m + k = \varepsilon m + k$ for number of inserts and $k' := k$ for number of deletes (we only remove items from consideration that never get deleted.)

# The catch

So far so good, but even with a small $\varepsilon$, and repeated application of our soft heap pass, the size of the problem won't keep shrinking once the number of remaining inserts becomes close to the number of original deletes.

We seem to be stuck.

# Back to basics

Going backwards through our list of days wasn't the only choice available to produce a sequence of operations.  We could also have gone forwards through the list of days, and for each day $d$:
- For each task $t$ with $\text{deadline}(t) = d$, append $\text{Insert}(t)$ to $s$.
- Afterwards, append enough $\text{DeleteMin}$ operations to cut the size of our heap down to $d$ (if necessary).

Conceptually, the backward pass's heap keeps track of items that might still get scheduled.  A $\text{DeleteMax}$ operation picks an item to schedule on that day.  The forward pass's heap keeps track of items that might still need to be dropped.  A $\text{DeleteMin}$ operation picks an item to drop from the schedule, because there were too many other more urgent items with short deadlines that needed to be scheduled first.

In the end, both approaches are equivalent and produce complementary left-over sets in their heaps at the end: either the items that we have to drop (for the forward pass), or the items that we can schedule in time (for the backward pass).  Each item will be in exactly one of the left-over sets.  (W.l.o.g. we assume that the items have distinct weights, or that ties will be broken in a consistent way.)

We described the forward and backward passes in the context of our concrete problem, but more generally for any sequence of $\text{Insert}$ and $\text{DeleteMax}$ (respectively $\text{Insert}$ and $\text{DeleteMin}$) operations, we can convert between the two representations in linear time (without comparing weights).

# Salvation

When the backward pass has $m$ inserts and $k$ deletes in some order, the forward pass has the same $m$ inserts (but in reverse order) interspersed with some $m - k$ deletes.

The backward pass removes at least $(1-\varepsilon)m - k$ items from consideration: they will definitely have to be dropped.  The forward pass removes at least $k - \varepsilon m$ different items from consideration: they can definitely be scheduled.

Overall $k$ nicely cancels out, and we are left with at most $m':=2\varepsilon m$ inserts after both passes (and around $k':= \varepsilon m$ deletes for both passes, but as long as $0 \leq k \leq m$ we can ignore the exact value.)

# The algorithm

The algorithm is now simple:
- Run both passes on the original problem instance in a combined $O(m)$ time.  Take note of the fate of the items we already nailed down.
- Reduce the problem instance to $m' \leq 2\varepsilon m$ inserts (and around $k' = \varepsilon m$ deletes).
- Unless $m'$ went to zero, repeat the process with the new instance.

The problem size shrinks by a constant factor of $2\varepsilon$ each iteration, the cost of each iteration is linear in the remaining problem size.  Overall, the well-known sum of the geometric series gives us a total runtime of $O(n)$, for any $\varepsilon < 1/2$.

# Appending: Nerding out

The backward and forwarded passes may seem a bit arbitrary.  Especially their conversion into each other, which we only sketched in the roughest of details.

They make more sense when viewed from the perspective of [matroids](https://en.wikipedia.org/wiki/Matroid) and their [duals](https://en.wikipedia.org/wiki/Dual_matroid).

Specifically the following family of matroids, which I will call 'heap matroids' reasons that will become clear later:

Fix a ground set $G$ of distinct elements, and a sequence of operations to non-deterministically build a set (starting from an empty set):
- $\text{Insert}(x)$: add $x$ to the set.
- $\text{DeleteAny}$: remove any element from the set.

All possible outcomes of this process together form the basis of a matroid over the groundset $G$.

Finding a maximum weight basis over this matroid is equivalent to always deleting the minimum element from our set, ie running a min-heap.  Always deleting the maximum element finds a minimum weight basis.

The family of heap matroids is closed under taking the [dual](https://en.wikipedia.org/wiki/Dual_matroid).  The conversion between backward and forward passes is the duality transformation.  Finding a minimum weight basis in a heap matroid is equivalent to finding a maximum weight basis in the dual matroid.

Soft heaps allow us to identify a subset of the optimum basis.

My 'heap matroids' are also known as 'nested matroids' or 'Schubert matroids'.  Alas, as far as I can tell, there are conflicting definitions, and only some definitions of 'nested matroids' and 'Schubert matroids' are equivalent to our 'heap matroids'.  That's why I coined a new name.
