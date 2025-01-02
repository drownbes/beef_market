use anyhow::anyhow;
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;
use ollama_rs::Ollama;

use crate::config::OllamaConfig;

pub struct OllamaRunner {
    ollama: Ollama,
    pub embedding_model: String,
}

impl OllamaRunner {
    pub fn new(cfg: &OllamaConfig) -> OllamaRunner {
        let ollama = Ollama::new(format!("http://{}", &cfg.host), cfg.port);
        OllamaRunner {
            ollama,
            embedding_model: cfg.embedding_model.clone(),
        }
    }

    pub async fn create_embedding(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let request = GenerateEmbeddingsRequest::new(self.embedding_model.clone(), text.into());
        let res = self.ollama.generate_embeddings(request).await?;
        res.embeddings
            .into_iter()
            .nth(0)
            .ok_or(anyhow!("Empty embeddings returned"))
    }
}
