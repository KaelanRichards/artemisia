use aurion_core::{NodeGraph, NodeError, Node};
use aurion_std_nodes::{ImageNode, BlendNode, BlendMode};

fn main() -> Result<(), NodeError> {
    let mut graph = NodeGraph::new();

    // Create nodes
    let image_node = Node::new(Box::new(ImageNode::new()));
    let blend_node = Node::new(Box::new(BlendNode::new(BlendMode::Normal)));

    // Add nodes to graph
    let image_id = graph.add_node(image_node);
    let blend_id = graph.add_node(blend_node);

    // Connect nodes
    graph.connect(&image_id, &blend_id, "input")?;

    println!("Graph created successfully!");
    Ok(())
}
