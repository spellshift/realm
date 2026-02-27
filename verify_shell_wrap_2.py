import time
import json
from playwright.sync_api import sync_playwright

def verify_shell_wrapping(page):
    # Intercept GraphQL queries to mock data
    def handle_graphql(route):
        try:
            # Handle potential JSON parse errors or empty body
            post_data = route.request.post_data_json
        except:
            route.continue_()
            return

        operation_name = post_data.get('operationName')

        if operation_name == 'GetMe':
            route.fulfill(json={
                "data": {
                    "me": {
                        "id": "1",
                        "username": "admin",
                        "isActivated": True
                    }
                }
            })
        elif operation_name == 'GetShell':
            variables = post_data.get('variables', {})
            shell_id = variables.get('id', '1')
            route.fulfill(json={
                "data": {
                    "node": {
                        "__typename": "Shell",
                        "id": shell_id,
                        "host": {"id": "1", "hostname": "test-host"},
                        "closedAt": None,
                    }
                }
            })
        else:
            route.continue_()

    # Enable request interception
    page.route("**/graphql", handle_graphql)

    # Go to the shell page
    page.goto("http://localhost:3000/shellv2/1")

    # Wait for the terminal to appear.
    page.wait_for_selector(".xterm-rows", state="visible", timeout=10000)

    # Generate a long string that will wrap
    # Assuming standard terminal width is around 80-100 chars,
    # but browser window size matters.
    # We'll type enough characters to definitely wrap 2-3 times.
    long_string = "a" * 300

    # Type the long string
    # We type it in chunks to allow redraws to happen
    for i in range(0, len(long_string), 10):
        chunk = long_string[i:i+10]
        page.keyboard.type(chunk)
        # Small delay to mimic fast typing but allow some processing
        time.sleep(0.05)

    # Wait a bit for the final redraw
    time.sleep(2)

    # Take a screenshot to verify the terminal content
    page.screenshot(path="verification_wrap_2.png")
    print("Screenshot saved to verification_wrap_2.png")

with sync_playwright() as p:
    browser = p.chromium.launch()
    page = browser.new_page()
    # Set a specific viewport size to control terminal width somewhat
    page.set_viewport_size({"width": 1024, "height": 768})

    try:
        verify_shell_wrapping(page)
    except Exception as e:
        print(f"Error: {e}")
        try:
            page.screenshot(path="error_wrap.png")
        except:
            pass
    finally:
        browser.close()
