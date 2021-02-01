extern crate env_logger;

use uciengine::analysis::*;

fn main() {
    env_logger::init();

    let mut ai = AnalysisInfo::new();

    let _ = ai.parse(
        "info depth 3 score mate 5 nodes 3000000000 time 3000 nps 1000000 pv e2e4 e7e5 g1f3",
    );

    println!("parsed ai {:?}", ai);

    println!(
        "bestmove {:?} ponder {:?} pv {:?}",
        ai.bestmove(),
        ai.ponder(),
        ai.pv()
    );

    let mut x = PvBuff::new().set("e2e4");

    println!("x = {:?}", x);

    x.set_trim("e2e4 e7e5 g1f3 b8c6", ' ');

    println!("x = {:?}", x);

    ai = AnalysisInfo::new();

    let _ = ai.parse("info depth 3 score mate 5 upperbound nodes 3000000000 time 3000 nps 1000000");

    if let Ok(json) = ai.to_json() {
        println!("ai as json {}", json);
    }
}
