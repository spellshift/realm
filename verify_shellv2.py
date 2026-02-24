import time
from playwright.sync_api import sync_playwright
import datetime

def run(playwright):
    browser = playwright.chromium.launch(headless=True)
    page = browser.new_page()

    # Intercept GraphQL requests
    def handle_route(route):
        request = route.request
        if "/graphql" in request.url:
            post_data = request.post_data_json
            if not post_data:
                route.continue_()
                return

            operation_name = post_data.get("operationName")
            print(f"Request: {operation_name}")

            if operation_name == "GetShell":
                print("Mocking GetShell")
                route.fulfill(json={
                    "data": {
                        "node": {
                            "__typename": "Shell",
                            "id": "123",
                            "closedAt": None,
                            "owner": {"id": "user1", "name": "Admin"},
                            "beacon": {
                                "id": "beacon1",
                                "name": "test-beacon",
                                "host": {
                                    "id": "host1",
                                    "name": "test-host"
                                }
                            },
                            "portals": []
                        }
                    }
                })
            elif operation_name == "GetBeaconStatus":
                print("Mocking GetBeaconStatus")
                # Calculate nextSeenAt to be 100 seconds in the future
                future_time = (datetime.datetime.utcnow() + datetime.timedelta(seconds=100)).isoformat() + "Z"

                route.fulfill(json={
                    "data": {
                        "node": {
                            "__typename": "Beacon",
                            "lastSeenAt": "2023-01-01T00:00:00Z",
                            "nextSeenAt": future_time,
                            "interval": 60
                        }
                    }
                })
            elif operation_name == "GetPortalStatus":
                print("Mocking GetPortalStatus")
                route.fulfill(json={
                    "data": {
                        "node": {
                            "__typename": "Portal",
                            "closedAt": None
                        }
                    }
                })
            elif operation_name == "GetMe":
                print("Mocking GetMe")
                route.fulfill(json={
                    "data": {
                        "me": {
                            "__typename": "User",
                            "id": "user1",
                            "name": "Admin",
                            "photoURL": "",
                            "isActivated": True,
                            "isAdmin": True
                        }
                    }
                })
            elif operation_name == "GetSearchFilters":
                print("Mocking GetSearchFilters")
                route.fulfill(json={
                    "data": {
                        "groupTags": {"edges": []},
                        "serviceTags": {"edges": []},
                        "beacons": {"edges": []},
                        "hosts": {"edges": []}
                    }
                })
            else:
                route.continue_()
        else:
            route.continue_()

    page.route("**/graphql", handle_route)

    # Set local storage token if needed (dummy)
    page.add_init_script("""
        localStorage.setItem("authToken", "dummy-token");
    """)

    print("Navigating to shellv2...")
    try:
        page.goto("http://localhost:3000/shellv2/123", timeout=60000)
    except Exception as e:
        print(f"Error navigating: {e}")
        browser.close()
        return

    print("Waiting for content...")
    try:
        # Wait for the specific elements we added
        page.wait_for_selector("text=BETA", timeout=30000)
        print("BETA Found")
        page.wait_for_selector("text=test-beacon @ test-host", timeout=30000)
        print("Title Found")

        # Check for link
        link = page.locator("a[href='/hosts/host1']")
        if link.count() > 0:
            print("Link found")
        else:
            print("Link NOT found")

        # Check for bug icon
        bug_link = page.locator("a[title='Report a bug']")
        if bug_link.count() > 0:
            print("Bug link found")
        else:
            print("Bug link NOT found")

        # Check timer text (in 1 min 40s approx)
        # We mocked 100 seconds = 1 min 40s
        # Note: The logic might produce "1 min 40s" or "1 min 39s" depending on execution time
        page.wait_for_selector("text=in 1 min", timeout=10000)
        print("Timer text found")

    except Exception as e:
        print(f"Timeout or error waiting for selector: {e}")
        page.screenshot(path="timeout.png")

    # Take screenshot
    page.screenshot(path="shellv2_verification.png")
    print("Screenshot saved to shellv2_verification.png")

    browser.close()

with sync_playwright() as playwright:
    run(playwright)
