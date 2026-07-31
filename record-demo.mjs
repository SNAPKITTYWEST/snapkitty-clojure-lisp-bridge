/**
 * Record demo.html as video using Puppeteer + FFmpeg
 * Usage: node record-demo.mjs
 */

import puppeteer from 'puppeteer';
import { spawn } from 'child_process';
import fs from 'fs';
import path from 'path';

const WIDTH = 1280;
const HEIGHT = 800;
const FPS = 30;
const OUTPUT = 'docs/demo.mp4';

async function recordDemo() {
  console.log('[Record] Starting demo video recording...');

  const browser = await puppeteer.launch({
    headless: true,
    args: ['--no-sandbox', '--disable-setuid-sandbox'],
  });

  const page = await browser.newPage();
  await page.setViewport({ width: WIDTH, height: HEIGHT });

  console.log('[Record] Loading demo.html...');
  await page.goto(`file://${path.resolve('docs/demo.html')}`, {
    waitUntil: 'networkidle2',
  });

  // Set up FFmpeg for video recording
  const ffmpeg = spawn('ffmpeg', [
    '-y',
    '-f', 'gdigrab',
    '-framerate', String(FPS),
    '-i', 'desktop',
    '-c:v', 'libx264',
    '-preset', 'medium',
    '-crf', '23',
    '-pix_fmt', 'yuv420p',
    OUTPUT,
  ]);

  ffmpeg.stderr.on('data', (data) => {
    console.log(`[FFmpeg] ${data}`);
  });

  // Wait for FFmpeg to start
  await new Promise(resolve => setTimeout(resolve, 1000));

  console.log('[Record] Running demo sequence...');

  // Click "Run Full Demo"
  await page.click('button[onclick="runDemo()"]');

  // Wait for demo to complete (~15 seconds)
  await page.waitForTimeout(18000);

  console.log('[Record] Stopping recording...');
  ffmpeg.stdin.write('q');

  await new Promise(resolve => {
    ffmpeg.on('close', resolve);
  });

  await browser.close();

  console.log(`[Record] ✓ Video saved: ${OUTPUT}`);
  console.log(`[Record] Size: ${fs.statSync(OUTPUT).size / 1024 / 1024}MB`);
}

recordDemo().catch(console.error);
