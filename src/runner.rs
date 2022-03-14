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
const FILE_LIMIT: usize = 2;

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
            RunResult::RunSuccess => write!(f, "Run Success"),
            RunResult::TimeLimitExceeded => write!(f, "Time Limit Exceeded"),
            RunResult::MemoryLimitExceeded => write!(f, "Memory Limit Exceeded"),
            RunResult::RunTimeError => write!(f, "Run Time Error"),
            RunResult::OutputLimitExceeded => write!(f, "Output Limit Exceeded"),
        }
    }
}

pub fn father_program(child_pid: Pid, config: &Config) -> JudgeResult {
    return listen_child_process(child_pid, config);
}

pub fn child_program(config: &Config) {
    set_child_limit(config);
    let _ = run_child_program(config);
}

fn run_child_program(config: &Config) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(&config.input_path)?;
    let output_file = File::create(&config.output_path)?;
    let _ = Command::new(&config.program_path)
        .stdin(Stdio::from(input_file))
        .stdout(Stdio::from(output_file))
        .output()
        .expect("failed to execute process");
    return Ok(());
}

fn listen_child_process(child_pid: Pid, config: &Config) -> JudgeResult {
    let memary_usage;
    let time_usage;
    let mut run_result = RunResult::RunSuccess;

    let res = pidrusage::<RUsageInfoV0>(child_pid.as_raw()).unwrap();
    memary_usage = res.memory_used();
    time_usage = res.ri_user_time;

    match waitpid(child_pid, Some(WaitPidFlag::WUNTRACED)).unwrap() {
        nix::sys::wait::WaitStatus::Exited(_, _) => {
            run_result = RunResult::RunSuccess;
        }
        nix::sys::wait::WaitStatus::Signaled(_, wtermsig, _) => match wtermsig {
            SIGXCPU | SIGKILL => run_result = RunResult::TimeLimitExceeded,
            SIGSEGV => run_result = RunResult::MemoryLimitExceeded,
            SIGXFSZ => run_result = RunResult::OutputLimitExceeded,
            _ => run_result = RunResult::RunTimeError,
        },
        _ => {}
    }

    if memary_usage > config.memory_limit {
        run_result = RunResult::MemoryLimitExceeded;
    }

    return JudgeResult {
        result: run_result,
        time: time_usage,
        memory: memary_usage,
    };
}

fn set_child_limit(config: &Config) {
    let time_limit = Some(config.time_limit as u64);
    setrlimit(Resource::RLIMIT_CPU, time_limit, time_limit).unwrap();

    let memory_limit = Some(config.memory_limit as u64);
    setrlimit(Resource::RLIMIT_AS, memory_limit, memory_limit).unwrap();
    setrlimit(Resource::RLIMIT_STACK, memory_limit, memory_limit).unwrap();

    let output_limit = Some(OUTPUT_LIMIT as u64);
    setrlimit(Resource::RLIMIT_FSIZE, output_limit, output_limit).unwrap();

    let file_limit = Some(FILE_LIMIT as u64);
    setrlimit(Resource::RLIMIT_NOFILE, file_limit, file_limit).unwrap();

    setrlimit(Resource::RLIMIT_NPROC, None, None).unwrap();
}
