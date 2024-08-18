# jam

Jonah's Automatic Mirrorlist

## About

jam is a powerful, yet simple command line tool for generating Arch mirrorlists.
It was designed largely as a way for me to update my mirrorlist on a daily
cron job, but also for me to write something useful in Rust, a language I am
interested in, but not skilled in.

## Installation

This is not yet in the AUR, so currently you must build the Rust application
locally and install it that way.

If you are unfamiliar with rust, please install Rust on your system, and then
run the following commands to build and install the application:

```bash
$ cargo build --release
$ cargo install --path .
```

## How to use

There are several command line options to help customize your experience with
the tool. You can use the `--help` flag to list these commands.

Here is what I'm using in my cron job:

```bash
$ jam -p https -c US -o /etc/pacman.d/mirrorlist
```

I'm setting my protocol to https only, my country to US, and setting the output
file path. If no output file path is provided, the list will be printed to
the console.

## Contributing

If anyone _wants_ to contribute, I will welcome PRs. I'm not a Rust programmer,
so I'm sure there's a bunch of things that could be optimized in my code, but
this is a pretty small program, so there's likely not much to do.

The "vision" for this software is to remain simple. There are some more options
that could be added to make it more powerful, but it's ultimately just an
interface with the Arch mirror API.
