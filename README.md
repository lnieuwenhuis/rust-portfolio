# Lars Portfolio

A Rust-first portfolio website for Lars Nieuwenhuis. The app uses Axum, Askama, SQLx, PostgreSQL, Tokio, secure cookie sessions, CSRF checks, and small vanilla CSS/JS.

## Stack

- Rust with Axum for routing and the web server
- Askama for type-safe server-rendered templates
- SQLx and PostgreSQL for projects
- First-login admin setup with Argon2 password storage
- Handcrafted CSS and a small JavaScript file for public UI polish
- Railway-ready build and run settings

## Environment

Copy `.env.example` to `.env` and fill in:

```env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/lars_portfolio
PGUSER=postgres
PGPASSWORD=postgres
ADMIN_USERNAME=lars
SESSION_SECRET=replace_with_64_char_random_secret
BASE_URL=http://localhost:3000
RUST_LOG=info
```

`PORT` is read automatically when present. Locally, the app defaults to `3000` and listens on `0.0.0.0:3000`.

`DATABASE_URL` is the base PostgreSQL connection string. When `PGUSER` or `PGPASSWORD` are present, the app injects them into `DATABASE_URL` before connecting.

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

4. Set a long random `SESSION_SECRET`.
5. Run the app:

   ```powershell
   cargo run
   ```

6. Open `http://localhost:3000`.

The app runs embedded SQLx migrations on startup and seeds safe draft projects only when the database is empty.

## Admin

Visit `/admin/login`. The first time, the form creates the admin password and stores a salted Argon2 hash in PostgreSQL. After that, the same screen becomes the normal login.

The default admin username is `lars` unless `ADMIN_USERNAME` is set. There is no default password.

The admin area can create, edit, publish or unpublish, reorder, and delete projects. JavaScript is not required for those workflows.

## Railway Deploy

1. Create a new Railway project.
2. Add a PostgreSQL service.
3. Import this GitHub repository.
4. Set these environment variables on the app service:

   ```env
   DATABASE_URL=...
   PGUSER=...
   PGPASSWORD=...
   ADMIN_USERNAME=lars
   SESSION_SECRET=...
   BASE_URL=https://your-domain.example
   RUST_LOG=info
   ```

5. Deploy. `railway.toml` uses Railpack, builds with `cargo build --release`, copies the compiled binary to Railpack's `./bin` runtime path, starts `./bin/lars-portfolio`, and checks `/health`.
6. Open `/admin/login` on the deployed domain and create the admin password.

## Useful Checks

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
```
