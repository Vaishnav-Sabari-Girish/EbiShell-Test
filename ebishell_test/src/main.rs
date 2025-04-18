use std::fs::File;
use std::io::{self, Write};
use std::os::unix::io::FromRawFd;
use std::process::{Child, Command, Stdio};
use inline_colorization::*;

fn main() {
    loop {
        print!("{color_green}>>> {color_reset}");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                if input == "quit" || input == "exit" {
                    break;
                }

                // Split input on '|' to detect piped commands
                let commands: Vec<&str> = input.split('|').map(|s| s.trim()).collect();

                if commands.len() > 2 {
                    println!("Error: Multiple pipes are not supported (e.g., 'cmd1 | cmd2 | cmd3')");
                    continue;
                }

                // Parse command into parts (command and arguments)
                let parts: Vec<String> = commands[0]
                    .split_whitespace()
                    .map(String::from)
                    .collect();

                if parts.is_empty() {
                    println!("Error: Invalid command format");
                    continue;
                }

                let cmd = parts[0].clone();
                let args = &parts[1..];

                if commands.len() == 1 {
                    // Non-piped command (e.g., "ls" or "ls -la")
                    match spawn_process(cmd, args, None, None) {
                        Ok(mut child) => {
                            if let Err(e) = child.wait() {
                                println!("Error waiting for process: {}", e);
                            }
                        }
                        Err(e) => println!("Error spawning process: {}", e),
                    }
                } else {
                    // Piped command (e.g., "ls -la | grep lock")
                    let right_parts: Vec<String> = commands[1]
                        .split_whitespace()
                        .map(String::from)
                        .collect();

                    if right_parts.is_empty() {
                        println!("Error: Invalid right command format");
                        continue;
                    }

                    let right_cmd = right_parts[0].clone();
                    let right_args = &right_parts[1..];

                    // Create pipe and spawn processes
                    let (left, right) = create_pipe();
                    match spawn_process(cmd, args, None, Some(right)) {
                        Ok(left_child) => {
                            match spawn_process(right_cmd, right_args, Some(left), None) {
                                Ok(right_child) => {
                                    // Wait for both processes to complete
                                    if let Err(e) = wait_for_processes(left_child, right_child) {
                                        println!("Error waiting for processes: {}", e);
                                    }
                                }
                                Err(e) => println!("Error spawning right process: {}", e),
                            }
                        }
                        Err(e) => println!("Error spawning left process: {}", e),
                    }
                }
            }
            Err(e) => println!("Error reading input: {}", e),
        }
    }
}

fn create_pipe() -> (File, File) {
    let mut pipe = [0, 0];
    unsafe {
        if libc::pipe(&mut pipe as *mut _) == -1 {
            panic!("Failed to create pipe");
        }
        (
            File::from_raw_fd(pipe[0]), // Safe: read end of pipe
            File::from_raw_fd(pipe[1]), // Safe: write end of pipe
        )
    }
}

fn spawn_process(
    cmd: String,
    args: &[String],
    stdin: Option<File>,
    stdout: Option<File>,
) -> io::Result<Child> {
    if cmd.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Empty command",
        ));
    }

    let mut command = Command::new(&cmd);
    command
        .args(args)
        .stdin(stdin.map(Stdio::from).unwrap_or(Stdio::inherit()))
        .stdout(stdout.map(Stdio::from).unwrap_or(Stdio::inherit()));

    let child = command.spawn()?;
    Ok(child)
}

fn wait_for_processes(mut left_child: Child, mut right_child: Child) -> io::Result<()> {
    left_child.wait()?;
    right_child.wait()?;
    Ok(())
}
