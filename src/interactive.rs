use anyhow::Result;
use dialoguer::{Confirm, theme::ColorfulTheme};

/// Prompts the user for confirmation before destructive operations
pub fn confirm_destructive_action(action: &str, target: &str) -> Result<bool> {
    let prompt = format!("Are you sure you want to {} {}?", action, target);
    
    let confirmation = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(false)
        .interact()?;
    
    Ok(confirmation)
}

/// Prompts for confirmation with a custom message
pub fn confirm_action(message: &str) -> Result<bool> {
    let confirmation = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .default(false)
        .interact()?;
    
    Ok(confirmation)
}

/// Prompts for confirmation with default yes
pub fn confirm_action_default_yes(message: &str) -> Result<bool> {
    let confirmation = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .default(true)
        .interact()?;
    
    Ok(confirmation)
}