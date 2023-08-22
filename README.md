# HoodBot

HoodBot is a simple discord bot written in Rust that can play music, and roll dice.

## Installation

Put a discord bot token in the directory where you plan to run HoodBot in 'token.txt'.

Also make sure to run the `install` script if you are on Ubuntu or Arch linux. Windows users should install ffmpeg, yt-dlp, cmake and whatever else on their own.

You might need to make it executable first with:

```bash
chmod +x ./install
```

### Install Rust if not already installed

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build and install HoodBot

To build and install hoodbot, run:

```bash
cargo install --path .
```

HoodBot can then be run using the `hoodbot` command if your cargo bin folder is in your path.

## Usage

The command prefix is `!`.

To queue a song:

```bash
!play <song-query>
```

To skip a song:

```bash
!skip
```

TODO: Document stuff better when there are more features
