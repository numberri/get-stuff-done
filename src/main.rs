use clap::{Parser, Subcommand};
use nom::character::complete::{digit1, one_of};
use nom::combinator::{map_res, opt};
use nom::sequence::pair;
use nom::IResult;
use chrono::{Local, Duration};
use serde::{Deserialize, Serialize};
use std::fs::{remove_file, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, exit};


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
    Directory { program_dir: PathBuf },

    /// starts a new session. takes flags -t "[hours]h[mins]m" or -f [file] -l [lines]
    Start {
        #[arg(short, long)]
        time: Option<String>,

        #[arg(short, long, value_name = "FILE")]
        file: Option<String>,

        #[arg(short, long)]
        lines: Option<u8>,
    },

    /// for sessions that watch a file, unlock will unlock the session or tell you how many more lines you need to write to unlock
    Unlock,
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
            update_dir(program_dir
                .clone()
                .into_os_string()
                .into_string()
                .unwrap());
                // good lord why did I do this
        }
        Some(Commands::Start { time, file, lines }) => {
            match time {
                Some(timer) => start_timed_session(timer.to_string()),
                None => {}
            }
            match file {
                Some(path) => {
                    match lines {
                    Some(num) => watch_file(path.to_string(), (*num).into()),
                    None => watch_file(path.to_string(), 10),
                };
                println!("Now watching {}. Enter \"gsd unlock\" to check status, and \"sudo gsd unlock\" once you meet the requirements.", path);
                exit(0);
            }
            None => println!("Please use either -t or -f [FILE] -l [LINES]")
            }
        }
        Some(Commands::Unlock) => {
            check_file_unlock();
        }
        None => start_timed_session("1h".to_string())
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
    cron.push_str(&mut format!("{} /usr/bin/rm {}/cron.tmp\n", cron_format, config.system.program_dir));
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

fn watch_file(file:String, lines: u32) {
    let _ = match remove_file("goal.tmp") {
        Ok(_) => {},
        Err(_) => {}
    };
    let watched_file = File::open(file.clone()).expect("Failed to open file");
    let buffered = BufReader::new(watched_file);
    let line_count = buffered.lines().count() + usize::try_from(lines).unwrap();
    let mut goal_tmp = File::create("goal.tmp").expect("Could not create file \"goal.tmp\"");
    let _ = goal_tmp.write_all(&format!("{}", line_count).into_bytes());
    let _ = match remove_file("path.tmp") {
        Ok(_) => {},
        Err(_) => {}
    };
    let mut path_tmp = File::create("path.tmp").expect("Could not create file \"path.tmp\"");
    let _ = path_tmp.write_all(&file.clone().into_bytes());
}

fn check_file_unlock() {
    let path = std::fs::read_to_string("path.tmp")
        .expect("Failed to open file path.tmp. This will only exist for a file-tracking session.");
    let watched = File::open(path).expect("Failed to open watched file");
    let buffered = BufReader::new(watched);
    let line_count = buffered.lines().count();
    let goal = std::fs::read_to_string("goal.tmp")
        .expect("Failed to open file goal.tmp. This will only exist for a file-tracking session.")
        .parse::<usize>()
        .unwrap();
    if line_count >= goal {
        println!("Goal reached - congratulations! Unlocking files now. If this fails, try this command with sudo.");

        let hosts_bak = std::fs::read_to_string("hosts.bak").expect("Failed to open hosts.bak");
        let mut hosts_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("/etc/hosts")
            .expect("Failed to open /etc/hosts - please run this program with sudo");
        let _ = hosts_file.write_all(&hosts_bak.into_bytes());

        Command::new("crontab")
            .arg("cron.bak")
            .spawn()
            .expect("Command \"crontab cron.bak\" failed");
        // again, I'm lazy right now so I shall have this work on my machine and have it work on
        // other people's machines later
        Command::new("systemctl")
        .arg("restart")
        .arg("dhcpcd")
        .spawn()
        .expect("Command failed");

        let _ = match remove_file("path.tmp") {
            Ok(_) => {},
            Err(_) => {}
        };
        let _ = match remove_file("goal.tmp") {
            Ok(_) => {},
            Err(_) => {}
        };
        let _ = match remove_file("cron.bak") {
            Ok(_) => {},
            Err(_) => {}
        };
    }
    else {
        println!("Write {} more lines to unblock apps and websites!", goal - line_count)
    }
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
