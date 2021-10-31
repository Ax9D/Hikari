use std::{cmp::Ordering, collections::HashSet};

use crate::{global::UnsafeGlobalState, GlobalState};

pub struct System {
    exec: Box<dyn FnMut(&UnsafeGlobalState) + 'static>
}
impl System {
    #[inline]
    pub fn run(&mut self, g_state: &UnsafeGlobalState) {
        (self.exec)(g_state);
    }
}
pub trait IntoSystem<Params>: 'static {
    fn into_system(self) -> System;
}

use crate::query::Fetch;

use crate::query::Query;

macro_rules! impl_into_system {
    ($($name: ident),*) => {
        #[allow(non_snake_case)]
        impl<'a, Func, Return, $($name: Query),*> IntoSystem<($($name,)*)> for Func
        where 
            Func:
                FnMut($($name),*) -> Return +
                FnMut($(<<$name as Query>::Fetch as Fetch>::Item),* ) -> Return + 
                Send + Sync + 'static {
            fn into_system(mut self) -> System { 
                System {
                    exec: Box::new(move |g_state| {
                        //($($name::get(g_state),)*)
                        let ($($name,)*) = unsafe { g_state.query::<($($name,)*)>() };
    
                        self($($name,)*);
                    })
                }
            }
        }
    };
}
impl_into_system!();
impl_into_system!(A);
impl_into_system!(A, B);
impl_into_system!(A, B, C);
impl_into_system!(A, B, C, D);
impl_into_system!(A, B, C, D, E);
impl_into_system!(A, B, C, D, E, F);
impl_into_system!(A, B, C, D, E, F, G);
impl_into_system!(A, B, C, D, E, F, G, H);

pub struct Task {
    name: String,
    system: System,
    before: HashSet<String>,
    after: HashSet<String>,
}
impl Task {
    pub fn new(name: &str, system: System) -> TaskBuilder {
        TaskBuilder {
            task: Task {
                name: name.to_owned(),
                system,
                before: HashSet::new(),
                after: HashSet::new()
            },
        }
    }
    #[inline]
    pub fn run(&mut self, g_state: &UnsafeGlobalState) {
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
        tasks.sort_by(|a, b| {
            if a.before.contains(&b.name) {
                Ordering::Greater
            } else if b.before.contains(&a.name) {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });

        self.schedule
    }
}
#[cfg(test)]
mod tests {
    use crate::borrow::Ref;

    use super::{IntoSystem, Task};

    fn do_stuff(x: Ref<i32>, y: Ref<f32>) -> i32 {
        todo!()
    }
    #[test]
    fn task_build() {
        let task = Task::new("Test", do_stuff.into_system());
    }
}