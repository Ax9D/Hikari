pub trait Script {
    fn initialize(&mut self);
    fn run(&mut self);
}
pub trait ScriptEngine {
    type S: Script;
    fn initialize(&self);
    fn execute(&self, script: Self::S);
}
