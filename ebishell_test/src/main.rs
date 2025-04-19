use std::fs::File;
use std::io::{self};
use std::os::unix::io::FromRawFd;
use std::process::{Child, Command, Stdio};
use inline_colorization::*;
use rustyline::error::ReadlineError;
use rustyline::{history::DefaultHistory, Editor};

fn main() {
    println!("{color_blue}Welcome to EbiShell{color_reset}");
    let mut rl = Editor::<(), DefaultHistory>::new().expect("Failed to create readline editor");
    let mut history: Vec<String> = Vec::new();

    loop {
        // Define the prompt with colors and emojis
        let prompt = format!(
            "🐚{color_bright_red}EbiShell{color_reset} 🍤 {color_green}>>> {color_reset}"
        );

        match rl.readline(&prompt) {
            Ok(input) => {
                let input = input.trim().to_string();
                if input.is_empty() {
                    continue;
                }
                if input == "quit" || input == "exit" {
                    break;
                }

                // Handle !! for previous command
                let final_input = if input == "!!" {
                    if let Some(last) = history.last() {
                        println!("{}", last);
                        last.clone()
                    } else {
                        println!("No previous command in history");
                        continue;
                    }
                } else {
                    input.clone()
                };

                // Add to history
                rl.add_history_entry(&final_input).unwrap();
                history.push(final_input.clone());

                // Split input on '|' to detect piped commands
                let commands: Vec<&str> = final_input.split('|').map(|s| s.trim()).collect();

                if commands.is_empty() {
                    println!("Error: Invalid command format");
                    continue;
                }

                // Parse each command into parts
                let mut command_parts: Vec<Vec<String>> = Vec::new();
                for cmd in commands.iter() {
                    let parts: Vec<String> = cmd.split_whitespace().map(String::from).collect();
                    if parts.is_empty() {
                        println!("Error: Invalid command format in pipeline");
                        continue;
                    }
                    command_parts.push(parts);
                }

                // Execute commands
                if commands.len() == 1 {
                    // Non-piped command
                    let cmd = command_parts[0][0].clone();
                    let args = &command_parts[0][1..];
                    match spawn_process(cmd, args, None, None) {
                        Ok(mut child) => {
                            if let Err(e) = child.wait() {
                                println!("Error waiting for process: {}", e);
                            }
                        }
                        Err(e) => println!("Error spawning process: {}", e),
                    }
                } else {
                    // Piped commands
                    let mut children: Vec<Child> = Vec::new();
                    let mut pipes = Vec::new();

                    // Create pipes
                    for _ in 0..(commands.len() - 1) {
                        pipes.push(create_pipe());
                    }

                    // Spawn processes
                    for i in 0..commands.len() {
                        let cmd = command_parts[i][0].clone();
                        let args = &command_parts[i][1..];

                        let stdin = if i == 0 {
                            None // First process uses inherited stdin
                        } else {
                            match pipes[i - 1].0.try_clone() {
                                Ok(file) => Some(file),
                                Err(e) => {
                                    println!("Error cloning pipe read end: {}", e);
                                    cleanup_processes(&mut children);
                                    break;
                                }
                            }
                        };

                        let stdout = if i == commands.len() - 1 {
                            None // Last process uses inherited stdout
                        } else {
                            match pipes[i].1.try_clone() {
                                Ok(file) => Some(file),
                                Err(e) => {
                                    println!("Error cloning pipe write end: {}", e);
                                    cleanup_processes(&mut children);
                                    break;
                                }
                            }
                        };

                        match spawn_process(cmd, args, stdin, stdout) {
                            Ok(child) => children.push(child),
                            Err(e) => {
                                println!("Error spawning process {}: {}", i + 1, e);
                                cleanup_processes(&mut children);
                                break;
                            }
                        }
                    }

                    // Wait for all processes only if all spawned successfully
                    if children.len() == commands.len() {
                        for mut child in children {
                            if let Err(e) = child.wait() {
                                println!("Error waiting for process: {}", e);
                            }
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(e) => {
                println!("Error reading input: {}", e);
                break;
            }
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

fn cleanup_processes(children: &mut Vec<Child>) {
    for mut child in children.drain(..) {
        let _ = child.kill();
        let _ = child.wait();
    }
}
