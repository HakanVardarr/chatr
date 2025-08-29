import socket
import ssl
import sys
import threading

SERVER_HOST = "176.96.131.170"
SERVER_PORT = 3030


SERVER_NAME = "screenshotpaylas.com"


def listen_for_messages(ssl_sock: ssl.SSLSocket):
    while True:
        try:
            data = ssl_sock.recv(1024)
            if not data:
                break
            msg = data.decode(errors="ignore").strip()
            sys.stdout.write(f"\r<< {msg}\n>> ")
            sys.stdout.flush()
        except OSError:
            break


def main():
    context = ssl.create_default_context()

    raw_sock = socket.create_connection((SERVER_HOST, SERVER_PORT))
    ssl_sock = context.wrap_socket(raw_sock, server_hostname=SERVER_NAME)

    print(f"Connected securely to server at {SERVER_HOST}:{SERVER_PORT}")

    threading.Thread(target=listen_for_messages, args=(ssl_sock,), daemon=True).start()

    try:
        while True:
            cmd = input(">> ").strip()
            if not cmd:
                continue

            ssl_sock.sendall((cmd + "\n").encode())

            if cmd.upper().startswith("QUIT"):
                print("Quitting...")
                break
    except KeyboardInterrupt:
        print("Interrupted, quitting...")
        ssl_sock.sendall(b"QUIT |\n")
    finally:
        ssl_sock.close()


if __name__ == "__main__":
    main()
