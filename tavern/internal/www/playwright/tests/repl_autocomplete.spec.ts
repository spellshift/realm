import { test, expect } from '@playwright/test';

test.describe('REPL Autocomplete', () => {
  test('should trigger autocomplete with Tab', async ({ page }) => {
    // Navigate to the REPL
    await page.goto('http://localhost:8080/assets/eldritch-repl/index.html');

    // Wait for prompt
    await expect(page.locator('.xterm-rows')).toContainText('>>>');

    // Type partial global
    await page.keyboard.type('pri');

    // Press Tab
    await page.keyboard.press('Tab');

    // Check if suggestion "print" appears in the output
    await expect(page.locator('.xterm-rows')).toContainText('print');
  });

  test('should cycle suggestions with Down arrow', async ({ page }) => {
    await page.goto('http://localhost:8080/assets/eldritch-repl/index.html');

    // Type 'a' (matches 'all', 'any', 'and', 'abs', etc.)
    await page.keyboard.type('a');
    await page.keyboard.press('Tab');

    // Wait for suggestions to appear
    // Cycle down
    await page.keyboard.press('ArrowDown');

    // Press Enter to accept
    await page.keyboard.press('Enter');

    // The buffer should now contain the selected suggestion.
    // It should replace 'a'.
    const content = await page.locator('.xterm-rows').textContent();
    expect(content).not.toMatch(/>>> a\s*$/);
    expect(content).toMatch(/>>> \w+/);
  });

  test('should complete property access', async ({ page }) => {
    await page.goto('http://localhost:8080/assets/eldritch-repl/index.html');

    // Define an object
    await page.keyboard.type('d = {"key": 1}');
    await page.keyboard.press('Enter');

    // Type 'd.'
    await page.keyboard.type('d.');
    await page.keyboard.press('Tab');

    // Should see 'keys', 'values', etc.
    await expect(page.locator('.xterm-rows')).toContainText('keys');
  });
});
