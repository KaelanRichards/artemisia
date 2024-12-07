use aurion_std_nodes::ImageNode;
use aurion_core::Node;

pub enum NodeType {
    Image,
}

impl NodeType {
    pub fn all() -> &'static [NodeType] {
        &[NodeType::Image]
    }

    pub fn name(&self) -> &'static str {
        match self {
            NodeType::Image => "Image",
        }
    }

    pub fn create(&self) -> Option<Node> {
        match self {
            NodeType::Image => Some(Node::new(Box::new(ImageNode::new()))),
        }
    }
} 