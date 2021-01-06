use log::{debug, log_enabled, info, Level};

use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt, AsyncWriteExt};
use std::process::Stdio;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::collections::HashMap;

use ::std::collections::VecDeque;
use ::std::sync::{Condvar, Mutex};

/// enum of possible position sepcifiers
#[derive(Debug)]
pub enum PosSpec{
	Startpos,
	Fen,
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
}

/// go job queue
/// https://www.poor.dev/posts/what-job-queue/
pub struct GoJobQueue {
	/// jobs
	jobs: Mutex<Option<VecDeque<GoJob>>>,
	/// cond var
	cvar: Condvar,
}

/// go job queue implementation
impl GoJobQueue {
	pub fn new() -> GoJobQueue {
		GoJobQueue {
			jobs: Mutex::new(Some(VecDeque::new())),
			cvar: Condvar::new(),
		}
	}
	
	/// enqueue go job
	pub fn enqueue_go_job(&self, go_job: GoJob) {
		let mut jobs = self.jobs.lock().unwrap();
		
		if let Some(queue) = jobs.as_mut() {
			queue.extend(vec!(go_job));
			self.cvar.notify_all();
		}
	}
	
	/// wait for go job
	pub fn wait_for_go_job(&self) -> Option<GoJob> {
		let mut jobs = self.jobs.lock().unwrap();
		
		loop {
			match jobs.as_mut()?.pop_front() {
				Some(job) => return Some(job),
				None => {
					jobs = self.cvar.wait(jobs).unwrap()
				}
			}
		}
	}
	
	/// end queue
	pub fn end(&self) {
		let mut jobs = self.jobs.lock().unwrap();
		*jobs = None;
		self.cvar.notify_all();
	}
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
	/// one minute thinking time for both sides, no increment
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
		}
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
	stdins: Vec<tokio::process::ChildStdin>,
	rxs: Vec<Receiver<String>>,	
}

/// uci engine pool implementation
impl UciEnginePool {
	pub fn new() -> UciEnginePool {
		UciEnginePool {
			stdins: vec!(),
			rxs: vec!(),
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
					let _ = tx.send(line);					
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
		
		self.stdins.push(stdin);
		
		let handle = self.stdins.len() - 1;
		
		let reader = BufReader::new(stdout).lines();
		
		let (tx, rx):(Sender<String>, Receiver<String>) = mpsc::channel();
		
		self.rxs.push(rx);

		tokio::spawn(async {
			let status = child.await
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
				
		if log_enabled!(Level::Info) {
			info!("spawned uci engine : {}", path);
		}		
		
		handle
	}
	
	/// issue engine command
	pub async fn issue_command<T>(&mut self, handle: usize, command: T) -> Result<(), Box<dyn std::error::Error>> 
	where T: core::fmt::Display {
		let command = format!("{}", command);
		
		if log_enabled!(Level::Info) {
			info!("issuing uci command : {}", command);
		}
		
		let result = self.stdins[handle].write_all(format!("{}\n", command).as_bytes()).await?;
		
		if log_enabled!(Level::Debug) {
			debug!("issue uci command result : {:?}", result);
		}

		Ok(())
	}
	
	/// start thinking based on go job and return best move and ponder if any, blocking
	pub async fn go(&mut self, handle: usize, go_job: GoJob) -> Result<GoResult, Box<dyn std::error::Error>> {
		for (key, value) in go_job.uci_options {
			let result = self.issue_command(handle, format!("setoption name {} value {}", key, value).to_string()).await;
			
			if log_enabled!(Level::Debug) {
				debug!("issue uci option command result : {:?}", result);
			}
		}
		
		let mut pos_command_moves = "".to_string();
		
		if let Some(pos_moves) = go_job.pos_moves {
			pos_command_moves = format!(" moves {}", pos_moves)
		}
		
		let pos_command:Option<String> = match go_job.pos_spec {
			Startpos => Some(format!("position startpos{}", pos_command_moves)),
			Fen => Some(format!("position fen {}{}", go_job.pos_fen.unwrap(), pos_command_moves)),
			_ => None
		};
		
		if let Some(pos_command) = pos_command {
			let result = self.issue_command(handle, pos_command).await;
		
			if log_enabled!(Level::Debug) {
				debug!("issue position command result : {:?}", result);
			}
		}
		
		let mut go_command = "go".to_string();
		
		for (key, value) in go_job.go_options {
			go_command = go_command + &format!(" {} {}", key, value);
		}
		
		let result = self.issue_command(handle, go_command).await;
		
		if log_enabled!(Level::Debug) {
			debug!("issue go command result : {:?}", result);
		}
		
		let result = self.rxs[handle].recv();
		
		if log_enabled!(Level::Debug) {
			debug!("recv bestmove result : {:?}", result);
		}
		
		let mut bestmove:Option<String> = None;
		let mut ponder:Option<String> = None;
		
		if let Ok(result) = result {
			let parts:Vec<&str> = result.split(" ").collect();
		
			if parts.len() > 1 {
				bestmove = Some(parts[1].to_string());
			}

			if parts.len() > 3 {
				ponder = Some(parts[3].to_string());
			}
		}
		
		Ok(GoResult {
			bestmove: bestmove,
			ponder: ponder,
		})
	}
}
