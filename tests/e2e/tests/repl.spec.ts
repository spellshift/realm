import { test, expect } from '@playwright/test';

test('End-to-end reverse shell repl test', async ({ page }) => {
  // Connect to tavern's UI using playwright at http://127.0.0.1:8000/createQuest
  console.log('Navigating to /createQuest');
  await page.goto('/createQuest');

  // Select the only visible beacon and click "continue"
  console.log('Waiting for beacons to load');
  await expect(page.getByText('Loading beacons...')).toBeHidden({ timeout: 15000 });

  // Select the checkbox. Using force: true because Chakra UI hides the actual input.
  // Define the locator for the beacon checkboxes
  const beacons = page.locator('.chakra-card input[type="checkbox"]');

  // Assert that exactly one beacon exists
  await expect(beacons).toHaveCount(1);

  // Select the beacon
  console.log('Selecting beacon');
  await beacons.first().check({ force: true });

  // Click Continue
  console.log('Clicking Continue (Beacon)');
  await page.getByRole('button', { name: 'Continue' }).click();

  // 3. Select the "Reverse Shell REPL" tome and click "continue"
  console.log('Selecting Tome');
  await expect(page.getByText('Loading tomes...')).toBeHidden();
  await page.getByText('Reverse Shell REPL').click();

  console.log('Clicking Continue (Tome)');
  await page.getByRole('button', { name: 'Continue' }).click();

  // 4. Select "Submit"
  console.log('Submitting Quest');
  await page.getByRole('button', { name: 'Submit' }).click();

  // 5. Wait at least 11 seconds for agent execution
  console.log('Waiting 12s for execution');
  await page.waitForTimeout(12000);

  // 6. Refresh the page
  console.log('Reloading page');
  await page.reload();

  // 7. See a "Shells" tab in the output, select it
  console.log('Clicking Shells tab');
  await page.getByRole('tab', { name: 'Shells' }).click();

  // 8. Select "Join shell instance"
  console.log('Joining shell instance');
  await page.getByRole('button', { name: 'Join shell instance' }).click();

  // 9. Perform tests to ensure it functions as expected
  console.log('Verifying shell connection');

  // Wait for terminal to be visible
  await expect(page.locator('#terminal')).toBeVisible({ timeout: 15000 });

  // Focus the terminal (clicking it helps ensure focus)
  await page.locator('#terminal').click();

  console.log('Sending command');
  // Type something.
  await page.waitForTimeout(1000); // Wait a bit for connection to be fully established
  await page.keyboard.type('print("Hello E2E")');
  await page.keyboard.press('Enter');

  // Verify output.
  console.log('Verifying output');
  // xterm rows usually contain the text.
  // We expect the command echoed, output, and a new prompt.
  // The prompt might contain trailing spaces or non-breaking spaces depending on rendering.
  await expect(page.locator('.xterm-rows')).toContainText('Hello E2E');
  await expect(page.locator('.xterm-rows')).toContainText('>>>');

  console.log('Test Complete');
});
