# 0-1 Knapsack Problem

The 0-1 knapsack problem is a classic optimization problem. Imagine you are a burglar with a knapsack that can carry a limited amount of weight. You break into a store with a variety of items, each with its own value and weight. Your goal is to choose a combination of items to put in your knapsack that maximizes the total value, without exceeding the knapsack's weight limit.

The "0-1" part of the name comes from the fact that for each item, you have two choices: either take it (1) or leave it (0). You cannot take a fraction of an item.

## Input

The input consists of the weight limit of the knapsack and the items.

-  The first line contains a single integer denoting the **weight limit** of the knapsack.
-  The second line is intentionally left blank.
-  Each subsequent line represents an item, consisting of two space-separated integers: `v w`.
    -   `v` is the **value** of the item.
    -   `w` is the **weight** of the item.

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
10

10 5
40 4
30 6
50 3
```

This means the knapsack has a weight limit of 10. There are four items:
- Item 0: value 10, weight 5
- Item 1: value 40, weight 4
- Item 2: value 30, weight 6
- Item 3: value 50, weight 3

The optimal solution is to pick items 1 and 3. The total value is 90 and the total weight is 7, which is within the limit.

The output for this solution would be:
```
1 3
```
