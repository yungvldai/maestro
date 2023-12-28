# maestro

![maestro](https://github.com/yungvldai/maestro/blob/main/media/cover.png)

`maestro` is a simple process manager and it can help you start multiple processes dependent on each other and then stop them in the right order. 

One of the use cases could be running in Docker. Although it is generally an anti-pattern to run multiple services in one container, sometimes it is a necessary requirement.

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
      - [`exit_code`](#exit_code)
      - [`delay`](#delay)
      - [`command`](#command)
      - [`http`](#http)
- [Recipes](#recipes)
  - [Using in Docker](#using-in-docker)
  - [Using environment variables in config](#using-environment-variables-in-config)
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

You can check existing builds and versions [here](https://github.com/yungvldai/maestro/releases).

```bash
export MAESTRO_VERSION=1.0.0 # Specify required version
export MAESTRO_BUILD=linux-musl # Specify required build

curl -o maestro.zip "https://github.com/yungvldai/maestro/releases/download/${MAESTRO_VERSION}/maestro-${MAESTRO_BUILD}.zip" && unzip maestro.zip && rm maestro.zip
```

## Operation

`maestro` will start all apps in the specified order and will be listening for signals. If `maestro` receives the appropriate signal, it will attempt to gracefully stop the started apps in reverse order. If any of the app stop on their own, exiting with a non-zero code (or if `maestro` fails to obtain an exit code), `maestro` will also attempt to stop the remaining apps, preserving the order, and then exit itself.

Exiting the `maestro` program will only occur when all processes are either never started (**INIT**) or already **STOPPED** (excluding SIGKILL, of course).

## Configuration

The configuration file `maestro.yml` must be placed either in the current working directory or in `/etc/maestro`.
The configuration file must be a valid YAML document.

### `pid`

You can specify the `pid` option; in this case, when `maestro` starts, it will write the ID of the main process (itself) to the file whose path you provide.

For example:

```yaml
pid: /var/run/maestro.pid
```

By default, it only prints it. Note that the PID log has an info severity level `info` (you can read about loggging levels below).

### `log_level`

`maestro` supports various levels of logging, such as: `debug`, `info`, `warn` and `error`. With this option, you can configure the messages you want to see during operation. Also, it may be controlled using `RUST_LOG` environment variable.

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

For redirecting to a file, the option should take a value - the path to the file. Ensure that the user who running the `maestro` has the capabiltiy to write to this file. Do not worry, `maestro` will create nested directories automatically.

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

When `maestro` receives SIGINT (2) or SIGTERM (15), it initiates the shutdown procedure. All apps are stopped in the order dictated by `depends_on`. Although `maestro` itself only handles SIGINT and SIGTERM, you can specify the signal that should be sent to the app for shutdown.

This can be a numeric signal identifier or one of the strings: `sigint`, `sigterm`, `int`, `term`, in any case. By default, `maestro` will send a SIGTERM to your app.

`maestro` will wait for all your apps to stop until it receives a SIGKILL itself. `maestro` will attempt to send a SIGKILL to your app if an error occurs when attempting to send the specified signal.

#### `user`

By default, all your apps will run under the current effective user id. However, you can change this behavior by providing the `user` option. You can pass a username (in this case, the `id` command must be supported in your OS), or directly provide a uid.

Under the hood, `maestro` will call setuid in the child process (app), so this mechanism has limitations. For example, if you run `maestro` without root privileges, you can only start other app under the same user as `maestro` itself (this information requires confirmation).

#### `depends_on`

`depends_on` allows you to specify apps that must be **READY** before the configured app starts. The readiness of an app is determined by the readiness probe (option `ready`, read below).

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

The app will be considered **READY** if it exits with the specified code. This is useful if you need to run a script before the app.

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

The app will be considered **READY** after the specified number of milliseconds following its launch.

```yaml
apps:
  - name: app
    command: ["node", "app.js"]
    ready:
      delay: 5000
```

##### `command`

`maestro` will execute the specified command at the specified interval (default: 1s). The app will be considered **READY** if the command returns a zero exit code at some point. Please note that the command execution is blocking.

Example: 

```yaml
apps:
  - name: app
    command: ["node", "app.js"]
    ready:
      command: ["npm", "run", "check-ready"]
      period: 1000 # may be omitted (default: 1000ms)
```

##### `http`

`maestro` will make the specified HTTP request at the specified interval (default: 1s). The app will be considered **READY** if a 2xx HTTP status is returned in response. Please note that the request is blocking, and a timeout of 1 second is set for it.

Example: 

```yaml
apps:
  - name: app
    command: ["node", "app.js", "--", "--port", "3000"]
    ready:
      url: http://localhost:3000/health-check
      method: GET # case-insensitive and may be omitted (default: GET)
      period: 1000 # may be omitted (default: 1000ms)
```

## Recipes

### Using in Docker

`maestro.yml`:

```yml
apps:
  - name: server
    command: ["./run.sh"]
    ready:
      url: http://localhost:3000/health-check
  - name: gateway
    command: ["nginx", "-g", "daemon off;"]
    depends_on:
      server
```

`Dockerfile`:

```Dockerfile
...

# Example platform & version
ENV MAESTRO_VERSION 1.0.3
ENV MAESTRO_BUILD linux-musl

RUN wget "https://github.com/yungvldai/maestro/releases/download/${MAESTRO_VERSION}/maestro-${MAESTRO_BUILD}.zip" && \
	unzip "maestro-${MAESTRO_BUILD}.zip" && \
	rm "maestro-${MAESTRO_BUILD}.zip" && \
	mv "maestro-${MAESTRO_BUILD}" /usr/local/bin/maestro && \
	chmod a+x /usr/local/bin/maestro && \
	mkdir -p /etc/maestro

COPY ./maestro.yml /etc/maestro/maestro.yml

CMD ["maestro"]
```

### Using environment variables in config

You can use [envsubst](https://linux.die.net/man/1/envsubst) to substitute environment variables into the config.

`maestro-template.yml`:

```yml
apps:
  - name: server
    command: ["./run.sh"]
    stdout: $LOGS_DIR/stdout.log # It will be /var/logs/stdout.log
    ready:
      url: http://localhost:3000/health-check
```


`run.sh`:

```bash
#!/bin/sh

set -e

export LOGS_DIR="/var/logs"

envsubst '$LOGS_DIR' < "./maestro-template.yml" > "./maestro.yml"
```

## Known issues

- Signals generated by keyboard interrupts (like Ctrl+C) are sent to every process in the foreground [process group](https://en.wikipedia.org/wiki/Process_group) of the current session. Therefore `maestro` cannot guarantee the correct order to stop apps when using signals generated by keyboard.
