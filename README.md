# maestro

Simple processes manager

<!-- TOC tocDepth:2..5 chapterDepth:2..6 -->

- [Features](#features)
- [Installation](#installation)
- [Operation](#operation)
- [Configuration](#configuration)
    - [`pid`](#pid)
    - [`log_level`](#log_level)
    - [`apps`](#apps)
        - [`stdout` & `stderr`](#stdout-stderr)
        - [`signal`](#signal)
        - [`user`](#user)
        - [`depends_on`](#depends_on)
        - [`ready`](#ready)
            - [exit_code](#exit_code)
            - [delay](#delay)
            - [command](#command)
            - [http](#http)
- [Known issues](#known-issues)

<!-- /TOC -->

## Features

- Running processes under different users
- The correct order to start and stop processes
- 4 types of process readiness probe: `delay`, `http`, `command` and `exit_code`
- Ability to specify a stop signal separately for each process
- Redirecting of stdout and stderr
- Simple YAML configuration

## Installation

## Operation

`maestro` will start all app in the specified order and will be listening for signals. If `maestro` receives the appropriate signal, it will attempt to gracefully stop the started apps in reverse order. If any of the applications stop on their own, exiting with a non-zero code (or if `maestro` fails to obtain an exit code), `maestro` will also attempt to stop the remaining apps, preserving the order, and then exit itself.

Exiting the `maestro` program will only occur when all processes are either never started (**INIT**) or already **STOPPED** (excluding SIGKILL, of course).

## Configuration

The configuration file `maestro.yml` must be placed either in the current working directory or in `/etc/maestro`.
The configuration file must be a valid YAML document.

### `pid`

You can specify the `pid` option; in this case, when `maestro` starts, it will write the ID of the main process (itself) to the file whose path you provide.

For example:

```yaml
pid: /var/run/maestro.pid;
```

By default, it only prints it. Note that the PID log has an info severity level `info` (you can read about loggging levels below).

### `log_level`

`maestro` supports various levels of logging, such as: `debug`, `info`, `warn` and `error`. With this option, you can configure the messages you want to see during operation.

### `apps`

Apps must be an array. The app must have a `name` (any valid YAML string) and `command` (array of strings).

Example of minimal app config:

```yaml
apps:
  - name: app
    command: ["node", "./app.js"]
```

#### `stdout` & `stderr`

You can redirect stdout and stderr to a file, to your terminal, or completely mute them. To do this, use the `stdout` and `stderr` options.

For redirecting to a file, the option should take a value - the path to the file. Ensure that the specified `user` for the app has the capabiltiy to write to this file.

Example:

```yaml
apps:
  - name: app
    stdout: /var/log/maestro/app/stdout.log
```

To display messages in the terminal from which `maestro` was launched, pass the keyword `inherit`:

```yaml
apps:
  - name: app
    stderr: inherit
```

By default, app logs are not written anywhere.

#### `signal`

When `maestro` receives SIGINT (2) or SIGTERM (15), it initiates the shutdown procedure. All apps are stopped in the order dictated by `depends_on`. Although `maestro` itself only handles SIGINT and SIGTERM, you can specify the signal that should be sent to the application for shutdown.

This can be a numeric signal identifier or one of the strings: `sigint`, `sigterm`, `int`, `term`, in any case. By default, `maestro` will send a SIGTERM to your app.

`maestro` will wait for all your apps to stop until it receives a SIGKILL itself. `maestro` will attempt to send a SIGKILL to your application if an error occurs when attempting to send the specified signal.

#### `user`

By default, all your applications will run under the current effective user id. However, you can change this behavior by providing the `user` option. You can pass a username (in this case, the `id` command must be supported in your OS), or directly provide a uid.

Under the hood, `maestro` will call setuid in the child process (app), so this mechanism has limitations. For example, if you run `maestro` without root privileges, you can only start other app under the same user as `maestro` itself (this information requires confirmation).

#### `depends_on`

`depends_on` allows you to specify apps that must be **READY** before the configured application starts. The readiness of an application is determined by the readiness probe (option `ready`, read below).

`depends_on` takes an array of strings - the names of other apps.

Example:

```yaml
apps:
  - name: db
    command: ["postgres"]
    ready: 
      command: ["./ping-postgres.sh"] # Pings postgres and exits with 0 or 1 code
  - name: server
    command: ["python", "server.py"]
    depends_on:
      - db
```

In the example above, the `server` won't be started until the `ping-postgres.sh` script exits with a code of 0.

When stopping `maestro`, it will wait for all dependents to stop before proceeding to stop the app.

#### `ready`

By default, apps are considered ready immediately after start. This can be changed by configuring a readiness probe using this option.

##### `exit_code`

The application will be considered **READY** if it exits with the specified code. This is useful if you need to run a script before the application.

Example: 

```yaml
apps:
  - name: migrations
    command: ["./run-migrations"]
    ready: 
      exit_code: 0
  - name: server
    command: ["python", "server.py"]
    depends_on:
      - migrations
```

##### `delay`

The application will be considered **READY** after the specified number of milliseconds following its launch.

```yaml
apps:
  - name: app
    command: ["node", "app.js"]
    ready:
      delay: 5000
```

##### `command`

##### `http`

## Known issues

- Signals generated by keyboard interrupts (like Ctrl+C) are sent to every process in the foreground [process group](https://en.wikipedia.org/wiki/Process_group) of the current session. Therefore `maestro` cannot guarantee the correct order to stop apps when using signals generated by keyboard.