use std::io::{Read, Write};

use anyhow::anyhow;
use clap::{Parser, Subcommand, command};
use clio::{Input, Output};
use serde_json::json;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// JTool - a powerful JSON tool to parse and stringify nested JSON objects
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Parse JSON input and return the JSON object
    Parse {
        #[clap(value_parser, default_value = "-")]
        input: Input,
        #[clap(value_parser, default_value = "-")]
        output: Output,
        #[clap(long, short)]
        prettify: bool,
    },
    /// Stringify JSON input and return the JSON string
    Stringify {
        #[clap(value_parser, default_value = "-")]
        input: Input,
        #[clap(value_parser, default_value = "-")]
        output: Output,
        #[clap(long, short)]
        paths: Option<Vec<String>>,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Parse {
            input,
            output,
            prettify,
        } => {
            let mut buf = String::new();
            let mut input_handle = input;
            input_handle.read_to_string(&mut buf)?;
            let mut output_handle = output;

            println!("{buf}");
            let json = core::parse::parse(buf)?;

            let json_str = if prettify {
                serde_json::to_string_pretty(&json)?
            } else {
                json.to_string()
            };

            write!(output_handle, "{}", json_str)?;

            // Add a newline if output to stdout
            if output_handle.path().is_std() {
                println!("")
            }

            Ok(())
        }
        Commands::Stringify {
            input,
            output,
            paths,
        } => {
            let mut buf = String::new();
            let mut input_handle = input;
            input_handle.read_to_string(&mut buf)?;
            let mut output_handle = output;

            let str_paths: Option<Vec<&str>> = paths
                .as_ref()
                .map(|vec| vec.iter().map(|s| s.as_ref()).collect());
            let val = json!(buf);
            let str = core::stringify::stringify(val, str_paths)?;

            write!(output_handle, "{}", str)?;

            // Add a newline if output to stdout
            if output_handle.path().is_std() {
                println!("")
            }

            Ok(())
        }
    }
}
