use uciengine::uciengine::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let go_job = GoJob::new()		
		.uci_opt("UCI_Variant".to_string(), "atomic".to_string())
		.uci_opt("Hash".to_string(), "128".to_string())
		.uci_opt("Threads".to_string(), "4".to_string())
		.tc(Timecontrol::default())
	;
	
	println!("go job {:?}", go_job);
	
	let mut uciengine = UciEngine::new("./stockfish12".to_string());
	
	let result = uciengine.go(go_job).await?;
	
	println!("result {:?}", result);
	
	Ok(())
}
