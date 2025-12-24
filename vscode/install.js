const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const projectRoot = path.resolve(__dirname, '..');
const lspCratePath = path.join(projectRoot, 'implants', 'lib', 'eldritchv2', 'eldritch-lsp');
// Workspace target dir is implants/target because implants/Cargo.toml defines the workspace
const targetPath = path.join(projectRoot, 'implants', 'target', 'release', 'eldritch-lsp');
const destDir = path.join(__dirname, 'bin');
const destPath = path.join(destDir, process.platform === 'win32' ? 'eldritch-lsp.exe' : 'eldritch-lsp');

console.log('Building eldritch-lsp...');
try {
    // We use --manifest-path. Cargo usually outputs to the workspace target dir unless --target-dir is specified.
    // Since implants/ is the workspace, output is implants/target.
    execSync('cargo build --release --manifest-path ' + path.join(lspCratePath, 'Cargo.toml'), { stdio: 'inherit' });
} catch (e) {
    console.error('Build failed:', e);
    process.exit(1);
}

if (!fs.existsSync(destDir)) {
    fs.mkdirSync(destDir);
}

console.log(`Copying binary to ${destPath}...`);
// Find the binary in target/release (it might have .exe extension on windows)
const srcPath = process.platform === 'win32' ? targetPath + '.exe' : targetPath;

if (fs.existsSync(srcPath)) {
    fs.copyFileSync(srcPath, destPath);
    console.log('Done.');
} else {
    console.error(`Binary not found at ${srcPath}`);
    process.exit(1);
}
