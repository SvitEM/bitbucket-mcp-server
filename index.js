#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const binaryPath = path.join(__dirname, 'target', 'release', 'bitbucket-mcp');
const usePrebuiltBinary = fs.existsSync(binaryPath);

const spawnConfig = usePrebuiltBinary
  ? { cmd: binaryPath, args: [], options: { stdio: ['inherit', 'inherit', 'inherit'] } }
  : { cmd: 'cargo', args: ['run', '--release'], options: { cwd: __dirname, stdio: ['inherit', 'inherit', 'inherit'] } };

const child = spawn(spawnConfig.cmd, spawnConfig.args, spawnConfig.options);

child.on('exit', (code) => {
  process.exit(code || 0);
});

child.on('error', (err) => {
  console.error('Failed to start Bitbucket MCP server:', err);
  if (!usePrebuiltBinary) {
    console.error('Make sure Rust and Cargo are installed.');
  }
  process.exit(1);
});
