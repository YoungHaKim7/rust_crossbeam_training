use crate::task::{Priority, Task};
use crate::task_db::DB;

#[derive(Debug, Copy, Clone)]
pub enum SortOrder {
    High,
    Low,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct TasksService {
    db: DB,
    db_filename: String,
}

impl Default for TasksService {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl TasksService {
    /// Create a new database
    /// `db_path` is the path to the database file. If it doesn't exist it is going to be created.
    /// If db_path is an empty string __""__, an in memory instance of database is going to be
    /// created instead.
    pub fn new(db_path: String) -> Self {
        Self {
            db: DB::create_and_return_connection(db_path.as_str()),
            db_filename: db_path,
        }
    }

    pub fn add_new_task(&self, task: &Task) {
        self.db.insert_task(task);
    }

    /// Get a task with `task_id`. Returns an Option containing the task or None
    /// if it couldn't find the task.
    pub fn get_task(&self, task_id: i32) -> Option<Task> {
        match self.db.get_task_by_id(task_id) {
            Ok(t) => Some(t),
            Err(e) => {
                eprintln!("{e}");
                if let Some(cause) = e.source() {
                    eprintln!("Underlying cause: {:?}", cause);
                }
                None
            }
        }
    }

    /// Returns all the tasks present in the database
    pub fn get_all_tasks(&self) -> Vec<Task> {
        self.db.get_all_tasks()
    }

    /// Return all the tasks sorted by `sort`
    pub fn get_all_tasks_sorted_by_priority(&self, sort: SortOrder) -> Vec<Task> {
        match sort {
            SortOrder::High => self.db.get_all_task_by_highest_priority(),
            SortOrder::Low => self.db.get_all_task_by_lowest_priority(),
        }
    }

    pub fn get_all_tasks_sorted_by_date(&self, sort: SortOrder) -> Vec<Task> {
        match sort {
            SortOrder::High => self.db.get_all_tasks_by_newest(),
            SortOrder::Low => self.db.get_all_tasks_by_oldest(),
        }
    }

    pub fn toggle_task_status(&self, task_id: i32, completed: bool) -> usize {
        self.db.toggle_task_completed(task_id, completed)
    }

    /// Change priority of the task
    pub fn change_priority(&self, task_id: i32, priority: &Priority) -> usize {
        self.db.update_task_priority(task_id, priority.to_owned())
    }

    pub fn update_task(&self, task: &Task) {
        self.db.update_task(task);
    }

    /// Delete a task with `task_id` number
    pub fn delete_task(&self, task_id: i32) -> usize {
        self.db.delete_task(task_id)
    }

    /// Number of tasks present in the database
    pub fn length(&self) -> i64 {
        self.db.get_record_count()
    }

    /// Check if there are no tasks at all
    pub fn is_empty(&self) -> bool {
        self.length() == 0
    }

    /// Clean the database, delete all tasks
    pub fn delete_all_tasks(&self) -> usize {
        self.db.clear()
    }
}
