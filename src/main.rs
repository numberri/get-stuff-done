use chrono::{Duration, Local};
use clap::{Parser, Subcommand};
use nom::character::complete::{digit1, one_of};
use nom::combinator::{map_res, opt};
use nom::sequence::pair;
use nom::IResult;
use serde::{Deserialize, Serialize};
use shellexpand;
use std::str;
use std::fs::{remove_file, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::process::{exit, Command, Stdio};
use std::path::Path;

#[derive(Parser)]
#[command(name = "get shit done")]
#[command(version = "1.1")]
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
            println!("Network command changed.")
        }
        Some(Commands::Start { time, file, lines }) => {
            match time {
                Some(timer) => start_timed_session(timer),
                None => {}
            }
            match file {
                Some(path) => {
                    match lines {
                        Some(num) => watch_file(path.to_string(), (*num).into()),
                        None => watch_file(path.to_string(), 10),
                    };
                    clean_file(&shellexpand::tilde("~/.config/gsd/session.sh").to_string());
                    update_hosts();
                    update_cron_blocklist();
                    run_sudo_script(&shellexpand::tilde("~/.config/gsd/session.sh").to_string());
                    println!("Now watching {}. Enter \"gsd unlock\" to check status, and \"sudo gsd unlock\" once you meet the requirements.", path);
                    exit(0);
                }
                None => println!("Please use either -t [HOURS]h[MINUTES]m or -f [FILE] -l [LINES]"),
            }
        }
        Some(Commands::Unlock) => {
            check_file_unlock();
        }
        None => start_timed_session("1h"),
    }
}

//fn init_config() {}

fn open_config() -> ConfigToml {
    let path = shellexpand::tilde("~/.config/gsd/config.toml").to_string();
    let contents = std::fs::read_to_string(path).expect("Config file most likely does not exist");
    // todo: make this create a default config on error
    toml::from_str(&contents).expect("Invalid config file")
}

// todo, add actual error handling instead of .unwrap() .unwrap() .unwrap()
fn add_to_blocklist(mode: char, block: String) {
    let mut config_contents =
        std::fs::read_to_string(shellexpand::tilde("~/.config/gsd/config.toml").to_string()).expect("Failed to open config file");
    let mut config: ConfigToml = toml::from_str(&config_contents).unwrap();
    if mode == 'w' {
        let websites: &mut Vec<String> = &mut config.blocklist.websites;
        websites.push(block);
    } else if mode == 'p' {
        let programs: &mut Vec<String> = &mut config.blocklist.programs;
        programs.push(block);
    } else {
        panic!("Option must be -w [website] or -p [program process name]");
    }
    config_contents = toml::to_string(&config).unwrap();
    let mut config_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(format!("{}", shellexpand::tilde("~/.config/gsd/config.toml").to_string()))
        .unwrap();
    write_line(&mut config_file, &config_contents);
}

fn update_network(cmd: String) {
    let mut config_contents =
        std::fs::read_to_string(shellexpand::tilde("~/.config/gsd/config.toml").to_string()).expect("Failed to open config file");
    let mut config: ConfigToml = toml::from_str(&config_contents).unwrap();
    let network_command: &mut String = &mut config.system.network_command;
    *network_command = cmd;
    config_contents = toml::to_string(&config).unwrap();
    let mut config_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(format!("{}", shellexpand::tilde("~/.config/gsd/config.toml").to_string()))
        .unwrap();
    write_line(&mut config_file, &config_contents);
}

fn update_hosts() {
    let config = open_config();
    let hosts = std::fs::read_to_string("/etc/hosts").expect("Failed to read /etc/hosts");
    clean_file("/tmp/gsd/hosts.bak");
    let mut hosts_backup = make_file("/tmp/gsd", "hosts.bak");
    hosts_backup.write_all(&hosts.clone().into_bytes());
    let mut session_script = make_file(&shellexpand::tilde("~/.config/gsd").to_string(), "session.sh");
    write_line(&mut session_script, "#!/bin/bash\n");
    for site in &config.blocklist.websites {
        write_line(&mut session_script, &format!("echo \"127.0.0.1 {}\" >> /etc/hosts", site));
    }
    write_line(&mut session_script, &config.system.network_command);
}

fn update_cron_blocklist() {
    let cron = Command::new("crontab")
        .arg("-l")
        .output()
        .expect("Failed to run command: \"crontab -l\"")
        .stdout;
    clean_file("/tmp/gsd/cron.bak");
    clean_file("/tmp/gsd/cron.tmp");
    let mut cron_backup = make_file("/tmp/gsd", "cron.bak");
    cron_backup.write_all(&cron);
    let mut cron_tmp = make_file("/tmp/gsd", "cron.tmp");
    cron_tmp.write_all(&cron);
    let config: ConfigToml = open_config();
    let mut session_script = make_file(&shellexpand::tilde("~/.config/gsd").to_string(), "session.sh");
    write_line(&mut session_script, "# the following lines are being appended to the root crontab");
    for program in &config.blocklist.programs {
        let killall = &find_cmd("killall");
        write_line(&mut cron_tmp, &format!("* * * * * {} {}", killall, program));
        write_line(&mut session_script, &format!("# * * * * * {} {}", killall, program));
    }
}

fn add_cron_time(time: &str) {
    // will always be used after cron_blocklist so cron.tmp already exists
    let (h, m) = parse_time(time);
    let unblock_dt = Local::now() + Duration::minutes(i64::from(m)) + Duration::hours(i64::from(h));
    let restore_cron = unblock_dt + Duration::minutes(1);
    let cron_format = format!("{} *", unblock_dt.format("%M %H %d %m"));
    let restore_format = format!("{} *", restore_cron.format("%M %H %d %m"));
    let config: ConfigToml = open_config();

    let mut session_script = make_file(&shellexpand::tilde("~/.config/gsd").to_string(), "session.sh");
    let mut cron_tmp = make_file("/tmp/gsd", "cron.tmp");

    let cron_cmds = vec![
        format!("# {} {} /tmp/gsd/hosts.bak /etc/hosts", cron_format, find_cmd("cp")),
        format!("# {} /usr/bin/{} /tmp/gsd/hosts.bak /etc/hosts", cron_format, config.system.network_command),
        format!("# {} {} /tmp/gsd/cron.tmp", cron_format, find_cmd("rm")),
        format!("# {} {} /tmp/gsd/cron.bak", restore_format, find_cmd("crontab"))
        ];

    for cmd in cron_cmds {
        write_line(&mut session_script, &cmd);
        write_line(&mut cron_tmp, &cmd);
    }
}

fn watch_file(file: String, lines: u32) {
    let files = vec!["path.tmp", "goal.tmp", "cron.bak"];
    for f in files {
        clean_file(&format!("/tmp/gsd/{}", f));
    }
    let watched_file = File::open(&file).expect("Failed to open file");
    let buffered = BufReader::new(watched_file);
    // todo : It would be good to add some error handling here on the off chance that u32 and usize are not
    // compatible. It is on my computer but I don't want to assume that
    let line_count = buffered.lines().count() + usize::try_from(lines).unwrap();
    let mut goal_tmp = make_file("/tmp/gsd", "goal.tmp");
    write_line(&mut goal_tmp, &line_count.to_string());
    let mut path_tmp = make_file("/tmp/gsd", "path.tmp");
    write_line(&mut path_tmp, &file);
}

fn check_file_unlock() {
    let mut path = std::fs::read_to_string("/tmp/gsd/path.tmp")
        .expect("Failed to open file path.tmp. This will only exist for a file-tracking session.");
    let watched = File::open(&trim_newline(&mut path)).expect(&format!("Failed to open watched file {}", &path));
    let buffered = BufReader::new(watched);
    let line_count = buffered.lines().count();
    let goal = trim_newline(&mut std::fs::read_to_string("/tmp/gsd/goal.tmp")
        .expect("Failed to open file goal.tmp. This will only exist for a file-tracking session."))
        .parse::<usize>()
        .unwrap();
    if line_count >= goal {
        println!("Goal reached - congratulations! Unlocking files now. You should get prompted to use sudo to update /etc/hosts and root crontab.");

        let config: ConfigToml = open_config();
        clean_file(&shellexpand::tilde("~/.config/gsd/restore.sh"));
        let mut restore_script = make_file(&shellexpand::tilde("~/.config/gsd").to_string(), "restore.sh");
        write_line(&mut restore_script, 
            "#!/bin/bash\n\ncp /tmp/gsd/hosts.bak /etc/hosts\n");
        write_line(&mut restore_script, &config.system.network_command);
        drop(restore_script);
        run_sudo_script(&shellexpand::tilde("~/.config/gsd/restore.sh"));
    } else {
        println!(
            "Write {} more lines to unblock apps and websites!",
            goal - line_count
        )
    }
}

fn int_parser(input: &str) -> IResult<&str, u32> {
    map_res(digit1, str::parse)(input)
}

fn parse_time(input: &str) -> (u32, u32) {
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

    (h.unwrap_or_default(), m.unwrap_or_default())
}

fn make_file(path:&str, file:&str) -> File {
    if !Path::new(&path).exists() {
        std::fs::create_dir_all(path).unwrap();
    }
    let full_file = format!("{}/{}", path, file);
    OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&full_file)
        .expect(&format!("Was unable to open or create the file {}", full_file))
}

fn clean_file(file:&str) {
    match remove_file(file) {
        Ok(_) => {}
        Err(_) => {}
    };
}

fn find_cmd(cmd: &str) -> String {
    let path_vec = String::from_utf8(Command::new("which")
    .arg(cmd)
    .output()
    .expect(&format!("Failed to run command \"which {}\"", cmd))
    .stdout);
    let mut path = path_vec.expect(&format!("Failed to format to string for the output of \"which {}\"", cmd));
    trim_newline(&mut path)
}

fn trim_newline(s: &mut String) -> String {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
    s.to_string()
}

fn run_sudo_script(script: &str) {
    // todo get this part in a seperate fucking function
    let mut edit = OpenOptions::new()
    .write(true)
    .append(true)
    .create(true)
    .open(script)
    .expect(&format!("Was unable to open or create the file {}", script));
    if let Err(e) = writeln!(edit, "crontab /tmp/gsd/cron.tmp") {
        eprintln!("Couldn't write to file: {}", e);
    }
    drop(edit);
    // ^ it's this part here that fucking sucks why did I put it in this functions
    Command::new("chmod")
    .arg("+x")
    .arg(script)
    .spawn()
    .expect("Command chmod +x failed");
    println!("This command needs to run sudo. This is so that it can update root crontabs, update /etc/hosts, and restart the network. If you want to know what this is running, please look at {}.", script);
    let mut cmd = Command::new("sudo")
        .arg(script)
        .stdout(Stdio::inherit())
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect(&format!("Failed to run \"sudo {}\"", script));
    cmd.wait().expect("cmd.wait failed... oops");
}

fn write_line(file: &mut File, line: &str) {
    if let Err(e) = writeln!(file, "{}", line) {
        eprintln!("Couldn't write to file: {}", e);
    }
}

fn start_timed_session(time: &str) {
    clean_file(&shellexpand::tilde("~/.config/gsd/session.sh").to_string());
    update_hosts();
    update_cron_blocklist();
    add_cron_time(time);
    run_sudo_script(&shellexpand::tilde("~/.config/gsd/session.sh").to_string());
    println!("Session started for {}. Stay productive!", time);
    exit(0)
}
