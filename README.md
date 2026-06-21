# Lars Portfolio

A Rust-first portfolio website for Lars Nieuwenhuis. The app uses Axum, Askama, SQLx, PostgreSQL, Tokio, secure cookie sessions, CSRF checks, and small vanilla CSS/JS.

## Stack

- Rust with Axum for routing and the web server
- Askama for type-safe server-rendered templates
- SQLx and PostgreSQL for projects
- Argon2 password verification for the single admin account
- Handcrafted CSS and a small JavaScript file for public UI polish
- Railway-ready build and run settings

## Environment

Copy `.env.example` to `.env` and fill in:

```env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/lars_portfolio
ADMIN_USERNAME=lars
ADMIN_PASSWORD_HASH=replace_with_argon2_hash
SESSION_SECRET=replace_with_64_char_random_secret
BASE_URL=http://localhost:3000
RUST_LOG=info
```

`PORT` is read automatically when present. Locally, the app defaults to `3000` and listens on `0.0.0.0:3000`.

Optional public GitHub and LinkedIn links are intentionally blank in `src/config.rs`. Add real public URLs there when ready.

## Local Setup

1. Install Rust.
2. Start PostgreSQL. With Docker available, run:

   ```powershell
   docker compose up -d
   ```

3. Copy the example environment file:

   ```powershell
   Copy-Item .env.example .env
   ```

4. Generate an Argon2 password hash:

   ```powershell
   cargo run --bin hash_password
   ```

5. Put the generated value in `ADMIN_PASSWORD_HASH` and set a long random `SESSION_SECRET`.
6. Run the app:

   ```powershell
   cargo run
   ```

7. Open `http://localhost:3000`.

The app runs embedded SQLx migrations on startup and seeds safe draft projects only when the database is empty.

## Admin

Visit `/admin/login` and sign in with `ADMIN_USERNAME` plus the password used to generate `ADMIN_PASSWORD_HASH`.

The admin area can create, edit, publish or unpublish, reorder, and delete projects. JavaScript is not required for those workflows.

## Railway Deploy

1. Create a new Railway project.
2. Add a PostgreSQL service.
3. Import this GitHub repository.
4. Set these environment variables on the app service:

   ```env
   DATABASE_URL=...
   ADMIN_USERNAME=lars
   ADMIN_PASSWORD_HASH=...
   SESSION_SECRET=...
   BASE_URL=https://your-domain.example
   RUST_LOG=info
   ```

5. Deploy. `railway.toml` uses Railpack, builds with `cargo build --release`, starts `./target/release/lars-portfolio`, and checks `/health`.

## Useful Checks

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
```
