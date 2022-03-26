pub mod mm;
pub mod task;
pub mod vma;

/// TODO: still don't know how this struct is used for
pub struct Process {
    task: task::Task,
    memory: mm::MemoryDescriptor,
}

impl Process {
    pub fn new_from_current() -> Self {
        let task = task::Task::new();
        let mm = task.get_memory_descriptor();

        Self {
            task: task,
            memory: mm,
        }
    }

    pub unsafe fn new_from_raw(task: *mut task::task_struct) -> Self {
        let task = task::Task::new_from_raw(task);
        let mm = task.get_memory_descriptor();

        Self {
            task: task,
            memory: mm,
        }
    }
}
