#![deny(warnings)]
use pyo3::{
    exceptions, ffi,
    prelude::*,
    types::{PyAny, PySequence},
    AsPyPointer,
};
use seqdiff::{diff_by, Diff};
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[pymodule]
fn seqdiff(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", VERSION)?;

    #[pyfn(m, "diff", "*", key = "None")]
    pub fn diff_py(
        py: Python,
        a: &PySequence,
        b: &PySequence,
        key: Option<&PyAny>,
    ) -> PyResult<(Diff, Diff)> {
        let a: Vec<_> = a.iter().unwrap().map(|x| x.unwrap()).collect();
        let b: Vec<_> = b.iter().unwrap().map(|x| x.unwrap()).collect();
        if let Some(f) = key {
            if !f.is_callable() {
                return Err(exceptions::ValueError::py_err(
                    "keyword argument `key` must be callable",
                ));
            }
            let is_eq =
                |x: &&PyAny, y: &&PyAny| f.call((*x, *y), None).unwrap().extract::<bool>().unwrap();
            Ok(diff_by(&a, &b, is_eq))
        } else {
            let is_eq = |x: &&PyAny, y: &&PyAny| unsafe {
                PyObject::from_owned_ptr_or_err(
                    py,
                    ffi::PyObject_RichCompare(x.as_ptr(), y.as_ptr(), ffi::Py_EQ),
                )
                .and_then(|obj| obj.is_true(py))
                .unwrap()
            };
            Ok(diff_by(&a, &b, is_eq))
        }
    }
    Ok(())
}
