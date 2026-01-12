use crate::Graph;
use crate::graph::Vertex;
use std::collections::HashMap;
use std::collections::HashSet;

pub fn dfs<T, Visitor>(graph: &Graph<T>, from: Vertex, mut visitor: Visitor)
where
    Visitor: FnMut(Vertex),
{
    dfs_rec(graph, from, &mut visitor, &mut HashSet::new());
}

fn dfs_rec<T, Visitor>(
    graph: &Graph<T>,
    v: Vertex,
    visitor: &mut Visitor,
    visited: &mut HashSet<Vertex>,
) where
    Visitor: FnMut(Vertex),
{
    if !visited.insert(v) {
        return;
    }
    visitor(v);
    graph
        .adj_list
        .get(v)
        .map(HashMap::keys)
        .into_iter()
        .flatten()
        .for_each(|&u| dfs_rec(graph, u, visitor, visited));
}

/// Traverse a tree from a given vertex `from`. `graph` must be a tree, cycles will cause infinite
/// recursion.
pub fn traverse_tree<T, Visitor>(graph: &Graph<T>, from: Vertex, mut visitor: Visitor)
where
    Visitor: FnMut(Vertex),
{
    traverse_tree_rec(graph, from, from, &mut visitor)
}

fn traverse_tree_rec<T, Visitor>(
    graph: &Graph<T>,
    node: Vertex,
    parent: Vertex,
    visitor: &mut Visitor,
) where
    Visitor: FnMut(Vertex),
{
    visitor(node);
    graph
        .adj_list
        .get(node)
        .map(HashMap::keys)
        .into_iter()
        .flatten()
        .filter(|&&n| n != parent)
        .for_each(|&neighbor| traverse_tree_rec(graph, neighbor, node, visitor));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Graph;

    #[test]
    fn test_dfs_single_vertex() {
        let graph: Graph<()> = Graph::new(1);
        let mut visited = Vec::new();
        dfs(&graph, 0, |v| visited.push(v));
        assert_eq!(visited, vec![0]);
    }

    #[test]
    fn test_dfs_simple_path() {
        let graph = Graph::new(4)
            .add_edge((0, 1))
            .add_edge((1, 2))
            .add_edge((2, 3));
        let mut visited = Vec::new();
        dfs(&graph, 0, |v| visited.push(v));

        let visited_set = HashSet::from_iter(visited);
        let expected_set = HashSet::from([0, 1, 2, 3]);
        assert_eq!(visited_set, expected_set);
    }

    #[test]
    fn test_dfs_multiple_components() {
        let graph = Graph::new(5)
            .add_edge((0, 1)) // component 1
            .add_edge((2, 3)); // component 2
        // vertex 4 is isolated

        // DFS from component 1
        let mut visited1 = Vec::new();
        dfs(&graph, 0, |v| visited1.push(v));
        let visited_set1 = HashSet::from_iter(visited1);
        let expected_set1 = HashSet::from([0, 1]);
        assert_eq!(visited_set1, expected_set1);

        // DFS from component 2
        let mut visited2 = Vec::new();
        dfs(&graph, 2, |v| visited2.push(v));
        let visited_set2 = HashSet::from_iter(visited2);
        let expected_set2 = HashSet::from([2, 3]);
        assert_eq!(visited_set2, expected_set2);
    }

    #[test]
    fn test_dfs_isolated_vertex() {
        let graph = Graph::new(3).add_edge((0, 2)); // 1 is isolated
        let mut visited = Vec::new();
        dfs(&graph, 1, |v| visited.push(v));
        assert_eq!(visited, vec![1]);
    }

    #[test]
    fn test_dfs_cycle() {
        let graph = Graph::new(4)
            .add_edge((0, 1))
            .add_edge((1, 2))
            .add_edge((2, 3))
            .add_edge((3, 0)); // Cycle

        let mut visited = Vec::new();
        dfs(&graph, 0, |v| visited.push(v));

        let visited_set = HashSet::from_iter(visited);
        let expected_set = HashSet::from([0, 1, 2, 3]);
        assert_eq!(visited_set, expected_set);
    }

    #[test]
    fn test_dfs_complete_graph() {
        let graph = Graph::full(4);
        let mut visited = Vec::new();
        dfs(&graph, 0, |v| visited.push(v));

        let visited_set = HashSet::from_iter(visited);
        let expected_set = HashSet::from([0, 1, 2, 3]);
        assert_eq!(visited_set, expected_set);
    }

    #[test]
    fn test_traverse_tree_single_node() {
        let graph: Graph<()> = Graph::new(1);
        let mut visited = Vec::new();
        traverse_tree(&graph, 0, |v| visited.push(v));
        assert_eq!(visited, vec![0]);
    }

    #[test]
    fn test_traverse_tree_path() {
        let graph = Graph::new(4)
            .add_edge((0, 1))
            .add_edge((1, 2))
            .add_edge((2, 3));
        let mut visited = Vec::new();
        traverse_tree(&graph, 0, |v| visited.push(v));
        let visited_set = HashSet::from_iter(visited);
        let expected_set = HashSet::from([0, 1, 2, 3]);
        assert_eq!(visited_set, expected_set);
    }

    #[test]
    fn test_traverse_tree_branching() {
        let graph = Graph::new(5)
            .add_edge((0, 1))
            .add_edge((0, 2))
            .add_edge((1, 3))
            .add_edge((1, 4));
        let mut visited = Vec::new();
        traverse_tree(&graph, 0, |v| visited.push(v));
        let visited_set = HashSet::from_iter(visited);
        let expected_set = HashSet::from([0, 1, 2, 3, 4]);
        assert_eq!(visited_set, expected_set);
    }

    #[test]
    fn test_traverse_tree_from_leaf() {
        let graph = Graph::new(5)
            .add_edge((0, 1))
            .add_edge((0, 2))
            .add_edge((1, 3))
            .add_edge((1, 4));
        let mut visited = Vec::new();
        traverse_tree(&graph, 3, |v| visited.push(v));
        let visited_set = HashSet::from_iter(visited);
        let expected_set = HashSet::from([0, 1, 2, 3, 4]);
        assert_eq!(visited_set, expected_set);
    }
}
