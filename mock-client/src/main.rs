//! Hello world server.
//!
//! A simple client that opens a TCP stream, writes "hello world\n", and closes
//! the connection.
//!
//! You can test this out by running:
//!
//!     ncat -l 25000
//!

#![warn(rust_2018_idioms)]

use std::env;
use std::error::Error;

use mock::pressure_ec2;

fn print_usage(program: &str, opts: &getopts::Options) {
    let brief = format!(
        r#"Echo benchmark.

Usage:
  {program} [ -a <address> ] [ -l <length> ] [ -c <number> ] [ -t <duration> ] [ -r <rw_ratio> ]
  {program} (-h | --help)
  {program} --version"#,
        program = program
    );
    print!("{}", opts.usage(&brief));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    let program = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "Print this help.");
    opts.optopt(
        "a",
        "address",
        "Target echo server address. Default: 127.0.0.1:25000",
        "<address>",
    );
    opts.optopt(
        "l",
        "length",
        "Test message length. Default: 1024",
        "<length>",
    );
    opts.optopt(
        "s",
        "start",
        "Measurement start time in seconds. Default: 0",
        "<start>",
    );
    opts.optopt(
        "t",
        "duration",
        "Test duration in seconds. Default: 10",
        "<duration>",
    );
    opts.optopt(
        "c",
        "number",
        "Test connection number. Default: 10",
        "<number>",
    );

    opts.optopt(
        "r",
        "rw_ratio",
        "Read write ratio from 0 to 100. Default: 100",
        "<rw_ratio>",
    );

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", e.to_string());
            print_usage(&program, &opts);
            return Err(e.into());
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, &opts);
        return Ok(());
    }

    let length = matches
        .opt_str("length")
        .unwrap_or_default()
        .parse::<usize>()
        .unwrap_or(1024);

    // if length > 4096 {
    //     println!("Please specify packet size equal or smaller than 4096 bytes.");
    //     return Ok(());
    // }
    let start = matches
        .opt_str("number")
        .unwrap_or_default()
        .parse::<u64>()
        .unwrap_or(0);
    let duration = matches
        .opt_str("duration")
        .unwrap_or_default()
        .parse::<u64>()
        .unwrap_or(10);
    let number = matches
        .opt_str("number")
        .unwrap_or_default()
        .parse::<u32>()
        .unwrap_or(10);
    let address = matches
        .opt_str("address")
        .unwrap_or_else(|| "127.0.0.1:25000".to_string());
    // .parse::<SocketAddr>()
    // .unwrap();

    let rw_ratio = matches
        .opt_str("rw_ratio")
        .unwrap_or_default()
        .parse::<u32>()
        .unwrap_or(100);

    println!("Benchmarking:");
    println!(
        "{} streams will send {}-byte packets to {} for {} sec.",
        number, length, address, duration
    );

    pressure_ec2(address.as_str(), start, duration, number, length, rw_ratio).await?;

    Ok(())
}
