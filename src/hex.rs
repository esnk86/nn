const COLUMNS: usize = 8;

pub fn dump(bytes: &Vec<u8>) {
	let chunks = get_chunks(bytes);
	let lines = get_lines(chunks);
	let rle = get_rle(lines);

	let mut addr = 0;

	for (len, line) in rle.iter() {
		if *len > 1 {
			println!("{addr:04x}: {line}");
			if *len > 2 {
				println!("...");
			}
		}
		addr += COLUMNS * (*len - 1);
		println!("{addr:04x}: {line}");
		addr += COLUMNS;
	}
}

fn get_chunks(bytes: &Vec<u8>) -> Vec<Vec<u8>> {
	let mut result = Vec::new();
	let mut i = 0;

	while i < bytes.len() {
		let j = usize::min(i + COLUMNS, bytes.len());
		result.push(bytes[i..j].iter().cloned().collect());
		i += COLUMNS;
	}

	result
}

fn get_lines(chunks: Vec<Vec<u8>>) -> Vec<String> {
	chunks.iter().map(|chunk| concat(chunk)).collect()
}

fn concat(chunk: &Vec<u8>) -> String {
	let mut result = String::new();

	for (i, byte) in chunk.iter().enumerate() {
		if i > 0 {
			result.push(' ');
		}
		result.push_str(&format!("{byte:02x}"));
	}

	result
}

fn get_rle(lines: Vec<String>) -> Vec<(usize, String)> {
	let mut result = Vec::new();
	let mut i = 0;

	while i < lines.len() {
		result.push(get_run(&lines, &mut i));
	}

	result
}

fn get_run(lines: &Vec<String>, p: &mut usize) -> (usize, String) {
	let mut result = (0, String::new());

	for i in *p .. lines.len() {
		if i == *p {
			result = (1, lines[*p].clone());
		} else if lines[*p].eq(&lines[i]) {
			result.0 += 1;
		} else {
			*p = i;
			return result;
		}
	}

	*p = lines.len();
	result
}
