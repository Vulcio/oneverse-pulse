"""Real Oneverse engine under pulsed; the cloud stays a local contract stand-in.

Set PULSE_TEST_ENGINE_DIR to a package-engine.sh output and PULSE_TEST_BINARY to
a freshly built pulsed. CI's engine job sets both; ordinary Python checks skip it.
"""
import http.server
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import tempfile
import threading
import time
import unittest
import urllib.error
import urllib.request


def port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def get(url):
    try:
        with urllib.request.urlopen(url, timeout=2) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as error:
        return error.code, error.read()


@unittest.skipUnless(os.environ.get("PULSE_TEST_ENGINE_DIR"), "requires a real packaged Oneverse engine")
class EngineIntegration(unittest.TestCase):
    def test_pairing_restart_shutdown_and_bundle_rejection(self):
        bundle = Path(os.environ["PULSE_TEST_ENGINE_DIR"]).resolve(strict=True)
        binary = Path(os.environ["PULSE_TEST_BINARY"]).resolve(strict=True)
        requests = []

        class Cloud(http.server.BaseHTTPRequestHandler):
            def do_POST(self):
                body = json.loads(self.rfile.read(int(self.headers.get("Content-Length", 0))))
                requests.append((self.path, body))
                if self.path == "/screens/pairings":
                    result = {"pairingId": "fixture", "code": "P1-PAIR", "secret": "local-fixture", "expiresAt": "2099-01-01T00:00:00Z"}
                    self.send_response(200)
                    self.send_header("Content-Type", "application/json")
                    self.end_headers()
                    self.wfile.write(json.dumps(result).encode())
                else:
                    self.send_response(202)
                    self.end_headers()

            def log_message(self, *args):
                pass

        cloud = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Cloud)
        self.addCleanup(cloud.server_close)
        self.addCleanup(cloud.shutdown)
        threading.Thread(target=cloud.serve_forever, daemon=True).start()
        with tempfile.TemporaryDirectory() as temp:
            data = Path(temp) / "data"
            daemon_port, room_port = port(), port()
            env = {k: v for k, v in os.environ.items() if not k.startswith(("PULSE_", "SONGVERSE_", "ONEVERSE_"))}
            env.update({"PULSE_ENGINE_DIR": str(bundle), "PULSE_DATA_DIR": str(data), "PULSE_DAEMON_PORT": str(daemon_port),
                        "PULSE_ROOM_PORT": str(room_port), "PULSE_ROOM_HOST": "127.0.0.1", "PULSE_NODE_ID": "pulse-ci-fixture",
                        "ONEVERSE_API_URL": f"http://127.0.0.1:{cloud.server_port}"})
            url = f"http://127.0.0.1:{daemon_port}"
            with (Path(temp) / "daemon.log").open("w+") as log:
                process = subprocess.Popen([str(binary)], env=env, stdout=log, stderr=log)
                child_pid = None
                try:
                    def wait_for(predicate):
                        deadline = time.monotonic() + 30
                        while time.monotonic() < deadline:
                            if process.poll() is not None:
                                log.seek(0)
                                self.fail("daemon exited: " + log.read())
                            try:
                                code, body = get(url + "/health")
                                state = json.loads(body)
                                if predicate(code, state):
                                    return state
                            except (OSError, ValueError):
                                pass
                            time.sleep(0.1)
                        log.seek(0)
                        self.fail("health timed out: " + log.read())

                    state = wait_for(lambda code, s: code == 200 and s["engine"]["health"]["pairing"] is not None)
                    child_pid = state["engine"]["pid"]
                    self.assertEqual(state["engine"]["health"]["pairing"]["code"], "P1-PAIR")
                    self.assertEqual(state["engine"]["health"]["pairing"]["expiresAt"], "2099-01-01T00:00:00Z")
                    self.assertEqual(requests[0], ("/screens/pairings", {"kind": "PULSE"}))
                    self.assertEqual(state["bundle"]["source_revision"], json.loads((bundle / "manifest.json").read_text())["source_revision"])
                    self.assertIsInstance(state["disk_available_bytes"], int)
                    self.assertEqual(data.stat().st_mode & 0o777, 0o700)
                    self.assertEqual(get(url + "/api/state")[0], 404)
                    code, page = get(f"http://127.0.0.1:{room_port}/")
                    self.assertEqual(code, 200)
                    self.assertEqual(page, (bundle / "console/pulse.html").read_bytes())
                    os.kill(child_pid, signal.SIGKILL)
                    state = wait_for(lambda code, s: code == 503 and s["engine"]["state"] == "backoff")
                    self.assertIsNone(state["engine"]["health"])
                    state = wait_for(lambda code, s: code == 200 and s["engine"]["pid"] != child_pid)
                    self.assertGreaterEqual(state["engine"]["restarts"], 1)
                    child_pid = state["engine"]["pid"]
                finally:
                    process.terminate()
                    try:
                        process.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait()
                        self.fail("daemon did not shut down")
                    if child_pid:
                        with self.assertRaises(ProcessLookupError):
                            os.kill(child_pid, 0)
            # An unrelated pre-existing room listener must never be mistaken for our child.
            with (Path(temp) / "occupied.log").open("w+") as log:
                process = subprocess.Popen([str(binary)], env={**env, "PULSE_ROOM_PORT": str(cloud.server_port)}, stdout=log, stderr=log)
                try:
                    state = wait_for(lambda code, s: code == 503 and s["engine"]["state"] == "backoff")
                    self.assertIsNone(state["engine"]["pid"])
                    self.assertIn("port already in use", state["engine"]["error"])
                finally:
                    process.terminate()
                    process.wait(timeout=10)
            # A partial bundle must fail before opening the daemon API or spawning anything.
            invalid = Path(temp) / "invalid"
            invalid.mkdir()
            (invalid / "manifest.json").write_text((bundle / "manifest.json").read_text())
            failed = subprocess.run([str(binary)], env={**env, "PULSE_ENGINE_DIR": str(invalid)}, capture_output=True, timeout=10)
            self.assertNotEqual(failed.returncode, 0)


if __name__ == "__main__":
    unittest.main()
