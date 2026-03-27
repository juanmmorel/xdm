import sys
import json
import struct
import socket
import os
import subprocess
import time
import threading

# Bridge between browser native messaging (stdin/stdout) and XDM IPC (HTTP on port 8597)

XDM_PORT = 8597
XDM_URL = f"http://127.0.0.1:{XDM_PORT}"

def log(msg):
    try:
        log_file = os.path.join(os.path.expanduser("~"), ".xdm_messaging_host.log")
        with open(log_file, "a") as f:
            f.write(f"{time.ctime()}: {msg}\n")
    except:
        pass

def send_to_browser(msg_dict, stream=None):
    if stream is None:
        stream = sys.stdout.buffer
    msg_json = json.dumps(msg_dict)
    msg_bytes = msg_json.encode('utf-8')
    stream.write(struct.pack('I', len(msg_bytes)))
    stream.write(msg_bytes)
    stream.flush()

def read_from_browser(stream=None):
    if stream is None:
        stream = sys.stdin.buffer
    text_length_bytes = stream.read(4)
    if not text_length_bytes:
        return None
    text_length = struct.unpack('I', text_length_bytes)[0]
    msg_bytes = stream.read(text_length)
    return json.loads(msg_bytes.decode('utf-8'))

def is_xdm_running():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        return s.connect_ex(('127.0.0.1', XDM_PORT)) == 0

def launch_xdm():
    try:
        # Try to find xdm-app in the same directory or parent
        base_dir = os.path.dirname(os.path.abspath(__file__))
        xdm_exe = os.path.join(base_dir, "xdm-app")
        if not os.path.exists(xdm_exe):
            xdm_exe = os.path.join(os.path.dirname(base_dir), "xdm-app")

        if not os.path.exists(xdm_exe):
            xdm_exe = "xdman" # Assume it's in PATH

        subprocess.Popen([xdm_exe, "--background"], env={**os.environ, "GTK_USE_PORTAL": "1"})
        log(f"Launched XDM: {xdm_exe}")
    except Exception as e:
        log(f"Failed to launch XDM: {e}")

def post_to_xdm(path, data):
    import urllib.request
    url = f"{XDM_URL}{path}"
    try:
        req = urllib.request.Request(url, data=json.dumps(data).encode('utf-8'), headers={'Content-Type': 'application/json'})
        with urllib.request.urlopen(req) as f:
            return json.loads(f.read().decode('utf-8'))
    except Exception as e:
        log(f"Error posting to XDM {path}: {e}")
        return None

def sync_loop():
    while True:
        try:
            if is_xdm_running():
                response = post_to_xdm("/sync", {})
                if response:
                    send_to_browser(response)
            else:
                send_to_browser({"enabled": False, "appExited": True})
        except Exception as e:
            log(f"Sync loop error: {e}")
        time.sleep(5)

def main():
    log("Started")
    if not is_xdm_running():
        launch_xdm()
        # Wait a bit for XDM to start
        for _ in range(10):
            if is_xdm_running():
                break
            time.sleep(1)

    threading.Thread(target=sync_loop, daemon=True).start()

    try:
        while True:
            msg = read_from_browser()
            if msg is None:
                break

            log(f"Received from browser: {msg}")

            path = "/download"
            if "vid" in msg:
                path = "/vid"
            elif "tab_update" in msg:
                path = "/tab-update"
            elif "clear" in msg:
                path = "/clear"
            elif "requestData" in msg:
                path = "/media"

            post_to_xdm(path, msg)
    except Exception as e:
        log(f"Main loop error: {e}")

if __name__ == "__main__":
    main()
