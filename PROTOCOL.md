# CProtocol (v1)

All messages are plain **UTF-8 text** terminated by a newline (`\n`).  
Fields are separated by the pipe character (`|`). Extra whitespace around fields should be ignored.  

---

## 1. Client → Server Commands

### HELLO
Identify yourself to the server. Must be sent before other commands.

```
HELLO | <username>
```

- `<username>`: must contain only ASCII alphabetic characters `[A-Za-z]+`

**Possible responses:**
```
WELCOME | <username> | <user_count>
OK      | SUCCESS
ERROR   | 03 | Invalid username.
ERROR   | 04 | You are already validated.
ERROR   | 05 | User already exists.
```

---

### MESSAGE
Send a public message to all connected users.

```
MESSAGE | <text>
```

- `<text>`: arbitrary UTF-8 text (excluding newline)

**Broadcast response to others:**
```
CHAT | <username> | <text>
OK   | SUCCESS
```

---

### QUIT
Gracefully disconnect from the server.

```
QUIT |
```

**Responses:**
- Connection is closed for the client
- Other clients receive:
```
LEFT | <username>
```

---

## 2. Server → Client Messages

### WELCOME
Sent after successful login.

```
WELCOME | <username> | <user_count>
```

- `<username>`: the logged-in user  
- `<user_count>`: number of currently connected users  

---

### CHAT
Public message from another user.

```
CHAT | <username> | <text>
```

---

### ERROR
Indicates an error or invalid command.

```
ERROR | <error_code> | <error_message>
```

**Common codes:**
- `01` → Protocol violation (`Please follow protocol.`)  
- `02` → Unknown/invalid command  
- `03` → Invalid username  
- `04` → Already validated  
- `05` → Username already exists  
- `06` → Not validated yet  
- `07` → Private messege to yourself
- `08` → User does not exists

---

### LEFT
Sent when a user leaves the chat.

```
LEFT | <username>
```

---

### SUCCESS
Indicates that a client command has been processed successfully.

```
OK | SUCCESS
```

---

## 3. Example Session

```
CLIENT → SERVER:  HELLO | Alice
SERVER → CLIENT:  WELCOME | Alice | 1

CLIENT → SERVER:  MESSAGE | Hello everyone!
SERVER → ALL:     CHAT | Alice | Hello everyone!
SERVER → Alice:   OK | Success

CLIENT → SERVER:  QUIT |
SERVER → ALL:     LEFT | Alice
```

---

## 4. Optional Extensions

The protocol can be extended with additional commands. For example, private messages:

```
PRIVATE | <to_username> | <text>
```

**Response to the recipient:**
```
PRIVATE | <from_username> | <text>
SUCCESS
```

---

⚡️ This protocol is intentionally simple and text-based, making it easy to debug and extend.  
