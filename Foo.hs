{-# LANGUAGE DeriveFunctor, DeriveFoldable, DeriveTraversable #-}

module PairingHeap 
  ( PairingHeap
  , Pool(..)
  , pool
  , sift
  , findMin
  , insert
  , merge
  , deleteMin
  , fromList
  ) where

import Data.List.Split (chunksOf)

-- Define the Pool data structure
-- A Pool contains one representative element and 'count' corrupted elements
data Pool a = Pool a Int
  deriving (Show, Eq, Functor, Foldable, Traversable)

-- Pooling operation to combine pools
pool :: Ord a => Pool a -> Pool a -> Pool a
pool (Pool item0 count0) (Pool item1 count1) = Pool (max item0 item1) (count0 + count1 + 1)
-- The +1 is because when we pool, one element becomes the representative
-- and the other element becomes corrupted

-- Define the pairing heap data structure with derived typeclasses
data PairingHeap a = Empty
                   | Node (Pool a) [PairingHeap a]
                   deriving (Show, Eq, Functor, Foldable, Traversable)

-- Find the minimum element (root of the heap)
findMin :: Ord a => PairingHeap a -> Maybe a
findMin Empty = Nothing
findMin (Node (Pool x _) _) = Just x

-- Insert an element into the heap
-- New elements start with corruption count 0 (just the representative)
insert :: Ord a => a -> PairingHeap a -> PairingHeap a
insert x heap = merge (Node (Pool x 0) []) heap

-- Merge two pairing heaps
merge :: Ord a => PairingHeap a -> PairingHeap a -> PairingHeap a
merge heap Empty = heap
merge Empty heap = heap
merge h1@(Node p1@(Pool x _) hs1) h2@(Node p2@(Pool y _) hs2)
  | x <= y    = Node p1 (h2 : hs1)  -- h2 becomes a subheap of h1
  | otherwise = Node p2 (h1 : hs2)  -- h1 becomes a subheap of h2

-- Parameter for chunksOf 
m :: Int
m = 4

-- Merge heaps within a chunk (using standard pairing heap approach)
mergeChunk :: Ord a => [PairingHeap a] -> PairingHeap a
mergeChunk [] = Empty
mergeChunk [h] = h
mergeChunk xs = mergeChunk $ map (foldr1 merge) (chunksOf 2 xs)

-- Helper function to combine heaps with sifting
combine :: Ord a => [PairingHeap a] -> PairingHeap a
combine [] = Empty
combine [heap] = heap
combine (x:y:xs) = combine (sift (merge x y) : xs)

-- Merge with sifting
mergeWithSift :: Ord a => [PairingHeap a] -> PairingHeap a
mergeWithSift xs = combine $ map mergeChunk $ chunksOf m xs

-- Sift operation for introducing controlled corruption
-- This combines children and pools their minimum with the current root
sift :: Ord a => PairingHeap a -> PairingHeap a
sift Empty = Empty
sift (Node p children) = 
  let newHeap = mergeWithSift children
  in case newHeap of
       Empty -> Node p []
       Node childPool childrenOfChildren -> 
         -- Pool the current node's pool with the result of merging its children
         -- This is where elements become corrupted
         Node (pool p childPool) childrenOfChildren

-- Remove the minimum element and restructure the heap
deleteMin :: Ord a => PairingHeap a -> PairingHeap a
deleteMin Empty = Empty
deleteMin (Node (Pool x count) children)
  | count > 0  = Node (Pool x (count - 1)) children  -- Remove one corrupted element
  | otherwise  = mergeWithSift children              -- Remove representative when no corrupted elements remain

-- Create a heap from a list of elements
fromList :: Ord a => [a] -> PairingHeap a
fromList = foldr insert Empty

-- Example usage:
example :: IO ()
example = do
  let heap = fromList [3, 1, 4, 1, 5, 9]
  print heap
  print $ findMin heap  -- Returns Just 1
  print $ deleteMin heap  -- Returns a new heap without the smallest element
  
  -- Examples using the derived typeclasses
  print $ fmap (*2) heap  -- Double all values using Functor
  print $ foldr (+) 0 heap  -- Sum all values using Foldable
  
  -- Pool examples
  let p1 = Pool 5 0
  let p2 = Pool 7 0
  print $ pool p1 p2  -- Should be Pool 7 1
  
  -- Sift example
  let heapToSift = Node (Pool 2 0) [Node (Pool 3 0) [], Node (Pool 4 0) []]
  print $ sift heapToSift  -- Should introduce corruption
  
  -- DeleteMin with corruption example
  let corruptedHeap = Node (Pool 1 2) [Node (Pool 5 0) []]
  print $ corruptedHeap                      -- Pool 1 2 - has representative value 1 and 2 corrupted elements
  print $ deleteMin corruptedHeap            -- :q
  Pool 1 1 - removed one corrupted element
  print $ deleteMin (deleteMin corruptedHeap)  -- Pool 1 0 - removed second corrupted element
  print $ deleteMin (deleteMin (deleteMin corruptedHeap))  -- Finally removes representative, restructures heap
  
  -- New mergeWithSift example with chunking and sift
  let multipleHeaps = map (\x -> Node (Pool x 0) []) [6, 3, 8, 2, 5, 1, 7, 4]
  print $ mergeWithSift multipleHeaps  -- Should show the effect of chunking and sifting
