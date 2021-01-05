#[derive(Debug)]
pub struct UciEngine {
	path: String,
}

impl UciEngine {
	fn new(set_path: String) -> UciEngine {
		UciEngine {
			path: set_path
		}
	}
}
