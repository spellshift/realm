
from playwright.sync_api import Page, expect, sync_playwright
import time

def verify_assets_page(page: Page):
    # 1. Arrange: Go to the Assets page.
    # Assuming local dev server runs on port 3000
    page.goto("http://localhost:3000/assets")

    # Wait for the page to load
    page.wait_for_timeout(5000)

    # 2. Check for Creator column
    # We expect to see "Creator" header
    expect(page.get_by_role("columnheader", name="Creator")).to_be_visible()

    # 3. Check for responsive hiding (simulate mobile by setting viewport)
    page.set_viewport_size({"width": 375, "height": 667})
    # Wait for layout adjustment
    page.wait_for_timeout(1000)

    # "Hash" and "Size" headers should NOT be visible on mobile
    expect(page.get_by_role("columnheader", name="Hash")).not_to_be_visible()
    expect(page.get_by_role("columnheader", name="Size")).not_to_be_visible()

    # "Creator" and "Name" SHOULD be visible
    expect(page.get_by_role("columnheader", name="Name")).to_be_visible()
    expect(page.get_by_role("columnheader", name="Creator")).to_be_visible()

    # Reset viewport
    page.set_viewport_size({"width": 1280, "height": 720})
    page.wait_for_timeout(1000)

    # 4. Open Create Link Modal
    # Click the first "Create Link" button (assuming there's at least one asset)
    # The button has an aria-label "Create Link"
    create_buttons = page.get_by_label("Create Link")
    if create_buttons.count() > 0:
        create_buttons.first.click()

        # 5. Verify "Limit Downloads" checkbox
        expect(page.get_by_text("Limit Downloads")).to_be_visible()

        # Checkbox should be unchecked by default (based on my code)
        # Note: Chakra UI checkbox structure might be complex, checking label visibility is a good start.
        # Let's try to find the input
        limit_input = page.locator("input[type='number']")
        # Should not be visible initially
        expect(limit_input).not_to_be_visible()

        # Click the checkbox
        page.locator("label:has-text('Limit Downloads')").click()

        # Now input should be visible with value 1
        expect(limit_input).to_be_visible()
        expect(limit_input).to_have_value("1")

        # Close modal
        page.get_by_role("button", name="Cancel").click()

    # 6. Check Asset Accordion (Expand a row)
    # We need a row with links. If we can't guarantee data, we assume there might be one.
    # We'll try to expand the first row if it has links.
    # My code adds a check: only render chevron if links > 0.
    # So we look for a chevron-down or chevron-right.

    # Take a screenshot of the main table first
    page.screenshot(path="/home/jules/verification/assets_page.png")

if __name__ == "__main__":
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page()
        try:
            verify_assets_page(page)
        except Exception as e:
            print(f"Error: {e}")
            page.screenshot(path="/home/jules/verification/error.png")
        finally:
            browser.close()
