import { test, expect } from '@playwright/test';

test('End-to-end portal test', async ({ page }) => {
  // Connect to tavern's UI
  console.log('Navigating to /createQuest');
  await page.goto('/createQuest');

  // Select the only visible beacon
  console.log('Waiting for beacons to load');
  await expect(page.getByText('Loading beacons...')).toBeHidden({ timeout: 15000 });
  const beacons = page.locator('.chakra-card input[type="checkbox"]');
  await expect(beacons).toHaveCount(1);
  console.log('Selecting beacon');
  await beacons.first().check({ force: true });
  await page.getByRole('button', { name: 'Continue' }).click();

  // Select the "SOCKS5 Relay" tome
  console.log('Selecting Tome');
  await expect(page.getByText('Loading tomes...')).toBeHidden();
  await page.getByText('SOCKS5 Relay').click();

  console.log('Clicking Continue (Tome)');
  await page.getByRole('button', { name: 'Continue' }).click();

  // Submit
  console.log('Submitting Quest');
  await page.getByRole('button', { name: 'Submit' }).click();

  // Wait for execution and completion
  console.log('Waiting for task completion');

  // We wait for the state to become COMPLETED.
  // We might need to reload to see the update.
  for (let i = 0; i < 20; i++) {
    await page.waitForTimeout(1000);
    // Check if COMPLETED is visible
    if (await page.getByText('State: COMPLETED').isVisible()) {
        console.log('Task completed successfully');
        return;
    }
    console.log('Reloading...');
    await page.reload();
  }

  // Final check
  await expect(page.getByText('State: COMPLETED')).toBeVisible();
});
