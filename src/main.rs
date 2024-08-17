use clap::{Parser, Subcommand};
use nom::character::complete::{digit1, one_of};
use nom::combinator::{map_res, opt};
use nom::sequence::pair;
use nom::IResult;
use chrono::{Local, Duration};
use serde::{Deserialize, Serialize};
use std::fs::{remove_file, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, exit};


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
    /// unblock websites and apps once you write a certain amount of lines on a file.
    /// this is intended for use without --time and including --lines, however it will
    /// default to 10 line and will unlock when either the file or time requirement is
    /// met.
    #[arg(short, long, value_name = "FILE")]
    file: Option<PathBuf>,

    /// specify a number of lines that you want changed before blocked websites and apps
    /// unlock.
    ///
    /// will do nothing if you use without --file. this specifies the amount of lines that
    /// you need to write before things are unblocked. will default to 10 if not provided.
    #[arg(short, long)]
    lines: Option<u8>,
}

#[derive(Subcommand)]
enum Commands {
    /// adds a website or program to the blocklist
    ///
    /// adds a single website or program to blocklist on config.toml. if you wish to add
    /// multiple at a time, it may be a good idea to manually add websites to this file.
    Add {
        /// specify a website to be added to the blacklist
        #[arg(short, long)]
        website: Option<String>,

        /// specify a program ro be added to the blacklist
        #[arg(short, long)]
        program: Option<String>,
    },

    /// changes command to restart NetworkManager, networkctl, etc. this is to update
    /// /etc/hosts
    ///
    /// specifies a command to run to reset network connection, which is neccesary to
    /// update /etc/hosts. some examples may be 'sudo /etc/init.d/nscd restart' or
    /// 'sudo systemctl restart dhcpcd'.
    Network { command: String },

    /// specifies the path of the program running. this is so that crontabs can point directly to cron.bak.
    Directory { program_dir: String },

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
    program_dir: String,
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
            update_network(command.to_string());
            println!("Network command changed. However, at this point this is hardcoded to \"systemctl restart dhcpcd\".")
        }
        Some(Commands::Directory { program_dir }) => {
            update_dir(program_dir.to_string());
        }
        Some(Commands::Start { time, file, lines }) => {
            match time {
                Some(timer) => start_timed_session(timer.to_string()),
                None => {}
            }
        }
        Some(Commands::Break) => {
            println!("To be implemented");
        }
        Some(Commands::Status) => {
            println!("Give the status\n");
        }
        None => {
            println!("A command was not used\n");
        }
    }
}

//fn init_config() {}

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

fn update_network(cmd: String) {
    let mut config_contents =
        std::fs::read_to_string("config.toml").expect("Failed to open config file");
    let mut config: ConfigToml = toml::from_str(&config_contents).unwrap();
    let network_command: &mut String = &mut config.system.network_command;
    *network_command = cmd;
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

fn update_dir(dir: String) {
    let mut config_contents =
        std::fs::read_to_string("config.toml").expect("Failed to open config file");
    let mut config: ConfigToml = toml::from_str(&config_contents).unwrap();
    let program_dir: &mut String = &mut config.system.program_dir;
    *program_dir = dir;
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

fn update_hosts() {
    let config_contents =
        std::fs::read_to_string("config.toml").expect("Failed to open config file");
    let config: ConfigToml = toml::from_str(&config_contents).unwrap();
    let mut hosts_file = OpenOptions::new()
        .write(true)
        .open("/etc/hosts")
        .expect("Failed to open /etc/hosts - please run this program with sudo");
    let mut hosts = std::fs::read_to_string("/etc/hosts").expect("Failed to read file");
    let _ = match remove_file("hosts.bak") {
        Ok(_) => {},
        Err(_) => {}
    };
    let mut hosts_backup = File::create("hosts.bak").expect("Could not create file");
    let _ = hosts_backup.write_all(&hosts.clone().into_bytes());
    for site in &config.blocklist.websites {
        hosts.push_str(&format!("\n127.0.0.1 {}", site))
    }
    let _ = hosts_file.write_all(&hosts.clone().into_bytes());
    // I'm lazy right now so I shall have this work on my machine and have it work on
    // other people's machines later
    Command::new("systemctl")
        .arg("restart")
        .arg("dhcpcd")
        .spawn()
        .expect("Command failed");
}

fn restore_hosts() {
    let mut hosts_file = OpenOptions::new()
        .write(true)
        .open("/etc/hosts")
        .expect("Failed to open /etc/hosts - please run this program with sudo");
    let hosts_backup =
        std::fs::read_to_string("hosts.bak").expect("Failed to read file - make sure it exists");
    let _ = hosts_file.write_all(&hosts_backup.into_bytes());
    Command::new("systemctl")
        .arg("restart")
        .arg("dhcpcd")
        .spawn()
        .expect("Command failed");
    remove_file("hosts.bak").expect("Failed to delete hosts.bak)");
}

fn update_cron_blocklist() {
    let mut cron = Command::new("crontab")
        .arg("-l")
        .output()
        .expect("Failed to run command: \"crontab -l\"")
        .stdout;
    let _ = match remove_file("cron.bak") {
        Ok(_) => {},
        Err(_) => {}
    };
    let mut cron_backup = File::create("cron.bak").expect("Could not create file \"cron.bak\"");
    let _ = cron_backup.write_all(&cron.clone());
    let config_contents =
        std::fs::read_to_string("config.toml").expect("Failed to open config file");
    let config: ConfigToml = toml::from_str(&config_contents).unwrap();
    for program in &config.blocklist.programs{
        cron.append(&mut format!("* * * * * /usr/bin/killall {}\n", program).into_bytes())
    };
    let _ = match remove_file("cron.tmp") {
        Ok(_) => {},
        Err(_) => {}
    };
    let mut cron_tmp = File::create("cron.tmp").expect("Could not create file \"cron.tmp\"");
    let _ = cron_tmp.write_all(&cron.clone());
    Command::new("crontab")
        .arg("cron.tmp")
        .spawn()
        .expect("Command \"crontab cron.tmp\" failed");
}

fn add_cron_time(time: String) {
    let cron_u = Command::new("crontab")
        .arg("-l")
        .output()
        .expect("Failed to run command: \"crontab -l\"")
        .stdout;
    let mut cron = String::from_utf8(cron_u).unwrap();
    let (h, m) = parse_time(time);
    let unblock_dt = Local::now()
        + Duration::minutes(i64::from(m))
        + Duration::hours(i64::from(h));
    let restore_cron = unblock_dt + Duration::minutes(1);
    let cron_format = format!("{} *", unblock_dt.format("%M %H %d %m"));
    let restore_format = format!("{} *", restore_cron.format("%M %H %d %m"));
    let config_contents =
        std::fs::read_to_string("config.toml").expect("Failed to open config file");
    let config: ConfigToml = toml::from_str(&config_contents).unwrap();
    cron.push_str(&mut format!("{} /usr/bin/crontab {}/cron.bak\n", restore_format, config.system.program_dir));
    cron.push_str(&mut format!("{} /usr/bin/cp {}/hosts.bak /etc/hosts\n", cron_format, config.system.program_dir));
    cron.push_str(&mut format!("{} /usr/bin/{}\n", cron_format, config.system.network_command));
    let _ = match remove_file("cron.tmp") {
        Ok(_) => {},
        Err(_) => {}
    };
    let mut cron_tmp = File::create("cron.tmp").expect("Could not create file \"cron.tmp\"");
    let _ = cron_tmp.write_all(&cron.clone().into_bytes());
    Command::new("crontab")
        .arg("cron.tmp")
        .spawn()
        .expect("Command \"crontab cron.tmp\" failed");
}

fn int_parser(input: &str) -> IResult<&str, u32> {
    map_res(digit1, str::parse)(input)
}

fn parse_time(input: String) -> (u32, u32) {
    let mut parser = pair(
        opt(pair(int_parser, one_of("Hh"))),
        opt(pair(int_parser, one_of("Mm"))),
    );
    let (input, (hours, minutes)) = parser(&input).unwrap();
    if input != "" {
        panic!("Malformed time input - please use format \"[number]h[number]m\"")
    }
    let h = hours.map(|t| t.0);
    let m = minutes.map(|t| t.0);

    (h.unwrap_or_default(),m.unwrap_or_default())
}

fn start_timed_session(time: String) {
    update_hosts();
    update_cron_blocklist();
    add_cron_time(time.clone());
    println!("Session started for {}. Stay productive!", time);
    exit(0)
}
