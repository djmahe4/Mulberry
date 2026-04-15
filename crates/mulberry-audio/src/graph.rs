use mulberry_core::Sample;

/// Unique identifier for a node in the [`AudioGraph`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// Trait implemented by every processing node in the audio graph.
pub trait AudioNode: Send {
    /// Process a block of audio.
    ///
    /// * `inputs`  – one slice per incoming connection (may be empty)
    /// * `output`  – buffer to fill with this node's output
    /// * `sample_rate` – current engine sample rate
    fn process(&mut self, inputs: &[&[Sample]], output: &mut [Sample], sample_rate: u32);

    /// Human-readable name for debugging / display.
    fn name(&self) -> &str;

    /// Reset internal state (e.g. on transport stop).
    fn reset(&mut self);
}

// CONTEXT7 REVIEW:
// Issue: Audio graph processing order
// Resolution: Simple linear processing order for v0.1 (nodes processed in insertion order)
// Why: Full topological sort adds complexity; linear order works for simple chains.
//      Topological sort planned for v0.2.

/// A simple node-based audio graph that routes audio between [`AudioNode`]s.
pub struct AudioGraph {
    nodes: Vec<Option<Box<dyn AudioNode>>>,
    connections: Vec<(NodeId, NodeId)>,
    output_node: Option<NodeId>,
    /// Scratch buffers used during processing – one per node slot.
    scratch: Vec<Vec<Sample>>,
}

impl AudioGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            connections: Vec::new(),
            output_node: None,
            scratch: Vec::new(),
        }
    }

    /// Insert a node and return its [`NodeId`].
    pub fn add_node(&mut self, node: Box<dyn AudioNode>) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Some(node));
        self.scratch.push(Vec::new());
        id
    }

    /// Declare a connection from one node's output to another node's input.
    pub fn connect(&mut self, from: NodeId, to: NodeId) {
        self.connections.push((from, to));
    }

    /// Set which node provides the final output of the graph.
    pub fn set_output(&mut self, node: NodeId) {
        self.output_node = Some(node);
    }

    /// Remove a node (and any connections referencing it).
    pub fn remove_node(&mut self, id: NodeId) {
        if id.0 < self.nodes.len() {
            self.nodes[id.0] = None;
            self.connections.retain(|&(from, to)| from != id && to != id);
            if self.output_node == Some(id) {
                self.output_node = None;
            }
        }
    }

    /// Process the entire graph and write the result into `output`.
    ///
    /// Nodes are processed in insertion order. Each node receives the mixed
    /// outputs of all nodes connected to its input.
    pub fn process(&mut self, output: &mut [Sample], sample_rate: u32) {
        let block_size = output.len();

        // Ensure scratch buffers are large enough.
        for buf in self.scratch.iter_mut() {
            buf.resize(block_size, 0.0);
            buf.iter_mut().for_each(|s| *s = 0.0);
        }

        // Process every node in insertion order.
        for idx in 0..self.nodes.len() {
            if self.nodes[idx].is_none() {
                continue;
            }

            // Collect input data into a temporary buffer to satisfy the borrow
            // checker – we need immutable access to upstream scratch buffers and
            // mutable access to `scratch[idx]` at the same time.
            let input_indices: Vec<usize> = self
                .connections
                .iter()
                .filter(|&&(_, to)| to == NodeId(idx))
                .map(|&(from, _)| from.0)
                .collect();

            let input_bufs: Vec<Vec<Sample>> = input_indices
                .iter()
                .map(|&i| self.scratch[i].clone())
                .collect();

            let input_refs: Vec<&[Sample]> = input_bufs.iter().map(|b| b.as_slice()).collect();

            // Temporarily take the node out so we can borrow scratch mutably.
            if let Some(mut node) = self.nodes[idx].take() {
                node.process(&input_refs, &mut self.scratch[idx], sample_rate);
                self.nodes[idx] = Some(node);
            }
        }

        // Copy the output node's scratch buffer to the final output.
        if let Some(out_id) = self.output_node {
            if out_id.0 < self.scratch.len() {
                let src = &self.scratch[out_id.0];
                let copy_len = output.len().min(src.len());
                output[..copy_len].copy_from_slice(&src[..copy_len]);
                return;
            }
        }

        // No output node – silence.
        output.iter_mut().for_each(|s| *s = 0.0);
    }
}

impl Default for AudioGraph {
    fn default() -> Self {
        Self::new()
    }
}
