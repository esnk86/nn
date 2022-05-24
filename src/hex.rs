const COLUMNS: usize = 8;

pub fn dump(bytes: &Vec<u8>) {
	let chunks = get_chunks(bytes);
	let strings = get_strings(chunks);
	let rle = get_rle(strings);

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

fn get_strings(chunks: Vec<Vec<u8>>) -> Vec<String> {
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

fn get_rle(strings: Vec<String>) -> Vec<(usize, String)> {
	let mut result = Vec::new();
	let mut i = 0;

	while i < strings.len() {
		result.push(get_run(&strings, &mut i));
	}

	result
}

fn get_run(strings: &Vec<String>, p: &mut usize) -> (usize, String) {
	let mut result = (0, String::new());

	for i in *p .. strings.len() {
		if i == *p {
			result = (1, strings[*p].clone());
		} else if strings[*p].eq(&strings[i]) {
			result.0 += 1;
		} else {
			*p = i;
			return result;
		}
	}

	*p = strings.len();
	result
}
