# Vertex Cover

In graph theory, a vertex cover is a subset of vertices that includes **at least** one endpoint of **every** edge. A **minimum vertex cover** is one of smallest possible size.

Find a vertex cover in an undirected graph.

## Input

The input consists of the graph. The graph is represented as a list of edges. An edge is represented as a tuple of 2 integers: `v u`, where `v` and `u` are the vertices making the edge.

`0 <= v, u < 65536`

Vertex IDs are a consecutive range starting from `0`, i.e., `0, 1, 2, ..., N - 1`, where `N` is an arbitrary number of vertices. No edge connects a vertex to itself. Any pair of vertices has **zero or one** edge. Note: the graph is not necessarily connected.

## Output

The output is expected to be whitespace separated list of integers denoting the vertices in the vertex cover.

## Validation and Scoring

The output must:

- be a valid vertex cover
- not contain duplicates or invalid vertex IDs

The score is the size of the vertex cover.

## Example

Consider the following graph:

<img src="/static/VC_0.svg" width="720">

The trivial vertex cover is the set of all vertices with score _8_. Here are two better vertex covers with scores _N_ and _M_.

<img src="/static/VC_1.svg" width="720">

<img src="/static/VC_2.svg" width="720">
