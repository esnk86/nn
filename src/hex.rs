const COLUMNS: usize = 8;

pub fn dump(bytes: &Vec<u8>) {
    let rows = get_rows(bytes);
    let lines = get_lines(rows);
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

fn get_rows(bytes: &Vec<u8>) -> Vec<Vec<u8>> {
    let mut rows = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        let j = usize::min(i + COLUMNS, bytes.len());
        rows.push(bytes[i..j].iter().cloned().collect());
        i += COLUMNS;
    }

    rows
}

fn get_lines(rows: Vec<Vec<u8>>) -> Vec<String> {
    rows.iter().map(concat).collect()
}

fn concat(row: &Vec<u8>) -> String {
    let mut line = String::new();

    for (i, byte) in row.iter().enumerate() {
        if i > 0 {
            line.push(' ');
        }
        line.push_str(&format!("{byte:02x}"));
    }

    line
}

fn get_rle(lines: Vec<String>) -> Vec<(usize, String)> {
    let mut rle = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        rle.push(get_run(&lines, &mut i));
    }

    rle
}

fn get_run(lines: &Vec<String>, p: &mut usize) -> (usize, String) {
    let mut run = (0, String::new());

    for i in *p .. lines.len() {
        if i == *p {
            run = (1, lines[*p].clone());
        } else if lines[*p].eq(&lines[i]) {
            run.0 += 1;
        } else {
            *p = i;
            return run;
        }
    }

    *p = lines.len();
    run
}
