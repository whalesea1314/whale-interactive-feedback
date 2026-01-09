#!/usr/bin/env node
/**
 * 构建前脚本：自动将版本号注入到窗口标题
 */
import { readFileSync, writeFileSync } from 'fs'
import { resolve, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const configPath = resolve(__dirname, '../src-tauri/tauri.conf.json')

const config = JSON.parse(readFileSync(configPath, 'utf-8'))
const version = config.version

// 更新窗口标题
if (config.app?.windows?.[0]) {
  config.app.windows[0].title = `Interactive Feedback (v${version})`
}

writeFileSync(configPath, JSON.stringify(config, null, 2) + '\n')
console.log(`✅ Window title updated to: Interactive Feedback (v${version})`)
