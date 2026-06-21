use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    print!("Password to hash: ");
    io::stdout().flush()?;

    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim_end_matches(['\r', '\n']);

    if password.is_empty() {
        anyhow::bail!("password cannot be empty");
    }

    let hash = lars_portfolio::auth::hash_password(password)
        .map_err(|error| anyhow::anyhow!("failed to hash password: {error}"))?;
    println!("{hash}");

    Ok(())
}
