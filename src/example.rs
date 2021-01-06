#[macro_use]
extern crate log;

use uciengine::uciengine::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	env_logger::init();
	
	info!("starting up");
	
	let go_job = GoJob::new()				
		.uci_opt("UCI_Variant", "atomic")
		.uci_opt("Hash", 128)
		.uci_opt("Threads", 4)
		.pos_fen("k7/8/8/8/8/8/R7/7K w - - 0 1")
		.pos_moves("h1h2")
		.tc(Timecontrol{
			wtime: 15000,
			winc: 0,
			btime: 15000,
			binc: 0
		})
	;
	
	println!("commands {:?}", go_job.to_commands());
		
	let mut pool = UciEnginePool::new();
	
	let engine = pool.create_engine("./stockfish12");
	
	/*let result = pool.go(engine, go_job).await?;
	
	println!("result {:?}", result);*/
	
	let mut rx = pool.enqueue_go_job(engine, go_job);
	
	let _ = rx.recv().await;
	
	Ok(())
}
