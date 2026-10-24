const fs = require('fs');
const path = require('path');

function create24BitBMP(width, height, drawFn) {
  const rowSize = Math.floor((24 * width + 31) / 32) * 4;
  const pixelArraySize = rowSize * height;
  const fileSize = 54 + pixelArraySize;
  const buffer = Buffer.alloc(fileSize);

  // Bitmap File Header
  buffer.write('BM', 0, 2, 'ascii');
  buffer.writeUInt32LE(fileSize, 2);
  buffer.writeUInt32LE(0, 6);
  buffer.writeUInt32LE(54, 10); // Offset to pixel data

  // DIB Header (BITMAPINFOHEADER)
  buffer.writeUInt32LE(40, 14); // Header size
  buffer.writeInt32LE(width, 18);
  buffer.writeInt32LE(height, 22); // Bottom-up
  buffer.writeUInt16LE(1, 26); // Planes
  buffer.writeUInt16LE(24, 28); // 24 bpp
  buffer.writeUInt32LE(0, 30); // BI_RGB (uncompressed)
  buffer.writeUInt32LE(pixelArraySize, 34);
  buffer.writeInt32LE(2835, 38); // ~72 DPI
  buffer.writeInt32LE(2835, 42);
  buffer.writeUInt32LE(0, 46);
  buffer.writeUInt32LE(0, 50);

  // Draw pixels (bottom-up)
  for (let y = 0; y < height; y++) {
    const rowOffset = 54 + y * rowSize;
    for (let x = 0; x < width; x++) {
      // Coordinate from top to bottom
      const topY = height - 1 - y;
      const [r, g, b] = drawFn(x, topY, width, height);
      const pixelOffset = rowOffset + x * 3;
      buffer[pixelOffset] = b;
      buffer[pixelOffset + 1] = g;
      buffer[pixelOffset + 2] = r;
    }
  }

  return buffer;
}

const installerDir = path.join(__dirname, '../installer');
if (!fs.existsSync(installerDir)) fs.mkdirSync(installerDir, { recursive: true });

// 1. Generate Sidebar BMP (164 x 314)
const sidebarBuffer = create24BitBMP(164, 314, (x, y, w, h) => {
  // Cyan vertical accent line on right edge
  if (x >= w - 3) return [0, 200, 255];

  // Hexagon logo around (82, 60)
  const cx = 82;
  const cy = 60;
  const dx = Math.abs(x - cx);
  const dy = Math.abs(y - cy);
  const dist = Math.sqrt(dx * dx + dy * dy);

  // Outer hexagon wireframe
  if (dist >= 26 && dist <= 29) {
    return [0, 210, 255];
  }
  // Inner diamond core
  if (dist <= 14) {
    return [0, 210, 255];
  }
  // Glow around logo
  if (dist < 45) {
    const glow = (45 - dist) / 45;
    const r = Math.round(6 + glow * 25);
    const g = Math.round(11 + glow * 80);
    const b = Math.round(20 + glow * 110);
    return [r, g, b];
  }

  // Dark obsidian gradient background with subtle blue undertone
  const ratio = y / h;
  const r = Math.round(6 + ratio * 8);
  const g = Math.round(11 + ratio * 14);
  const b = Math.round(20 + ratio * 24);

  // Subtle tech divider lines
  if (y === 115 || y === 116 || y === 270 || y === 271) {
    return [15, 35, 60];
  }

  return [r, g, b];
});

fs.writeFileSync(path.join(installerDir, 'installerSidebar.bmp'), sidebarBuffer);

// 2. Generate Header BMP (150 x 57)
const headerBuffer = create24BitBMP(150, 57, (x, y, w, h) => {
  // Logo on right side around (125, 28)
  const cx = 125;
  const cy = 28;
  const dx = Math.abs(x - cx);
  const dy = Math.abs(y - cy);
  const dist = Math.sqrt(dx * dx + dy * dy);

  if (dist >= 14 && dist <= 16) {
    return [0, 210, 255];
  }
  if (dist <= 7) {
    return [0, 210, 255];
  }

  // Dark obsidian background
  const r = 8;
  const g = 13;
  const b = 24;
  return [r, g, b];
});

fs.writeFileSync(path.join(installerDir, 'installerHeader.bmp'), headerBuffer);

console.log('Successfully generated installerSidebar.bmp and installerHeader.bmp!');
