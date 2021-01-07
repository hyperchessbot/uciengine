# uciengine

[![documentation](https://docs.rs/uciengine/badge.svg)](https://docs.rs/uciengine) [![Crates.io](https://img.shields.io/crates/v/uciengine.svg)](https://crates.io/crates/uciengine) [![Crates.io (recent)](https://img.shields.io/crates/dr/uciengine)](https://crates.io/crates/uciengine)

Rust uci engine wrapper.

# Usage

```rust
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
		.uci_opt("UCI_Variant", "chess")
		.pos_startpos()
		.go_opt("depth", 12)
	;
			
	let engine = UciEngine::new("./stockfish12");
	
	let ( engine_clone1 , engine_clone2 ) = ( engine.clone(), engine.clone() );
	
	tokio::spawn(async move {	
		let engine = engine_clone1;
	
		let go_result = engine.go(go_job1).recv().await;

		println!("go result 1 {:?}", go_result);
	});
	
	tokio::spawn(async move {		
		let engine = engine_clone2;
	
		let go_result = engine.go(go_job2).recv().await;

		println!("go result 2 {:?}", go_result);
		
		engine.quit();
	});
	
	tokio::time::sleep(tokio::time::Duration::from_millis(20000)).await;
		
	Ok(())
}
```

# Logging

```bash
export RUST_LOG=info
# or
export RUST_LOG=debug
```
