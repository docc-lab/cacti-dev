use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

use uuid::Uuid;
use petgraph::{dot::Dot, Direction, graph::NodeIndex};
use crypto::digest::Digest;
use crypto::sha2::Sha256;

use trace::Event;
use trace::EventEnum;
use osprofiler::OSProfilerDAG;
use trace::DAGEdge;
use trace::DAGNode;
use trace::EdgeType;

#[derive(Debug, Clone)]
pub struct CriticalPath {
    pub g: OSProfilerDAG,
    pub start_node: NodeIndex,
    pub end_node: NodeIndex,
    duration: Duration,
    pub is_hypothetical: bool,
    hash: RefCell<Option<String>>
}

impl CriticalPath {
    pub fn from_trace(dag: &OSProfilerDAG) -> CriticalPath {
        let mut path = CriticalPath {
            duration: Duration::new(0, 0),
            g: OSProfilerDAG::new(dag.base_id),
            start_node: NodeIndex::end(),
            end_node: NodeIndex::end(),
            is_hypothetical: false,
            hash: RefCell::new(None)
        };
        let mut cur_node = dag.end_node;
        let mut end_nidx = path.g.g.add_node(dag.g[cur_node].clone());
        path.end_node = end_nidx;
        loop {
            let next_node = dag.g.neighbors_directed(cur_node, Direction::Incoming).max_by_key(|&nidx| dag.g[nidx].span.timestamp).unwrap();
            let start_nidx = path.g.g.add_node(dag.g[next_node].clone());
            path.g.g.add_edge(start_nidx, end_nidx, dag.g[dag.g.find_edge(next_node, cur_node).unwrap()].clone());
            if next_node == dag.start_node {
                path.start_node = start_nidx;
                break;
            }
            cur_node = next_node;
            end_nidx = start_nidx;
        }
        path.add_synthetic_nodes(dag);
        path
    }

    /// This method returns all possible critical paths that
    /// are generated by splitting the critical path into two at every
    /// concurrent part of the trace.
    pub fn all_possible_paths(dag: &OSProfilerDAG) -> Vec<CriticalPath> {
        let mut result = Vec::new();
        for end_node in dag.possible_end_nodes() {
            let mut path = CriticalPath {
                g: OSProfilerDAG::new(dag.base_id),
                start_node: NodeIndex::end(),
                end_node: NodeIndex::end(),
                duration: Duration::new(0, 0),
                is_hypothetical: true,
                hash: RefCell::new(None)
            };
            let cur_node = end_node;
            let end_nidx = path.g.g.add_node(dag.g[cur_node].clone());
            path.end_node = end_nidx;
            result.extend(CriticalPath::possible_paths_helper(dag, cur_node, end_nidx, path));
        }
        for i in &mut result {
            i.add_synthetic_nodes(dag);
            i.filter_incomplete_spans();
        }
        result
    }

    pub fn hash(&self) -> String {
        if self.hash.borrow().is_none() {
            self.calculate_hash();
        }
        self.hash.borrow().as_ref().unwrap().clone()
    }

    fn calculate_hash(&self) {
        let mut hasher = Sha256::new();
        let mut cur_node = self.start_node;
        loop {
            hasher.input_str(&self.g.g[cur_node].span.tracepoint_id);
            cur_node = match self.next_node(cur_node) {
                Some(node) => node,
                None => break
            };
        }
        *self.hash.borrow_mut() = Some(hasher.result_str());
    }

    fn possible_paths_helper(dag: &OSProfilerDAG, cur_node: NodeIndex, end_nidx: NodeIndex,
        mut path: CriticalPath) -> Vec<CriticalPath> {
        let next_nodes: Vec<_> = dag.g.neighbors_directed(cur_node, Direction::Incoming).collect();
        if next_nodes.len() == 0 {
            panic!("Path finished too early");
        } else if next_nodes.len() == 1 {
            let next_node = next_nodes[0];
            let start_nidx = path.g.g.add_node(dag.g[next_node].clone());
            path.g.g.add_edge(start_nidx, end_nidx, dag.g[dag.g.find_edge(next_node, cur_node).unwrap()].clone());
            if next_node == dag.start_node {
                path.start_node = start_nidx;
                vec![path]
            } else {
                CriticalPath::possible_paths_helper(dag, next_node, start_nidx, path)
            }
        } else {
            let mut result = Vec::new();
            for next_node in next_nodes {
                let mut new_path = path.clone();
                let start_nidx = new_path.g.g.add_node(dag.g[next_node].clone());
                new_path.g.g.add_edge(start_nidx, end_nidx, dag.g[dag.g.find_edge(next_node, cur_node).unwrap()].clone());
                if next_node == dag.start_node {
                    path.start_node = start_nidx;
                    result.push(new_path);
                } else {
                    result.extend(CriticalPath::possible_paths_helper(dag, next_node, start_nidx, new_path));
                }
            }
            result
        }
    }

    pub fn filter_incomplete_spans(&mut self) {
        let mut cur_node = self.start_node;
        let mut span_map = HashMap::new();
        let mut nodes_to_remove = Vec::new();
        let mut exits = HashMap::<Uuid,NodeIndex>::new();
        loop {
            let cur_trace_id = self.g.g[cur_node].span.trace_id;
            let existing_node_id = span_map.get(&cur_trace_id);
            match self.g.g[cur_node].span.variant {
                EventEnum::Entry => {
                    match existing_node_id {
                        Some(_) => {
                            nodes_to_remove.push(cur_node.clone());
                            assert!(exits.get(&cur_trace_id).is_none());
                        },
                        None => {
                            span_map.insert(cur_trace_id.clone(), cur_node.clone());
                        }
                    }
                },
                EventEnum::Annotation => {},
                EventEnum::Exit => {
                    match existing_node_id {
                        Some(_) => {
                            span_map.remove(&cur_trace_id);
                        },
                        None => {
                            match exits.get(&cur_trace_id) {
                                Some(node) => {
                                    nodes_to_remove.push(node.clone());
                                },
                                None => {
                                    println!("Current path: {}", Dot::new(&self.g.g));
                                    println!("Node to remove: {:?}", cur_node);
                                    panic!("We shouldn't have any incomplete spans");
                                }
                            }
                        }
                    }
                    exits.insert(cur_trace_id.clone(), cur_node.clone());
                }
            }
            cur_node = match self.next_node(cur_node) {
                Some(nidx) => nidx,
                None => break
            }
        }
        for nidx in nodes_to_remove {
            self.remove_node(nidx);
        }
    }

    /// We add synthetic nodes for spans with exit nodes off the critical path
    /// e.g.,
    /// A_start -> B_start -> C_start -> C_end -> ... rest of the path
    ///                   \-> D_start -> B_end -> A_end
    /// We add B_end and A_end (in that order) right before C_start
    fn add_synthetic_nodes(&mut self, dag: &OSProfilerDAG) {
        let mut cur_nidx = self.start_node;
        let mut cur_dag_nidx = dag.start_node;
        let mut active_spans = Vec::new();
        loop {
            let cur_node = &self.g.g[cur_nidx];
            let cur_dag_node = &dag.g[cur_dag_nidx];
            assert!(cur_node.span.trace_id == cur_dag_node.span.trace_id);
            match cur_node.span.variant {
                EventEnum::Entry => {
                    active_spans.push(cur_dag_node.span.clone());
                },
                EventEnum::Annotation => {},
                EventEnum::Exit => {
                    match active_spans.iter()
                        .rposition(|span| span.trace_id == cur_node.span.trace_id) {
                            Some(idx) => {
                                active_spans.remove(idx);
                            },
                            None => {
                                self.add_synthetic_start_node(cur_nidx, cur_dag_nidx, dag);
                            }
                        };
                }
            }
            let next_nidx = match self.next_node(cur_nidx) {
                Some(nidx) => nidx,
                None => break
            };
            let next_dag_nodes = dag.g.neighbors_directed(cur_dag_nidx, Direction::Outgoing).collect::<Vec<_>>();
            if next_dag_nodes.len() == 1 {
                cur_dag_nidx = next_dag_nodes[0];
            } else {
                assert!(next_dag_nodes.len() != 0);
                let mut unfinished_spans = Vec::new();
                for next_dag_nidx in next_dag_nodes {
                    if dag.g[next_dag_nidx].span.trace_id == self.g.g[next_nidx].span.trace_id {
                        cur_dag_nidx = next_dag_nidx;
                    } else {
                        unfinished_spans.extend(self.get_unfinished(
                                &active_spans, next_nidx, next_dag_nidx, dag));
                    }
                }
                for span in unfinished_spans.iter().rev() {
                    self.add_node_after(cur_nidx, span);
                    cur_nidx = self.next_node(cur_nidx).unwrap();
                }
            }
            cur_nidx = next_nidx;
        }
    }

    /// We encountered an end node for a span that did not start on our critical path
    /// We should go back, and add a corresponding synthetic start node after the correct
    /// synchronization point
    fn add_synthetic_start_node(&mut self, start_nidx: NodeIndex,
        start_dag_nidx: NodeIndex, dag: &OSProfilerDAG) {
        let span_to_add = self.g.g[start_nidx].span.clone();
        // Find synch. point
        let mut cur_nidx = start_nidx;
        let mut cur_dag_nidx = start_dag_nidx;
        loop {
            assert!(dag.g[cur_dag_nidx].span.trace_id == self.g.g[cur_nidx].span.trace_id);
            let prev_dag_nodes = dag.g
                .neighbors_directed(cur_dag_nidx, Direction::Incoming).collect::<Vec<_>>();
            let mut prev_nidx = cur_nidx;
            loop {
                prev_nidx = self.prev_node(prev_nidx).unwrap();
                if self.g.g[prev_nidx].span.parent_id != Uuid::nil() {
                    break;
                }
            }
            if prev_dag_nodes.len() == 1 {
                cur_dag_nidx = prev_dag_nodes[0];
                // We may have added other synthetic nodes to self, so iterate self
                // until we find matches in the dag
            } else {
                assert!(!prev_dag_nodes.is_empty());
                let mut found_start = false;
                for prev_dag_nidx in prev_dag_nodes {
                    if dag.g[prev_dag_nidx].span.trace_id == self.g.g[prev_nidx].span.trace_id {
                        cur_dag_nidx = prev_dag_nidx;
                    } else {
                        if self.find_start_node(&span_to_add, prev_dag_nidx, dag) {
                            found_start = true;
                        }
                    }
                }
                if found_start {
                    self.add_node_after(prev_nidx, &span_to_add);
                    return;
                }
            }
            cur_nidx = prev_nidx;
        }
    }

    fn find_start_node(&self, span: &Event, start_nidx: NodeIndex, dag: &OSProfilerDAG) -> bool {
        let mut cur_dag_nidx = start_nidx;
        loop {
            if dag.g[cur_dag_nidx].span.trace_id == span.trace_id {
                return true;
            }
            let prev_dag_nodes = dag.g
                .neighbors_directed(cur_dag_nidx, Direction::Incoming).collect::<Vec<_>>();
            if prev_dag_nodes.len() == 1 {
                cur_dag_nidx = prev_dag_nodes[0];
            } else {
                if prev_dag_nodes.is_empty() {
                    return false;
                }
                for prev_node in prev_dag_nodes {
                    if self.find_start_node(span, prev_node, dag) {
                        return true;
                    }
                }
                return false;
            }
        }
    }

    /// Get all of the active spans that are not finished in the rest of the critical path.
    /// A synthetic node will be added after all unfinished spans.
    ///
    /// The end of the unfinished span needs to be accessible through dag_nidx, otherwise we
    /// would be adding an erroneous edge
    fn get_unfinished(&self, spans: &Vec<Event>, nidx: NodeIndex,
        dag_nidx: NodeIndex, dag: &OSProfilerDAG) -> Vec<Event> {
        let mut unfinished = spans.clone();
        let mut cur_nidx = nidx;
        loop {
            for (idx, span) in unfinished.iter().enumerate() {
                if span.trace_id == self.g.g[cur_nidx].span.trace_id {
                    unfinished.remove(idx);
                    break;
                }
            }
            cur_nidx = match self.next_node(cur_nidx) {
                Some(nidx) => nidx,
                None => break
            };
        }
        unfinished.retain(|span| dag.can_reach_from_node(span.trace_id, dag_nidx));
        unfinished
    }

    pub fn next_node(&self, nidx: NodeIndex) -> Option<NodeIndex> {
        let mut matches = self.g.g.neighbors_directed(nidx, Direction::Outgoing);
        let result = matches.next();
        assert!(matches.next().is_none());
        result
    }

    pub fn prev_node(&self, nidx: NodeIndex) -> Option<NodeIndex> {
        let mut matches = self.g.g.neighbors_directed(nidx, Direction::Incoming);
        let result = matches.next();
        assert!(matches.next().is_none());
        result
    }

    fn remove_node(&mut self, nidx: NodeIndex) {
        let next_node = self.next_node(nidx);
        let prev_node = self.prev_node(nidx);
        match next_node {
            Some(next_nidx) => {
                self.g.g.remove_edge(self.g.g.find_edge(nidx, next_nidx).unwrap());
                match prev_node {
                    Some(prev_nidx) => {
                        self.g.g.remove_edge(self.g.g.find_edge(prev_nidx, nidx).unwrap());
                        self.g.g.add_edge(prev_nidx, next_nidx, DAGEdge{
                            duration: (self.g.g[next_nidx].span.timestamp
                                       - self.g.g[prev_nidx].span.timestamp).to_std().unwrap(),
                            variant: EdgeType::ChildOf});
                    },
                    None => {
                        self.start_node = next_nidx;
                    }
                }
            },
            None => {
                match prev_node {
                    Some(prev_nidx) => {
                        self.g.g.remove_edge(self.g.g.find_edge(prev_nidx, nidx).unwrap());
                        self.end_node = prev_nidx;
                    },
                    None => {
                        panic!("Something went wrong here");
                    }
                }
            }
        }
        self.g.g.remove_node(nidx);
    }

    /// Modifies the span to be exit/end, and changes timestamp
    fn add_node_after(&mut self, after: NodeIndex, node: &Event) {
        let next_node = self.next_node(after);
        let new_node = self.g.g.add_node(DAGNode{span: Event{
            tracepoint_id: node.tracepoint_id.clone(),
            variant: match node.variant {
                EventEnum::Entry => EventEnum::Exit,
                EventEnum::Exit => EventEnum::Entry,
                EventEnum::Annotation => panic!("don't give me annotation")
            },
            trace_id: node.trace_id,
            timestamp: self.g.g[after].span.timestamp + chrono::Duration::nanoseconds(1),
            parent_id: Uuid::nil()
        }});
        self.g.g.add_edge(after, new_node, DAGEdge{
            duration: Duration::new(0, 1), variant: EdgeType::ChildOf});
        match next_node {
            Some(next_nidx) => {
                let old_edge = self.g.g.find_edge(after, next_nidx).unwrap();
                let old_duration = self.g.g[old_edge].duration;
                self.g.g.remove_edge(old_edge);
                self.g.g.add_edge(new_node, next_nidx, DAGEdge{
                    duration: old_duration, variant: EdgeType::ChildOf});
            },
            None => {
                self.end_node = new_node;
            }
        }
    }
}
