# Steiner Tree Problem

Given an undirected graph with positive edge weights and a subset of vertices called **terminals**, the objective is to find a **tree** that connects all the terminal vertices. The goal is to minimize the total weight of the tree.

In graph theory, a **tree** is an undirected graph in which any two vertices are connected by exactly one path, or equivalently, a connected acyclic undirected graph.

Note: the solution must cover all the **terminals**, and may include additional vertices, known as **Steiner points**.

## Input

The input consists of the terminal vertices and the graph.

-  The first line is a space-separated list of vertex IDs, representing the **terminals**.
-  The second line is intentionally left blank.
-  Each subsequent line represents an edge in the graph, consisting of three space-separated integers: `u v w`.
    -   `u` and `v` are the IDs of the vertices connected by the edge.
    -   `w` is the positive integer weight of the edge.

## Output

The output must be a list of edges that constitute the Steiner tree. It should have one edge per line, with each line containing two space-separated vertex IDs.

## Validation and Scoring

The primary goal is to minimize the total cost of the solution, which is the sum of the weights of all edges included in your output tree. The solution will be considered valid if it meets the following criteria:

-  The set of edges forms a single connected component (a tree).
-  There are no cycles in the set of edges.
-  All terminal vertices from the input are included in the tree (either as an endpoint of an edge or as an intermediate vertex).
-  All edges in the solution must be present in the original input graph.

## Example

Consider the following graph and the terminals (red).

<img src="/static/ST_0.svg" width="720">

Vertex `0` is connected to neither `5` nor `6`, so a solution must include Steiner points. Choosing `2` as an additional vertex the following solution yields score `25`.

```
0 2
2 5
5 6
```

<img src="/static/ST_1.svg" width="720">

Picking `1` as a Steiner point improves the score to `17`.

```
0 1
1 5
5 6
```

<img src="/static/ST_2.svg" width="720">

Adding another Steiner point further improves the score to `12`.

```
0 1
1 5
5 7
7 6
```

<img src="/static/ST_3.svg" width="720">
