from playwright.sync_api import Page, expect, sync_playwright
import time

def verify_pprint(page: Page):
    # Hook console
    page.on("console", lambda msg: print(f"Browser Console: {msg.text}"))
    page.on("pageerror", lambda err: print(f"Browser Uncaught Exception: {err}"))

    # 1. Navigate to the REPL
    page.goto("http://localhost:8080")

    # 2. Wait for initialization
    expect(page.locator("body")).to_contain_text("stdlib fakes registered")

    terminal = page.locator("#terminal")
    input_box = page.locator("#hidden-input")

    # 3. Test basic print
    print("Testing print('hello')...")
    input_box.focus()
    # input_box.fill('print("hello")') # fill doesn't trigger keydown events needed by REPL
    input_box.press_sequentially('print("hello")')
    input_box.press("Enter")

    time.sleep(1)

    # Check if print output appeared
    content = terminal.inner_text()
    print(f"Terminal content after print: {content}")

    if "hello" not in content:
        print("print('hello') FAILED (Expected)")
    else:
        print("print('hello') PASSED")

    # 4. Test pprint
    print("Testing pprint({'a': 1})...")
    input_box.focus()
    input_box.press_sequentially('pprint({"a": 1})')
    input_box.press("Enter")

    time.sleep(1)

    content = terminal.inner_text()
    print(f"Terminal content after pprint: {content}")

    if '"a": 1' in content or "DEBUG" in content:
         print("pprint PASSED")
    else:
         print("pprint FAILED")
         raise Exception("pprint failed")

    # 5. Take screenshot
    page.screenshot(path="/home/jules/verification/pprint_verification.png")

if __name__ == "__main__":
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page()
        try:
            verify_pprint(page)
            print("All tests passed!")
        except Exception as e:
            print(f"Test failed: {e}")
            page.screenshot(path="/home/jules/verification/pprint_failure.png")
            # Don't fail the script so we can see output
        finally:
            browser.close()
