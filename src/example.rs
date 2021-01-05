use uciengine::uciengine::*;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let uciengine = UciEngine::new("stockfish12")
	
	println!("{}", uciengine);
	
	Ok(())
}
