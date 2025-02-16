use std::env;
use crate::app::{AppContext, InputFieldType, InputMode};
use crate::date::{TaskDate, DATE_FORMAT};
use crate::task::Task;
use anyhow::{anyhow, Context, Result};
use crate::export::{export_tasks_to_icalendar, write_to_file};

pub trait Command {
    fn execute(&self, app: &mut AppContext) -> Result<()>;
}

/// Enter in add command input mode
pub struct EnterAddModeCommand;

impl Command for EnterAddModeCommand {
    fn execute(&self, app: &mut AppContext) -> Result<()> {
        app.input_mode = InputMode::Adding;
        app.input_field = InputFieldType::Title;
        Ok(())
    }
}

/// Add a new task
pub struct AddTaskCommand;

impl Command for AddTaskCommand {
    fn execute(&self, app: &mut AppContext) -> Result<()> {
        let mut t = Task::new();
        if app.input_title.is_empty() {
            return Err(anyhow!("You must insert at least a title for the task"));
        }
        if !app.input_date.is_empty() {
            t.date = TaskDate::try_from(app.input_date.drain(..).collect::<String>())
                .context("Invalid date format, use dd-mm-yyyy")?;
        }
        t.title = app.input_title.drain(..).collect();
        t.description = app.input_description.drain(..).collect();
        app.tasks_service.add_new_task(&t);
        app.refresh_task_list();
        Ok(())
    }
}

/// Start editing a task, move cursor to Title input field
/// and set InputMode equal to InputMode::EditingExisting
pub struct StartEditingExistingTaskCommand;

impl Command for StartEditingExistingTaskCommand {
    fn execute(&self, app: &mut AppContext) -> Result<()> {
        if let Some(index) = app.task_list.state.selected() {
            app.input_title = app.task_list.items[index].title.clone();
            app.input_description = app.task_list.items[index].description.clone();
            app.input_date = app.task_list.items[index]
                .date
                .clone()
                .0
                .map(|d| d.format(DATE_FORMAT).to_string())
                .unwrap_or(String::new());
            app.input_mode = InputMode::EditingExisting;
            app.input_field = InputFieldType::Title;
        }
        Ok(())
    }
}

/// Finish editing an existing task, set InputMode back to Normal
pub struct FinishEditingExistingTaskCommand;

impl Command for FinishEditingExistingTaskCommand {
    fn execute(&self, app: &mut AppContext) -> Result<()> {
        if let Some(index) = app.task_list.state.selected() {
            if app.input_title.is_empty() {
                return Err(anyhow!("You must insert at least a title for the task"));
            }
            if !app.input_date.is_empty() {
                app.task_list.items[index].date =
                    TaskDate::try_from(app.input_date.drain(..).collect::<String>())
                        .context("Invalid date format, use dd-mm-yyyy")?;
            } else {
                app.task_list.items[index].date = TaskDate(None)
            }
            app.task_list.items[index].title = app.input_title.drain(..).collect();
            app.task_list.items[index].description = app.input_description.drain(..).collect();
            app.tasks_service.update_task(&app.task_list.items[index])
        }
        Ok(())
    }
}


/// Toggle completed for selected task status
pub struct ToggleTaskStatusCommand;

impl Command for ToggleTaskStatusCommand {
    fn execute(&self, app: &mut AppContext) -> Result<()> {
        if let Some(index) = app.task_list.state.selected() {
            let item = &mut app.task_list.items[index];
            item.completed = match item.completed {
                true => false,
                false => true,
            };
            let _ = app
                .tasks_service
                .toggle_task_status(item.id, item.completed);
        };
        Ok(())
    }
}

/// Switch between priorities
pub struct ToggleItemPriorityCommand;

impl Command for ToggleItemPriorityCommand {
    fn execute(&self, app: &mut AppContext) -> Result<()> {
        if let Some(index) = app.task_list.state.selected() {
            let item = &mut app.task_list.items[index];
            item.priority = item.priority.next();
            app.tasks_service.change_priority(item.id, &item.priority);
        }
        Ok(())
    }
}

pub struct DeleteTaskCommand;

impl Command for DeleteTaskCommand {
    fn execute(&self, app: &mut AppContext) -> Result<()> {
        if let Some(index) = app.task_list.state.selected() {
            app.tasks_service.delete_task(app.task_list.items[index].id);
            app.task_list.items.remove(index);
        }
        Ok(())
    }
}

/// Stop adding or editing the current task, clear the input fields and
/// set InputMode back to Normal
pub struct StopEditingCommand;

impl Command for StopEditingCommand {
    fn execute(&self, app: &mut AppContext) -> Result<()> {
        app.input_mode = InputMode::View;
        app.input_title.clear();
        app.input_description.clear();
        app.input_date.clear();
        app.error = None;
        Ok(())
    }
}

pub struct EnterExportModeCommand;
impl Command for EnterExportModeCommand {
    fn execute(&self, app: &mut AppContext) -> Result<()> {
        app.input_mode = InputMode::Export;
        app.error = None;
        let mut current_dir = env::current_dir().context("Could not access to the current directory")?;
        current_dir.set_file_name("task_rustler.ics");
        app.input_export_path = current_dir.display().to_string();
        Ok(())
    }
}

pub struct FinishingExportCommand;
impl Command for FinishingExportCommand {
    fn execute(&self, app: &mut AppContext) -> Result<()> {
        let calendar = export_tasks_to_icalendar("task rustler", &app.task_list.items);
        write_to_file(app.input_export_path.as_str(), calendar.to_string().as_str())
    }
}

pub struct ExitExportModeCommand;
impl Command for ExitExportModeCommand {
    fn execute(&self, app: &mut AppContext) -> Result<()> {
        app.input_mode = InputMode::View;
        app.input_export_path.clear();
        app.error = None;
        Ok(())
    }
}
