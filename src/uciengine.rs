use log::{debug, error, info, log_enabled, Level};

use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::*;

use crate::analysis::*;

/// enum of possible position specifiers
#[derive(Debug)]
pub enum PosSpec {
    /// starting position
    Startpos,
    /// position from fen
    Fen,
    /// position not specified
    No,
}

use PosSpec::*;

/// go command job
#[derive(Debug)]
pub struct GoJob {
    /// uci options as key value pairs
    uci_options: HashMap<String, String>,
    /// position specifier
    pos_spec: PosSpec,
    /// position fen
    pos_fen: Option<String>,
    /// position moves
    pos_moves: Option<String>,
    /// go command options as key value pairs
    go_options: HashMap<String, String>,
    /// custom command
    custom_command: Option<String>,
    /// ponder ( go option )
    ponder: bool,
    /// ponderhit ( ponderhit uci commend )
    ponderhit: bool,
    /// pondermiss ( alias to awaited stop )
    pondermiss: bool,
    /// result sender
    rtx: Option<oneshot::Sender<GoResult>>,
}

/// time control ( all values are in milliseconds )
#[derive(Debug)]
pub struct Timecontrol {
    /// white time
    pub wtime: usize,
    /// white increment
    pub winc: usize,
    /// black time
    pub btime: usize,
    /// black increment
    pub binc: usize,
}

/// implementation of time control
impl Timecontrol {
    /// create default time control
    /// ( one minute thinking time for both sides, no increment )
    pub fn default() -> Self {
        Self {
            wtime: 60000,
            winc: 0,
            btime: 60000,
            binc: 0,
        }
    }
}

/// go command job implementation
impl GoJob {
    /// create new GoJob with defaults
    pub fn new() -> Self {
        Self {
            pos_spec: No,
            pos_fen: None,
            pos_moves: None,
            uci_options: HashMap::new(),
            go_options: HashMap::new(),
            rtx: None,
            custom_command: None,
            ponder: false,
            ponderhit: false,
            pondermiss: false,
        }
    }

    /// set custom command and return self,
    /// if set, other settings will be ignored
    /// and only this single command will be sent,
    /// returns self
    pub fn custom<T>(mut self, command: T) -> Self
    where
        T: core::fmt::Display,
    {
        self.custom_command = Some(format!("{}", command));

        self
    }

    /// convert go job to commands
    pub fn to_commands(&self) -> Vec<String> {
        let mut commands: Vec<String> = vec![];

        if self.ponderhit {
            commands.push(format!("{}", "ponderhit"));

            return commands;
        }

        if self.pondermiss {
            commands.push(format!("{}", "stop"));

            return commands;
        }

        if let Some(command) = &self.custom_command {
            commands.push(format!("{}", command));

            return commands;
        }

        for (key, value) in &self.uci_options {
            commands.push(format!("setoption name {} value {}", key, value));
        }

        let mut pos_command_moves = "".to_string();

        if let Some(pos_moves) = &self.pos_moves {
            pos_command_moves = format!(" moves {}", pos_moves)
        }

        let pos_command: Option<String> = match self.pos_spec {
            Startpos => Some(format!("position startpos{}", pos_command_moves)),
            Fen => {
                let fen = match &self.pos_fen {
                    Some(fen) => fen,
                    _ => "",
                };
                Some(format!("position fen {}{}", fen, pos_command_moves))
            }
            _ => None,
        };

        if let Some(pos_command) = pos_command {
            commands.push(pos_command);
        }

        let mut go_command = "go".to_string();

        for (key, value) in &self.go_options {
            go_command = go_command + &format!(" {} {}", key, value);
        }

        if self.ponder {
            go_command = go_command + &format!(" {}", "ponder");
        }

        commands.push(go_command);

        commands
    }

    /// set ponder and return self
    pub fn set_ponder(mut self, value: bool) -> Self {
        self.ponder = value;

        self
    }

    /// set ponder to true and return self
    pub fn ponder(mut self) -> Self {
        self.ponder = true;

        self
    }

    /// set ponderhit and return self
    pub fn ponderhit(mut self) -> Self {
        self.ponderhit = true;

        self
    }

    /// set pondermiss and return self
    pub fn pondermiss(mut self) -> Self {
        self.pondermiss = true;

        self
    }

    /// set position fen and return self
    pub fn pos_fen<T>(mut self, fen: T) -> Self
    where
        T: core::fmt::Display,
    {
        self.pos_spec = Fen;
        self.pos_fen = Some(format!("{}", fen).to_string());

        self
    }

    /// set position startpos and return self
    pub fn pos_startpos(mut self) -> Self {
        self.pos_spec = Startpos;

        self
    }

    /// set position moves and return self,
    /// moves should be a space separated string of uci moves,
    /// as described by the UCI protocol
    ///
    /// ### Example
    /// ```
    /// use uciengine::uciengine::GoJob;
    ///
    /// let go_job = GoJob::new()
    ///                .pos_startpos()
    ///                .pos_moves("e2e4 e7e5 g1f3");
    /// ```
    pub fn pos_moves<T>(mut self, moves: T) -> Self
    where
        T: core::fmt::Display,
    {
        self.pos_moves = Some(format!("{}", moves));

        self
    }

    /// set uci option as key value pair and return self
    pub fn uci_opt<K, V>(mut self, key: K, value: V) -> Self
    where
        K: core::fmt::Display,
        V: core::fmt::Display,
    {
        self.uci_options
            .insert(format!("{}", key), format!("{}", value));

        self
    }

    /// set go option as key value pair and return self
    pub fn go_opt<K, V>(mut self, key: K, value: V) -> Self
    where
        K: core::fmt::Display,
        V: core::fmt::Display,
    {
        self.go_options
            .insert(format!("{}", key), format!("{}", value));

        self
    }

    /// set time control and return self
    pub fn tc(mut self, tc: Timecontrol) -> Self {
        self.go_options
            .insert("wtime".to_string(), format!("{}", tc.wtime));
        self.go_options
            .insert("winc".to_string(), format!("{}", tc.winc));
        self.go_options
            .insert("btime".to_string(), format!("{}", tc.btime));
        self.go_options
            .insert("binc".to_string(), format!("{}", tc.binc));

        self
    }
}

/// go command result
#[derive(Debug)]
pub struct GoResult {
    /// best move if any
    pub bestmove: Option<String>,
    /// ponder if any
    pub ponder: Option<String>,
    /// analysis info
    pub ai: AnalysisInfo,
}

/// uci engine
pub struct UciEngine {
    gtx: mpsc::UnboundedSender<GoJob>,
    ai: std::sync::Arc<std::sync::Mutex<AnalysisInfo>>,
}

/// uci engine implementation
impl UciEngine {
    /// create new uci engine
    pub fn new<T>(path: T) -> std::sync::Arc<UciEngine>
    where
        T: core::fmt::Display,
    {
        // you can use anything that can be converted to string as path
        let path = path.to_string();

        // spawn engine process
        let mut child = Command::new(path.as_str())
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .expect("failed to spawn engine");

        // obtain process stdout
        let stdout = child
            .stdout
            .take()
            .expect("child did not have a handle to stdout");

        // obtain process stdin
        let stdin = child
            .stdin
            .take()
            .expect("child did not have a handle to stdin");

        // stdout reader
        let reader = BufReader::new(stdout).lines();

        // channel for receiving bestmove result
        let (tx, rx) = mpsc::unbounded_channel::<String>();

        tokio::spawn(async move {
            // run engine process and wait for exit code
            let status = child
                .wait()
                .await
                .expect("engine process encountered an error");

            if log_enabled!(Level::Info) {
                info!("engine process exit status : {}", status);
            }
        });

        let ai = std::sync::Arc::new(std::sync::Mutex::new(AnalysisInfo::new()));

        let ai_clone = ai.clone();

        tokio::spawn(async move {
            let mut reader = reader;
            let ai = ai_clone;

            loop {
                match reader.next_line().await {
                    Ok(line_opt) => {
                        if let Some(line) = line_opt {
                            if log_enabled!(Level::Debug) {
                                debug!("uci engine out : {}", line);
                            }

                            {
                                let mut ai = ai.lock().unwrap();

                                let _ = ai.parse(line.to_owned());

                                debug!("{:?}", ai);
                            }

                            if line.len() >= 8 {
                                if &line[0..8] == "bestmove" {
                                    let send_result = tx.send(line);

                                    if log_enabled!(Level::Debug) {
                                        debug!("send bestmove result {:?}", send_result);
                                    }
                                }
                            }
                        } else {
                            if log_enabled!(Level::Debug) {
                                debug!("engine returned empty line option");
                            }

                            break;
                        }
                    }
                    Err(err) => {
                        if log_enabled!(Level::Error) {
                            error!("engine read error {:?}", err);
                        }

                        break;
                    }
                }
            }

            if log_enabled!(Level::Debug) {
                debug!("engine read terminated");
            }
        });

        // channel for sending go jobs
        let (gtx, grx) = mpsc::unbounded_channel::<GoJob>();

        let ai_clone = ai.clone();

        tokio::spawn(async move {
            let mut stdin = stdin;
            let mut grx = grx;
            let mut rx = rx;
            let ai = ai_clone;

            while let Some(go_job) = grx.recv().await {
                if log_enabled!(Level::Debug) {
                    debug!("received go job {:?}", go_job);
                }

                for command in go_job.to_commands() {
                    let command = format!("{}\n", command);

                    if log_enabled!(Level::Debug) {
                        debug!("issuing engine command : {}", command);
                    }

                    let write_result = stdin.write_all(command.as_bytes()).await;

                    if log_enabled!(Level::Debug) {
                        debug!("write result {:?}", write_result);
                    }
                }

                if go_job.custom_command.is_none() && (!go_job.ponder) {
                    {
                        let mut ai = ai.lock().unwrap();

                        *ai = AnalysisInfo::new();
                    }

                    let recv_result = rx.recv().await.unwrap();

                    if log_enabled!(Level::Debug) {
                        debug!("recv result {:?}", recv_result);
                    }

                    let parts: Vec<&str> = recv_result.split(" ").collect();

                    let send_ai: AnalysisInfo;

                    {
                        let ai = ai.lock().unwrap();

                        send_ai = *ai;
                    }

                    let mut go_result = GoResult {
                        bestmove: None,
                        ponder: None,
                        ai: send_ai,
                    };

                    if parts.len() > 1 {
                        go_result.bestmove = Some(parts[1].to_string());
                    }

                    if parts.len() > 3 {
                        go_result.ponder = Some(parts[3].to_string());
                    }

                    let send_result = go_job.rtx.unwrap().send(go_result);

                    if log_enabled!(Level::Debug) {
                        debug!("result of send go result {:?}", send_result);
                    }
                }
            }
        });

        if log_enabled!(Level::Info) {
            info!("spawned uci engine : {}", path);
        }

        std::sync::Arc::new(UciEngine { gtx: gtx, ai: ai })
    }

    /// get analysis info
    pub fn get_ai(&self) -> AnalysisInfo {
        let ai = self.ai.lock().unwrap();

        *ai
    }

    /// issue go command
    pub fn go(&self, go_job: GoJob) -> oneshot::Receiver<GoResult> {
        let mut go_job = go_job;

        let (rtx, rrx): (oneshot::Sender<GoResult>, oneshot::Receiver<GoResult>) =
            oneshot::channel();

        go_job.rtx = Some(rtx);

        let send_result = self.gtx.send(go_job);

        if log_enabled!(Level::Debug) {
            debug!("send go job result {:?}", send_result);
        }

        rrx
    }

    /// quit engine
    pub fn quit(&self) {
        self.go(GoJob::new().custom("quit"));
    }
}
