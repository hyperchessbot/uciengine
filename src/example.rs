use uciengine::uciengine::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut uciengine = UciEngine::new("./stockfish12".to_string());
	
	let _ = uciengine.go().await?;
	
	std::thread::sleep(std::time::Duration::from_millis(5000));
	
	println!("done");
	
	Ok(())
}
