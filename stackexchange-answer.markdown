A few things I found out so far.

We can reduce ourselves to solving the following related problem:

<!-- language: lang-hs -->

    newtype Slot = Slot Int
    newtype Schedule a = Schedule [(Slot, [a])]

    findSchedule :: Ord a => Schedule a -> Schedule (a, Bool)

I.e. give the input data already sorted by deadline, but allow an arbitrary non-negative number of tasks to be done on each day.  Give the output by just marking the elements on whether they can be scheduled in time, or not.

The following function can check whether a schedule given in this format is feasible, ie whether all items still in the schedule can be scheduled before their deadlines:

<!-- language: lang-hs -->

    leftOverItems :: Schedule a -> [Int]
    leftOverItems (Schedule sch) = scanr op 0 sch where
      op (Slot s, items) itemsCarried = max 0 (length items - s + itemsCarried)

    feasible schedule = head (leftOverItems schedule) == 0

If we have a proposed candidate solution, and all items left out, we can check in linear time whether the candidate is optimal, or whether there are any items in the left-out set that would improve the solution.  We call these *light* items, in analogy to the [terminology in Minimum Spanning Tree algorithm](https://en.wikipedia.org/wiki/Expected_linear_time_MST_algorithm#F-heavy_and_F-light_edges)


<!-- language: lang-hs -->

    carry1 :: Ord a => Schedule a -> [Bound a]
    carry1 (Schedule sch) = map (maybe Top Val . listToMaybe) . scanr op [] $ sch where
      op (Slot s, items) acc = remNonMinN s (foldr insertMin acc items)

    -- We only care about the number of items, and the minimum item.
    -- insertMin inserts an item into a list, keeping the smallest item at the front.
    insertMin :: Ord a => a -> [a] -> [a]
    insertMin a [] = [a]
    insertMin a (b:bs) = min a b : max a b : bs

    -- remNonMin removes an item from the list,
    -- only picking the minimum at the front, if it's the only element.
    remNonMin :: [a] -> [a]
    remNonMin [] = []
    remNonMin [x] = []
    remNonMin (x:y:xs) = x : xs

    remNonMinN :: Int -> [a] -> [a]
    remNonMinN n l = iterate remNonMin l !! n

    data Bound a = Bot | Val a | Top
      deriving (Eq, Ord, Show, Functor)

    -- The curve of minimum reward needed for each deadline to make the cut:
    curve :: Ord a => Schedule a -> [Bound a]
    curve = zipWith min <$> runMin <*> carry1

    -- Same curve extended to infinity (in case the Schedules have a different length)
    curve' :: Ord a => Schedule a -> [Bound a]
    curve' = ((++) <*> repeat . last) . curve

    -- running minimum of items on left:
    runMin :: Ord a => Schedule a -> [Bound a]
    runMin = scanl1 min . map minWithBound . items . fmap Val

    minWithBound :: Ord a => [Bound a] -> Bound a
    minWithBound = minimum . (Top:)

    -- The pay-off for our efforts, this function uses
    -- the candidate solution to classify the left-out items
    -- into whether they are definitely _not_ in
    -- the optimal schedule (heavy items), or might be in it (light items).
    heavyLight :: Ord a => Schedule a -> Schedule a -> ([[a]],[[a]])
    heavyLight candidate leftOut =
        unzip . zipWith light1 (curve' candidate) . items $ leftOut
      where
        light1 pivot = partition (\item -> pivot < Val item)

`heavyLight` not only checks a proposed schedules for optimality, it also gives you a list of items that can improve a non-optimal schedule.
