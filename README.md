# CookyCardz (formerly ChefDeck)

CookyCardz is a recipe manager tool. It allows you to manage your recipes, organize them with tags, schedule them, and generate grocery lists. It is both a cloud-based web app and a desktop application with optional cloud syncing.

## Status
The web app should be fairly stable. The desktop app also works well, though the cloud syncing remains brittle and sometimes reorders and/or duplicates recipes, so that feature is still a work in progress. Note that the desktop app does not yet compile on ARM windows due to dependencies that don't support the architecture.

## Quick repo summary

- CookyCardz uses Turborepo.
- The Next.js web application is at: `apps/web`
- The desktop Tauri app is at: `apps/desktop`
- Shared code is at: `packages/shared`
- The grocery list generator code is at: `packages/groceryify`

## Prerequisites

- Node.js and `npm`
    - You'll also need `generate-license-file` installed globally through npm.
- Rust toolchain (install via `rustup`)
    - The `wasm32-unknown-unknown` target is also required.
    - You'll also need to install `cargo about` with the `cli` feature and `wasm-bindgen-cli`. On Windows, you will need to either install LLVM Clang or download prebuilt binaries for these.
    - For the desktop app, you'll need `sqlx-cli`, installable with `cargo`.
- For Tauri (desktop):
  - Platform-specific native build tools:
    - macOS: Xcode command line tools
    - Linux: GTK dev packages (varies by distro), as well as `libsecret-tools`.
    - Windows: Visual Studio / Build Tools
  - (Optional) Tauri CLI — you can use the local CLI in devDependencies or install globally

## Environment variables
You'll need to set the following environment variables for the web app (put `.env` file in `apps/web`):

- `SUPABASE_URL` - URL of your supabase database. You can use the sql scripts in the `apps/web/src/db` folder to set up the tables and procedures.
- `SUPABASE_ANON_KEY` - Your supabase anon key.
- `SUPABASE_SERVICE_ROLE_KEY` - Your supabase service role key. Used only for account deletion.
- `NEXT_PUBLIC_SITE_URL` - The URL of your website. This will be http://localhost:3000 in development.

For the desktop app Rust backend you'll need the following (put `.env` file in `apps/desktop/src-tauri`):
- `DATABASE_URL` - SQLx database connection URL for the sqlite database to use for compile-time query checking. You'll need to create an sqlite database somewhere, set this variable to `sqlite:/path/to/db`, and run the migrations on that database with the `sqlx` cli in order to compile the app. The reason behind this is that `sqlx` performs compile-time type-checking of SQL queries, so the app can't compile without a database to check against. Once the app runs once, you can set this variable to the actual database used by the app (in the app data directory) and delete the test database if you wish.
- `API_URL` - URL of the API in the web app. For dev, this would be `http://localhost:3000/api`.

## Installing dependencies

From the repository root:

- Using npm:
  ```bash
  npm install
  ```

- Or using pnpm (recommended for monorepos):
  ```bash
  pnpm install
  ```

This installs dependencies for all workspace packages.

## Development
### Runing both apps (recommended)

At the repo root you can use the root scripts that orchestrate workspaces via Turborepo:

```bash
# start both web and desktop dev tasks in parallel
npm run dev
```

The root script delegates to each app's `dev` script. Useful when you want shortcuts to run both at once.

### Runing a single workspace

- Run only the web app:
  ```bash
  # from repo root
  npm run dev:web
  ```

- Run only the desktop renderer (Vite preview / HMR):
  ```bash
  # from repo root
  npm run dev:desktop
  ```

## Build / production

From the root, run:
```bash
npm run build
```
this builds both web and desktop apps.

## Licensing
- CookyCardz is licensed under the [MIT License](LICENSE).
- Third Party Licenses for the web app are located in [ThirdPartyNotices.txt](ThirdPartyNotices.txt) in the repo root.
- Third Party Licenses for the desktop app are located in [licenses-js.txt](apps/desktop/src/licenses-js.txt) and [licenses-rust.txt](apps/desktop/src/licenses-rust.txt) in `apps/desktop/src`.
- Grocery list generation depends on WordNet, a registered trademark of Princeton University. CookyCardz is neither affiliated with nor endorsed by Princeton University.
    - Princeton University "About WordNet." WordNet. Princeton University. 2010.
