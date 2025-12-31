# Maximum Clique

In graph theory, a clique is a subset of vertices of an undirected graph such that every two distinct vertices in the clique are adjacent. A **maximum clique** is a clique of the largest possible size in a given graph.

Find a clique in an undirected graph.

## Input

The input consists of the graph. The graph is represented as a list of edges. An edge is represented as a tuple of 2 integers: `v u`, where `v` and `u` are the vertices making the edge.

Vertex IDs are a consecutive range starting from `0`, i.e., `0, 1, 2, ..., N - 1`, where `N` is an arbitrary number of vertices. No edge connects a vertex to itself. Any pair of vertices has **zero or one** edge. The graph is not necessarily connected.

## Output

The output is expected to be a whitespace separated list of integers denoting the vertices in the clique.

## Validation and Scoring

The output must:

- be a valid clique (every vertex in the output must be connected to every other vertex in the output).
- not contain duplicates or invalid vertex IDs.

The score is the size of the clique. The objective is to maximize the score.

## Example

Consider the following graph:

<img src="" width="720">

A clique of size 3 could be `{0, 1, 2}`. A larger clique of size 4 could be `{2, 3, 4, 5}`. The maximum clique for this graph has size 5.

<img src="" width="720">

<img src="" width="720">
