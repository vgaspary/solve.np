# Maximum Cut

In graph theory, a **cut** is a partition of the vertices of a graph into two disjoint subsets. Any cut determines a **cut-set**, the set of edges that have one endpoint in each subset of the partition. These edges are said to **cross** the cut.

Find a cut in a **weighted** undirected graph. The objective is to maximize the total weight of the cut-set.

## Input

The input consists of the graph represented as a list of edges. An edge is represented as a tuple of 3 integers: `u v w`, where `u` and `v` are the vertices making the edge, and `w` is the weight of the edge.

Vertex IDs are a consecutive range starting from `0`, i.e., `0, 1, 2, ..., N - 1`, where `N` is an arbitrary number of vertices. No edge connects a vertex to itself. Any pair of vertices has **zero or one** edge. The graph is not necessarily connected.

## Output

The output is expected to be a whitespace separated list of integers denoting one of the disjoint subsets of the cut.

## Validation and Scoring

The output must:

- be a proper, non-empty subset of all vertices
- not contain duplicates or invalid vertex IDs

The score is total weight of the cut-set, the edges connecting vertices in the two subsets of the partition. The objective is to maximize the score.

## Example

Consider the following grpah:

<img src="/static/MaxCut_0.svg" width="720">

The following is a cut yielding score `39`.

<img src="/static/MaxCut_1.svg" width="720">

This is an even better cut with score `41` (still not the best).

<img src="/static/MaxCut_2.svg" width="720">
