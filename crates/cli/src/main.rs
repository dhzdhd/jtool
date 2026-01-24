use core::{
    compare::{compare, print_diff},
    parse::parse,
    stringify::stringify,
};
use std::io::{Read, Write};

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
    #[command(alias = "p")]
    Parse {
        /// Input file, defaults to STDIN
        #[clap(value_parser, default_value = "-")]
        input: Input,
        /// Output file, defaults to STDOUT
        #[clap(value_parser, default_value = "-")]
        output: Output,
        /// Prettify and format output JSON
        #[clap(long, short)]
        prettify: bool,
    },
    /// Stringify JSON input and return the JSON string
    #[command(alias = "s")]
    Stringify {
        /// Input file, defaults to STDIN
        #[clap(value_parser, default_value = "-")]
        input: Input,
        /// Output file, defaults to STDOUT
        #[clap(value_parser, default_value = "-")]
        output: Output,
        /// List of key hierarchy sequences separated by (.) for nested stringification
        #[clap(long, short)]
        paths: Option<Vec<String>>,
    },
    /// Trim extra spaces and newlines from JSON
    #[command(aliases = ["r", "rem"])]
    RemoveSpaces {},
    /// Compare two JSON's and generate a diff
    #[command(aliases = ["c", "diff", "d"])]
    Compare {
        /// Old file
        #[clap(value_parser)]
        old: Input,
        /// New file
        #[clap(value_parser)]
        new: Input,
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

            let json = parse(buf)?;

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
            let val = parse(buf)?;
            let str = stringify(val, str_paths)?;

            write!(output_handle, "{}", str)?;

            // Add a newline if output to stdout
            if output_handle.path().is_std() {
                println!("")
            }

            Ok(())
        }
        Commands::RemoveSpaces {} => Ok(()),
        Commands::Compare { old, new } => {
            let mut old_buf = String::new();
            let mut new_buf = String::new();
            let mut old_handle = old;
            let mut new_handle = new;

            old_handle.read_to_string(&mut old_buf)?;
            new_handle.read_to_string(&mut new_buf)?;

            let diff = compare(old_buf.as_str(), new_buf.as_str())?;
            print_diff(&diff);

            Ok(())
        }
    }
}
