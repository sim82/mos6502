pub mod hexdump {
    use std::io::BufRead;

    pub fn read() -> Vec<u8> {
        let mut out = Vec::new();
        for line in std::io::stdin().lock().lines() {
            let line = line.unwrap();

            if line.chars().nth(4).unwrap() != ':' {
                panic!("missing address in hexdump: {}", line);
            }
            let mut address =
                usize::from_str_radix(&line.chars().take(4).collect::<String>(), 16).unwrap();

            for b in line.chars().skip(6).collect::<String>().split(' ') {
                // println!("b: {}", b);
                if b.is_empty() {
                    continue;
                }

                if out.len() <= address {
                    out.resize(address + 1, 0u8);
                }
                out[address] = u8::from_str_radix(b, 16).unwrap();
                address += 1;
            }
        }
        out
    }

    pub fn dump(data: &[u8]) {
        let chunk_size = 16;
        for (i, chunk) in data.chunks(chunk_size).enumerate() {
            if chunk.iter().any(|c| *c != 0) {
                println!(
                    "{:04x}: {}",
                    i * chunk_size,
                    chunk
                        .iter()
                        .map(|b| format!("{:02x}", *b))
                        .collect::<Vec<_>>()
                        .join(" ")
                );
            }
        }
    }
}
