use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt, AsyncWriteExt};
use std::process::Stdio;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

#[derive(Debug)]
pub struct UciEngine {
	path: String,
	stdin: tokio::process::ChildStdin,
	rx: Receiver<String>,
}

impl UciEngine {
	pub fn new(path: String) -> UciEngine {		
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
		
		let (tx, rx):(Sender<String>, Receiver<String>) = mpsc::channel();

		tokio::spawn(async {
			let status = child.await
				.expect("child process encountered an error");

			println!("child status was: {}", status);
		});

		tokio::spawn(async {
			match UciEngine::read_stdout(tx, reader).await {
				Ok(result) => println!("reader ok {:?}", result),
				Err(err) => println!("reader err {:?}", err)
			}
		});

		println!("spawned uci engine {}", path);
		
		UciEngine {
			path: path,
			stdin: stdin,
			rx: rx,
		}
	}
	
	pub async fn read_stdout(
		tx: Sender<String>,
		mut reader: tokio::io::Lines<tokio::io::BufReader<tokio::process::ChildStdout>>
	) -> Result<(), Box<dyn std::error::Error>> {
		while let Some(line) = reader.next_line().await? {
			println!("Line: {}", line);
			if line.len() >= 8 {
				if &line[0..8] == "bestmove" {
					let send_result = tx.send(line);
					println!("send result {:?}", send_result);
				}	
			}
		}

		Ok(())
	}
		
	pub async fn issue_command(&mut self, command: String) -> Result<(), Box<dyn std::error::Error>> {
		println!("issuing command {}", command);
		
		let result = self.stdin.write_all(command.as_bytes()).await?;
		
		println!("issue command result {:?}", result);

		Ok(())
	}
	
	pub async fn go(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		let _ = self.issue_command("go depth 5\n".to_string()).await?;
		
		let result = self.rx.recv();
		
		println!("go command result {:?}", result);
		
		Ok(())
	}
}
