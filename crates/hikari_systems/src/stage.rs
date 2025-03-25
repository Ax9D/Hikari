use std::collections::{HashMap, HashSet};

use crate::{
    function::{Function, IntoFunction},
    GlobalState,
};
pub struct Task<Return> {
    name: String,
    function: Function<Return>,
    before: HashSet<String>,
    after: HashSet<String>,
}
impl<Return> Task<Return> {
    pub fn new<Params>(name: &str, function: impl IntoFunction<Params, Return>) -> Self {
        Self {
            name: name.to_owned(),
            function: function.into_function(),
            before: HashSet::new(),
            after: HashSet::new(),
        }
    }
    pub unsafe fn with_raw_function(name: &str, function: Function<Return>) -> Self {
        Self {
            name: name.to_owned(),
            function,
            before: HashSet::new(),
            after: HashSet::new(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn before(mut self, task_name: &str) -> Self {
        self.before.insert(task_name.to_owned());

        self
    }
    pub fn after(mut self, task_name: &str) -> Self {
        self.after.insert(task_name.to_owned());

        self
    }
    fn validate(&self) -> Result<(), String> {
        let intersection = self.before.intersection(&self.after);

        if intersection.count() > 0 {
            Err(format!(
                "Task cannot run both before and after another a task!"
            ))
        } else {
            Ok(())
        }
    }
}
pub struct Schedule<Return> {
    functions: Vec<(String, Function<Return>)>,
}

impl<Return> Schedule<Return> {
    pub fn new() -> ScheduleBuilder<Return> {
        ScheduleBuilder { stages: Vec::new() }
    }
    #[inline]
    pub fn execute(&mut self, state: &mut GlobalState) {
        for _ret in self.execute_iter(state) {}
    }
    #[inline]
    pub fn execute_iter<'a>(&'a mut self, state: &'a mut GlobalState) -> impl Iterator<Item = Return> + 'a {
        self.functions.iter_mut().map(|(_name, function)| {
            hikari_dev::profile_scope!(_name);

            unsafe {
                function.run(state.raw())
            }
        })
    }
    pub fn execute_parallel(&mut self, _state: &mut GlobalState) {
        todo!()
    }
}

struct Stage<Return> {
    name: String,
    tasks: Vec<Task<Return>>,
}
impl<Return> Stage<Return> {
    pub fn validate(&self) -> Result<(), String> {
        for task in &self.tasks {
            task.validate()?;
        }

        Ok(())
    }
}

pub struct ScheduleBuilder<Return> {
    stages: Vec<Stage<Return>>
}

impl<Return> ScheduleBuilder<Return> {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }
    pub fn create_stage(&mut self, name: &str) -> &mut Self {
        if self.stages.iter().find(|st| st.name == name).is_none() {
            self.stages.push(Stage {
                name: name.to_string(),
                tasks: Vec::new(),
            });
        } else {
            panic!("Stage with name {} already exists", name);
        }

        self
    }
    pub fn add_task(&mut self, stage: &str, task: Task<Return>) -> &mut Self {
        self.stages
            .iter_mut()
            .find(|st| st.name == stage)
            .unwrap_or_else(|| panic!("Stage {:?} not found", stage))
            .tasks
            .push(task);

        self
    }
    fn validate(&self) -> Result<(), String> {
        for stage in &self.stages {
            stage.validate()?;
        }

        for stage in &self.stages {
            let mut task_names: HashSet<String> = HashSet::new();
            for task in &stage.tasks {
                if task_names.contains(&task.name) {
                    return Err(format!(
                        "Task names must be unique, {:?}, appears more than once in stage {:?}",
                        task.name, stage.name
                    ));
                } else {
                    task_names.insert(task.name.clone());
                }
            }
        }

        Ok(())
    }
    fn build_graph(mut tasks: Vec<Task<Return>>) -> TaskGraph<Return> {
        let mut edges: HashMap<String, HashSet<String>> = HashMap::new();

        for task in &tasks {
            for from_node in &task.after {
                edges
                    .entry(from_node.clone())
                    .or_default()
                    .insert(task.name().to_string());
            }

            for to_node in &task.before {
                edges
                    .entry(task.name().to_string())
                    .or_default()
                    .insert(to_node.to_string());
            }
        }

        let nodes = tasks
            .drain(..)
            .map(|task| (task.name().to_string(), task))
            .collect();

        TaskGraph { nodes, edges }
    }
    pub fn build(self) -> Result<Schedule<Return>, String> {
        self.validate()?;

        let mut functions = Vec::new();
        print!("Exec order: ");
        for stage in self.stages {
            let graph = Self::build_graph(stage.tasks);
            let mut tasks = graph.into_topological_order();

            tasks.drain(..).for_each(|task| {
                print!("{} ", task.name());
                functions.push((task.name, task.function));
            });
        }
        println!();

        Ok(Schedule { functions })
    }
}

struct TaskGraph<Return> {
    nodes: HashMap<String, Task<Return>>,
    edges: HashMap<String, HashSet<String>>,
}
impl<Return> TaskGraph<Return> {
    fn topo_sort_(
        node_name: &str,
        visited: &mut HashSet<String>,
        adj_list: &HashMap<String, HashSet<String>>,
        stack: &mut Vec<String>,
    ) {
        visited.insert(node_name.to_string());

        if let Some(connected_nodes) = adj_list.get(node_name) {
            for node_name in connected_nodes {
                if !visited.contains(node_name) {
                    Self::topo_sort_(node_name, visited, adj_list, stack);
                }
            }
        }

        stack.push(node_name.to_string());
    }
    fn into_topological_order(mut self) -> Vec<Task<Return>> {
        let mut order = Vec::new();

        let mut visited = HashSet::new();

        let adj_list = &self.edges;

        for node in self.nodes.values() {
            if !visited.contains(node.name()) {
                Self::topo_sort_(node.name(), &mut visited, &adj_list, &mut order)
            }
        }

        order.reverse();

        let mut topo = Vec::new();

        for node_name in order {
            topo.push(self.nodes.remove(&node_name).unwrap());
        }

        topo
    }
}

#[cfg(test)]
mod tests {
    use crate::StateBuilder;

    use super::{Schedule, Task};

    fn update(x: &f32, y: &i32) {
        println!("{:?} {}", x, y);
    }
    fn render(y: &i32, s: &mut &'static str, r: &mut RenderData) {
        println!("{} {} {}", y, s, r.state);
        r.state = !r.state;
    }
    struct RenderData {
        state: bool,
    }
    #[test]
    fn stage_build() {
        let mut global = StateBuilder::new();
        global.add_state(420);
        global.add_state(69_f32);
        global.add_state("hello there");
        global.add_state(RenderData { state: false });
        let mut global = global.build();

        let mut task_schedule = Schedule::new();
        task_schedule.create_stage("Update");

        let update_stage = Task::new("Update", &update);

        task_schedule.add_task("Update", update_stage);

        let render_stage = Task::new("Render", &render);

        task_schedule.add_task("Update", render_stage);

        let mut schedule = task_schedule.build().unwrap();

        for _ in 0..50 {
            schedule.execute(&mut global);
        }
    }
}
