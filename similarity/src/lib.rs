use std::path::Path;

use pyo3::prelude::*;

pub fn compare_files(file1: &str, file2: &str) -> Result<f32, Box<dyn std::error::Error>> {
    Python::with_gil(|py| -> PyResult<f32> {
        let file1_path = Path::new(file1).canonicalize()?;
        let file2_path = Path::new(file2).canonicalize()?;

        let script_dir = Path::new("./scripts").canonicalize()?;
        let sys_path = py.import("sys")?.getattr("path")?;
        sys_path.call_method1("append", (script_dir.to_str().unwrap(),))?;

        let similarity = py.import("similarity")?;
        let comparar_dois_codigos = similarity.getattr("comparar_dois_codigos")?;

        let result = comparar_dois_codigos
            .call1((file1_path.to_str().unwrap(), file2_path.to_str().unwrap()))?;

        result.extract::<f32>()
    })
    .map_err(|e| e.into())
}
