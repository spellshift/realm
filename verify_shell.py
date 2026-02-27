import time
import json
from playwright.sync_api import sync_playwright

def verify_shell(page):
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
            print("Intercepted GetMe")
            route.fulfill(json={
                "data": {
                    "me": {
                        "id": "1",
                        "username": "admin",
                        "isActivated": True  # Add isActivated based on memory
                    }
                }
            })
        elif operation_name == 'GetShell':
            print("Intercepted GetShell")
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
        elif operation_name == 'GetBeaconStatus': # Mock Beacon Status if needed
             print("Intercepted GetBeaconStatus")
             route.fulfill(json={
                "data": {
                     "node": {
                         "lastSeenAt": "2024-01-01T00:00:00Z" # Old enough/recent enough? Logic might depend.
                     }
                }
             })
        else:
            route.continue_()

    # Enable request interception
    page.route("**/graphql", handle_graphql)

    # Go to the shell page
    # Using a high ID to avoid conflicts if needed, but 1 is fine with mocks
    page.goto("http://localhost:3000/shellv2/1")

    # Wait for the terminal to appear.
    # If the access gate is passed, we should see the shell UI.
    try:
        page.wait_for_selector(".xterm-rows", state="visible", timeout=10000)
    except Exception as e:
        print("Timeout waiting for .xterm-rows. Dumping page content...")
        # print(page.content())
        raise e

    # Simulate typing 'echo hello' quickly
    # This will trigger the fast-path logic
    page.keyboard.type("echo hello")

    # Wait a bit for the debounced redraw to potentially happen
    time.sleep(1)

    # Take a screenshot to verify the terminal content
    page.screenshot(path="verification.png")
    print("Screenshot saved to verification.png")

with sync_playwright() as p:
    browser = p.chromium.launch()
    page = browser.new_page()
    try:
        verify_shell(page)
    except Exception as e:
        print(f"Error: {e}")
        # Take a screenshot even on error if possible
        try:
            page.screenshot(path="error_retry.png")
        except:
            pass
    finally:
        browser.close()
