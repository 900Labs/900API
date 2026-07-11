import { execFileSync } from 'node:child_process'
import { existsSync, readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'

const files = execFileSync('git', ['ls-files', '*.md'], { encoding: 'utf8' })
  .trim()
  .split('\n')
  .filter(Boolean)
  .filter((file) => existsSync(file))

const failures = []
const linkPattern = /\[[^\]]*\]\(([^)]+)\)/g

for (const file of files) {
  const content = readFileSync(file, 'utf8')
  for (const match of content.matchAll(linkPattern)) {
    const rawTarget = match[1].trim().replace(/^<|>$/g, '')
    if (!rawTarget || /^(https?:|mailto:|#)/.test(rawTarget)) continue

    const target = decodeURIComponent(rawTarget.split('#')[0])
    if (!target) continue
    const absolute = resolve(dirname(file), target)
    if (!existsSync(absolute)) {
      const line = content.slice(0, match.index).split('\n').length
      failures.push(`${file}:${line}: missing local link target ${rawTarget}`)
    }
  }
}

if (failures.length > 0) {
  console.error(failures.join('\n'))
  process.exit(1)
}

console.log(`Checked local links in ${files.length} Markdown files`)
