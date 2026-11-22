import { test, expect } from '@playwright/test';

test('End-to-end asset upload and execution test', async ({ page }) => {
  // 1. Navigate to the /assets page
  console.log('Navigating to /assets');
  await page.goto('/assets');

  // 2. Click "Upload Assets"
  console.log('Clicking Upload Assets');
  await page.getByRole('button', { name: 'Upload Assets' }).click();
  const uploadDialog = page.locator('[role="dialog"]');
  await expect(uploadDialog).toBeVisible();

  // 3. Upload a file named test_script.sh
  console.log('Uploading test_script.sh');
  const fileContent = Buffer.from('#!/bin/sh\necho "E2E Test Passed"\n', 'utf-8');
  await uploadDialog.getByTestId('file-upload-input').setInputFiles({
    name: 'test_script.sh',
    mimeType: 'text/x-sh',
    buffer: fileContent,
  });

  // 4. Click the "Upload" button in the modal and wait for upload request to complete
  console.log('Confirming Upload');
  const uploadResponsePromise = page.waitForResponse(
    (resp) => resp.url().includes('/cdn/upload') && resp.ok()
  );
  await uploadDialog.getByRole('button', { name: 'Upload', exact: true }).click();
  await uploadResponsePromise;

  // 5. Close upload modal and wait for the uploaded asset row to appear in the table
  console.log('Waiting for upload to complete');
  const cancelButton = uploadDialog.getByRole('button', { name: 'Cancel' });
  await expect(cancelButton).toBeEnabled();
  await cancelButton.click();
  await expect(uploadDialog).toBeHidden();

  // 6. Create a Link for the specific asset row
  console.log('Creating Link for asset');
  const assetRow = page.locator('[data-index]').filter({ hasText: 'test_script.sh' }).first();
  await expect(assetRow).toBeVisible();
  const createLinkButton = assetRow.getByRole('button', { name: 'Create Link' });
  await expect(createLinkButton).toBeVisible();
  await createLinkButton.click();

  // 7. Wait for Create Link Modal
  console.log('Waiting for Create Link Modal');
  const createLinkDialog = page.locator('[role="dialog"]');
  await expect(createLinkDialog.getByText('Create link for test_script.sh')).toBeVisible();

  // 8. Submit Create Link Form
  console.log('Submitting Create Link Form');
  await createLinkDialog.getByRole('button', { name: 'Create Link' }).click();

  // 9. Get the generated link
  console.log('Getting generated link');
  await expect(createLinkDialog.getByText('Link Created')).toBeVisible();
  const linkTextElement = createLinkDialog.locator('.break-all.select-all');
  await expect(linkTextElement).toBeVisible();
  const downloadUrl = (await linkTextElement.innerText()).trim();
  expect(() => new URL(downloadUrl)).not.toThrow();
  console.log('Generated Link:', downloadUrl);

  // Close the modal
  await createLinkDialog.getByRole('button', { name: 'Close', exact: true }).click();
  await expect(createLinkDialog).toBeHidden();

  // 10. Navigate to /quests and open Create Quest modal
  console.log('Navigating to /quests');
  await page.goto('/quests');

  // 11. Click "Create a quest" button to open the modal
  console.log('Opening Create Quest modal');
  await page.getByRole('button', { name: 'Create a quest' }).click();
  const questDialog = page.locator('[role="dialog"]');
  await expect(questDialog).toBeVisible();

  // 12. Wait for beacons to load
  console.log('Waiting for beacons to load');
  await expect(questDialog.getByText('Loading beacons...')).toBeHidden({ timeout: 15000 });

  // Select the first beacon via its Chakra Checkbox label to avoid force-clicking the hidden input
  const beaconCheckbox = questDialog.getByLabel(/Select beacon/).first();
  const beaconLabel = questDialog.locator('label').filter({ has: beaconCheckbox }).first();
  await expect(beaconLabel).toBeVisible({ timeout: 15000 });

  // 13. Select the beacon
  console.log('Selecting beacon');
  await beaconLabel.click();
  await expect(beaconCheckbox).toBeChecked();

  // 14. Click Continue (Beacon Step)
  console.log('Clicking Continue (Beacon)');
  await questDialog.locator('[aria-label="continue beacon step"]').click();

  // 15. Select the "Shell command" tome to download and execute the asset and capture its output
  console.log('Selecting Tome');
  await expect(questDialog.getByText('Loading tomes...')).toBeHidden();

  const searchInput = questDialog.getByPlaceholder('Tome name, description & params');
  await searchInput.fill('Shell command');

  const tomeRow = questDialog.locator('[role="button"]').filter({ hasText: 'Shell command' }).first();
  await expect(tomeRow).toBeVisible();
  await tomeRow.click();

  // 16. Fill in the cmd parameter to fetch, chmod, and execute the uploaded asset via its public link
  console.log('Filling parameters');
  const cmdInput = questDialog.locator('#cmd');
  await expect(cmdInput).toBeVisible();
  await cmdInput.fill(
    `tmp=$(mktemp) && trap 'rm -f "$tmp"' EXIT && curl -fsSL '${downloadUrl}' -o "$tmp" && chmod +x "$tmp" && "$tmp"`
  );

  // 17. Click Continue (Tome Step)
  console.log('Clicking Continue (Tome)');
  await questDialog.locator('[aria-label="continue tome step"]').click();

  // 18. Submit Quest
  console.log('Submitting Quest');
  await questDialog.locator('[aria-label="submit quest"]').click();
  await expect(questDialog).toBeHidden({ timeout: 15000 });

  // 19. Wait for polled execution output and verify the uploaded script actually executed
  console.log('Waiting for execution output');
  const outputPanel = page.locator('[aria-label="output panel"] [aria-label="task output"]');
  await expect(outputPanel).toBeVisible({ timeout: 15000 });
  await expect(outputPanel).toContainText('E2E Test Passed', { timeout: 30000 });
  await expect(outputPanel).toContainText('status: 0', { timeout: 30000 });

  console.log('Test Complete');
});
