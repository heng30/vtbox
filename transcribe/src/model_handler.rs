use anyhow::Result;
use reqwest::{Client, Proxy};
use std::time::Duration;

pub struct ModelHandler {
    model_name: String, // list of downloaded models
    models_dir: String, // path to the models directory
}

impl ModelHandler {
    pub async fn new(
        model_name: &str,
        models_dir: &str,
        proxy_info: Option<(&str, u16)>,
    ) -> Result<ModelHandler> {
        let model_handler = ModelHandler {
            model_name: model_name.to_string(),
            models_dir: models_dir.to_string(),
        };

        if model_handler.is_model_existing() {
            return Ok(model_handler);
        }

        let _ = model_handler.setup_directory();
        match model_handler.download_model(proxy_info).await {
            Err(e) => {
                let _ = std::fs::remove_file(format!("{}/{}", models_dir, model_name));
                Err(e)
            }
            _ => Ok(model_handler),
        }
    }

    fn setup_directory(&self) -> Result<()> {
        let path = std::path::Path::new(&self.models_dir);
        if !path.exists() {
            let _ = std::fs::create_dir_all(path)?;
        }
        Ok(())
    }

    fn is_model_existing(&self) -> bool {
        match std::fs::metadata(format!("{}/{}", self.models_dir, self.model_name)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub async fn download_model(&self, proxy_info: Option<(&str, u16)>) -> Result<()> {
        if !self.is_model_existing() {
            self.setup_directory()?;
        }

        let base_url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";
        let url = format!("{}/{}", base_url, &self.model_name);

        let client = if let Some((ip, port)) = proxy_info {
            let proxy = Proxy::all(format!("socks5://{}:{}", ip, port))?;
            Client::builder().proxy(proxy).build()?
        } else {
            Client::new()
        };

        let response = client
            .get(&url)
            .timeout(Duration::from_secs(30))
            .send()
            .await?;

        let mut file =
            std::fs::File::create(format!("{}/{}", &self.models_dir, &self.model_name))?;
        let mut content = std::io::Cursor::new(response.bytes().await?);
        std::io::copy(&mut content, &mut file)?;
        Ok(())
    }

    pub fn get_model_dir(&self) -> String {
        format!("{}/{}", &self.models_dir, &self.model_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_model_exists_existent_path() {
        let path = std::path::Path::new("test_models/ggml-tiny.bin");
        if !path.exists() {
            let _ = std::fs::create_dir_all(path);
        }

        let test_model = ModelHandler::new("ggml-tiny.bin", "test_models/", None).await.unwrap();
        let result = test_model.is_model_existing();
        assert_eq!(result, true);
    }

    #[tokio::test]
    async fn test_setup_directory_happy_case() {
        let path = std::path::Path::new("test_models/ggml-tiny.bin");
        if !path.exists() {
            let _ = std::fs::create_dir_all(path);
        }

        let test_model = ModelHandler::new("ggml-tiny.bin", "test_models/", None).await.unwrap();
        let result = test_model.setup_directory();
        assert_eq!(result.is_ok(), true);
        let _ = std::fs::remove_dir_all("test_models/");
    }

    #[tokio::test]
    async fn test_download_model_happy_case() {
        fn prep_test_dir() {
            let path = std::path::Path::new("test_dir/");
            if !path.exists() {
                let _ = std::fs::create_dir_all(path);
            }
        }

        prep_test_dir();

        let model_handler = ModelHandler::new("ggml-tiny.bin", "test_dir/", None).await.unwrap();

        let _result = model_handler.download_model(None).await;

        let is_file_existing = match std::fs::metadata("test_dir/ggml-tiny.bin") {
            Ok(_) => true,
            Err(_) => false,
        };

        assert_eq!(is_file_existing, true);

        let _ = std::fs::remove_dir_all("test_dir/");
    }
}
