use std::io::{self, Write};

fn main() {
    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, 
            Ok(_) => {
                if input.to_lowercase().trim() == "quit" || input.to_lowercase().trim() == "exit" {
                    break;
                }
            },
            Err(e) => println!("Error Reading input: {}", e),
        }
    }
}
