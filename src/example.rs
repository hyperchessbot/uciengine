#[macro_use]
extern crate log;

use uciengine::uciengine::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	env_logger::init();
	
	info!("starting up");
	
	let go_job1 = GoJob::new()				
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
	
	let go_job2 = GoJob::new()						
		.pos_startpos()
		.pos_moves("e2e4")
		.tc(Timecontrol{
			wtime: 15000,
			winc: 0,
			btime: 15000,
			binc: 0
		})
	;
			
	let pool = std::sync::Arc::new(UciEnginePool::new());
		
	let engine = std::sync::Arc::new(pool.create_engine("./stockfish12"));
	
	let engine_clone1 = engine.clone();
	let engine_clone2 = engine.clone();
	
	tokio::spawn(async move {	
		let engine = engine_clone1;
		
		let mut rx = UciEnginePool::enqueue_go_job(engine, go_job1);
	
		let go_result = rx.recv().await;

		println!("go result 1 {:?}", go_result);
	});
	
	tokio::spawn(async move {		
		let engine = engine_clone2;
		
		let mut rx = UciEnginePool::enqueue_go_job(engine, go_job2);
	
		let go_result = rx.recv().await;

		println!("go result 2 {:?}", go_result);
	});
	
	tokio::time::sleep(tokio::time::Duration::from_millis(10000)).await;
		
	Ok(())
}
