macro_rules! gen_str_buff {
	($(#[$attr:meta] => $type:ident, $size:expr),*) => { $(
	    #[$attr]
	    #[derive(Debug, Clone, Copy)]
		pub struct $type {
			pub len: usize,
			pub buff: [u8; $size],
		}

		impl $type {
			fn new() -> Self {
				Self {
					len: 0,
					buff: [0; $size]
				}
			}
		}

		impl std::convert::From<&str> for $type {
			fn from(value: &str) -> Self {
				let bytes = value.as_bytes();
				let mut len = bytes.len();
				if len > $size{
					len = $size;
				}

				let mut buff = $type::new();

                buff.len = len;
				buff.buff[0..len].copy_from_slice(&bytes[0..len]);

				buff
			}
		}

		impl std::convert::From<$type> for String {
			fn from(buff: $type) -> String {
				std::str::from_utf8(&buff.buff[0..buff.len]).unwrap().to_string()
			}
		}

		impl std::fmt::Display for $type {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		        write!(f, "{}", String::from(*self))
		    }
		}
	)* }
}

gen_str_buff!(
/// UciBuff
=> UciBuff, 10
);
