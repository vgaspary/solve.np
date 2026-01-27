# 0-1 Knapsack Problem

The 0-1 knapsack problem is a classic optimization problem. Imagine you are a burglar with a knapsack that can carry a limited amount of weight. You break into a store with a variety of items, each with its own value and weight. Your goal is to choose a combination of items to put in your knapsack that maximizes the total value, without exceeding the knapsack's weight limit.

The "0-1" part of the name comes from the fact that for each item, you have two choices: either take it (1) or leave it (0). You cannot take a fraction of an item.

## Input

The input consists of the weight limit of the knapsack and the items.

-  The first line contains a single integer denoting the **weight limit** of the knapsack.
-  The second line is intentionally left blank.
-  Each subsequent line represents an item, consisting of two space-separated integers: `w v`.
    -   `w` is the **weight** of the item.
    -   `v` is the **value** of the item.

## Output

The output represents the subset of picked items. It must be a space-separated list of indices of the items (zero indexed) in the input file.

## Validation and Scoring

The goal is to maximize the total value of the items in the knapsack. The solution will be considered valid if it meets the following criteria:

-  The indices are valid.
-  The indices are unique.
-  The total weight of the items does not exceed the weight limit of the knapsack.

## Example

Consider the following input:

```
32

2 6
8 9
10 13
6 11
6 10
4 9
3 6
3 4
15 16
7 8
```

This means the knapsack has a weight limit of 32. There are 10 items:
- Item 0: weight 2, value 6
- Item 1: weight 8, value 9
- Item 2: weight 10, value 13
- Item 3: weight 6, value 11
- Item 4: weight 6, value 10
- Item 5: weight 4, value 9
- Item 6: weight 3, value 6
- Item 7: weight 3, value 4
- Item 8: weight 15, value 16
- Item 9: weight 7, value 8

<img src="/static/K_0.svg" width="720">

One way to fill up the knapsack would be picking the items `1 3 6 8`. The score would be `42`.

<img src="/static/K_1.svg" width="720">

We can do even better with `0 2 3 4 5 6`. That would improve the score up to `55`. Note, this way the knapsack is not fully loaded.

<img src="/static/K_2.svg" width="720">
