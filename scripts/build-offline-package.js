#!/usr/bin/env node
/**
 * Cross-platform offline package build script
 * Downloads OpenClaw binaries from GitHub releases for all supported platforms
 *
 * Usage:
 *   node scripts/build-offline-package.js [options]
 *
 * Options:
 *   --version, -v    Specify version to download (default: latest)
 *   --output, -o     Output directory (default: ./offline-packages)
 *   --platform, -p   Download specific platform only
 *   --help, -h       Show help
 */

import fs from 'fs';
import path from 'path';
import https from 'https';
import { fileURLToPath } from 'url';

// Get __dirname equivalent in ES modules
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Supported platforms configuration
const PLATFORMS = [
  { name: 'macos-arm64', ext: 'tar.gz', archiveType: 'tar' },
  { name: 'macos-x64', ext: 'tar.gz', archiveType: 'tar' },
  { name: 'windows-arm64', ext: 'zip', archiveType: 'zip' },
  { name: 'windows-x64', ext: 'zip', archiveType: 'zip' },
  { name: 'linux-arm64', ext: 'tar.gz', archiveType: 'tar' },
  { name: 'linux-x64', ext: 'tar.gz', archiveType: 'tar' },
];

const GITHUB_REPO = 'openclaw-ai/openclaw';
const DEFAULT_OUTPUT_DIR = './offline-packages';

// Colors for terminal output
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  dim: '\x1b[2m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
};

/**
 * Logger utility with color support
 */
class Logger {
  static info(message) {
    console.log(`${colors.blue}[INFO]${colors.reset} ${message}`);
  }

  static success(message) {
    console.log(`${colors.green}[SUCCESS]${colors.reset} ${message}`);
  }

  static warn(message) {
    console.log(`${colors.yellow}[WARN]${colors.reset} ${message}`);
  }

  static error(message) {
    console.log(`${colors.red}[ERROR]${colors.reset} ${message}`);
  }

  static progress(platform, message) {
    console.log(`${colors.cyan}[${platform}]${colors.reset} ${message}`);
  }

  static header(message) {
    console.log(`\n${colors.bright}${colors.cyan}${message}${colors.reset}\n`);
  }
}

/**
 * Parse command line arguments
 */
function parseArgs() {
  const args = process.argv.slice(2);
  const options = {
    version: 'latest',
    output: DEFAULT_OUTPUT_DIR,
    platform: null,
    help: false,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    switch (arg) {
      case '--version':
      case '-v':
        options.version = args[++i];
        break;
      case '--output':
      case '-o':
        options.output = args[++i];
        break;
      case '--platform':
      case '-p':
        options.platform = args[++i];
        break;
      case '--help':
      case '-h':
        options.help = true;
        break;
    }
  }

  return options;
}

/**
 * Show help message
 */
function showHelp() {
  console.log(`
${colors.bright}OpenClaw Offline Package Builder${colors.reset}

Downloads OpenClaw binaries from GitHub releases for offline installation.

${colors.bright}Usage:${colors.reset}
  node scripts/build-offline-package.js [options]

${colors.bright}Options:${colors.reset}
  --version, -v    Specify version to download (default: latest)
  --output, -o     Output directory (default: ./offline-packages)
  --platform, -p   Download specific platform only (e.g., macos-arm64)
  --help, -h       Show this help message

${colors.bright}Supported Platforms:${colors.reset}
  ${PLATFORMS.map((p) => p.name).join('\n  ')}

${colors.bright}Examples:${colors.reset}
  node scripts/build-offline-package.js
  node scripts/build-offline-package.js --version v1.2.0
  node scripts/build-offline-package.js --platform macos-arm64
  node scripts/build-offline-package.js -o ./packages -v v1.0.0
`);
}

/**
 * Ensure directory exists
 */
function ensureDir(dirPath) {
  if (!fs.existsSync(dirPath)) {
    fs.mkdirSync(dirPath, { recursive: true });
    Logger.info(`Created directory: ${dirPath}`);
  }
}

/**
 * Get the download URL for a specific platform and version
 */
function getDownloadUrl(platform, version) {
  const versionTag = version === 'latest' ? 'latest' : version;
  const filename = `openclaw-${platform.name}.${platform.ext}`;

  if (version === 'latest') {
    return `https://github.com/${GITHUB_REPO}/releases/latest/download/${filename}`;
  }
  return `https://github.com/${GITHUB_REPO}/releases/download/${versionTag}/${filename}`;
}

/**
 * Follow redirects and download file
 */
function downloadFile(url, outputPath, platform) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(outputPath);
    let redirected = false;

    const request = https.get(url, { headers: { 'User-Agent': 'OpenClaw-Manager' } }, (response) => {
      // Handle redirects
      if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
        if (!redirected) {
          redirected = true;
          Logger.progress(platform.name, `Following redirect to: ${response.headers.location}`);
          downloadFile(response.headers.location, outputPath, platform)
            .then(resolve)
            .catch(reject);
          return;
        }
      }

      if (response.statusCode !== 200) {
        reject(new Error(`HTTP ${response.statusCode}: ${response.statusMessage}`));
        return;
      }

      const totalSize = parseInt(response.headers['content-length'], 10) || 0;
      let downloadedSize = 0;
      let lastProgress = 0;

      response.on('data', (chunk) => {
        downloadedSize += chunk.length;
        if (totalSize > 0) {
          const progress = Math.round((downloadedSize / totalSize) * 100);
          if (progress - lastProgress >= 10) {
            Logger.progress(platform.name, `Download progress: ${progress}%`);
            lastProgress = progress;
          }
        }
      });

      response.pipe(file);

      file.on('finish', () => {
        file.close();
        resolve();
      });
    });

    request.on('error', (err) => {
      fs.unlink(outputPath, () => {});
      reject(err);
    });

    file.on('error', (err) => {
      fs.unlink(outputPath, () => {});
      reject(err);
    });

    request.setTimeout(30000, () => {
      request.destroy();
      fs.unlink(outputPath, () => {});
      reject(new Error('Download timeout'));
    });
  });
}

/**
 * Download package for a specific platform
 */
async function downloadPlatform(platform, version, outputDir) {
  const url = getDownloadUrl(platform, version);
  const outputPath = path.join(outputDir, `openclaw-${platform.name}.${platform.ext}`);

  Logger.progress(platform.name, `Starting download from: ${url}`);

  try {
    await downloadFile(url, outputPath, platform);

    // Verify file was created and has content
    const stats = fs.statSync(outputPath);
    if (stats.size === 0) {
      throw new Error('Downloaded file is empty');
    }

    const sizeMB = (stats.size / (1024 * 1024)).toFixed(2);
    Logger.success(`${platform.name}: Downloaded ${sizeMB} MB`);

    return {
      platform: platform.name,
      path: outputPath,
      size: stats.size,
      success: true,
    };
  } catch (error) {
    Logger.error(`${platform.name}: ${error.message}`);

    // Clean up failed download
    if (fs.existsSync(outputPath)) {
      fs.unlinkSync(outputPath);
    }

    return {
      platform: platform.name,
      error: error.message,
      success: false,
    };
  }
}

/**
 * Create metadata file for the offline packages
 */
function createMetadata(outputDir, version, results) {
  const metadata = {
    version: version,
    createdAt: new Date().toISOString(),
    githubRepo: GITHUB_REPO,
    packages: results
      .filter((r) => r.success)
      .map((r) => ({
        platform: r.platform,
        filename: path.basename(r.path),
        size: r.size,
        sizeFormatted: `${(r.size / (1024 * 1024)).toFixed(2)} MB`,
      })),
    failed: results.filter((r) => !r.success).map((r) => r.platform),
  };

  const metadataPath = path.join(outputDir, 'metadata.json');
  fs.writeFileSync(metadataPath, JSON.stringify(metadata, null, 2));
  Logger.info(`Metadata saved to: ${metadataPath}`);

  return metadata;
}

/**
 * Main function
 */
async function main() {
  const options = parseArgs();

  if (options.help) {
    showHelp();
    process.exit(0);
  }

  Logger.header('OpenClaw Offline Package Builder');
  Logger.info(`Version: ${options.version}`);
  Logger.info(`Output directory: ${options.output}`);

  // Ensure output directory exists
  ensureDir(options.output);

  // Filter platforms if specific one requested
  const platformsToDownload = options.platform
    ? PLATFORMS.filter((p) => p.name === options.platform)
    : PLATFORMS;

  if (platformsToDownload.length === 0) {
    Logger.error(`Unknown platform: ${options.platform}`);
    Logger.info(`Supported platforms: ${PLATFORMS.map((p) => p.name).join(', ')}`);
    process.exit(1);
  }

  Logger.info(`Platforms to download: ${platformsToDownload.map((p) => p.name).join(', ')}`);
  console.log();

  // Download all platforms
  const results = [];
  for (const platform of platformsToDownload) {
    const result = await downloadPlatform(platform, options.version, options.output);
    results.push(result);
  }

  console.log();
  Logger.header('Build Summary');

  // Create metadata
  const metadata = createMetadata(options.output, options.version, results);

  // Print summary
  const successCount = results.filter((r) => r.success).length;
  const failCount = results.filter((r) => !r.success).length;

  Logger.info(`Total packages: ${results.length}`);
  Logger.success(`Successful: ${successCount}`);
  if (failCount > 0) {
    Logger.error(`Failed: ${failCount}`);
    results
      .filter((r) => !r.success)
      .forEach((r) => Logger.error(`  - ${r.platform}: ${r.error}`));
  }

  const totalSize = results
    .filter((r) => r.success)
    .reduce((sum, r) => sum + r.size, 0);
  Logger.info(`Total size: ${(totalSize / (1024 * 1024)).toFixed(2)} MB`);

  if (successCount === 0) {
    Logger.error('No packages were downloaded successfully');
    process.exit(1);
  }

  Logger.success(`\nOffline packages built successfully in: ${path.resolve(options.output)}`);
}

// Run main function
main().catch((error) => {
  Logger.error(`Unexpected error: ${error.message}`);
  console.error(error);
  process.exit(1);
});
