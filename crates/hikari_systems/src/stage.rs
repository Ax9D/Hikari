use std::{collections::{HashSet, HashMap}};

use crate::{function::{Function, IntoFunction}, GlobalState};
pub struct Stage {
    name: String,
    functions: Vec<Function>,
    before: HashSet<String>,
    after: HashSet<String>,
}
impl Stage {
    pub fn new(name: &str) -> Self {
        Self {
                name: name.to_owned(),
                functions: Vec::new(),
                before: HashSet::new(),
                after: HashSet::new(),
            }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn before(&mut self, task_name: &str) -> &mut Self {
        self.before.insert(task_name.to_owned());

        self
    }
    pub fn after(&mut self, task_name: &str) -> &mut Self {
        self.after.insert(task_name.to_owned());

        self
    }
    pub fn add_function<Params>(&mut self, function: impl IntoFunction<Params>) -> &mut Self{
        self.functions.push(function.into_function());

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
pub struct Schedule {
    functions: Vec<Function>,
}

impl Schedule {
    pub fn new() -> ScheduleBuilder {
        ScheduleBuilder {
            stages: Vec::new()
        }
    }
    #[inline]
    pub fn execute(&mut self, state: &mut GlobalState) {
        for function in &mut self.functions {
            unsafe {
                function.run(state.raw());
            }
        }
    }
    pub fn execute_parallel(&mut self, _state: &mut GlobalState) {
        todo!()
    }
 
}

pub struct ScheduleBuilder {
    stages: Vec<Stage>,
}

impl ScheduleBuilder {
    pub fn new() -> Self {
        Self {
            stages: Vec::new()
        }
    }
    pub fn add_stage(&mut self, stage: Stage) -> &mut Self {
        self.stages.push(stage);

        self
    }
    pub fn add_to_stage<Params>(&mut self, task_name: &str, function: impl IntoFunction<Params>) -> &mut Self {
        let task = self.stages.iter_mut().find(|task| task.name() == task_name).expect("Task not found");
        task.add_function(function);

        self
    }
    fn validate(&self) -> Result<(), String> {
        for task in &self.stages {
            task.validate()?;
        }

        let mut task_names: HashSet<String> = HashSet::new();

        for task in &self.stages {
            if task_names.contains(&task.name) {
                return Err(format!(
                    "Task names must be unique, {:?}, appears more than once",
                    task.name
                ));
            } else {
                task_names.insert(task.name.clone());
            }
        }

        Ok(())
    }
    fn build_stage_graph(mut stages: Vec<Stage>) -> TaskGraph {
        let mut edges: HashMap<String, HashSet<String>> = HashMap::new();

        for stage in &stages {
            for from_node in &stage.after {
                edges.entry(from_node.clone())
                .or_default()
                .insert(stage.name().to_string());
            }

            for to_node in &stage.before {
                edges.entry(stage.name().to_string())
                .or_default()
                .insert(to_node.to_string());
            }
        }

        let nodes = stages
                                        .drain(..)
                                        .map(|task| (task.name().to_string(), task))
                                        .collect();


        TaskGraph {
            nodes,
            edges
        }
    }
    pub fn build(self) -> Result<Schedule, String> {
        self.validate()?;
        let graph = Self::build_stage_graph(self.stages);
    
        let tasks = graph.into_topological_order();
        
        let mut functions = Vec::new();

        print!("Exec order: ");
        for mut task in tasks {
            print!("{} ", task.name());
            task.functions.drain(..).for_each(|function| functions.push(function));
        }

        println!();
        Ok(Schedule {
            functions
        })
    }
}

struct TaskGraph {
    nodes: HashMap<String, Stage>,
    edges: HashMap<String, HashSet<String>>
}
impl TaskGraph {
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
    fn into_topological_order(mut self) -> Vec<Stage> {
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
    use crate::{StateBuilder, Ref, RefMut};

    use super::{Stage, Schedule};

    fn update(x: Option<RefMut<f32>>, y: Ref<i32>) {      
        println!("{:?} {}", x, y);
    }
    fn render(y: Ref<i32>, s: Ref<&'static str>, mut r: RefMut<RenderData>) {
        println!("{} {} {}", y, s, r.state);
        r.state = !r.state;
    }
    struct RenderData {
        state: bool
    }
    #[test]
    fn stage_build() {
        let mut global = StateBuilder::new();
        global.add_state(420);
        global.add_state(69_f32);
        global.add_state("hello there");
        global.add_state(RenderData{state: false});
        let mut global = global.build();

        let mut task_schedule = Schedule::new();

        let mut update_stage = Stage::new("Update");
        update_stage.add_function(&update);
        
        task_schedule.add_stage(update_stage);

        let mut render_stage = Stage::new("Render");

        render_stage.after("Update");
        render_stage.add_function(&render);

        task_schedule.add_stage(render_stage);
        
        let mut schedule = task_schedule.build().unwrap();

        for _ in 0..50 {
            schedule.execute(&mut global);
        }
    }
}
