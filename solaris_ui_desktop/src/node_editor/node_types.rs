use aurion_core::{Node, NodeData};
use aurion_std_nodes::{ImageNode, AiImageGenNode, ColorAdjustNode};

#[derive(Debug, Clone, Copy)]
pub enum NodeType {
    Image,
    AiImageGen,
    ColorAdjust,
}

impl NodeType {
    pub fn all() -> &'static [NodeType] {
        &[
            NodeType::Image,
            NodeType::AiImageGen,
            NodeType::ColorAdjust,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            NodeType::Image => "Image",
            NodeType::AiImageGen => "AI Image Generation",
            NodeType::ColorAdjust => "Color Adjustment",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            NodeType::Image => "Holds and displays an image",
            NodeType::AiImageGen => "Generates images using AI",
            NodeType::ColorAdjust => "Adjusts image colors",
        }
    }

    pub fn create(&self) -> Option<Node> {
        let node_data: Box<dyn NodeData> = match self {
            NodeType::Image => Box::new(ImageNode::new()),
            NodeType::AiImageGen => Box::new(AiImageGenNode::new("".to_string())),
            NodeType::ColorAdjust => Box::new(ColorAdjustNode::new(1.0, 1.0, 1.0)),
        };

        Some(Node::new(node_data))
    }

    pub fn input_slots(&self) -> Vec<(&'static str, &'static str)> {
        match self {
            NodeType::Image => vec![],
            NodeType::AiImageGen => vec![],
            NodeType::ColorAdjust => vec![
                ("input", "Image input to adjust"),
            ],
        }
    }

    pub fn output_slots(&self) -> Vec<(&'static str, &'static str)> {
        match self {
            NodeType::Image => vec![
                ("output", "Image output"),
            ],
            NodeType::AiImageGen => vec![
                ("output", "Generated image"),
            ],
            NodeType::ColorAdjust => vec![
                ("output", "Adjusted image"),
            ],
        }
    }

    pub fn parameters(&self) -> Vec<NodeParameter> {
        match self {
            NodeType::Image => vec![],
            NodeType::AiImageGen => vec![
                NodeParameter::String {
                    name: "prompt",
                    description: "Text prompt for image generation",
                    default: "".to_string(),
                },
            ],
            NodeType::ColorAdjust => vec![
                NodeParameter::Float {
                    name: "brightness",
                    description: "Image brightness adjustment",
                    default: 1.0,
                    min: 0.0,
                    max: 2.0,
                },
                NodeParameter::Float {
                    name: "contrast",
                    description: "Image contrast adjustment",
                    default: 1.0,
                    min: 0.0,
                    max: 2.0,
                },
                NodeParameter::Float {
                    name: "saturation",
                    description: "Image saturation adjustment",
                    default: 1.0,
                    min: 0.0,
                    max: 2.0,
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeParameter {
    Float {
        name: &'static str,
        description: &'static str,
        default: f32,
        min: f32,
        max: f32,
    },
    String {
        name: &'static str,
        description: &'static str,
        default: String,
    },
}

impl NodeParameter {
    pub fn name(&self) -> &'static str {
        match self {
            NodeParameter::Float { name, .. } => name,
            NodeParameter::String { name, .. } => name,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            NodeParameter::Float { description, .. } => description,
            NodeParameter::String { description, .. } => description,
        }
    }
} 