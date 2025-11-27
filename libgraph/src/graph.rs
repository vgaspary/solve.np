use itertools::Itertools;
use rand::prelude::*;
use std::collections::HashMap;

pub type Vertex = usize;
pub type Edge = (Vertex, Vertex);
pub struct Graph<Metadata = ()> {
    pub adj_list: Vec<HashMap<usize, Metadata>>,
}

impl<Metadata> Graph<Metadata>
where
    Metadata: Clone,
{
    pub fn new(num_vertices: usize) -> Self {
        Self {
            adj_list: vec![HashMap::new(); num_vertices],
        }
    }

    pub fn add_edge_with_metadata(mut self, (u, v): Edge, metadata: Metadata) -> Self {
        if u != v {
            self.adj_list[u].insert(v, metadata.clone());
            self.adj_list[v].insert(u, metadata);
        }
        self
    }

    pub fn edges(&self) -> impl Iterator<Item = Edge> {
        self.adj_list
            .iter()
            .enumerate()
            .map(|(u, vs)| vs.keys().map(move |&v| (u, v)))
            .flatten()
            .filter(|(u, v)| u < v)
    }

    pub fn num_vertices(&self) -> usize {
        self.adj_list.len()
    }

    pub fn edges_with_metadata(&self) -> impl Iterator<Item = (Edge, Metadata)> {
        self.adj_list
            .iter()
            .enumerate()
            .map(|(u, vs)| {
                vs.iter()
                    .map(move |(&v, metadata)| ((u, v), metadata.clone()))
            })
            .flatten()
            .filter(|((u, v), _)| u < v)
    }
}

impl Graph<()> {
    pub fn hamiltonian_cycle(num_vertices: usize, rng: &mut impl Rng) -> Self {
        if num_vertices <= 1 {
            return Self::new(num_vertices);
        }
        let mut vertices: Vec<_> = (0..num_vertices).collect();
        vertices.shuffle(rng);
        (0..num_vertices)
            .map(|i| (vertices[i], vertices[(i + 1) % num_vertices]))
            .fold(Self::new(num_vertices), |graph, edge| graph.add_edge(edge))
    }

    pub fn minimum_spanning_tree(num_vertices: usize, rng: &mut impl Rng) -> Self {
        if num_vertices <= 1 {
            return Self::new(num_vertices);
        }
        (1..num_vertices)
            .map(|i| (i, rng.random_range(0..i)))
            .fold(Self::new(num_vertices), |graph, edge| graph.add_edge(edge))
    }

    pub fn full(num_vertices: usize) -> Self {
        (0..num_vertices)
            .cartesian_product(0..num_vertices)
            .filter(|(u, v)| u < v)
            .fold(Self::new(num_vertices), |graph, edge| graph.add_edge(edge))
    }

    pub fn add_edge(mut self, (u, v): Edge) -> Self {
        if u != v {
            self.adj_list[u].insert(v, ());
            self.adj_list[v].insert(u, ());
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng; // For deterministic tests

    fn get_sorted_edges(graph: &Graph) -> Vec<Edge> {
        graph.edges().sorted().collect()
    }

    #[test]
    fn test_new() {
        let graph = Graph::new(0);
        assert_eq!(graph.adj_list.len(), 0);
        assert_eq!(get_sorted_edges(&graph).len(), 0);

        let graph = Graph::new(3);
        assert_eq!(graph.adj_list.len(), 3);
        for i in 0..3 {
            assert!(graph.adj_list[i].is_empty());
        }
        assert_eq!(get_sorted_edges(&graph).len(), 0);
    }

    #[test]
    fn test_add_edge() {
        let graph = Graph::new(3).add_edge((0, 1));
        assert!(graph.adj_list[0].contains_key(&1));
        assert!(graph.adj_list[1].contains_key(&0));
        assert_eq!(get_sorted_edges(&graph), vec![(0, 1)]);

        let graph = Graph::new(3).add_edge((0, 1)).add_edge((0, 2));
        assert!(graph.adj_list[0].contains_key(&2));
        assert!(graph.adj_list[2].contains_key(&0));
        assert_eq!(get_sorted_edges(&graph), vec![(0, 1), (0, 2)]);

        let graph = Graph::new(3)
            .add_edge((0, 1))
            .add_edge((0, 2))
            .add_edge((1, 2));
        assert!(graph.adj_list[1].contains_key(&2));
        assert!(graph.adj_list[2].contains_key(&1));
        assert_eq!(get_sorted_edges(&graph), vec![(0, 1), (0, 2), (1, 2)]);

        // Add existing edge (should not change anything)
        let graph = Graph::new(3)
            .add_edge((0, 1))
            .add_edge((0, 2))
            .add_edge((1, 2))
            .add_edge((0, 1));
        assert_eq!(get_sorted_edges(&graph), vec![(0, 1), (0, 2), (1, 2)]);

        // Self loop not supported.
        let graph = Graph::new(1).add_edge((0, 0));
        assert!(graph.adj_list[0].is_empty());
        assert_eq!(get_sorted_edges(&graph), vec![]);
    }

    #[test]
    fn test_new_full_graph() {
        let graph = Graph::full(0);
        assert!(get_sorted_edges(&graph).is_empty());

        let graph = Graph::full(1);
        assert!(get_sorted_edges(&graph).is_empty());

        let graph = Graph::full(3);
        assert_eq!(get_sorted_edges(&graph), vec![(0, 1), (0, 2), (1, 2)]);

        let graph = Graph::full(4);
        assert_eq!(
            get_sorted_edges(&graph),
            vec![(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]
        );
    }

    #[test]
    fn test_new_hamiltonian_cycle() {
        let mut rng = StdRng::seed_from_u64(42); // Deterministic RNG for testing

        let graph = Graph::hamiltonian_cycle(0, &mut rng);
        assert_eq!(graph.adj_list.len(), 0);
        assert_eq!(get_sorted_edges(&graph).len(), 0);

        let graph = Graph::hamiltonian_cycle(1, &mut rng);
        assert_eq!(graph.adj_list.len(), 1);
        assert_eq!(get_sorted_edges(&graph).len(), 0);

        let graph = Graph::hamiltonian_cycle(2, &mut rng);
        assert_eq!(graph.adj_list.len(), 2);
        assert_eq!(get_sorted_edges(&graph).len(), 1); // One edge (v0, v1)
        for i in 0..2 {
            assert_eq!(
                graph.adj_list[i].len(),
                1,
                "Vertex {} in 2-cycle should have degree 1",
                i
            );
        }

        // Test case: 3 vertices (triangle)
        let graph = Graph::hamiltonian_cycle(3, &mut rng);
        assert_eq!(graph.adj_list.len(), 3);
        assert_eq!(get_sorted_edges(&graph).len(), 3); // 3 edges in a 3-cycle
        for i in 0..3 {
            assert_eq!(
                graph.adj_list[i].len(),
                2,
                "Vertex {} in 3-cycle should have degree 2",
                i
            );
        }

        // Test case: 5 vertices
        let graph = Graph::hamiltonian_cycle(5, &mut rng);
        assert_eq!(graph.adj_list.len(), 5);
        assert_eq!(get_sorted_edges(&graph).len(), 5); // 5 edges in a 5-cycle
        for i in 0..5 {
            assert_eq!(
                graph.adj_list[i].len(),
                2,
                "Vertex {} in 5-cycle should have degree 2",
                i
            );
        }
    }

    #[test]
    fn test_new_minimum_spanning_tree() {
        let mut rng = StdRng::seed_from_u64(42); // Deterministic RNG

        // Test case: 0 vertices
        let graph = Graph::minimum_spanning_tree(0, &mut rng);
        assert_eq!(graph.adj_list.len(), 0);
        assert_eq!(get_sorted_edges(&graph).len(), 0);

        // Test case: 1 vertex (0 edges)
        let graph = Graph::minimum_spanning_tree(1, &mut rng);
        assert_eq!(graph.adj_list.len(), 1);
        assert!(graph.adj_list[0].is_empty());
        assert_eq!(get_sorted_edges(&graph).len(), 0);

        // Test case: 2 vertices (1 edge)
        let graph = Graph::minimum_spanning_tree(2, &mut rng);
        assert_eq!(graph.adj_list.len(), 2);
        assert_eq!(get_sorted_edges(&graph).len(), 1); // N-1 edges
        let degrees: Vec<usize> = graph.adj_list.iter().map(|n| n.len()).collect();
        assert_eq!(degrees.iter().filter(|&&d| d == 1).count(), 2); // Both vertices have degree 1

        // Test case: 5 vertices (4 edges for a tree)
        let graph = Graph::minimum_spanning_tree(5, &mut rng);
        assert_eq!(graph.adj_list.len(), 5);
        assert_eq!(get_sorted_edges(&graph).len(), 4); // N-1 edges
        // Check degrees: A path graph (which this creates) on N>=2 vertices has 2 nodes of degree 1 (endpoints)
        // and N-2 nodes of degree 2 (internal).
        let degrees: Vec<usize> = graph.adj_list.iter().map(|n| n.len()).collect();
        assert_eq!(
            degrees.iter().filter(|&&d| d == 1).count(),
            2,
            "MST on 5 vertices (path) should have 2 degree-1 nodes"
        );
        assert_eq!(
            degrees.iter().filter(|&&d| d == 2).count(),
            3,
            "MST on 5 vertices (path) should have 3 degree-2 nodes"
        );
    }

    #[test]
    fn test_add_edge_with_metadata() {
        let graph = Graph::new(3).add_edge_with_metadata((0, 1), 10);
        assert_eq!(graph.adj_list[0].get(&1), Some(&10));
        assert_eq!(graph.adj_list[1].get(&0), Some(&10));

        let mut edges = graph.edges_with_metadata().collect::<Vec<_>>();
        edges.sort_by_key(|k| k.0);
        assert_eq!(edges, vec![((0, 1), 10)]);

        let graph = graph.add_edge_with_metadata((0, 2), 20);
        assert_eq!(graph.adj_list[0].get(&2), Some(&20));
        assert_eq!(graph.adj_list[2].get(&0), Some(&20));

        let mut edges = graph.edges_with_metadata().collect::<Vec<_>>();
        edges.sort_by_key(|k| k.0);
        assert_eq!(edges, vec![((0, 1), 10), ((0, 2), 20)]);

        // Test updating metadata
        let graph = graph.add_edge_with_metadata((0, 1), 100);
        assert_eq!(graph.adj_list[0].get(&1), Some(&100));
        assert_eq!(graph.adj_list[1].get(&0), Some(&100));

        let mut edges = graph.edges_with_metadata().collect::<Vec<_>>();
        edges.sort_by_key(|k| k.0);
        assert_eq!(edges, vec![((0, 1), 100), ((0, 2), 20)]);

        // Self loop not supported.
        let graph = Graph::new(1).add_edge_with_metadata((0, 0), 5);
        assert!(graph.adj_list[0].is_empty());
        assert!(graph.edges_with_metadata().next().is_none());
    }
}
