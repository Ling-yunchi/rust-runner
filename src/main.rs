use std::env;

use rust_runner::{
    config::Config,
    runner::{self, JudgeResult},
};

use nix::unistd::{fork, ForkResult};

fn main() {
    let pid = unsafe { fork() };

    let args: Vec<String> = env::args().collect();
    let config = Config::parse(&args).unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    let mut res: Option<JudgeResult> = None;

    match pid {
        Ok(ForkResult::Parent { child, .. }) => {
            res = Some(
                runner::father_program(child, &config).unwrap_or_else(|err| {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }),
            )
        }
        Ok(ForkResult::Child) => runner::child_program(&config).unwrap_or_else(|err| {
            eprintln!("{}", err);
            std::process::exit(1);
        }),
        Err(_) => {
            eprintln!("Fork failed");
            std::process::exit(1);
        }
    }

    match res {
        Some(res) => println!("{}", res),
        None => std::process::exit(1),
    }
}
