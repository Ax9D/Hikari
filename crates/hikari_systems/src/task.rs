use std::{collections::HashSet};

use crate::{global::UnsafeGlobalState, system::Function};

pub struct Task {
    name: String,
    system: Function,
    before: HashSet<String>,
    after: HashSet<String>,
}
impl Task {
    pub fn new(name: &str, system: Function) -> TaskBuilder {
        TaskBuilder {
            task: Task {
                name: name.to_owned(),
                system,
                before: HashSet::new(),
                after: HashSet::new(),
            },
        }
    }
    #[inline]
    pub unsafe fn run(&mut self, g_state: &UnsafeGlobalState) {
        self.system.run(g_state);
    }
}
pub struct TaskBuilder {
    task: Task,
}
impl TaskBuilder {
    pub fn before(mut self, task_name: &str) -> Self {
        self.task.before.insert(task_name.to_owned());

        self
    }
    pub fn after(mut self, task_name: &str) -> Self {
        self.task.after.insert(task_name.to_owned());

        self
    }
    fn validate(self) -> Result<Self, String> {
        let intersection = self.task.before.intersection(&self.task.after);
        if intersection.count() > 0 {
            Ok(self)
        } else {
            Err(format!(
                "Task cannot run both before and after another a task!"
            ))
        }
    }
    pub fn build(self) -> Task {
        self.task
    }
}

pub struct TaskSchedule {
    tasks: Vec<Task>,
}

impl TaskSchedule {}

pub struct TaskScheduleBuilder {
    schedule: TaskSchedule,
}

impl TaskScheduleBuilder {
    pub fn add_task(mut self, task: Task) -> Self {
        self.schedule.tasks.push(task);

        self
    }
    fn validate(self) -> Result<Self, String> {
        let mut task_names: HashSet<String> = HashSet::new();

        for task in &self.schedule.tasks {
            if task_names.contains(&task.name) {
                return Err(format!(
                    "Task names must be unique, {:?}, appears more than once",
                    task.name
                ));
            } else {
                task_names.insert(task.name.clone());
            }
        }

        Ok(self)
    }
    fn add_all_dependencies_(task_ix: usize, tasks: &[Task], new_deps: &mut Vec<String>) {
        for dependency in &tasks[task_ix].before {
            if let Some((task_ix, task)) = tasks
                .iter()
                .enumerate()
                .find(|(_, task)| &task.name == dependency)
            {
                task.before
                    .iter()
                    .for_each(|new_dep| new_deps.push(new_dep.clone()));

                Self::add_all_dependencies_(task_ix, tasks, new_deps);
            }
        }
    }
    // fn add_all_dependencies(tasks: &[Task]) {
    //     for (task_ix, _) in tasks.iter().enumerate() {
    //         Self::add_all_dependencies_(task_ix, tasks, new_deps)
    //     }
    // }
    pub fn build(mut self) -> TaskSchedule {
        let tasks = &mut self.schedule.tasks;

        self.schedule
    }
}
#[cfg(test)]
mod tests {
    use crate::{GlobalState, global::Ref, global::RefMut, system::IntoFunction};

    use super::{Task};

    fn do_stuff(mut x: RefMut<f32>, y: Ref<i32>) {
        (*x)+=*y as f32;
        println!("Works {}  {}", *x, *y);
        
    }
    #[test]
    fn task_build() {
        let global = GlobalState::new()
        .add_state(420)
        .add_state(69_f32)
        .build();

        let mut task = Task::new("Hk_Renderer_Update", do_stuff.into_function()).build();

        unsafe { task.run(global.raw()); }
    }
}
