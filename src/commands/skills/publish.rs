use anyhow::{Context, Result};
use base64::Engine;
use chrono::Utc;
use indicatif::ProgressBar;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use agentlink_protocol::skill::{
    CreateSkillSubmissionRequest, SkillAuthor, SkillManifest, SkillManifestFile,
};

use crate::api::ApiClient;
use crate::config::Config;
use crate::utils::output::{print_error, print_success};

pub async fn publish_skill_bundle(config: &Config, path: &str, is_update: bool) -> Result<()> {
    let client = ApiClient::new(config)?;

    // 1. Validate directory
    let skill_dir = Path::new(path);
    if !skill_dir.is_dir() {
        print_error(&format!("'{}' is not a directory", path));
        return Ok(());
    }

    let skill_md_path = skill_dir.join("SKILL.md");
    if !skill_md_path.exists() {
        print_error("SKILL.md not found in the specified directory");
        return Ok(());
    }

    // 2. Parse and validate SKILL.md frontmatter
    let skill_md_content = std::fs::read_to_string(&skill_md_path)
        .with_context(|| "Failed to read SKILL.md")?;
    let (name, version, description) = parse_and_validate_frontmatter(&skill_md_content)?;

    // 3. Get current user info for author
    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Verifying agent identity...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let user = client.get_current_user().await?;
    let agent_id = if user.user_type == agentlink_protocol::UserType::Agent {
        Some(user.id)
    } else {
        None
    };
    spinner.finish_and_clear();

    // 4. Create zip bundle in memory with progress
    let pb = ProgressBar::new_spinner();
    pb.set_message("Packing skill bundle...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let (bundle_bytes, manifest_files) = create_zip_bundle(skill_dir, &name)
        .with_context(|| "Failed to create skill bundle")?;

    pb.finish_with_message(format!("Packed {} files", manifest_files.len()));

    // 5. Base64 encode
    let bundle_base64 = base64::engine::general_purpose::STANDARD.encode(&bundle_bytes);

    // 6. Build manifest
    let manifest = SkillManifest {
        name: name.clone(),
        version: version.clone(),
        description: description.clone(),
        author: SkillAuthor {
            user_id: user.id,
            agent_id,
        },
        source: "agentlink-cli".to_string(),
        cli_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: Utc::now(),
        files: manifest_files,
    };

    // 7. Submit with progress
    let action = if is_update { "Updating" } else { "Uploading" };
    let submit_pb = ProgressBar::new_spinner();
    submit_pb.set_message(format!("{} skill bundle...", action));
    submit_pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let req = CreateSkillSubmissionRequest {
        name,
        version,
        description,
        category: Some("general".to_string()),
        bundle_base64,
        manifest,
    };

    match client.create_skill_submission(req).await {
        Ok(resp) => {
            submit_pb.finish_and_clear();
            if is_update {
                print_success(&format!(
                    "Skill update submitted successfully. Submission ID: {}",
                    resp.id
                ));
            } else {
                print_success(&format!(
                    "Skill submitted successfully. Submission ID: {}",
                    resp.id
                ));
            }
        }
        Err(e) => {
            submit_pb.finish_and_clear();
            if is_update {
                print_error(&format!("Failed to submit skill update: {}", e));
            } else {
                print_error(&format!("Failed to submit skill: {}", e));
            }
        }
    }

    Ok(())
}

fn parse_and_validate_frontmatter(content: &str) -> Result<(String, String, Option<String>)> {
    let trimmed = content.trim();
    if !trimmed.starts_with("---") {
        anyhow::bail!("SKILL.md does not start with YAML frontmatter (---)");
    }

    let after_first = &trimmed[3..];
    let Some(end_pos) = after_first.find("---") else {
        anyhow::bail!("SKILL.md frontmatter not closed with ---");
    };

    let yaml_part = &after_first[..end_pos].trim();
    let frontmatter: serde_yaml::Value = serde_yaml::from_str(yaml_part)
        .with_context(|| "Failed to parse SKILL.md frontmatter as YAML")?;

    let name = frontmatter["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing 'name' in SKILL.md frontmatter"))?
        .trim()
        .to_string();
    let version = frontmatter["version"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing 'version' in SKILL.md frontmatter"))?
        .trim()
        .to_string();
    let description = frontmatter["description"].as_str().map(|s| s.trim().to_string());

    // Validate name: lowercase, alphanumeric, hyphens only
    if name.is_empty() {
        anyhow::bail!("'name' in SKILL.md frontmatter cannot be empty");
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        anyhow::bail!(
            "'name' must contain only lowercase letters, digits, and hyphens (got: {})",
            name
        );
    }
    if name.len() > 100 {
        anyhow::bail!("'name' must not exceed 100 characters");
    }

    // Validate version: strict semver X.Y.Z (no leading zeros)
    let version_parts: Vec<&str> = version.split('.').collect();
    if version_parts.len() != 3
        || version_parts.iter().any(|p| {
            p.parse::<u64>().is_err()
                || p.starts_with('+')
                || p.starts_with('-')
                || (p.len() > 1 && p.starts_with('0'))
        })
    {
        anyhow::bail!(
            "'version' must follow semantic versioning format X.Y.Z without leading zeros (got: {})",
            version
        );
    }

    // Validate description
    if description.as_ref().map(|d| d.is_empty()).unwrap_or(true) {
        anyhow::bail!("'description' is required and cannot be empty");
    }

    Ok((name, version, description))
}

fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("sha256:{}", hex_encode(&result))
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn create_zip_bundle(dir: &Path, _skill_name: &str) -> Result<(Vec<u8>, Vec<SkillManifestFile>)> {
    let mut buf = Vec::new();
    let mut manifest_files = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        walk_and_zip(dir, dir, &mut zip, options, &mut manifest_files)?;
        zip.finish()?;
    }
    Ok((buf, manifest_files))
}

fn walk_and_zip(
    base: &Path,
    current: &Path,
    zip: &mut zip::ZipWriter<std::io::Cursor<&mut Vec<u8>>>,
    options: zip::write::SimpleFileOptions,
    manifest_files: &mut Vec<SkillManifestFile>,
) -> Result<()> {
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let name_in_zip = path.strip_prefix(base)?;

        if path.is_file() {
            let mut f = File::open(&path)?;
            let mut file_bytes = Vec::new();
            f.read_to_end(&mut file_bytes)?;

            let hash = sha256_bytes(&file_bytes);
            let path_str = name_in_zip.to_string_lossy().to_string();
            manifest_files.push(SkillManifestFile {
                path: path_str,
                hash,
            });

            zip.start_file_from_path(name_in_zip, options)?;
            zip.write_all(&file_bytes)?;
        } else if path.is_dir() {
            zip.add_directory_from_path(name_in_zip, options)?;
            walk_and_zip(base, &path, zip, options, manifest_files)?;
        }
    }
    Ok(())
}
