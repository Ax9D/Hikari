use std::{cmp::Ordering, collections::HashSet};

use crate::{GlobalState, global::UnsafeGlobalState};

pub trait System: 'static {
    fn run(&mut self);
}

pub trait IntoSystem {

}

pub struct Task {
    name: String,
    exec: Box<dyn System>,
    dependencies: HashSet<String>
}
impl Task {
    pub fn new(name: &str, exec: impl System) -> TaskBuilder {
        TaskBuilder {
            task: Task {
                name: name.to_owned(),
                exec: Box::new(exec),
                dependencies: HashSet::new()
            }
        }
    }
}
pub struct TaskBuilder {
    task: Task
}
impl TaskBuilder {
    pub fn depends_on(self, task: &Task) -> Self {
        self.depends_on_by_name(&task.name)
    } 
    pub fn depends_on_by_name(mut self, task_name: &str) -> Self {
        self.task.dependencies.insert(task_name.to_owned());

        self
    }
    pub fn build(self) -> Task {
        self.task
    }
}

pub struct TaskSchedule {
    tasks: Vec<Task>
}

impl TaskSchedule {
   
}

pub struct TaskScheduleBuilder {
    schedule: TaskSchedule
}

impl TaskScheduleBuilder {
    pub fn add_task(mut self, task: Task) -> Self{
        self.schedule.tasks.push(task);

        self
    }
    fn validate(self) -> Result<Self, String>{
        let mut task_names: HashSet<String> = HashSet::new();

        for task in &self.schedule.tasks {
            if task_names.contains(&task.name) {
                return Err(format!("Task names must be unique, {:?}, appears more than once", task.name));
            }
            else {
                task_names.insert(task.name.clone());
            }
        }

        Ok(self)
    }
    fn add_all_dependencies_(task_ix: usize, tasks: &[Task], new_deps: &mut Vec<String>) {
        for dependency in &tasks[task_ix].dependencies {
            if let Some((task_ix, task)) = tasks.iter().enumerate().find(|(_, task)| &task.name == dependency) {
                
                task.dependencies.iter().for_each(|new_dep|new_deps.push(new_dep.clone()));

                Self::add_all_dependencies_(task_ix, tasks, new_deps);
            }
        }
    }
    // fn add_all_dependencies(tasks: &[Task]) {
    //     for (task_ix, _) in tasks.iter().enumerate() {
    //         Self::add_all_dependencies_(task_ix, tasks, new_deps)
    //     }
    // }
    pub fn build(mut self) -> TaskSchedule{
        let tasks = &mut self.schedule.tasks;
        tasks.sort_by(|a,b| {
            if a.dependencies.contains(&b.name) {
                Ordering::Greater 
            } else if b.dependencies.contains(&a.name) {
                Ordering::Less
            }
            else {
                Ordering::Equal
            }
        });

        self.schedule
    }
}