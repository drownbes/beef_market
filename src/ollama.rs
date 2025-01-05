use anyhow::anyhow;
use ollama_rs::generation::chat::request::ChatMessageRequest;
use ollama_rs::generation::chat::ChatMessage;
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

    pub async fn generate(&self, prompt: &str, message: &str) -> anyhow::Result<Option<String>> {
        let msg = format!("{} \n\n {}", prompt, message);
        let request = ChatMessageRequest::new(
            "llama3.1:8b".to_string(),
            vec![ChatMessage::user(msg.to_string())],
        );
        let res = self.ollama.send_chat_messages(request).await?;
        Ok(res.message.map(|x| x.content))
    }

    pub async fn guess_beef_cut(&self, name: &str) -> anyhow::Result<Option<(String, i64)>> {
        let answer_str = self.generate(PROMPT, name).await?;
        let res = answer_str.and_then(|ans| {
            if ans.trim() == "NOPE" {
                None
            } else {
                let mut parts = ans.split(",");
                let cut = parts.next()?.trim();
                let per: i64 = parts.next()?.trim().parse().ok()?;
                Some((cut.to_string(), per))
            }
        });
        Ok(res)
    }
}

const PROMPT: &str = "
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

Remember that is important. Your mother would be fucked by the ugly ape if you add comments. You should always answer in format: Beef cut, confidence level. And remember don't comment anything, just answer. If product is not beef cut answer NOPE.
";

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate() {
        let ollama = OllamaRunner::new(&OllamaConfig {
            host: "127.0.0.1".into(),
            port: 11434,
            embedding_model: "snowflake-arctic-embed2".into(),
        });
        let answer = ollama
            .generate(PROMPT, "Veise antrekoot, RAKVERE LK, kg")
            .await
            .unwrap();
        dbg!(answer);
    }

    #[tokio::test]
    async fn test_beef_cut_detector() {
        let ollama = OllamaRunner::new(&OllamaConfig {
            host: "127.0.0.1".into(),
            port: 11434,
            embedding_model: "snowflake-arctic-embed2".into(),
        });
        let answer = ollama
            .guess_beef_cut("Veise antrekoot, RAKVERE LK, kg")
            .await
            .unwrap();
        dbg!(answer);
    }
}
