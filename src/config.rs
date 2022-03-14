use std::error::Error;

pub struct Config {
    pub program_path: String,
    pub input_path: String,
    pub output_path: String,
    pub time_limit: u64,
    pub memory_limit: u64,
}

impl Config {
    pub fn parse(args: &[String]) -> Result<Config, Box<dyn Error>> {
        if args.len() != 6 {
            return Err("Usage: rust-runner <program_path> <input_path> <output_path> <time_limit> <memory_limit>".into());
        }

        let program_path = args[1].clone();
        let input_path = args[2].clone();
        let output_path = args[3].clone();
        let time_limit = args[4].parse::<u64>()?;
        let memory_limit = args[5].parse::<u64>()?;

        Ok(Config {
            program_path,
            input_path,
            output_path,
            time_limit,
            memory_limit,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        let args = vec![
            "rust_runner".to_string(),
            "./a.out".to_string(),
            "in.txt".to_string(),
            "out.txt".to_string(),
            "1000".to_string(),
            "1000".to_string(),
        ];
        let config = Config::parse(&args).unwrap();
        assert_eq!(config.program_path, "./a.out");
        assert_eq!(config.input_path, "in.txt");
        assert_eq!(config.output_path, "out.txt");
        assert_eq!(config.time_limit, 1000);
        assert_eq!(config.memory_limit, 1000);
    }

    #[test]
    fn test_parse_error() {
        let args = vec![
            "rust_runner".to_string(),
            "./a.out".to_string(),
            "in.txt".to_string(),
            "out.txt".to_string(),
            "1000".to_string(),
            "-1000".to_string(),
        ];
        let config = Config::parse(&args).unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            Config {
                program_path: "".to_string(),
                input_path: "".to_string(),
                output_path: "".to_string(),
                time_limit: 0,
                memory_limit: 0,
            }
        });
        assert_eq!(config.program_path, "");
        assert_eq!(config.input_path, "");
        assert_eq!(config.output_path, "");
        assert_eq!(config.time_limit, 0);
        assert_eq!(config.memory_limit, 0);
    }
}
