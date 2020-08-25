# Laminar Heartbeat Demonstration

This creates a server (`:12350`) and a client (`:12355`). Each are configured to send heartbeats every 2 seconds and timeout after 5 seconds.

```
idle_connection_timeout: Duration::from_secs(5),
heartbeat_interval: Some(Duration::from_secs(2)),
```

The client, on startup will send a `Ping` to the server. The server can optionally respond with a `pong` message.

What we see is that if the server responds with a `pong` message, heartbeats flow in both direction and the connection is maintained on both ends.

If the server does not respond with a `pong` message, heartbeats only flow from the client to the server, until the client times out. The server sends a new `Connection` event for each heartbeat that it receives, and never reports a timeout event after the client stops sending heartbeat messages.

## Running It

Run with `cargo run -- -s` to start a server, or `cargo run -- -s -p` to start a server.
Run with `cargo run -- -c` to start a client. The client sends a message to the server on startup, so make sure the server is already running.

## Ping

### Capture

![ping capture](screenshots/ping-capture.png)

### Server

```
$ cargo run -- -s
[164.4µs] Bound to 127.0.0.1:12350
        [2.43s] Connection from: 127.0.0.1:12355
[2.43s] Received "Ping" from V4(127.0.0.1)
        [4.42s] Connection from: 127.0.0.1:12355
        [6.42s] Connection from: 127.0.0.1:12355
```

### Client

```
$ cargo run -- -c
[147.0µs] Bound to 127.0.0.1:12355
        [269.30µs] Sent: Ping
        [5.00s] Timeout from: 127.0.0.1:12350
```

## Ping w/ Server Pong

### Capture

![ping-pong capture](screenshots/ping-pong-capture.png)

### Server

```
$ cargo run -- -s -p
[116.8µs] Bound to 127.0.0.1:12350
        [1.32s] Connection from: 127.0.0.1:12355
[1.32s] Received "Ping" from V4(127.0.0.1)
        [1.32s] Sent: pong
        [64.33s] Timeout from: 127.0.0.1:12355
```

### Client

```
$ cargo run -- -c
[154.3µs] Bound to 127.0.0.1:12355
        [273.42µs] Sent: Ping
[24.87ms] Received "pong" from V4(127.0.0.1)

# client interrupted
```
