# uciengine

[![documentation](https://docs.rs/uciengine/badge.svg)](https://docs.rs/uciengine) [![Crates.io](https://img.shields.io/crates/v/uciengine.svg)](https://crates.io/crates/uciengine) [![Crates.io (recent)](https://img.shields.io/crates/dr/uciengine)](https://crates.io/crates/uciengine)

Rust uci engine wrapper. Allows doing multiple searches from parallel asyncs. Searches are queued and done one by one in a way opaque to the receiver of the result. Analysis mode / streaming results while searching is not supported. You issue a go / ponderhit / pondermiss command and await on bestmove / ponder. ( Pondermiss is a fancy name for awaited stop command, to reflect the use case of a failed ponder. )

# Usage

```rust
extern crate env_logger;

use uciengine::uciengine::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let go_job1 = GoJob::new()
        .uci_opt("UCI_Variant", "atomic")
        .uci_opt("Hash", 128)
        .uci_opt("Threads", 4)
        .pos_fen("k7/8/8/8/8/8/R7/7K w - - 0 1")
        .pos_moves("h1h2")
        .tc(Timecontrol {
            wtime: 15000,
            winc: 0,
            btime: 15000,
            binc: 0,
        });

    let go_job2 = GoJob::new()
        .uci_opt("UCI_Variant", "chess")
        .pos_startpos()
        .go_opt("depth", 12);

    let engine = UciEngine::new("./stockfish12");

    // make two clones of the engine, so that we can move its clones to async blocks
    let (engine_clone1, engine_clone2) = (engine.clone(), engine.clone());

    // make two parallel async go commands, to demonstrate that they will be queued and processed one a time
    tokio::spawn(async move {
        let go_result1 = engine_clone1.go(go_job1).await;

        println!("go result 1 {:?}", go_result1);
    });

    tokio::spawn(async move {
        let go_result2 = engine_clone2.go(go_job2).await;

        println!("go result 2 {:?}", go_result2);
    });

    // wait enough for go commands to complete in the background
    tokio::time::sleep(tokio::time::Duration::from_millis(20000)).await;

    // quit engine
    engine.quit();

    // wait for engine to quit gracefully
    tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;

    Ok(())
}
```

# Logging

```bash
export RUST_LOG=info
# or
export RUST_LOG=debug
```
