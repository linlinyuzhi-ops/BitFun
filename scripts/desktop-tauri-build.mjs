#!/usr/bin/env node
/** Runs `tauri build` from src/apps/desktop with CI=true. */
import { spawnSync } from 'child_process';
import { fileURLToPath } from 'url';
import { basename, dirname, isAbsolute, join, relative, resolve, sep } from 'path';
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  statSync,
  writeFileSync,
} from 'fs';
import { ensureFlashgrepBinary } from './prepare-flashgrep-resource.mjs';
import { readStageTimings } from './build-stage-timings.mjs';
import { extractProductConfigArg } from './product-customization/cli.mjs';
import { productBuildEnvironment } from './product-customization/projections.mjs';
import { resolveProductDefinition } from './product-customization/resolver.mjs';
import { resolveReleaseChannel } from './release-channel.mjs';
import {
  WEB_FONT_PROFILE_ENV,
  fontProfileForDesktopTarget,
} from './web-font-profile.mjs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');

function tauriBuildArgsFromArgv() {
  const args = process.argv.slice(2);
  // `node script.mjs -- --foo` leaves a leading `--`; strip so `tauri build` sees the same argv as before.
  let i = 0;
  while (i < args.length && args[i] === '--') {
    i += 1;
  }
  return args.slice(i);
}

const SKIP_AUDITS_FLAG = '--skip-audits';
export const SKIP_AUDITS_ENV = 'OPENBITFUN_SKIP_AUDITS';

/** Same accepted values as the frontend pipelines read from the environment. */
function envRequestsSkipAudits(value = process.env[SKIP_AUDITS_ENV]) {
  return ['1', 'true', 'yes'].includes(String(value ?? '').toLowerCase());
}

/**
 * Strips this wrapper's own `--skip-audits` flag out of the forwarded arguments.
 * The flag may appear before or after the `--` cargo separator, so it is removed
 * from every position: neither `tauri build` nor `cargo` knows about it.
 */
export function extractSkipAuditsFlag(args) {
  if (!args.includes(SKIP_AUDITS_FLAG)) {
    return { skipAudits: false, args };
  }
  return { skipAudits: true, args: args.filter((arg) => arg !== SKIP_AUDITS_FLAG) };
}

export function createStageTimer(startedAtMs = Date.now()) {
  const stages = [];
  let cursor = startedAtMs;
  return {
    stages,
    mark(name, now = Date.now()) {
      stages.push({ name, ms: Math.max(0, now - cursor) });
      cursor = now;
    },
    elapsedMs(now = Date.now()) {
      return Math.max(0, now - startedAtMs);
    },
  };
}

function formatStageMs(ms) {
  return `${(ms / 1000).toFixed(1)}s`;
}

function formatStageLine(name, ms, indent) {
  return `[build-timing] ${indent}${name.padEnd(24)} ${formatStageMs(ms)}`;
}

export function reportBuildStages(
  timer,
  { profile, target, skipAudits, frontendStages = null, now = Date.now() }
) {
  const lines = [
    `[build-timing] profile=${profile} target=${target ?? 'host'} skip-audits=${skipAudits ? 'true' : 'false'}`,
  ];
  for (const stage of timer.stages) {
    lines.push(formatStageLine(stage.name, stage.ms, '  '));
  }
  if (frontendStages?.length > 0) {
    lines.push('[build-timing]   frontend stages (inside tauri-build):');
    for (const stage of frontendStages) {
      lines.push(formatStageLine(stage.name, stage.ms, '    '));
    }
  }
  lines.push(formatStageLine('total', timer.elapsedMs(now), '  '));
  console.log(lines.join('\n'));
}

function buildProfileFromTauriArgs(args) {
  if (args.includes('--debug')) return 'debug';
  return optionValue(args, '--profile') || 'release';
}

async function main() {
const buildStartedAtMs = Date.now();
  const timer = createStageTimer(buildStartedAtMs);
  const { productConfig, forwardArgs: argsWithFlag } = extractProductConfigArg(
    tauriBuildArgsFromArgv()
  );
  const { skipAudits: skipAuditsFlag, args: forward } = extractSkipAuditsFlag(argsWithFlag);
  // The environment form is the same switch, so a developer can export it once
  // instead of adding the flag to every packaging command.
  const skipAudits = skipAuditsFlag || envRequestsSkipAudits();
  if (skipAudits) {
    process.env[SKIP_AUDITS_ENV] = '1';
    console.log('[tauri-build] Skipping frontend audits (pure CI gates).');
  }
  const bundleOnly = forward.includes('--bundle-only');
  if (bundleOnly) forward.splice(forward.indexOf('--bundle-only'), 1);
  const resolution = resolveProductDefinition({ rootDir: ROOT, productConfig, member: 'desktop' });
  Object.assign(process.env, productBuildEnvironment(resolution));
  console.log(`[product] ${resolution.assembly.member} ${resolution.assembly.assemblyDigest}`);
  const fontProfile = configureDesktopWebFontProfile(forward);
  console.log(`[font-profile] ${fontProfile}`);
  const releaseChannel = resolveReleaseChannel(process.env.OPENBITFUN_RELEASE_CHANNEL);
  console.log(`[release] channel=${releaseChannel.channel}`);
  timer.mark('product+release');

  const desktopDir = join(ROOT, 'src', 'apps', 'desktop');
if (!bundleOnly) preparePluginHost();
  timer.mark('plugin-host');
  const flashgrepBinary = prepareMacOSFlashgrepForSigning(
    ensureFlashgrepBinary({ target: optionValue(forward, '--target') || rustHostTargetTriple() }),
    desktopDir,
  );
  process.env.FLASHGREP_DAEMON_BIN = flashgrepBinary;
  timer.mark('flashgrep');
  // Tauri CLI reads CI and rejects numeric "1" (common in CI providers).
  process.env.CI = 'true';
  if (process.platform === 'darwin' && requestsDmgBundle(forward)) {
    // Tauri otherwise passes --skip-jenkins under CI, which drops the branded
    // Finder background and icon positions from the generated DMG.
    process.env.TAURI_BUNDLER_DMG_IGNORE_CI = 'true';
  }

  const tauriConfig = prepareTauriConfig(join(desktopDir, 'tauri.conf.json'), {
    desktopDir,
    flashgrepBinary,
    resolution,
    releaseChannel,
  });
  timer.mark('config');
  const tauriBin = join(ROOT, 'node_modules', '.bin', 'tauri');
const tauriArgs = [bundleOnly ? 'bundle' : 'build', '--config', tauriConfig, ...forward];
  const summaryOptions = {
    profile: buildProfileFromTauriArgs(forward),
    target: optionValue(forward, '--target'),
    skipAudits,
  };
  let attemptStartedAtMs = Date.now();
  let r = runTauriBuild(tauriBin, tauriArgs, desktopDir);

  const maxMacDmgBuildAttempts = 3;
  for (
    let attempt = 1;
    attempt < maxMacDmgBuildAttempts
      && !r.error
      && shouldRetryMacDmgBuild(r, forward, desktopDir, attemptStartedAtMs);
    attempt += 1
  ) {
    const retryDelaySeconds = attempt * 10;
    console.warn(
      `[tauri-build] DMG bundling failed after the macOS app bundle was refreshed; retrying build attempt ${attempt + 1}/${maxMacDmgBuildAttempts} in ${retryDelaySeconds} seconds.`
    );
    await new Promise((resolveRetry) => setTimeout(resolveRetry, retryDelaySeconds * 1_000));
    attemptStartedAtMs = Date.now();
    r = runTauriBuild(tauriBin, tauriArgs, desktopDir);
  }

  timer.mark('tauri-build');
  // Tauri runs the frontend pipeline inside the stage above, so its nested costs
  // are only readable once the build command has returned.
  const frontendStages = readStageTimings(
    join(desktopDir, 'gen', 'frontend-timings.json'),
    { notBeforeMs: buildStartedAtMs - 1_000 }
  );

  if (r.error) {
    console.error(r.error);
    reportBuildStages(timer, { ...summaryOptions, frontendStages });
    process.exit(1);
  }

  // Keep only the latest useful Cargo caches for this build profile after tauri build ends.
  try {
    const { profileFromTauriBuildArgs, runGcBestEffort, targetFromTauriBuildArgs } = await import(
      './cargo-target-gc.mjs'
    );
    runGcBestEffort({
      rootDir: ROOT,
      profile: profileFromTauriBuildArgs(forward),
      triple: targetFromTauriBuildArgs(forward),
    });
  } catch (error) {
    console.warn(`[target-gc] skipped: ${error.message || String(error)}`);
  }
  timer.mark('target-gc');

  reportBuildStages(timer, { ...summaryOptions, frontendStages });

  if (r.status === 0 && forward.includes('--no-bundle')) {
    console.warn(
      '[tauri-build] No bundle was produced. The raw desktop executable depends on its adjacent frontend, flashgrep, mobile-web, and resources directories and must not be distributed by itself.'
    );
  }

  process.exit(r.status ?? 1);
}

function rustHostTargetTriple() {
  const result = spawnSync('rustc', ['-vV'], {
    cwd: ROOT,
    encoding: 'utf8',
    shell: false,
    windowsHide: true,
  });
  if (result.error || result.status !== 0) {
    const detail = result.error?.message || result.stderr || `exit status ${result.status}`;
    throw new Error(`Could not determine the Rust host target: ${detail}`);
  }
  const host = String(result.stdout).match(/^host:\s*(\S+)$/m)?.[1];
  if (!host) throw new Error('rustc -vV did not report a host target triple.');
  return host;
}

function preparePluginHost() {
  const result = spawnSync('pnpm', ['run', 'plugin-host:prepare'], {
    cwd: ROOT,
    env: process.env,
    stdio: 'inherit',
    shell: true,
  });
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(`OpenCode extension Host preparation failed with exit code ${result.status}`);
  }
}

function runTauriBuild(tauriBin, args, desktopDir) {
  return spawnSync(tauriBin, args, {
    cwd: desktopDir,
    env: process.env,
    stdio: 'inherit',
    shell: true,
  });
}

export function shouldRetryMacDmgBuild(
  result,
  args,
  desktopDir,
  buildStartedAtMs,
  runtime = {}
) {
  const platform = runtime.platform ?? process.platform;
  const githubActions = runtime.githubActions ?? process.env.GITHUB_ACTIONS;
  if (
    result.status === 0 ||
    platform !== 'darwin' ||
    githubActions !== 'true' ||
    args.includes('--no-bundle') ||
    !requestsDmgBundle(args)
  ) {
    return false;
  }

  const configuredTargetDir = runtime.cargoTargetDir ?? process.env.CARGO_TARGET_DIR;
  const targetDir = configuredTargetDir
    ? isAbsolute(configuredTargetDir)
      ? configuredTargetDir
      : resolve(desktopDir, configuredTargetDir)
    : join(runtime.root ?? ROOT, 'target');
  const target = optionValue(args, '--target');
  const profile = args.includes('--debug') ? 'debug' : optionValue(args, '--profile') || 'release';
  const bundleDir = join(
    targetDir,
    ...(target ? [target] : []),
    profile,
    'bundle',
    'macos'
  );

  const freshAfterMs = buildStartedAtMs - 1_000;
  try {
    return readdirSync(bundleDir, { withFileTypes: true }).some((entry) => {
      if (!entry.isDirectory() || !entry.name.endsWith('.app')) {
        return false;
      }

      const appDir = join(bundleDir, entry.name);
      if (statSync(appDir).mtimeMs >= freshAfterMs) {
        return true;
      }

      // The Rust cache can restore an existing app directory without changing
      // its own mtime. Tauri still refreshes the executable inside it before
      // codesigning, so use that file as the reliable bundling boundary.
      const executableDir = join(appDir, 'Contents', 'MacOS');
      return readdirSync(executableDir, { withFileTypes: true }).some(
        (executable) =>
          executable.isFile()
          && statSync(join(executableDir, executable.name)).mtimeMs >= freshAfterMs
      );
    });
  } catch {
    return false;
  }
}

function requestsDmgBundle(args) {
  const bundles = optionValue(args, '--bundles');
  return bundles === undefined || bundles.split(',').some((bundle) => bundle.trim() === 'dmg');
}

function optionValue(args, option) {
  const inlinePrefix = `${option}=`;
  for (let i = 0; i < args.length; i += 1) {
    if (args[i] === option) {
      return args[i + 1];
    }
    if (args[i].startsWith(inlinePrefix)) {
      return args[i].slice(inlinePrefix.length);
    }
  }
  return undefined;
}

export function configureDesktopWebFontProfile(
  args,
  { env = process.env, platform = process.platform } = {},
) {
  const profile = fontProfileForDesktopTarget({
    target: optionValue(args, '--target'),
    platform,
  });
  env[WEB_FONT_PROFILE_ENV] = profile;
  return profile;
}

export function prepareMacOSFlashgrepForSigning(
  flashgrepBinary,
  desktopDir,
  runtime = {},
) {
  const platform = runtime.platform ?? process.platform;
  const signingIdentity = runtime.signingIdentity ?? process.env.APPLE_SIGNING_IDENTITY;
  if (platform !== 'darwin' || !signingIdentity) {
    return flashgrepBinary;
  }

  const signedDir = join(desktopDir, 'gen', 'signed-resources', 'flashgrep');
  const signedBinary = join(signedDir, basename(flashgrepBinary));
  mkdirSync(signedDir, { recursive: true });
  copyFileSync(flashgrepBinary, signedBinary);
  chmodSync(signedBinary, statSync(signedBinary).mode | 0o111);

  const run = runtime.spawnSync ?? spawnSync;
  const result = run(
    'codesign',
    [
      '--force',
      '--sign',
      signingIdentity,
      '--options',
      'runtime',
      '--timestamp',
      signedBinary,
    ],
    { encoding: 'utf8', shell: false, windowsHide: true },
  );
  if (result.error || result.status !== 0) {
    const detail = result.error?.message || result.stderr || `exit status ${result.status}`;
    throw new Error(`Failed to sign bundled flashgrep binary: ${detail}`);
  }

  console.log(`[tauri-build] Signed bundled flashgrep binary: ${signedBinary}`);
  return signedBinary;
}

// The cloud private key remains in SimplySign; only its certificate selector is
// passed to Tauri. Authenticode runs before Tauri creates updater signatures.
export function configureWindowsSigning(config, env = process.env, platform = process.platform) {
  if (platform !== 'win32' || !env.WINDOWS_CERTIFICATE_THUMBPRINT) return;
  const thumbprint = env.WINDOWS_CERTIFICATE_THUMBPRINT.replace(/\s/g, '').toUpperCase();
  if (!/^[0-9A-F]{40}$/.test(thumbprint)) {
    throw new Error('WINDOWS_CERTIFICATE_THUMBPRINT must be a SHA-1 certificate fingerprint.');
  }
  config.bundle ??= {};
  config.bundle.windows = {
    ...config.bundle.windows,
    certificateThumbprint: thumbprint,
    digestAlgorithm: 'sha256',
    timestampUrl: 'http://time.certum.pl',
    tsp: true,
  };
}

export function prepareTauriConfig(
  baseConfigPath,
  { desktopDir, flashgrepBinary, resolution, releaseChannel }
) {
  const config = JSON.parse(readFileSync(baseConfigPath, 'utf8'));
  if (resolution) {
    const productName =
      resolution.productNames[resolution.assembly.fallbackLocale]
      ?? resolution.productNames[resolution.assembly.defaultLocale];
    config.productName = productName;
    config.mainBinaryName = resolution.assembly.binaryName;
    config.identifier = resolution.assembly.bundleId;
  }
  configureWindowsSigning(config);
  injectTargetFlashgrepResource(config, desktopDir, flashgrepBinary);
  // The DeepSeek bridge is not a compile-time resource: cargo check and
  // desktop:dev must not require packages/dsh-acp/dist-profile. Official
  // packaging injects it here; frontend:build-all (beforeBuildCommand)
  // compiles the profile before Tauri copies resources.
  injectDshProfileResource(config);
  injectExternalFrontendResource(config);

  const release = releaseChannel
    ?? resolveReleaseChannel(process.env.OPENBITFUN_RELEASE_CHANNEL);
  const primaryEndpoint =
    process.env.TAURI_UPDATER_ENDPOINT || release.primaryUpdaterEndpoint;
  const fallbackEndpoint =
    process.env.TAURI_UPDATER_FALLBACK_ENDPOINT || release.fallbackUpdaterEndpoint;
  process.env.OPENBITFUN_RELEASE_CHANNEL = release.channel;
  process.env.OPENBITFUN_UPDATER_PRIMARY_ENDPOINT = primaryEndpoint;
  process.env.OPENBITFUN_UPDATER_FALLBACK_ENDPOINT = fallbackEndpoint;

  const enabled = ['1', 'true', 'yes'].includes(
    String(process.env.OPENBITFUN_ENABLE_UPDATER_ARTIFACTS || '').toLowerCase()
  );

  if (enabled) {
    const pubkey = process.env.TAURI_UPDATER_PUBKEY;
    if (!pubkey) {
      console.error('OPENBITFUN_ENABLE_UPDATER_ARTIFACTS is set, but TAURI_UPDATER_PUBKEY is missing.');
      process.exit(1);
    }
    if (!process.env.TAURI_SIGNING_PRIVATE_KEY) {
      console.error('OPENBITFUN_ENABLE_UPDATER_ARTIFACTS is set, but TAURI_SIGNING_PRIVATE_KEY is missing.');
      process.exit(1);
    }

    // Fallback endpoint used when GitHub is unreachable (not when no update is found).
    // Tauri updater iterates endpoints and only falls through on network/HTTP errors;
    // a 204 (no update) or a successfully parsed manifest stops the loop.
    config.bundle = {
      ...(config.bundle || {}),
      createUpdaterArtifacts: true,
    };
    config.plugins = {
      ...(config.plugins || {}),
      updater: {
        endpoints: [primaryEndpoint, fallbackEndpoint],
        pubkey,
        windows: {
          installMode: 'quiet',
        },
      },
    };
    console.log(
      `[tauri-build] Updater artifacts enabled for ${release.channel}: ${primaryEndpoint} (fallback: ${fallbackEndpoint})`
    );
  }

  const generatedDir = join(desktopDir, 'gen');
  mkdirSync(generatedDir, { recursive: true });
  const generatedConfig = join(
    generatedDir,
    resolution
      ? `tauri.${resolution.assembly.assemblyDigest}.generated.conf.json`
      : 'tauri.generated.conf.json',
  );
  writeFileSync(generatedConfig, `${JSON.stringify(config, null, 2)}\n`, 'utf8');
  return generatedConfig;
}

const DSH_PROFILE_RESOURCE_SOURCE = '../../../packages/dsh-acp/dist-profile';
const DSH_PROFILE_RESOURCE_TARGET = 'resources/dsh-profile';
const EXTERNAL_FRONTEND_RESOURCE_SOURCE = '../../../dist';
const EXTERNAL_FRONTEND_RESOURCE_TARGET = 'frontend/dist';

function injectDshProfileResource(config) {
  const resources = { ...(config.bundle?.resources || {}) };
  resources[DSH_PROFILE_RESOURCE_SOURCE] = DSH_PROFILE_RESOURCE_TARGET;
  config.bundle = {
    ...(config.bundle || {}),
    resources,
  };
}

function injectExternalFrontendResource(config) {
  const resources = { ...(config.bundle?.resources || {}) };
  resources[EXTERNAL_FRONTEND_RESOURCE_SOURCE] = EXTERNAL_FRONTEND_RESOURCE_TARGET;
  config.bundle = {
    ...(config.bundle || {}),
    resources,
  };
}

function injectTargetFlashgrepResource(config, desktopDir, flashgrepBinary) {
  const resources = { ...(config.bundle?.resources || {}) };
  delete resources['../../../resources/flashgrep'];

  for (const binaryPath of bundledFlashgrepResources(flashgrepBinary)) {
    const source = toTauriPath(relative(desktopDir, binaryPath));
    resources[source] = `flashgrep/${basename(binaryPath)}`;
  }
  config.bundle = {
    ...(config.bundle || {}),
    resources,
  };
}

function bundledFlashgrepResources(primaryBinary) {
  return primaryBinary ? [primaryBinary] : [];
}

function toTauriPath(value) {
  return value.split(sep).join('/');
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((e) => {
    console.error(e);
    process.exit(1);
  });
}
