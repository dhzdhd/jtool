use std::io::{Read, Write};

use anyhow::anyhow;
use clap::{Parser, Subcommand, command};
use clio::{Input, Output};

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
    },
    /// Stringify JSON input and return the JSON string
    Stringify {
        #[clap(value_parser, default_value = "-")]
        input: Input,
        #[clap(value_parser, default_value = "-")]
        output: Output,
        #[clap(long, short, default_value = "true")]
        prettify: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Parse { input, output } => {
            let mut buf = String::new();
            let mut input_handle = input;
            input_handle.read_to_string(&mut buf)?;
            let mut output_handle = output;

            let json = core::parse(buf)?;
            let json_str = json.to_string();
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
            prettify,
        } => Err(anyhow!("stringify not supported yet")),
    }
}
