import { test, expect } from '@playwright/test';

test('End-to-end portal provisioning test', async ({ page }) => {
  // Connect to tavern's UI using playwright at http://127.0.0.1:8000/createQuest
  console.log('Navigating to /createQuest');
  await page.goto('/createQuest');

  // Select the only visible beacon and click "continue"
  console.log('Waiting for beacons to load');
  await expect(page.getByText('Loading beacons...')).toBeHidden({ timeout: 15000 });

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

  // 3. Select the "SOCKS5 Relay" tome and click "continue"
  console.log('Selecting Tome');
  await expect(page.getByText('Loading tomes...')).toBeHidden();
  // Use exact match or check if it exists
  await expect(page.getByText('SOCKS5 Relay')).toBeVisible();
  await page.getByText('SOCKS5 Relay').click();

  console.log('Clicking Continue (Tome)');
  await page.getByRole('button', { name: 'Continue' }).click();

  // 4. Select "Submit"
  console.log('Submitting Quest');
  await page.getByRole('button', { name: 'Submit' }).click();

  // 5. Wait for execution.
  // We expect "Portal created" to appear in the output.
  console.log('Waiting for "Portal created" message');
  await expect(page.getByText('Portal created')).toBeVisible({ timeout: 30000 });

  console.log('Portal created successfully');
});
