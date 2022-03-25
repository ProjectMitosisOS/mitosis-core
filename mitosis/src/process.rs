pub mod mm;
pub mod vma;
pub mod task;

#[allow(dead_code)]
pub struct Process {
    task: task::Task,
}

impl Process {
    pub fn new() -> Self {
        Self {
            task: task::Task::new(),
        }
    }

    pub fn new_from_task(task: task::Task) -> Self {
        Self {
            task: task,
        }
    }

    pub fn get_task(&self) -> &task::Task {
        &self.task
    }
}

// TODO: implement `generate_descriptor` for `Process`
