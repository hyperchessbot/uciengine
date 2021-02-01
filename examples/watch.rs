extern crate env_logger;

use uciengine::uciengine::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let go_job = GoJob::new()
        .uci_opt("UCI_Variant", "chess")
        .pos_startpos()
        .pos_moves("e2e4 e7e5")
        .go_opt("depth", 10);

    let engine = UciEngine::new("stockfish12.exe");

    // start engine detached
    let _ = engine.go(go_job);

    let mut arx = engine.atx.subscribe();

    loop {
        let rec_result = arx.recv().await;

        println!("rec result {:?}", rec_result);

        if let Ok(rec_result) = rec_result {
            if rec_result.done {
                break;
            }
        }
    }

    Ok(())
}
