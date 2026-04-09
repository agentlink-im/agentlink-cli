use colored::Colorize;
use comfy_table::{ContentArrangement, Table};

use crate::models::UserResponse;

/// 打印成功消息
pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message.green());
}

/// 打印错误消息
pub fn print_error(message: &str) {
    println!("{} {}", "✗".red().bold(), message.red());
}

/// 打印警告消息
pub fn print_warning(message: &str) {
    println!("{} {}", "!".yellow().bold(), message.yellow());
}

/// 打印表格
pub fn print_table(headers: Vec<&str>, rows: Vec<Vec<String>>) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // 添加表头
    table.set_header(headers);

    // 添加行
    for row in rows {
        table.add_row(row);
    }

    println!("{}", table);
}

/// 打印用户信息
pub fn print_user_info(user: &UserResponse) {
    println!();
    println!("{}", "Authenticated Identity".bold().underline());
    println!();
    println!("{}: {}", "ID".bold(), user.id);
    println!("{}: {}", "LinkID".bold(), user.linkid);
    if let Some(profile) = &user.profile {
        if let Some(email) = &profile.email {
            println!("{}: {}", "Email".bold(), email);
        }
        if let Some(headline) = &profile.headline {
            println!("{}: {}", "Headline".bold(), headline);
        }
        if let Some(display_context) = &profile.display_context {
            println!("{}: {}", "Context".bold(), display_context);
        }
    }
    println!("{}: {}", "Type".bold(), user.user_type);
    println!(
        "{}: {}",
        "Status".bold(),
        format!("{:?}", user.status).to_lowercase()
    );
    println!("{}: {}", "Verified".bold(), user.is_verified);
    println!();
}
