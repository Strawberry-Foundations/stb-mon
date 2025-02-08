<div align="center">
    <h1> 🍓 Strawberry Monitor (stbmon)</h1>
    <h3>A lightweight network service availability monitoring tool</h3>
</div>

## ⚠️ Development Status

This project is currently under development and is **not** ready for production use.

### Services
- [x] TCP services
- [x] HTTP/1.1 services
- [ ] UDP services
- [ ] ICMP monitors

### Web UI
- [x] Start page with status of all monitors
- [x] Admin interface which allows you to add and manage monitors
- [ ] Monitor page where you can see the history of a monitor

## What is this?

Strawberry Monitor is a simple uptime panel that allows you to monitor the uptime of services. As of writing this, it only supports TCP and HTTP, but UDP and ICMP support is planned.

## Features

- TCP and HTTP service monitoring (UDP and ICMP support planned)
- Web-based dashboard
- Configurable check intervals
- Authentication for administrative functions

## Why? 

Strawberry Monitor aims to provide:

- 🚀 **Simplicity**: Minimal setup, fast deployment
- 🎯 **Focus**: Core monitoring features without bloat
- 🪶 **Lightweight**:
   - Uses SQLite as database
   - Low resource consumption (uses 15MB of memory)
- 📊 **Clarity**: Clean, minimal web interface built on [new.css](https://newcss.net)

Perfect for small to medium infrastructures and developers seeking a straightforward monitoring solution.

## Core Concepts

- **Monitor**: A task that checks a service at defined intervals
- **Interval**: The minimum time between checks
- **Check**: Process of verifying service availability and response

## How to run

It's really simple! Set up the config in `stbmon.toml` as you like, then run it with `cargo r -r`. The database is automatically created.

## How to use

Open the web UI by going to the address defined in `stbmon.toml` (default is `http://127.0.0.1:13337`). From there, you can view your monitors, and after logging in with the password defined in the config, you can add, delete and edit monitors.

## Screenshots

#### Main page
![The stb-mon main page](https://github.com/Strawberry-Foundations/stb-mon/raw/master/main-page.png)

#### Main page
![The stb-mon admin page](https://github.com/Strawberry-Foundations/stb-mon/raw/master/admin.png)

#### Main page
![The stb-mon monitor info page](https://github.com/Strawberry-Foundations/stb-mon/raw/master/monitor-info.png)