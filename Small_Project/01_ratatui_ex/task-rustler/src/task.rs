use crate::date::TaskDate;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Priority {
    Low = 3 ,
    Medium = 2,
    High = 1,
}

impl Priority {
    pub fn next(&self) -> Self {
        match self {
            Priority::Low => Priority::Medium,
            Priority::Medium => Priority::High,
            Priority::High => Priority::Low,
        }
    }

    pub fn from_u8(value: u8) -> Option<Priority> {
        match value {
            3 => Some(Priority::Low),
            2 => Some(Priority::Medium),
            1 => Some(Priority::High),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Priority::Low => 3,
            Priority::Medium => 2,
            Priority::High => 1,
        }
    }
}

impl Display for Priority {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Low => write!(f, "↓"),
            Priority::Medium => write!(f, "="),
            Priority::High => write!(f, "↑"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Task {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub priority: Priority,
    pub date: TaskDate,
}

impl Default for Task {
    fn default() -> Self {
        let mut t = Task::new();
        t.title = "Test task title".to_string();
        t.description = "Test task description".to_string();
        t
    }
}

impl Task {
    pub fn new() -> Self {
        Self {
            id: 0,
            title: String::new(),
            description: String::new(),
            completed: false,
            priority: Priority::Low,
            date: TaskDate(None),
        }
    }
}
