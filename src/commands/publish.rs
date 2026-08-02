use anyhow::{Result, anyhow};
use chrono::Utc;
use git2::{Cred, FetchOptions, PushOptions, RemoteCallbacks, Repository, Signature};
use octocrab::Octocrab;
use std::fs;

use crate::{auth::storage, manifest::Manifest, terminal};

const OWNER: &str = "beaglesoftware";
const REPO: &str = "cakes";

pub async fn execute() -> Result<()> {
    let auth =
        storage::load()?.ok_or_else(|| anyhow!("Not authenticated. Run `cakeman auth` first."))?;

    let token = auth.github_token;

    let github = Octocrab::builder().personal_token(token.clone()).build()?;

    let user = github.current().user().await?;

    let username = user.login;

    terminal::info("Forking registry repository...");

    match github.repos(OWNER, REPO).create_fork().send().await {
        Ok(_) => {}

        Err(err) => {
            terminal::hint("Fork already exists, continuing...");

            terminal::info(&format!("{:?}", err));
        }
    }

    let fork_url = format!("https://github.com/{}/{}.git", username, REPO);

    let repo_dir = tempfile::tempdir()?;

    terminal::info("Cloning fork...");

    let repo = Repository::clone(&fork_url, repo_dir.path())?;

    let manifest = fs::read_to_string("Cake.toml").map_err(|_| anyhow!("Cake.toml not found"))?;

    let parsed: Manifest = toml::from_str(&manifest)?;

    parsed.validate()?;

    let value: toml::Value = toml::from_str(&manifest)?;

    let package = value
        .get("package")
        .ok_or_else(|| anyhow!("Missing [package]"))?;

    let name = package
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing package name"))?;

    let first_char = name
        .chars()
        .next()
        .ok_or_else(|| anyhow!("Invalid package name"))?
        .to_ascii_lowercase();

    let branch = format!("add-{}-{}", name, Utc::now().timestamp());

    terminal::info("Creating branch...");

    let head = repo.head()?;
    let commit = head.peel_to_commit()?;

    repo.branch(&branch, &commit, false)?;

    repo.set_head(&format!("refs/heads/{}", branch))?;

    repo.checkout_head(None)?;

    let manifest_path = repo_dir
        .path()
        .join("manifests")
        .join(first_char.to_string())
        .join(name)
        .join("Cake.toml");

    fs::create_dir_all(manifest_path.parent().unwrap())?;

    fs::write(&manifest_path, manifest)?;

    terminal::info("Committing manifest...");

    let mut index = repo.index()?;

    index.add_path(manifest_path.strip_prefix(repo_dir.path())?)?;

    index.write()?;

    let tree_id = index.write_tree()?;

    let tree = repo.find_tree(tree_id)?;

    let signature = Signature::now(&username, &format!("{}@users.noreply.github.com", username))?;

    let parent = repo.head()?.peel_to_commit()?;

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &format!("Add {} package", name),
        &tree,
        &[&parent],
    )?;

    terminal::info("Pushing branch...");

    let mut callbacks = RemoteCallbacks::new();

    let push_token = token.clone();

    callbacks.credentials(move |_url, username, _allowed| {
        Cred::userpass_plaintext(username.unwrap_or("git"), &push_token)
    });

    let mut push_options = PushOptions::new();

    push_options.remote_callbacks(callbacks);

    let mut remote = repo.find_remote("origin")?;

    remote.push(
        &[format!("refs/heads/{}:refs/heads/{}", branch, branch)],
        Some(&mut push_options),
    )?;

    terminal::info("Opening pull request...");

    let pr = github
        .pulls(OWNER, REPO)
        .create(
            &format!("Add {}", name),
            &format!("{}:{}", username, branch),
            "main",
        )
        .body("🍰 Published through Cakeman.")
        .send()
        .await?;

    terminal::success("Pull request created successfully!");

    if let Some(url) = pr.html_url {
        terminal::hint(&format!("Pull request: {}", url));
    }

    Ok(())
}
