use log::{debug, log_enabled, info, Level};

use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt, AsyncWriteExt};
use std::process::Stdio;
use std::collections::HashMap;
use tokio::sync::mpsc::{Sender, Receiver};

/// enum of possible position sepcifiers
#[derive(Debug)]
pub enum PosSpec{
	/// starting position
	Startpos,
	/// position from fen
	Fen,
	/// position not specified
	No
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
	/// result sender
	rtx: Option<Sender<GoResult>>,
}

/// time control
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
	pub fn default() -> Timecontrol {
		Timecontrol {
			wtime: 60000,
			winc: 0,
			btime: 60000,
			binc: 0,
		}
	}
}

/// go command job implementation
impl GoJob {
	/// create new GoJob with reasonable defaults
	pub fn new() -> GoJob {
		GoJob {
			pos_spec: No,
			pos_fen: None,
			pos_moves: None,
			uci_options: HashMap::new(),
			go_options: HashMap::new(),
			rtx: None,
		}
	}
	
	/// to commands
	pub fn to_commands(&self) -> Vec<String> {
		let mut commands:Vec<String> = vec!();
		
		for (key, value) in &self.uci_options {
			commands.push(format!("setoption name {} value {}", key, value));			
		}
		
		let mut pos_command_moves = "".to_string();
		
		if let Some(pos_moves) = &self.pos_moves {
			pos_command_moves = format!(" moves {}", pos_moves)
		}
		
		let pos_command:Option<String> = match self.pos_spec {
			Startpos => Some(format!("position startpos{}", pos_command_moves)),
			Fen => {
				let fen = match &self.pos_fen {
					Some(fen) => fen,
					_ => "",
				};				
				Some(format!("position fen {}{}", fen, pos_command_moves))
			},
			_ => None
		};
		
		if let Some(pos_command) = pos_command {
			commands.push(pos_command);
		}
		
		let mut go_command = "go".to_string();
		
		for (key, value) in &self.go_options {
			go_command = go_command + &format!(" {} {}", key, value);
		}
		
		commands.push(go_command);
		
		commands
	}
	
	/// set position fen and return self
	pub fn pos_fen<T>(mut self, fen: T) -> GoJob where
	T: core::fmt::Display {
		self.pos_spec = Fen;
		self.pos_fen = Some(format!("{}", fen).to_string());
		
		self
	}
	
	/// set position startpos and return self
	pub fn pos_startpos(mut self) -> GoJob {
		self.pos_spec = Startpos;
		
		self
	}
	
	/// set position moves and return self
	pub fn pos_moves<T>(mut self, moves: T) -> GoJob where
	T: core::fmt::Display {
		self.pos_moves = Some(format!("{}", moves));
		
		self
	}
	
	/// set uci option as key value pair and return self
	pub fn uci_opt<K,V>(mut self, key:K, value:V) -> GoJob where
	K: core::fmt::Display, V: core::fmt::Display {
		self.uci_options.insert(format!("{}",key), format!("{}", value));
		
		self
	}
	
	/// set go option as key value pair and return self
	pub fn go_opt(mut self, key:String, value:String) -> GoJob {
		self.go_options.insert(key, value);
		
		self
	}
	
	/// set time control and return self
	pub fn tc(mut self, tc: Timecontrol) -> GoJob {
		self.go_options.insert("wtime".to_string(), format!("{}", tc.wtime));
		self.go_options.insert("winc".to_string(),  format!("{}", tc.winc));
		self.go_options.insert("btime".to_string(), format!("{}", tc.btime));
		self.go_options.insert("binc".to_string(),  format!("{}", tc.binc));
		
		self
	}
}

/// go command result
#[derive(Debug)]
pub struct GoResult {
	/// best move if any
	bestmove: Option<String>,
	/// ponder if any
	ponder: Option<String>,
}

/// uci engine pool
pub struct UciEnginePool {		
	/// senders of go jobs
	gtxs: Vec<tokio::sync::mpsc::UnboundedSender<GoJob>>,
}

/// uci engine pool implementation
impl UciEnginePool {
	/// create new uci engine pool
	pub fn new() -> UciEnginePool {
		UciEnginePool {						
			gtxs: vec!(),
		}
	}
	
	/// read stdout of engine process
	async fn read_stdout(
		tx: Sender<String>,
		mut reader: tokio::io::Lines<tokio::io::BufReader<tokio::process::ChildStdout>>
	) -> Result<(), Box<dyn std::error::Error>> {
		while let Some(line) = reader.next_line().await? {
			if log_enabled!(Level::Info) {
				info!("uci engine out : {}", line);
			}	
			
			if line.len() >= 8 {
				if &line[0..8] == "bestmove" {
					let send_result = tx.send(line).await;					
					
					if log_enabled!(Level::Debug) {
						debug!("send bestmove result {:?}", send_result);
					}
				}	
			}
		}

		Ok(())
	}
	
	/// create new engine and return its handle
	pub fn create_engine<T>(&mut self, path: T) -> usize 
	where T : core::fmt::Display {
		let path = format!("{}", path);
		
		let mut cmd = Command::new(path.as_str());
		
		cmd.stdout(Stdio::piped());
		cmd.stdin(Stdio::piped());
	
		let mut child = cmd.spawn()
        	.expect("failed to spawn command");
		
		let stdout = child.stdout.take()
        	.expect("child did not have a handle to stdout");
	
		let stdin = child.stdin.take()
			.expect("child did not have a handle to stdin");
		
		let reader = BufReader::new(stdout).lines();
		
		let (tx, rx):(Sender<String>, Receiver<String>) = tokio::sync::mpsc::channel(1);
		
		tokio::spawn(async move {
			let status = child.wait().await
				.expect("child process encountered an error");

			if log_enabled!(Level::Debug) {
				debug!("child exit status : {}", status);
			}			
		});

		tokio::spawn(async {
			match UciEnginePool::read_stdout(tx, reader).await {
				Ok(result) => {
					if log_enabled!(Level::Debug) {
						debug!("reader ok {:?}", result)
					}		
				},
				Err(err) => {
					if log_enabled!(Level::Debug) {
						debug!("reader err {:?}", err)
					}		
				}
			}
		});
		
		let (gtx, grx) = tokio::sync::mpsc::unbounded_channel::<GoJob>();
		
		self.gtxs.push(gtx);
		
		let handle = self.gtxs.len() - 1;
		
		tokio::spawn(async move {				
			let mut stdin = stdin;
			let mut grx = grx;
			let mut rx = rx;
			while let Some(go_job) = grx.recv().await {
				if log_enabled!(Level::Debug) {
					debug!("received go job {:?}", go_job);
				}
				
				for command in go_job.to_commands() {
					let command = format!("{}\n", command);
					
					if log_enabled!(Level::Debug) {
						debug!("issuing engine command {}", command);
					}
					
					let write_result = stdin.write_all(command.as_bytes()).await;
					
					if log_enabled!(Level::Debug) {
						debug!("write result {:?}", write_result);
					}
				}
				
				let recv_result = rx.recv().await.unwrap();
				
				if log_enabled!(Level::Debug) {
					debug!("recv result {:?}", recv_result);
				}
				
				let parts:Vec<&str> = recv_result.split(" ").collect();
				
				let mut go_result = GoResult{
					bestmove: None,
					ponder: None,
				};
				
				if parts.len() > 1 {
					go_result.bestmove = Some(parts[1].to_string());
				}
				
				if parts.len() > 3 {
					go_result.ponder = Some(parts[3].to_string());
				}
				
				let send_result = go_job.rtx.unwrap().send(go_result).await;
				
				if log_enabled!(Level::Debug) {
					debug!("result of send go result {:?}", send_result);
				}
			}
		});
				
		if log_enabled!(Level::Info) {
			info!("spawned uci engine : {}", path);
		}		
		
		handle
	}
	
	/// enqueue go job
	pub fn enqueue_go_job(&mut self, handle: usize, go_job: GoJob) -> Receiver<GoResult> {	
		let mut go_job = go_job;
		
		let (rtx, rrx):(Sender<GoResult>, Receiver<GoResult>) = tokio::sync::mpsc::channel(1);
		
		go_job.rtx = Some(rtx);
		
		let send_result = self.gtxs[handle].send(go_job);		
		
		if log_enabled!(Level::Debug) {
			debug!("send go job result {:?}", send_result);
		}
		
		rrx
	}
}
