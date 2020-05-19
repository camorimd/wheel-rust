![Rust](https://github.com/camorimd/wheel-rust/workflows/Rust/badge.svg)

# Introduction
This command line tool is a simple tool to let a Twitch streamer do a giveaway for his followers and viewers.
To use this command line you will need to register an application in [Twitch Console](https://dev.twitch.tv) and
get a Client-ID and Client-Secret, it's free do to.

Once you've done it, you will need to create a file called: app, inside that file you will need to write 
{client-id}:{client-secret}. 

# Usage
```bash
wheel 1.0.0
Claudio Amorim <camorimd@gmail.com>
A simple tool to let Twitch Streamers do giveaways

USAGE:
    wheel.exe [FLAGS] <channel>

FLAGS:
    -m, --drop         Drop moderators from giveaway
    -e, --extra        Give viewers extra tickets
    -f, --followers    Add followers to the giveaway
    -h, --help         Prints help information
    -s, --subs         Add subs to the giveaway
    -V, --version      Prints version information
    -v, --viewers      Add viewers to the giveaway

ARGS:
    <channel>    User channel
```

# Features
* Reads client-id and client-secret from file
* All the channel followers will receive a ticket
* All the viewers will get an extra ticket
* You can add a discarded.txt file with the usernames of the people you don't want to be in the giveaway.
* Let the user be able to select who enters the giveaway
..* Viewers
..* Followers
* Select if you want a to give viewers extra tickets
* Select if you want to auto-discard moderators

# Roadmap
* let the user be able to select subs for the giveaway
* Let the program write in the stream chat the result of the giveaway


# License
MIT License

Copyright (c) 2020 Claudio Amorim

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
