use reqwest::blocking::Client;
use serde::Serialize;

#[derive(Serialize)]
struct ComfyRequest {
    prompt: String,
    // Add other required fields based on ComfyUI’s workflow
}

pub struct AiImageGenNode {
    prompt: String,
}

impl AiImageGenNode {
    pub fn new(prompt: &str) -> Self {
        Self { prompt: prompt.to_string() }
    }

    pub fn run(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let client = Client::new();
        let request_body = ComfyRequest {
            prompt: self.prompt.clone(),
        };

        // Adjust URL/endpoint according to ComfyUI’s API
        let resp = client.post("http://127.0.0.1:8188/run_workflow")
            .json(&request_body)
            .send()?
            .error_for_status()?;

        let resp_bytes = resp.bytes()?;
        Ok(resp_bytes.to_vec())
    }
}
