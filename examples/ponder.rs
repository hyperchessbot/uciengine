extern crate env_logger;

use uciengine::uciengine::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let go_job = GoJob::new()
        .uci_opt("UCI_Variant", "chess")
        .pos_startpos()
        .pos_moves("e2e4 e7e5")
        .ponder()
        .tc(Timecontrol {
            wtime: 15000,
            winc: 0,
            btime: 15000,
            binc: 0,
        });

    let engine = UciEngine::new("stockfish12.exe");

    // start engine detached
    let _ = engine.go(go_job);

    // do something in the meanwhile
    println!("doing something");

    // issue ponderhit
    let result = engine.go(GoJob::new().ponderhit()).await;

    // issue pondermiss
    //let result = engine.go(GoJob::new().pondermiss()).recv().await;

    println!("{:?}", result);

    Ok(())
}
