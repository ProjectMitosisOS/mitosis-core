/// The kern_wrappers module containers handful wrappers over kernel_structures used by MITOSIS.
/// These includes:
/// * mm_struct - abstracted in mm::MemoryDescriptor 
/// * task_struct - abstracted in task::Task
/// * vma_struct - abstracted in vma::VMA
/// 
/// vma_iters module also includes useful code for iterating pages belonging to a VMA
pub mod mm;
pub mod task;
pub mod vma;
pub mod vma_iters;
pub mod page;

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

impl Process {
    #[inline]
    pub fn get_memory_descriptor(&self) -> &mm::MemoryDescriptor { 
        &self.memory
    }

    #[inline]
    pub fn get_task(&self) -> &task::Task { 
        &self.task
    }
}
