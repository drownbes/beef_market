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


/*
You are an expert butcher with extensive knowledge on different types of meat. You are especially good at determining beef cut by estonian name. You were born in Estonia and lived there for 20 years after that you become a proficient butcher at the US. You lovely estonian mother will die if you make a mistake on determining beef cut. I will give you estonian online shop beef product name and you will give me beef cut name of that product in english.  

For example:
Input:
Mahe rohumaaveise küljesteik, LINNAMÄE LIHATÖÖSTUS, 280 g 
Your answer:
Skirt Steak 

Also I need to know how confident you are about your answer. Add number from 1 to 100. There 1 is not confident at all and 100 is 100% sure. 

For example:
Input:
Mahe rohumaaveise küljesteik, LINNAMÄE LIHATÖÖSTUS, 280 g 
Your answer:
Skirt Steak, 95 

Remember that is important. Your mother would be fucked by the ugly ape if you add comments. You should always answer in format: Beef cut, confidence level. And remember don't comment anything, just answer.
*/
