use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::api::ApiClient;
use crate::config::Config;
use crate::utils::output::{print_error, print_success};

pub async fn sync_skills(config: &Config, dir: Option<PathBuf>) -> Result<()> {
    let client = ApiClient::new(config)?;

    let skills_dir = match dir {
        Some(d) => d,
        None => {
            let data_dir =
                dirs::data_dir().context("Failed to get data directory")?;
            data_dir.join("agentlink").join("skills")
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

        match write_skill_to_dir(&skill, &skill_dir).await {
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

async fn write_skill_to_dir(
    skill: &agentlink_protocol::user::InstalledSkill,
    skill_dir: &PathBuf,
) -> Result<()> {
    std::fs::create_dir_all(skill_dir).with_context(|| {
        format!(
            "Failed to create skill directory: {}",
            skill_dir.display()
        )
    })?;

    let skill_meta = serde_json::json!({
        "id": skill.id,
        "name": skill.name,
        "category": skill.category,
        "runtime_type": skill.runtime_type,
        "install_payload": skill.install_payload,
        "local_config": skill.local_config,
        "installed_at": skill.installed_at,
    });

    let meta_path = skill_dir.join("skill.json");
    std::fs::write(&meta_path, serde_json::to_string_pretty(&skill_meta)?)
        .with_context(|| {
            format!(
                "Failed to write skill metadata: {}",
                meta_path.display()
            )
        })?;

    Ok(())
}
