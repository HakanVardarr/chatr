import socket
import sys
import threading

SERVER_HOST = "127.0.0.1"
SERVER_PORT = 3030


def listen_for_messages(sock: socket.socket):
    while True:
        try:
            data = sock.recv(1024)
            if not data:
                break
            msg = data.decode().strip()
            sys.stdout.write(f"\r<< {msg}\n>> ")
            sys.stdout.flush()
        except OSError:
            break


def main():
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect((SERVER_HOST, SERVER_PORT))
    print(f"Connected to server at {SERVER_HOST}:{SERVER_PORT}")

    threading.Thread(target=listen_for_messages, args=(sock,), daemon=True).start()

    try:
        while True:
            cmd = input(">> ").strip()
            if not cmd:
                continue

            sock.sendall((cmd + "\n").encode())

            if cmd.upper().startswith("QUIT"):
                print("Quitting...")
                break
    except KeyboardInterrupt:
        print("Interrupted, quitting...")
        sock.sendall(b"QUIT |\n")
    finally:
        sock.close()


if __name__ == "__main__":
    main()
