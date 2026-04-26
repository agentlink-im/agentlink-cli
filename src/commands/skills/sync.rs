use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::config::Config;
use crate::utils::output::{print_error, print_success};

pub async fn sync_skills(config: &Config, dir: Option<PathBuf>) -> Result<()> {
    let client = config.to_client()?;

    let skills_dir = match dir {
        Some(d) => d,
        None => {
            let home_dir =
                dirs::home_dir().context("Failed to get home directory")?;
            home_dir.join(".agents").join("skills")
        }
    };

    std::fs::create_dir_all(&skills_dir).with_context(|| {
        format!(
            "Failed to create skills directory: {}",
            skills_dir.display()
        )
    })?;

    println!("Syncing skills to {}...", skills_dir.display());

    let skills = client
        .users
        .list_installed_skills()
        .await
        .with_context(|| "Failed to fetch installed skills")?;

    if skills.is_empty() {
        println!("No skills installed.");
        return Ok(());
    }

    let mut synced = 0;
    let mut failed = 0;

    for skill in skills {
        let skill_dir = skills_dir.join(&skill.name);

        match download_and_extract_skill(&client, &skill, &skill_dir).await {
            Ok(_) => {
                println!("  ✓ {}", skill.name);
                synced += 1;
            }
            Err(e) => {
                print_error(&format!(
                    "Failed to sync {}: {}",
                    skill.name, e
                ));
                failed += 1;
            }
        }
    }

    println!();
    if failed == 0 {
        print_success(&format!(
            "Synced {} skill(s) successfully.",
            synced
        ));
    } else {
        println!("Synced {} skill(s), {} failed.", synced, failed);
    }

    Ok(())
}

async fn download_and_extract_skill(
    client: &agentlink_rust_sdk::AgentLinkClient,
    skill: &agentlink_protocol::user::InstalledSkill,
    skill_dir: &std::path::Path,
) -> Result<()> {
    // Clean and recreate the skill directory
    if skill_dir.exists() {
        std::fs::remove_dir_all(skill_dir)
            .with_context(|| format!("Failed to remove old skill dir: {}", skill_dir.display()))?;
    }
    std::fs::create_dir_all(skill_dir)
        .with_context(|| format!("Failed to create skill directory: {}", skill_dir.display()))?;

    // Download the .skills bundle
    let bundle_bytes = client
        .skills
        .download_skill_bundle(&skill.id.to_string())
        .await
        .with_context(|| format!("Failed to download skill bundle for {}", skill.name))?;

    // Extract zip contents
    extract_zip(&bundle_bytes, skill_dir)
        .with_context(|| format!("Failed to extract skill bundle for {}", skill.name))?;

    // Write metadata file for reference
    let meta_path = skill_dir.join("skill.json");
    std::fs::write(&meta_path, serde_json::to_string_pretty(skill)?)
        .with_context(|| {
            format!(
                "Failed to write skill metadata: {}",
                meta_path.display()
            )
        })?;

    Ok(())
}

fn extract_zip(data: &[u8], dest_dir: &std::path::Path) -> Result<()> {
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .context("Failed to open zip archive")?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .context("Failed to read zip entry")?;
        let out_path = dest_dir.join(file.mangled_name());

        if file.is_dir() {
            std::fs::create_dir_all(&out_path)
                .with_context(|| format!("Failed to create directory: {}", out_path.display()))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
            }
            let mut out_file = std::fs::File::create(&out_path)
                .with_context(|| format!("Failed to create file: {}", out_path.display()))?;
            std::io::copy(&mut file, &mut out_file)
                .with_context(|| format!("Failed to write file: {}", out_path.display()))?;
        }
    }

    Ok(())
}
