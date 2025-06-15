# Travelling Salesman

The travelling salesman, starting from a certain city, wants to visit all the other cities exactly once and return. Given a map of the cities and the roads connecting them, he wants to make the trip as short as possible.

Find the shortest path in the graph that starts and ends in a specific vertex and visits **all** other vertices **exactly once**. This kind of a path is also known as a Hamiltonian cycle.

## Input

The input consists of the graph. The **start vertex** is always the one with ID `0`. The graph is represented as a list of edges. An edge is represented as a tuple of 3 integers: `v u w`, where `v` and `u` are the vertices making the edge, and `w` is the weight of the edge. All values are represented as 16-bit unsigned integers.

`0 <= v, u, w < 65536`

Vertex IDs are consecutive range starting from `0`, i.e. `0, 1, 2, ..., N - 1, N`, where `N` is an arbitrary number. No edge connects a vertex to itself.  Any pair of vertices has **zero or one** edge. The graph is connected, i.e. any vertex is reachable from any other.

## Output

The output is space (`' '`) separated list of integers denoting vertex IDs.

## Validation and Scoring

The output must:

- start and end with `0`
- must be of length `V + 1`, where `V` is the number of vertices
- contain only vertex IDs
- contain all vertex IDs only once (except for `0`)

Scoring is based on the length of the path.

## Example

Consider the following graph:

IMAGE

The travelling salesman starts from town "0" and has a few options of traversing the whole map. This happens to be shortest path.

IMAGE
