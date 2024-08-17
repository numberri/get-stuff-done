# get shit done: blocking websites and programs so you can, well, get shit done

## What is this?

This is a project that I wrote for the 2024 UQ Computing Society hackathon. I wrote this because I couldn't get work done as I kept on getting distracted by websites such as Discord, Reddit, GBATemp, and other social media/forum/messaging sites.

This will automate changes to crontabs and /etc/hosts so that it will unlock after you get some work done on a coding file, or after a certain amount of time. To use it, you need Linux, and a working cron daemon.

## How do I set it up?

Ideally, I want to make it so that you can set up entirely though commands: using `gsd network` to specify a command to be run (under sudo) to restart the network and apply changes to /etc/hosts, and use `gsd add -w` and `gsd add -p` to block websites and programs, respectively. However, this relies on having a `config.toml` file which I haven't gotten around to making a generator for. For this, an example of what this file should look like is provided below:

```toml
[system]
# command to restart network service to refresh /etc/hosts. doesn't include sudo.
network_command = 'systemctl restart dhcpcd'

[blocklist]
# make sure to include www. if it is part of the URL. check on a per-website basis: comapre
# www.reddit.com and discord.com
websites = [
  'discord.com',
  'www.reddit.com',
  'gbatemp.net',
  'www.facebook.com',
  'www.tumblr.com',
  'tech.lgbt',
]
# programs should be what the process name is called. e.g. 'Discord' will kill discord
# proccesses but 'discord' won't
programs = [
  'Discord',
  'steam',
]
```

## What are the limitations of this?

- You can only track plaintext files. If you want 100 words written on some essay, then the track may not work for you.
- On a similar note, this only works for lines, not words at the current moment. This may be changed as I have to write LaTeX files quite often...
- It's easy to fudge. You can just write meaningless comments to increase line count to unlock websites, and there's nothing stopping you from reverting changes to crontabs and /etc/hosts. If you want this, there's a Perl program out there for you called Lockout: <https://thomer.com/lockout/> which will lock you out of root; this is however a big risk to take. get shit done is safer!

## Why did you decide to write this in Rust?

"hey Avery you should try writing something in Rust sometime it's such a cool language"
