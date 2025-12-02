from playwright.sync_api import Page, expect, sync_playwright
import time

def verify_repl_suggestions(page: Page):
    # Navigate to the REPL
    page.goto("http://localhost:8080/index.html")

    # Wait for the REPL to initialize and print the welcome message
    expect(page.locator("#terminal")).to_contain_text("stdlib fakes registered")

    # The actual input is a hidden textarea
    input_el = page.locator("#hidden-input")

    # Type 'process' which should have completions like process.list, etc. if we were typing more.
    # But let's just type 'p' and hit tab? Or type 'process.'?
    # The backend logic says: if suggestions not empty, render.
    # Let's type 'p' and hit Tab.

    input_el.focus()
    input_el.type("process.")

    # Trigger completion with Tab
    input_el.press("Tab")

    # Wait for suggestions to appear
    suggestions_el = page.locator("#suggestions")
    expect(suggestions_el).to_be_visible()

    # Check if we have some expected suggestions
    # process.list should be one of them if fake stdlib is loaded.
    expect(suggestions_el).to_contain_text("list")

    # Take a screenshot
    page.screenshot(path="/home/jules/verification/repl_suggestions.png")

    print("Verification successful!")

if __name__ == "__main__":
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page()
        try:
            verify_repl_suggestions(page)
        finally:
            browser.close()
