use std::io::{self, Write};
use std::process::Command;

fn main() {
    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, 
            Ok(_) => {
                let mut parts = input.trim().split_whitespace();
                let command = parts.next().unwrap_or("");
                if command == "quit" || command == "exit" {
                    break;
                }

                let output = Command::new(command)
                    .args(parts)
                    .output()
                    .expect("Failed to execute command");

                io::stdout().write_all(&output.stdout).unwrap();
                io::stderr().write_all(&output.stderr).unwrap();
            },

            Err(e) => println!("Error Reading input: {}", e),
        }
    }
}
