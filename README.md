# Eremit

![eremit logo](eremit_logo.jpeg)

I decided to learn Rust because I don't know how to use a high-performance compiled language and this often stops me when I'm trying to do real-time audio or precise manipulation of time and audio. I know next to nothing about language and I'm learning as I go. I'm not even aware of all that language has to offer, so I just run into the walls and correct accordingly!

## Architecture

**Eremit** is a small live coding environment contained in a single binary:
- the interpreter is **Lua** (easy to integrate thanks to [mlua](https://github.com/khvzak/mlua)).
- the clock is using **Ableton Link** thanks to [rusty_link](https://github.com/anzbert/rusty_link).
- Classic I/O (WIP) with [rosc](https://github.com/klingtnet/rosc) and [midir](https://github.com/Boddlnagg/midir).
- (**TODO**) a scheduler happily scheduling I/O.

**Eremit** is designed to be a very compact and resilient programme. I'd like to be able to offer it as a binary for Linux / Mac / Windows. Communication with the interpreter is done automatically via the terminal. You'll need to create a small plugin for VSCode or Vim/Neovim to communicate with **Eremit** from your favorite editor. For the moment, there's so little we can do that it's not necessary. 

## Goals

- Robust timing and synchronization with modern **DAWs**
- Little memory footprint and running **Eremit** on small devices
- Flexible time model for sequencing patterns
- Learning as much as possible about Rust...

## Setup and Compiling

Compilation is a two step process, three with cloning the project:
1) `git clone https://github.com/Bubobubobubobubo/Eremit && cd Eremit`.
2) `git submodule update --init --recursive` for (_rusty link_).
3) `cargo build` or `cargo run`
