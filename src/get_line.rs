pub fn get_line() -> Option<String> {
	let mut line = String::new();

	match std::io::stdin().read_line(&mut line) {
		Ok(0) => None,
		Ok(_) => Some(line.trim_end().to_string()),
		Err(err) => {
			println!("{}", err);
			None
		},
	}
}
