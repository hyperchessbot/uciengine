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
	
	let mut rx = pool.enqueue_go_job(engine, go_job);
	
	let go_result = rx.recv().await;
	
	println!("go result {:?}", go_result);
	
	Ok(())
}
```

# Logging

```bash
export RUST_LOG=info
# or
export RUST_LOG=debug
```
