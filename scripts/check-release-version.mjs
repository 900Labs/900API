import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const SEMVER = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/

function readJsonVersion(root, relativePath) {
  const parsed = JSON.parse(readFileSync(resolve(root, relativePath), 'utf8'))
  if (typeof parsed.version !== 'string') {
    throw new Error(`${relativePath} does not contain a string version`)
  }
  return parsed.version
}

function readPackageLockVersion(root) {
  const relativePath = 'package-lock.json'
  const parsed = JSON.parse(readFileSync(resolve(root, relativePath), 'utf8'))
  const rootVersion = parsed.packages?.['']?.version
  if (typeof parsed.version !== 'string' || typeof rootVersion !== 'string') {
    throw new Error(`${relativePath} does not contain package versions`)
  }
  if (parsed.version !== rootVersion) {
    throw new Error(`${relativePath} package versions do not agree`)
  }
  return parsed.version
}

function readCargoVersion(root, relativePath) {
  const content = readFileSync(resolve(root, relativePath), 'utf8')
  const match = content.match(/^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m)
  if (!match) throw new Error(`${relativePath} does not contain a package version`)
  return match[1]
}

function readCargoLockVersion(root, packageName) {
  const relativePath = 'Cargo.lock'
  const content = readFileSync(resolve(root, relativePath), 'utf8')
  const packages = content.split('[[package]]').slice(1)
  for (const entry of packages) {
    const name = entry.match(/^\s*name\s*=\s*"([^"]+)"/m)?.[1]
    if (name !== packageName) continue
    const version = entry.match(/^\s*version\s*=\s*"([^"]+)"/m)?.[1]
    if (version) return version
  }
  throw new Error(`${relativePath} does not contain ${packageName}`)
}

export function validateReleaseVersions(tag, root = process.cwd()) {
  if (typeof tag !== 'string' || !tag.startsWith('v')) {
    throw new Error(`Release tag must use the form vX.Y.Z, received: ${tag || '<empty>'}`)
  }

  const tagVersion = tag.slice(1)
  if (!SEMVER.test(tagVersion)) {
    throw new Error(`Release tag is not valid semantic versioning: ${tag}`)
  }

  const versions = new Map([
    ['package.json', readJsonVersion(root, 'package.json')],
    ['package-lock.json', readPackageLockVersion(root)],
    ['src-tauri/tauri.conf.json', readJsonVersion(root, 'src-tauri/tauri.conf.json')],
    ['src-tauri/Cargo.toml', readCargoVersion(root, 'src-tauri/Cargo.toml')],
    ['crates/900api-cli/Cargo.toml', readCargoVersion(root, 'crates/900api-cli/Cargo.toml')],
    ['crates/900api-core/Cargo.toml', readCargoVersion(root, 'crates/900api-core/Cargo.toml')],
    ['Cargo.lock (api900)', readCargoLockVersion(root, 'api900')],
    ['Cargo.lock (api900-cli)', readCargoLockVersion(root, 'api900-cli')],
    ['Cargo.lock (api900-core)', readCargoLockVersion(root, 'api900-core')],
  ])

  for (const [file, version] of versions) {
    if (version !== tagVersion) {
      throw new Error(`${file} version ${version} does not match release tag ${tag}`)
    }
  }

  return tagVersion
}

const scriptPath = process.argv[1] ? pathToFileURL(resolve(process.argv[1])).href : ''
if (import.meta.url === scriptPath) {
  try {
    const version = validateReleaseVersions(process.argv[2] || process.env.GITHUB_REF_NAME)
    console.log(`Release tag and package versions agree on ${version}`)
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error))
    process.exit(1)
  }
}
