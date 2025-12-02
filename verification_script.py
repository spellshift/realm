import threading
import http.server
import socketserver
import os
import time
from playwright.sync_api import sync_playwright, expect

PORT = 8000
DIRECTORY = "implants/lib/eldritchv2/eldritch-repl/www"

class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIRECTORY, **kwargs)

def run_server():
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(("", PORT), Handler) as httpd:
        print(f"Serving at port {PORT}")
        httpd.serve_forever()

def verify():
    server_thread = threading.Thread(target=run_server, daemon=True)
    server_thread.start()
    time.sleep(2)

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page()

        try:
            page.goto(f"http://localhost:{PORT}")

            expect(page.locator("#terminal")).to_contain_text("stdlib fakes registered", timeout=10000)

            page.click("body")

            # Type 'zzzzzz'
            page.keyboard.type("zzzzzz")
            page.keyboard.press("Control+Space")

            # Wait a bit for debug print to appear
            time.sleep(1)

            # Get text content of terminal to see debug
            terminal_text = page.locator("#terminal").inner_text()
            print("Terminal content:")
            print(terminal_text)

            os.makedirs("/home/jules/verification", exist_ok=True)
            page.screenshot(path="/home/jules/verification/repl_debug.png")

        except Exception as e:
            print(f"Verification failed: {e}")
            raise e
        finally:
            browser.close()

if __name__ == "__main__":
    verify()
