use anyhow::Result;
use transcribe::model_handler;
use transcribe::transcriber;

// "ggml-tiny.bin", "ggml-base.bin", "ggml-small.bin", "ggml-medium.bin", "ggml-large.bin",
#[tokio::main]
async fn main() -> Result<()> {
    let m = model_handler::ModelHandler::new("ggml-tiny.bin", "models", None).await?;
    // let m = model_handler::ModelHandler::new("ggml-tiny.bin", "models", Some(("127.0.0.1", 1084))).await?;
    let trans = transcriber::Transcriber::new(m)?;
    let result = trans.transcribe("src/test_data/test.mp3", None)?;
    let text = result.get_text();
    let start = result.get_start_timestamp();
    let end = result.get_end_timestamp();
    println!("start[{}]-end[{}] {}", start, end, text);

    Ok(())
}
