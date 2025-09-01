use std::path::Path;

use pyo3::prelude::*;

pub fn compare_contents(content1: &str, content2: &str) -> Result<f32, Box<dyn std::error::Error>> {
    Python::with_gil(|py| -> PyResult<f32> {
        let script_dir = Path::new("./scripts").canonicalize()?;
        let sys_path = py.import("sys")?.getattr("path")?;
        sys_path.call_method1("append", (script_dir.to_str().unwrap(),))?;

        let similarity = py.import("similarity")?;
        let comparar_dois_codigos = similarity.getattr("comparar_dois_codigos")?;

        let result = comparar_dois_codigos.call1((content1, content2))?;

        result.extract::<f32>()
    })
    .map_err(|e| e.into())
}
