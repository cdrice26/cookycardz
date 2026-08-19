import { execSync } from 'child_process';
import fs from 'fs';

// 1. generate-license-file (auto-accept prompt)
execSync(
  'generate-license-file --input package.json --output ThirdPartyNotices.txt',
  {
    input: 'yes\n',
    stdio: ['pipe', 'inherit', 'inherit']
  }
);

// 2. cargo about generate, append to file
const cargoOutputPath = 'ThirdPartyNotices.cargo.txt';
try {
  execSync(
    `cargo about generate -m packages/groceryify/Cargo.toml --target wasm32-unknown-unknown packages/groceryify/about.hbs --features wasm -o ${cargoOutputPath}`,
    { stdio: 'inherit' }
  );
  fs.appendFileSync(
    'ThirdPartyNotices.txt',
    fs.readFileSync(cargoOutputPath, 'utf8')
  );
} finally {
  if (fs.existsSync(cargoOutputPath)) {
    fs.unlinkSync(cargoOutputPath);
  }
}

// 3. append wordnet license
const out = 'ThirdPartyNotices.txt';
const a = fs.readFileSync(out, 'utf8');
const b = fs.readFileSync(
  'packages/groceryify/resources/nounexc_license',
  'utf8'
);

// 4. append Ionicons license
const c = fs.readFileSync('apps/web/IoniconsLicense', 'utf8');
fs.writeFileSync(out, a + b + c);
