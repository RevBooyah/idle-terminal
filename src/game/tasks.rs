use rand::Rng;
use serde::{Deserialize, Serialize};

use super::resources::Resources;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTask {
    pub definition: TaskDefinition,
    pub remaining_ticks: u32,
    pub input: String,
    pub selected_option: usize,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDefinition {
    pub name: String,
    pub kind: TaskKind,
    pub reward: Resources,
    pub time_limit_ticks: u32,
    pub difficulty: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskKind {
    TypeCommand {
        command: String,
    },
    IncidentResponse {
        question: String,
        options: Vec<String>,
        correct: usize,
    },
}

impl ActiveTask {
    pub fn new(definition: TaskDefinition) -> Self {
        let remaining = definition.time_limit_ticks;
        Self {
            definition,
            remaining_ticks: remaining,
            input: String::new(),
            selected_option: 0,
            completed: false,
        }
    }

    pub fn tick(&mut self) {
        if self.remaining_ticks > 0 {
            self.remaining_ticks -= 1;
        }
    }

    pub fn is_expired(&self) -> bool {
        self.remaining_ticks == 0 && !self.completed
    }

    pub fn time_fraction(&self) -> f64 {
        self.remaining_ticks as f64 / self.definition.time_limit_ticks as f64
    }

    pub fn check_completion(&mut self) -> bool {
        match &self.definition.kind {
            TaskKind::TypeCommand { command } => {
                if self.input == *command {
                    self.completed = true;
                    true
                } else {
                    false
                }
            }
            TaskKind::IncidentResponse { correct, .. } => {
                if self.selected_option == *correct {
                    self.completed = true;
                    true
                } else {
                    false
                }
            }
        }
    }
}

/// How many ticks to wait before spawning a new task after completion/expiry
pub const TASK_COOLDOWN_TICKS: u32 = 20; // 5 seconds at 4Hz

pub fn generate_random_task(rng: &mut impl Rng) -> TaskDefinition {
    let tasks = task_pool();
    let idx = rng.gen_range(0..tasks.len());
    tasks.into_iter().nth(idx).unwrap()
}

fn task_pool() -> Vec<TaskDefinition> {
    vec![
        // TypeCommand tasks
        TaskDefinition {
            name: "Restart Service".into(),
            kind: TaskKind::TypeCommand {
                command: "sudo systemctl restart nginx".into(),
            },
            reward: Resources {
                compute: 50.0,
                ..Default::default()
            },
            time_limit_ticks: 120, // 30 seconds
            difficulty: 1,
        },
        TaskDefinition {
            name: "Deploy Hotfix".into(),
            kind: TaskKind::TypeCommand {
                command: "git push origin hotfix".into(),
            },
            reward: Resources {
                compute: 40.0,
                ..Default::default()
            },
            time_limit_ticks: 100,
            difficulty: 1,
        },
        TaskDefinition {
            name: "Check Disk Usage".into(),
            kind: TaskKind::TypeCommand {
                command: "df -h".into(),
            },
            reward: Resources {
                storage: 30.0,
                ..Default::default()
            },
            time_limit_ticks: 60,
            difficulty: 1,
        },
        TaskDefinition {
            name: "Flush DNS Cache".into(),
            kind: TaskKind::TypeCommand {
                command: "sudo systemd-resolve --flush-caches".into(),
            },
            reward: Resources {
                bandwidth: 60.0,
                ..Default::default()
            },
            time_limit_ticks: 120,
            difficulty: 2,
        },
        TaskDefinition {
            name: "Kill Process".into(),
            kind: TaskKind::TypeCommand {
                command: "kill -9 $(pgrep zombie)".into(),
            },
            reward: Resources {
                compute: 80.0,
                ..Default::default()
            },
            time_limit_ticks: 120,
            difficulty: 2,
        },
        TaskDefinition {
            name: "View Logs".into(),
            kind: TaskKind::TypeCommand {
                command: "tail -f /var/log/syslog".into(),
            },
            reward: Resources {
                compute: 35.0,
                ..Default::default()
            },
            time_limit_ticks: 100,
            difficulty: 1,
        },
        TaskDefinition {
            name: "SSL Certificate".into(),
            kind: TaskKind::TypeCommand {
                command: "certbot renew --dry-run".into(),
            },
            reward: Resources {
                compute: 70.0,
                bandwidth: 30.0,
                ..Default::default()
            },
            time_limit_ticks: 120,
            difficulty: 2,
        },
        // IncidentResponse tasks
        TaskDefinition {
            name: "502 Bad Gateway".into(),
            kind: TaskKind::IncidentResponse {
                question: "Server returning 502. What do you check first?".into(),
                options: vec![
                    "Check upstream service health".into(),
                    "Restart the database".into(),
                    "Clear browser cache".into(),
                    "Increase disk space".into(),
                ],
                correct: 0,
            },
            reward: Resources {
                compute: 100.0,
                ..Default::default()
            },
            time_limit_ticks: 60,
            difficulty: 2,
        },
        TaskDefinition {
            name: "High CPU Alert".into(),
            kind: TaskKind::IncidentResponse {
                question: "CPU at 99%. What's your first step?".into(),
                options: vec![
                    "Add more RAM".into(),
                    "Run top to identify the process".into(),
                    "Reboot immediately".into(),
                    "Ignore it".into(),
                ],
                correct: 1,
            },
            reward: Resources {
                compute: 80.0,
                ..Default::default()
            },
            time_limit_ticks: 60,
            difficulty: 1,
        },
        TaskDefinition {
            name: "Disk Full".into(),
            kind: TaskKind::IncidentResponse {
                question: "Disk at 100%. Quickest safe fix?".into(),
                options: vec![
                    "Delete /var/log/*.log".into(),
                    "Find and clean old logs with logrotate".into(),
                    "Buy a new disk".into(),
                    "Compress the root partition".into(),
                ],
                correct: 1,
            },
            reward: Resources {
                storage: 120.0,
                ..Default::default()
            },
            time_limit_ticks: 60,
            difficulty: 2,
        },
        TaskDefinition {
            name: "DNS Resolution Failure".into(),
            kind: TaskKind::IncidentResponse {
                question: "Users can't resolve your domain. What do you check?".into(),
                options: vec![
                    "Check DNS records and nameservers".into(),
                    "Restart the web server".into(),
                    "Update the SSL certificate".into(),
                    "Clear the CDN cache".into(),
                ],
                correct: 0,
            },
            reward: Resources {
                bandwidth: 90.0,
                ..Default::default()
            },
            time_limit_ticks: 60,
            difficulty: 2,
        },
        TaskDefinition {
            name: "Memory Leak".into(),
            kind: TaskKind::IncidentResponse {
                question: "App memory grows 100MB/hour. Best approach?".into(),
                options: vec![
                    "Add swap space".into(),
                    "Profile with valgrind/heaptrack".into(),
                    "Set a cron to restart hourly".into(),
                    "Upgrade to more RAM".into(),
                ],
                correct: 1,
            },
            reward: Resources {
                compute: 150.0,
                ..Default::default()
            },
            time_limit_ticks: 60,
            difficulty: 3,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_command_completion() {
        let def = TaskDefinition {
            name: "Test".into(),
            kind: TaskKind::TypeCommand {
                command: "ls -la".into(),
            },
            reward: Resources::default(),
            time_limit_ticks: 100,
            difficulty: 1,
        };
        let mut task = ActiveTask::new(def);
        task.input = "ls -la".into();
        assert!(task.check_completion());
        assert!(task.completed);
    }

    #[test]
    fn test_incident_response_completion() {
        let def = TaskDefinition {
            name: "Test".into(),
            kind: TaskKind::IncidentResponse {
                question: "?".into(),
                options: vec!["A".into(), "B".into(), "C".into()],
                correct: 1,
            },
            reward: Resources::default(),
            time_limit_ticks: 100,
            difficulty: 1,
        };
        let mut task = ActiveTask::new(def);
        task.selected_option = 1;
        assert!(task.check_completion());
    }

    #[test]
    fn test_task_expiry() {
        let def = TaskDefinition {
            name: "Test".into(),
            kind: TaskKind::TypeCommand {
                command: "test".into(),
            },
            reward: Resources::default(),
            time_limit_ticks: 2,
            difficulty: 1,
        };
        let mut task = ActiveTask::new(def);
        assert!(!task.is_expired());
        task.tick();
        task.tick();
        assert!(task.is_expired());
    }
}
