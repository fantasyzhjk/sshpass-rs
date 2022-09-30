use anyhow::Result;
use clap::Parser;
use console::Term;
use expectrl::{check, spawn, stream::stdin::Stdin, Error};
use std::{env, io::stdout};

/// A sshpass implementation in Rust 
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Provide password as argument (security unwise)
    #[arg(short, long)]
    password: Option<String>,
    /// Password is passed as env-var
    #[arg(short, long, default_value_t = String::from("SSHPASS"))]
    env: String,
    /// SSH command that runs
    #[arg(last = true)]
    command: Vec<String>,
    
    // TODO: Take password to use from file
    // #[arg(short, long)]
    // file: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let command = args.command.join(" ");
    let password = if let Some(password) = args.password {
        Some(password)
    } else if let Ok(password) = env::var(args.env) {
        Some(password)
    } else {
        None
    };

    let mut ssh = spawn(&command).expect(&format!("Unknown command: {:?}", command));

    loop {
        match check!(
            &mut ssh,
            _ = "(yes/no/[fingerprint])" => {
                ssh.send_line("yes")?;
            },
            _ = "password:" => {
                if let Some(password) = password {
                    ssh.send_line(password)?;
                } else {
                    print!("password:")
                }
                break;
            },
        ) {
            Err(Error::Eof) => break,
            result => result.expect("Check output failed"),
        };
    }

    let _term = Term::stdout();
    let mut stdin = Stdin::open().expect("Failed to create stdin");
    ssh.interact(&mut stdin, stdout())
        .on_idle(|_state| {
            #[cfg(not(target_os = "windows"))]
            {
                let (rows, cols) = _term.size();
                _state
                    .session
                    .set_window_size(cols, rows)
                    .expect("Update window size failed");
            }
            Ok(())
        })
        .spawn()
        .expect("Failed to start interact");

    stdin.close().expect("Failed to close a stdin");
    Ok(())
}
