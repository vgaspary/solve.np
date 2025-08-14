# Graph Coloring

The Graph Coloring problem involves assigning a color to each vertex of a graph such that no two adjacent vertices share the same color. The goal is to find a valid coloring that uses as few colors as possible. The minimum number of colors is known as the **chromatic number** of the graph.

Find an assignment of colors to the vertices of a given graph such that no two vertices connected by an edge have the same color.

## Input

The input consists of the graph. The graph is represented as a list of edges. An edge is represented as a tuple of 2 integers: `v u`, where `v` and `u` are the vertices making the edge. All values are represented as 16-bit unsigned integers.

`0 <= v, u < 65536`

Vertex IDs are a consecutive range starting from `0`, i.e., `0, 1, 2, ..., N - 1`, where `N` is an arbitrary number of vertices. No edge connects a vertex to itself. Any pair of vertices has **zero or one** edge. The graph is connected, i.e., any vertex is reachable from any other.

## Output

The output is a space (`' '`) separated list of integers denoting the color assigned to each vertex, ordered by vertex ID starting from 0. The colors should be represented by non-negative integers, starting from 0.

## Validation and Scoring

The output must:

- Be a list of `V` integers, where `V` is the number of vertices.
- Contain only non-negative integers representing colors.
- Satisfy the constraint that for every edge `(u, v)` in the input graph, the color assigned to vertex `u` must be different from the color assigned to vertex `v`.

Scoring will be based on the number of colors used.

## Example

Consider the following graph:

<img src="/static/GC_not_colored.svg" width="720">

The trivial solution is to pick a unique color for each vertex. The fewest colors we can use to color the graph (chromatic number) is _4_. Here are two ways to color the graph using 4 colors.

<img src="/static/GC_4_colored.svg" width="720">

<img src="/static/GC_4_colored_2.svg" width="720">
