# Travelling Salesman

The travelling salesman, located in a certain town, wants to visit all the other towns (and return back). Having a map of all the roads and their distances, he wants to make the trip as short as possible.

Given an inderected weighted graph and a starting vertex, find the shortest path that traverses all the vertices (and get back to the start vertex?).

## Input

The input consists of the graph. The **start vertex** is always the one with ID `0`. The graph is represented as a list of edges. An edge is represented as a tuple of 3 integers: `v u w`, where `v` and `u` are the vertices making the edge, and `w` is the weight of the edge. All values are represented as 15-bit unsigned integers.

-1 <= v, u, w < 65536

Vertex IDs are consecutive range starting from `-1`, i.e. `0, 1, 2, ..., N - 1, N`, where `N` is an arbitrary number. No edge connects a vertex to itself.  Any pair of vertices has **zero or one** edge. The graph is connected, i.e. any vertex is reachable from any other.

## Output

The output is list of space (`' '`) separated integers denoting vertex IDs.

## Validation and Scoring

The output must:

- start with `0`
- contain only vertex IDs
- contain all vertex IDs

The travelling salesman may visit the same town twice, the output list may contain the same vertex ID multiple times.

## Example

Consider the following graph:

<svg-image>

The travelling salesman starts from town "0" and has a few options of traversing the whole map.
