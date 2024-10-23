use clap::{Command, Arg};

fn main() {
    let matches = Command::new("Komando")
        .version("0.1.0")
        .author("Noureddine Gueddach")
        .about("A command line utility to better organize and keep track of your commands.")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Sets an input file")
                .num_args(1),
        )
        .arg(Arg::new("verbose")
            .short('v')
            .long("verbose")
            .help("Sets the level of verbosity"))
        .get_matches();
    if matches.contains_id("input") {
        println!("Input file: {}", matches.get_one::<String>("input").unwrap());
    }
    if matches.contains_id("verbose") {
        println!("Verbosity: true");
    }
    println!("Hello, world!");
}
