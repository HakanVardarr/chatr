# 💬 chatr

`chatr` is a simple asynchronous chat server and client implementation written in Rust using [Tokio](https://tokio.rs).  
It demonstrates how to build a text‑based protocol, handle multiple clients concurrently, and support both public and private messaging.

---

## Features

- Async TCP server with [Tokio](https://tokio.rs)
- Custom text‑based protocol
- User validation and session management
- Public chat 
- Private messages (direct user to user)
- Error codes and graceful disconnects

---

## Protocol

The chat protocol used by `chatr` is text‑based and easy to debug.  
See the full specification here: [Protocol Documentation](./PROTOCOL.md)

---


## License

MIT License. See [LICENSE](./LICENSE) for details.
