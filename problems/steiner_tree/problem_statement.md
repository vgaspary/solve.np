# Steiner Tree Problem

## Description

The Steiner Tree problem is a classic combinatorial optimization problem. Given an undirected graph with non-negative edge weights and a subset of vertices called "terminals", the objective is to find a minimum-weight subgraph that connects all the terminal vertices.

This connecting subgraph must be a tree. It can include vertices that are not terminals; these are known as "Steiner vertices". The inclusion of these Steiner vertices can help reduce the total weight of the tree.

## Input Format

The input is provided as a text file describing the graph and the set of terminal vertices.

1.  The first line is a space-separated list of integer vertex IDs, representing the **terminals**.
2.  The second line is intentionally left blank.
3.  Each subsequent line represents an edge in the graph, consisting of three space-separated integers: `u v w`.
    *   `u` and `v` are the integer IDs of the vertices connected by the edge.
    *   `w` is the non-negative integer weight of the edge.

### Example Input

```
0 5 7 3

0 2 7
1 6 10
...
```

In this example, vertices `0`, `5`, `7`, and `3` are the terminals. The lines following the blank line define the edges and their weights.

## Output Format

Your solution should be a list of edges that constitute the Steiner tree. The output file should have one edge per line, with each line containing two space-separated vertex IDs.

### Example Output

```
0 1
1 5
6 7
3 6
```

## Goal

The primary goal is to minimize the total cost of the solution, which is the sum of the weights of all edges included in your output tree. The solution will be considered valid if it meets the following criteria:

1.  The set of edges forms a single connected component (a tree).
2.  There are no cycles in the set of edges.
3.  All terminal vertices from the input are included in the tree (either as an endpoint of an edge or as an intermediate vertex).
4.  All edges in the solution must be present in the original input graph.
