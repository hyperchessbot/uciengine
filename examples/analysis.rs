extern crate env_logger;

use uciengine::analysis::*;

fn main() {
    env_logger::init();

    let mut ai = AnalysisInfo::new();

    ai.parse("info depth 3 score mate 5 nodes 3000000000 time 3000 nps 1000000 pv e2e4 e7e5 g1f3");

    println!("parsed ai {:?}", ai);

    println!(
        "bestmove {:?} ponder {:?} pv {:?}",
        ai.bestmove(),
        ai.ponder(),
        ai.pv()
    );
}
