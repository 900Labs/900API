import { readdirSync, statSync } from 'node:fs'
import { resolve } from 'node:path'
import { pathToFileURL } from 'node:url'

const SEMVER = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/

function normalizeVersion(expectedVersion) {
  const version = expectedVersion?.startsWith('v') ? expectedVersion.slice(1) : expectedVersion
  if (typeof version !== 'string' || !SEMVER.test(version)) {
    throw new Error(`Expected version must be semantic versioning, received: ${expectedVersion || '<empty>'}`)
  }
  return version
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

function requiredArtifacts(version) {
  const escapedVersion = escapeRegExp(version)
  return [
    {
      name: 'macOS arm64 DMG',
      pattern: new RegExp(`^900API_${escapedVersion}_aarch64\\.dmg$`),
    },
    {
      name: 'macOS Intel DMG',
      pattern: new RegExp(`^900API_${escapedVersion}_x64\\.dmg$`),
    },
    {
      name: 'Linux AppImage',
      pattern: new RegExp(`^900API_${escapedVersion}_amd64\\.AppImage$`),
    },
    {
      name: 'Debian package',
      pattern: new RegExp(`^900API_${escapedVersion}_amd64\\.deb$`),
    },
    {
      name: 'RPM package',
      pattern: new RegExp(`^900API-${escapedVersion}-[0-9A-Za-z][0-9A-Za-z._+~]*\\.x86_64\\.rpm$`),
    },
    {
      name: 'Windows MSI',
      pattern: new RegExp(
        `^900API_${escapedVersion}_x64_[A-Za-z]{2,3}(?:-[A-Za-z0-9]{2,8})+\\.msi$`,
      ),
    },
    {
      name: 'Windows setup executable',
      pattern: new RegExp(`^900API_${escapedVersion}_x64-setup\\.exe$`),
    },
  ]
}

export function validateReleaseAssets(directory, expectedVersion) {
  const version = normalizeVersion(expectedVersion)
  const root = resolve(directory)
  const files = readdirSync(root, { withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => entry.name)
    .sort()

  const validated = []
  const allowedFiles = new Set(['SHA256SUMS.txt'])

  for (const requirement of requiredArtifacts(version)) {
    const candidates = files.filter((file) => requirement.pattern.test(file))
    if (candidates.length === 0) {
      throw new Error(`Missing required release artifact: ${requirement.name}`)
    }
    if (candidates.length > 1) {
      throw new Error(`Duplicate required release artifact: ${requirement.name}: ${candidates.join(', ')}`)
    }

    const [file] = candidates
    if (statSync(resolve(root, file)).size === 0) {
      throw new Error(`${requirement.name} artifact is empty: ${file}`)
    }

    allowedFiles.add(file)
    validated.push(file)
  }

  const optionalArchives = [
    `900API_${version}_aarch64.app.tar.gz`,
    `900API_${version}_x64.app.tar.gz`,
  ]
  const includedArchives = optionalArchives.filter((file) => files.includes(file))
  for (const file of includedArchives) {
    if (statSync(resolve(root, file)).size === 0) {
      throw new Error(`Optional app archive is empty: ${file}`)
    }
    allowedFiles.add(file)
  }

  const unexpected = files.filter((file) => !allowedFiles.has(file))
  if (unexpected.length > 0) {
    throw new Error(`Unexpected release asset: ${unexpected.join(', ')}`)
  }

  return { version, validated, optionalArchives: includedArchives }
}

const scriptPath = process.argv[1] ? pathToFileURL(resolve(process.argv[1])).href : ''
if (import.meta.url === scriptPath) {
  try {
    const directory = process.argv[2]
    if (!directory) throw new Error('Usage: npm run validate:release-assets -- <directory> <version>')
    const result = validateReleaseAssets(directory, process.argv[3])
    console.log(`Validated ${result.validated.length} required release artifacts for ${result.version}`)
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error))
    process.exit(1)
  }
}
