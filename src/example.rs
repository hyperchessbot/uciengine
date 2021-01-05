use uciengine::uciengine::*;

#[tokio::main]
async fn main() {
	let uciengine = UciEngine::new("stockfish12".to_string());
	
	println!("{:?}", uciengine);
}
