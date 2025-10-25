const fs = require('fs');
const { build } = require('esbuild');

// Read the HTML file content
let htmlContent = fs.readFileSync('src/ui.html', 'utf8');

// Read the TypeScript code
let code = fs.readFileSync('src/code.ts', 'utf8');

// Replace __html__ with the actual HTML content
code = code.replace(/__html__/g, JSON.stringify(htmlContent));

// Write temporary file
fs.writeFileSync('src/code.temp.ts', code);

// Build with esbuild
build({
  entryPoints: ['src/code.temp.ts'],
  bundle: true,
  outfile: 'dist/code.js',
  format: 'iife',
  target: 'es2019', // Compatible with Figma plugin environment
  tsconfig: 'tsconfig.json',
}).then(() => {
  // Clean up temporary file
  fs.unlinkSync('src/code.temp.ts');
  console.log('Build complete!');
}).catch((error) => {
  // Clean up temporary file even on error
  if (fs.existsSync('src/code.temp.ts')) {
    fs.unlinkSync('src/code.temp.ts');
  }
  console.error('Build failed:', error);
  process.exit(1);
});
