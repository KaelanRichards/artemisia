use aurion_std_nodes;

fn main() {
    println!("Starting Polaris Application for Artemisia...");

    let ai_node = aurion_std_nodes::AiImageGenNode::new("A picturesque sunset over mountains");
    match ai_node.run() {
        Ok(data) => {
            println!("Received {} bytes of AI-generated image data", data.len());
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
