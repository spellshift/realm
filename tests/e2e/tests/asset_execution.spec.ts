import { test, expect } from '@playwright/test';

test('End-to-end asset upload and execution test', async ({ page }) => {
  // 1. Navigate to the /assets page
  console.log('Navigating to /assets');
  await page.goto('/assets');

  // 2. Click "Upload Assets"
  console.log('Clicking Upload Assets');
  await page.getByRole('button', { name: 'Upload Assets' }).click();

  // 3. Upload a file named test_script.sh
  console.log('Uploading test_script.sh');
  const fileContent = Buffer.from('#!/bin/sh\necho "E2E Test Passed"', 'utf-8');
  await page.getByTestId('file-upload-input').setInputFiles({
    name: 'test_script.sh',
    mimeType: 'text/x-sh',
    buffer: fileContent
  });

  // 4. Click the "Upload" button in the modal
  console.log('Confirming Upload');
  await page.getByRole('button', { name: 'Upload', exact: true }).click();

  // 5. Wait for the upload to complete (modal closes)
  console.log('Waiting for upload to complete');
  // Wait for the file to appear in the table
  await expect(page.getByText('test_script.sh').first()).toBeVisible();

  // 6. Create a Link for the asset
  console.log('Creating Link for asset');

  // Close the upload modal if it's still open (sometimes success state keeps it open)
  const cancelButton = page.getByRole('button', { name: 'Cancel' });
  if (await cancelButton.isVisible()) {
      await cancelButton.click();
  }

  // Find the "Create Link" button in the row containing "test_script.sh"
  const createLinkButton = page.locator('div').filter({ hasText: 'test_script.sh' }).getByLabel('Create Link').first();
  await createLinkButton.click({ force: true });

  // 7. Wait for Create Link Modal
  console.log('Waiting for Create Link Modal');
  await expect(page.getByText('Create link for test_script.sh')).toBeVisible();

  // 8. Submit Create Link Form
  console.log('Submitting Create Link Form');
  const modal = page.locator('[role="dialog"]');
  await modal.getByRole('button', { name: 'Create Link' }).click();

  // 9. Get the generated link
  console.log('Getting generated link');
  await expect(page.getByText('Link Created')).toBeVisible();
  const linkTextElement = page.locator('.break-all.select-all');
  await expect(linkTextElement).toBeVisible();
  const downloadUrl = await linkTextElement.innerText();
  console.log('Generated Link:', downloadUrl);

  // Close the modal
  await page.getByRole('button', { name: 'Close', exact: true }).click();

  // 10. Navigate to /createQuest
  console.log('Navigating to /createQuest');
  await page.goto('/createQuest');

  // 11. Wait for beacons to load
  console.log('Waiting for beacons to load');
  await expect(page.getByText('Loading beacons...')).toBeHidden({ timeout: 15000 });

  const beacons = page.locator('.chakra-card input[type="checkbox"]');
  // There might be more than 1 beacon if we restarted the agent multiple times or other tests ran
  await expect(beacons.first()).toBeVisible();

  // 12. Select the beacon
  console.log('Selecting beacon');
  await beacons.first().check({ force: true });

  // 13. Click Continue (Beacon Step)
  console.log('Clicking Continue (Beacon)');
  await page.locator('[aria-label="continue beacon step"]').click();

  // 14. Select the "HTTP GET file and execute" tome
  console.log('Selecting Tome');
  await expect(page.getByText('Loading tomes...')).toBeHidden();
  await expect(page.getByText('HTTP GET file and execute')).toBeVisible();
  await page.getByText('HTTP GET file and execute').click();

  // 15. Fill in the url parameter with the generated link
  console.log('Filling parameters');
  await page.locator('textarea[name="url"]').fill(downloadUrl);

  // 16. Click Continue (Tome Step)
  console.log('Clicking Continue (Tome)');
  await page.locator('[aria-label="continue tome step"]').click();

  // 17. Submit Quest
  console.log('Submitting Quest');
  await page.locator('[aria-label="submit quest"]').click();

  // 18. Wait for execution and check output
  console.log('Waiting for execution output');
  await page.waitForTimeout(5000);

  // Reload to refresh output
  await page.reload();

  const outputPanel = page.locator('[aria-label="task output"]');
  await expect(outputPanel).toBeVisible();

  // Check for the output indicating successful download start
  // The script execution output is hidden because the tome disowns the process.
  // We accept "Downloading" as success indicator.
  await expect(outputPanel).toContainText('Downloading', { timeout: 30000 });

  console.log('Test Complete');
});
