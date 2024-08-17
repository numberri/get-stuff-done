use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs::{write, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "get shit done")]
#[command(version = "1.0")]
#[command(about = "blocks websites and programs while you get work done", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// specify a time for a session to run for
    ///
    /// intended for use without file or line,
    #[arg(short, long)]
    time: Option<String>,

    /// specify a file to watch for lines changed
    ///
    /// unblock websites and apps once you write a certain amount of lines on a file. this is
    /// intended for use without --time and including --lines, however it will default to 10 lines
    /// and will unlock when either the file or time requirement is met.
    #[arg(short, long, value_name = "FILE")]
    file: Option<PathBuf>,

    /// specify a number of lines that you want changed before blocked websites and apps unlock.
    ///
    /// will panic if you use without --file. this specifies the amount of lines that you need to
    /// write before things are unblocked. will default to 10 if not provided.
    #[arg(short, long)]
    lines: Option<u8>,
}

#[derive(Subcommand)]
enum Commands {
    /// adds a website or program to the blocklisrt
    ///
    /// adds a single website or program to blocklist on config.toml. if you wish to add multiple at a time,
    /// it may be a good idea to manually add websites to this file.
    Add {
        /// specify a website to be added to the blacklist
        #[arg(short, long)]
        website: Option<String>,

        /// specify a program ro be added to the blacklist
        #[arg(short, long)]
        program: Option<String>,
    },

    /// changes command to restart NetworkManager, networkctl, etc. this is to update /etc/hosts
    ///
    /// specifies a command to run to reset network connection, which is neccesary to update
    /// /etc/hosts. some examples may be 'sudo /etc/init.d/nscd restart' or
    /// 'sudo systemctl restart dhcpcd'.
    Network { command: PathBuf },

    /// starts a new session. equivlent to gsd withourt any commands.
    Start {
        #[arg(short, long)]
        time: Option<String>,

        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,

        #[arg(short, long)]
        lines: Option<u8>,
    },

    /// pauses current session for 5 minutes
    Break,

    /// determines if a session is running and checks conditions to unlock
    Status,
}

#[derive(Deserialize, Serialize)]
struct ConfigToml {
    system: System,
    blocklist: Blocklist,
}

#[derive(Deserialize, Serialize)]
struct System {
    network_command: String,
}

#[derive(Deserialize, Serialize)]
struct Blocklist {
    websites: Vec<String>,
    programs: Vec<String>,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Add { website, program }) => {
            match website {
                Some(url) => add_to_blocklist('w', url.to_string()),
                None => {}
            };
            match program {
                Some(process_name) => add_to_blocklist('p', process_name.to_string()),
                None => {}
            }
        }
        Some(Commands::Network { command }) => {
            println!("Command to reset /etc/hosts is: {:?}\n", command)
        }
        Some(Commands::Start { time, file, lines }) => {
            println!("Time: {:?}\n File:{:?}\n Lines:{:?}\n", time, file, lines)
        }
        Some(Commands::Break) => {
            println!("Take a break\n");
        }
        Some(Commands::Status) => {
            println!("Give the status\n");
        }
        None => {
            println!("A command was not used\n");
        }
    }
}

fn init_config() {}

// todo, add actual error handling instead of .unwrap() .unwrap() .unwrap()
fn add_to_blocklist(mode: char, block: String) {
    let mut config_contents =
        std::fs::read_to_string("config.toml").expect("Failed to open config file");
    let mut config: ConfigToml = toml::from_str(&config_contents).unwrap();
    if mode == 'w' {
        let websites: &mut Vec<String> = &mut config.blocklist.websites;
        websites.push(block);
        config_contents = toml::to_string(&config).unwrap();
        let mut config_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("config.toml")
            .unwrap();
        if let Err(e) = writeln!(config_file, "{}", config_contents) {
            eprintln!("Couldn't write to file: {}", e);
        }
    } else if mode == 'p' {
        let programs: &mut Vec<String> = &mut config.blocklist.programs;
        programs.push(block);
        config_contents = toml::to_string(&config).unwrap();
        let mut config_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("config.toml")
            .unwrap();
        if let Err(e) = writeln!(config_file, "{}", config_contents) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }
}

/*
Using this as space to draft out what I want CLI arguments to look like

Commands:
    add <-w -p> <name>    adds a website or program to the blocklist
network-command <command>       changes command to restart network manager (updates /etc/hosts)

    start <-t | -f | -h>            starts a new session
    break                           pasues current session for 5 minutes - perhaps limit to 1-2 a session?
    status                          determines if a session is running. if it is, check conditions
                                    to unlock session

    help                            displays a help message :)

Options:
    -t, --time, --timer <XHXM>      sets a timer to block websites. in the form e.g. 2h30m, 45m, 1h
    -f --file <PATH> <NUM>          watches a file for difference in lines
    -h, --help                      shows a help message :)

no arguments will be equivlent to 'gsd start -t 1h'
*/
