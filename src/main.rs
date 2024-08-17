use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "get shit done")]
#[command(version = "1.0")]
#[command(about = "blocks websites and programs while you get work done", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// adds a website or program to the blocklisrt
    Add {
        /// specify a website to be added to the blacklist
        #[arg(short, long)]
        website: Option<String>,

        /// specify a program ro be added to the blacklist
        #[arg(short, long)]
        program: Option<String>,
    },

    /// changes command to restart NetworkManager, networkctl, etc. this is to update /etc/hosts
    Network { command: String },

    /// starts a new session
    Start {
        #[arg(short, long)]
        time: Option<String>,

        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,
    },

    /// pauses current session for 5 minutes
    Break,

    /// determines if a session is running and checks conditions to unlock
    Status,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Add { website, program }) => {
            println!("Website: {:?}\n Program: {:?}\n", website, program);
        }
        Some(Commands::Network { command }) => {
            println!("Command to reset /etc/hosts is: {:?}\n", command)
        }
        Some(Commands::Start { time, file }) => {
            println!("Start a session\n")
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
