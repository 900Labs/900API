import { describe, expect, it } from 'vitest'
import { validateReleaseVersions } from './check-release-version.mjs'

describe('release version validation', () => {
  it('accepts the tag matching every package version', () => {
    expect(validateReleaseVersions('v0.2.0')).toBe('0.2.0')
  })

  it('rejects a tag that does not match repository versions', () => {
    expect(() => validateReleaseVersions('v0.2.1')).toThrow('does not match release tag')
  })

  it('rejects malformed tags', () => {
    expect(() => validateReleaseVersions('release-0.2.0')).toThrow('form vX.Y.Z')
    expect(() => validateReleaseVersions('v01.2.0')).toThrow('not valid semantic versioning')
  })
})
