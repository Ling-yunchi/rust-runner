use std::{
    error::Error,
    fmt,
    fs::File,
    process::{Command, Stdio},
};

use libproc::libproc::pid_rusage::{pidrusage, PIDRUsage, RUsageInfoV0};
use nix::{
    sys::{
        resource::{setrlimit, Resource},
        signal::Signal::{SIGKILL, SIGSEGV, SIGXCPU, SIGXFSZ},
        wait::{waitpid, WaitPidFlag},
    },
    unistd::Pid,
};

use crate::config::Config;

const OUTPUT_LIMIT: usize = 1024 * 100;
const FILE_LIMIT: usize = 5;

pub struct JudgeResult {
    result: RunResult,
    time: u64,
    memory: u64,
}

impl fmt::Display for JudgeResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.result, self.time, self.memory)
    }
}

pub enum RunResult {
    RunSuccess,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    RunTimeError,
    OutputLimitExceeded,
}

impl fmt::Display for RunResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunResult::RunSuccess => write!(f, "RunSuccess"),
            RunResult::TimeLimitExceeded => write!(f, "TimeLimitExceeded"),
            RunResult::MemoryLimitExceeded => write!(f, "MemoryLimitExceeded"),
            RunResult::RunTimeError => write!(f, "RunTimeError"),
            RunResult::OutputLimitExceeded => write!(f, "OutputLimitExceeded"),
        }
    }
}

pub fn father_program(
    child_pid: Pid,
    config: &Config,
) -> core::result::Result<JudgeResult, Box<dyn Error>> {
    return listen_child_process(child_pid, config);
}

pub fn child_program(config: &Config) -> core::result::Result<(), Box<dyn Error>> {
    let _ = set_child_limit(config)?;
    let _ = run_child_program(config)?;
    Ok(())
}

fn run_child_program(config: &Config) -> core::result::Result<(), Box<dyn Error>> {
    let input_file = File::open(&config.input_path)?;
    let output_file = File::create(&config.output_path)?;
    let status = Command::new(&config.program_path)
        .stdin(Stdio::from(input_file))
        .stdout(Stdio::from(output_file))
        .status()?;
    if !status.success() {
        return Err("Program exit with non-zero status".into());
    }
    return Ok(());
}

fn listen_child_process(
    child_pid: Pid,
    config: &Config,
) -> core::result::Result<JudgeResult, Box<dyn Error>> {
    let memary_usage;
    let time_usage;
    let mut run_result;

    let res = pidrusage::<RUsageInfoV0>(child_pid.as_raw())?;
    memary_usage = res.memory_used();
    time_usage = res.ri_user_time;

    match waitpid(child_pid, Some(WaitPidFlag::WUNTRACED))? {
        nix::sys::wait::WaitStatus::Exited(_, _) => {
            run_result = RunResult::RunSuccess;
        }
        nix::sys::wait::WaitStatus::Signaled(_, wtermsig, _) => match wtermsig {
            SIGXCPU | SIGKILL => run_result = RunResult::TimeLimitExceeded,
            SIGSEGV => run_result = RunResult::MemoryLimitExceeded,
            SIGXFSZ => run_result = RunResult::OutputLimitExceeded,
            _ => run_result = RunResult::RunTimeError,
        },
        _ => {
            return Err("Child process is not exited".into());
        }
    }

    if memary_usage > config.memory_limit {
        run_result = RunResult::MemoryLimitExceeded;
    }

    Ok(JudgeResult {
        result: run_result,
        time: time_usage,
        memory: memary_usage,
    })
}

fn set_child_limit(config: &Config) -> core::result::Result<(), Box<dyn Error>> {
    let time_limit = Some(config.time_limit as u64);
    setrlimit(Resource::RLIMIT_CPU, time_limit, time_limit)?;

    let memory_limit = Some(config.memory_limit as u64);
    setrlimit(Resource::RLIMIT_AS, memory_limit, memory_limit)?;
    setrlimit(Resource::RLIMIT_STACK, memory_limit, memory_limit)?;

    let output_limit = Some(OUTPUT_LIMIT as u64);
    setrlimit(Resource::RLIMIT_FSIZE, output_limit, output_limit)?;

    let file_limit = Some(FILE_LIMIT as u64);
    setrlimit(Resource::RLIMIT_NOFILE, file_limit, file_limit)?;

    setrlimit(Resource::RLIMIT_CORE, None, None)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out() {
        let res = JudgeResult {
            result: RunResult::RunSuccess,
            time: 101,
            memory: 100,
        };
        println!("{}", res);
    }
}
