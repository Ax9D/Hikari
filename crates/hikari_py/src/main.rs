use pyo3::prelude::*;
struct GameObject {
    update: Py<PyAny>,
}

fn execute_script(object: &mut GameObject, python: Python) -> PyResult<()> {
    object.update.call0(python)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pyo3::prepare_freethreaded_python();

    let guard = Python::acquire_gil();
    let python = guard.python();
    python
        .run("import sys\nprint(sys.executable, sys.path)\n", None, None)
        .unwrap();

    let script = PyModule::from_code(
        python,
        &std::fs::read_to_string("translate.py")?,
        "translate.py",
        "translate",
    )?;
    let n = 50_000;
    
    let mut objects = Vec::new();
    
    let script_class = script.getattr("Translate")?;

    for _ in 0..n {
        let script_instance = script_class.call1((rand::random::<f32>(), rand::random::<f32>()))?;
        let update = script_instance.getattr("update")?.to_object(python);
        objects.push(GameObject { update });
    }

    let now = std::time::Instant::now();
    for object in objects.iter_mut() {
        //script_instance.call_method1("update", (0.5, )).expect("Failed to update");
        execute_script(object, python)?;
    }
    let elapsed = now.elapsed();
    println!("Per call: {:?} Total: {:?}", elapsed/n, elapsed);

    Ok(())
}
