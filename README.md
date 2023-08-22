# HoodBot

HoodBot is a simple discord bot written in Rust that can play music, and roll dice.

## Installation

Put a discord bot token in the directory where you plan to run HoodBot in 'token.txt'.

Also make sure to run the `install` script if you are on Ubuntu or Arch linux. Windows users should install ffmpeg, yt-dlp, cmake and whatever else on their own.

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
