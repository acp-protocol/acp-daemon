//! @acp:module "Primer Handler"
//! @acp:summary "AI bootstrap primer endpoint"
//! @acp:domain daemon
//! @acp:layer api

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct PrimerQuery {
    /// Token budget for the primer (default: 200)
    budget: Option<u32>,
    /// Required capabilities (comma-separated)
    capabilities: Option<String>,
}

/// Tier level for content selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Minimal,
    Standard,
    Full,
}

impl Tier {
    pub fn from_budget(remaining: u32) -> Self {
        if remaining < 80 {
            Tier::Minimal
        } else if remaining < 300 {
            Tier::Standard
        } else {
            Tier::Full
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::Minimal => "minimal",
            Tier::Standard => "standard",
            Tier::Full => "full",
        }
    }
}

#[derive(Serialize)]
pub struct PrimerResponse {
    total_tokens: u32,
    tier: String,
    commands_included: usize,
    content: String,
}

/// Bootstrap block content (~20 tokens)
struct Bootstrap {
    awareness: &'static str,
    workflow: &'static str,
    expansion: &'static str,
    tokens: u32,
}

impl Default for Bootstrap {
    fn default() -> Self {
        Self {
            awareness: "This project uses ACP. @acp:* comments are directives for you.",
            workflow: "Before editing: acp constraints <path>",
            expansion: "More: acp primer --budget N",
            tokens: 20,
        }
    }
}

/// Command documentation with tiered content
struct Command {
    name: &'static str,
    critical: bool,
    priority: u32,
    capabilities: &'static [&'static str],
    minimal: TierLevel,
    standard: Option<TierLevel>,
    full: Option<TierLevel>,
}

struct TierLevel {
    tokens: u32,
    template: &'static str,
}

/// GET /primer - Get AI bootstrap primer
pub async fn get_primer(
    State(state): State<AppState>,
    Query(query): Query<PrimerQuery>,
) -> Json<PrimerResponse> {
    let budget = query.budget.unwrap_or(200);
    let capabilities: Vec<String> = query
        .capabilities
        .map(|s| s.split(',').map(|c| c.trim().to_string()).collect())
        .unwrap_or_default();

    let cache = state.cache_async().await;
    let bootstrap = Bootstrap::default();
    let commands = get_default_commands();

    // Filter commands by capabilities
    let filtered_commands: Vec<&Command> = if capabilities.is_empty() {
        commands.iter().collect()
    } else {
        commands
            .iter()
            .filter(|cmd| {
                cmd.capabilities.is_empty()
                    || cmd
                        .capabilities
                        .iter()
                        .any(|cap| capabilities.contains(&cap.to_string()))
            })
            .collect()
    };

    // Sort by (critical desc, priority asc)
    let mut sorted_commands = filtered_commands;
    sorted_commands.sort_by(|a, b| match (a.critical, b.critical) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => a.priority.cmp(&b.priority),
    });

    // Calculate remaining budget after bootstrap
    let remaining_budget = budget.saturating_sub(bootstrap.tokens);
    let tier = Tier::from_budget(remaining_budget);

    // Select commands within budget
    let mut used_tokens = bootstrap.tokens;
    let mut selected_commands: Vec<(&Command, &TierLevel)> = Vec::new();

    for cmd in sorted_commands {
        // Get the appropriate tier level
        let tier_level = match tier {
            Tier::Full => cmd
                .full
                .as_ref()
                .or(cmd.standard.as_ref())
                .unwrap_or(&cmd.minimal),
            Tier::Standard => cmd.standard.as_ref().unwrap_or(&cmd.minimal),
            Tier::Minimal => &cmd.minimal,
        };

        let cmd_tokens = tier_level.tokens;

        // Critical commands are always included
        if cmd.critical || used_tokens + cmd_tokens <= budget {
            used_tokens += cmd_tokens;
            selected_commands.push((cmd, tier_level));
        }
    }

    // Build output content
    let mut content = String::new();

    // Bootstrap block
    content.push_str(bootstrap.awareness);
    content.push('\n');
    content.push_str(bootstrap.workflow);
    content.push('\n');
    content.push_str(bootstrap.expansion);
    content.push_str("\n\n");

    // Commands
    for (cmd, tier_level) in &selected_commands {
        content.push_str(cmd.name);
        content.push('\n');
        content.push_str(tier_level.template);
        content.push_str("\n\n");
    }

    // Add project warnings if we have budget
    if used_tokens + 30 < budget {
        let mut warnings = Vec::new();

        // Check for frozen/restricted symbols
        for (name, symbol) in &cache.symbols {
            if let Some(ref constraints) = symbol.constraints {
                if constraints.level == "frozen" || constraints.level == "restricted" {
                    warnings.push(format!(
                        "{}: {} ({})",
                        name,
                        constraints.level,
                        constraints.directive.chars().take(50).collect::<String>()
                    ));
                }
            }
        }

        if !warnings.is_empty() {
            content.push_str("Project Warnings\n");
            for warning in warnings.iter().take(3) {
                content.push_str(&format!("  - {}\n", warning));
                used_tokens += 15;
                if used_tokens >= budget {
                    break;
                }
            }
        }
    }

    Json(PrimerResponse {
        total_tokens: used_tokens,
        tier: tier.as_str().to_string(),
        commands_included: selected_commands.len(),
        content: content.trim().to_string(),
    })
}

/// Get the default command set for the primer
fn get_default_commands() -> Vec<Command> {
    vec![
        Command {
            name: "acp constraints <path>",
            critical: true,
            priority: 1,
            capabilities: &["shell"],
            minimal: TierLevel {
                tokens: 8,
                template: "  Returns: lock level + directive",
            },
            standard: Some(TierLevel {
                tokens: 25,
                template: "  Returns: lock level + directive\n  Levels: frozen (refuse), restricted (ask), normal (proceed)\n  Use: Check before ANY file modification",
            }),
            full: Some(TierLevel {
                tokens: 45,
                template: "  Returns: lock level + directive\n  Levels: frozen (refuse), restricted (ask), normal (proceed)\n  Use: Check before ANY file modification\n  Example:\n    $ acp constraints src/auth/session.ts\n    frozen - Core auth logic; security-critical",
            }),
        },
        Command {
            name: "acp query file <path>",
            critical: false,
            priority: 2,
            capabilities: &["shell"],
            minimal: TierLevel {
                tokens: 6,
                template: "  Returns: purpose, constraints, symbols",
            },
            standard: Some(TierLevel {
                tokens: 20,
                template: "  Returns: purpose, constraints, symbols, dependencies\n  Options: --json for machine-readable output\n  Use: Understand file context before working with it",
            }),
            full: Some(TierLevel {
                tokens: 35,
                template: "  Returns: purpose, constraints, symbols, dependencies\n  Options: --json for machine-readable output\n  Use: Understand file context before working with it\n  Example:\n    $ acp query file src/payments/processor.ts",
            }),
        },
        Command {
            name: "acp query symbol <name>",
            critical: false,
            priority: 3,
            capabilities: &["shell"],
            minimal: TierLevel {
                tokens: 6,
                template: "  Returns: signature, purpose, constraints, callers",
            },
            standard: Some(TierLevel {
                tokens: 18,
                template: "  Returns: signature, purpose, constraints, callers/callees\n  Options: --json for machine-readable output\n  Use: Understand function/method before modifying",
            }),
            full: None,
        },
        Command {
            name: "acp query domain <name>",
            critical: false,
            priority: 4,
            capabilities: &["shell"],
            minimal: TierLevel {
                tokens: 5,
                template: "  Returns: domain files, cross-cutting concerns",
            },
            standard: Some(TierLevel {
                tokens: 15,
                template: "  Returns: domain files, cross-cutting concerns\n  Options: --json for machine-readable output\n  Use: Understand architectural boundaries",
            }),
            full: None,
        },
        Command {
            name: "acp map [path]",
            critical: false,
            priority: 5,
            capabilities: &["shell"],
            minimal: TierLevel {
                tokens: 5,
                template: "  Returns: directory tree with purposes",
            },
            standard: Some(TierLevel {
                tokens: 15,
                template: "  Returns: directory tree with purposes and constraints\n  Options: --depth N, --inline (show todos/hacks)\n  Use: Navigate unfamiliar codebase",
            }),
            full: None,
        },
        Command {
            name: "acp expand <text>",
            critical: false,
            priority: 6,
            capabilities: &["shell"],
            minimal: TierLevel {
                tokens: 5,
                template: "  Expands $variable references to full paths",
            },
            standard: Some(TierLevel {
                tokens: 12,
                template: "  Expands $variable references to full paths\n  Options: --mode inline|annotated\n  Use: Resolve variable shortcuts in instructions",
            }),
            full: None,
        },
        Command {
            name: "acp attempt start <id>",
            critical: false,
            priority: 7,
            capabilities: &["shell"],
            minimal: TierLevel {
                tokens: 5,
                template: "  Creates checkpoint for safe experimentation",
            },
            standard: Some(TierLevel {
                tokens: 15,
                template: "  Creates checkpoint for safe experimentation\n  Related: acp attempt fail <id>, acp attempt verify <id>\n  Use: Track and revert failed approaches",
            }),
            full: None,
        },
        Command {
            name: "acp primer --budget <N>",
            critical: false,
            priority: 8,
            capabilities: &["shell"],
            minimal: TierLevel {
                tokens: 5,
                template: "  Get more context (this command)",
            },
            standard: Some(TierLevel {
                tokens: 10,
                template: "  Get more context within token budget\n  Options: --capabilities shell,mcp\n  Use: Request more detailed primer",
            }),
            full: None,
        },
    ]
}
