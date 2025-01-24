# 🍓 Strawberry Monitor (stbmon)
## ⚠️ Strawberry Monitor is NOT completed and in a working state
## What is this?
Strawberry Monitor is a simple uptime panel that allows you to monitor the uptime of services. As of writing this, it only supports TCP, but UDP and ICMP support is planned.
## Why?
🐈

## Terms
- Monitor: a "task" which checks a service at a given interval
- Interval: the minimum time between checks
- Check: the process of stbmon running a monitor and checking if it is up and if it responds like it should

## How to run
It's really simple! Set up the config in `stbmon.toml` as you like, then run it with `cargo r -r`. The database is automatically created.

## How to use
Open the web UI by going to the address defined in `stbmon.toml` (default is `http://127.0.0.1:13337`). From there, you can view your monitors, and after logging in with the password defined in the config, you can add, delete and edit monitors.
