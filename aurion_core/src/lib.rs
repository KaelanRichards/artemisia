pub trait Node {
    fn compute(&self, inputs: &[&dyn std::any::Any]) -> Box<dyn std::any::Any>;
}

pub struct NodeGraph {}

impl NodeGraph {
    pub fn new() -> Self { Self {} }
    pub fn evaluate(&self) {
        // Future: implement evaluation logic
    }
}
