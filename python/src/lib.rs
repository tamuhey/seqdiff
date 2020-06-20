use pyo3::{ffi, prelude::*, types::PySequence, AsPyPointer};
use seqdiff::{diff_by, Diff};

#[pymodule]
fn seqdiff(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", "0.1.0")?;

    #[pyfn(m, "diff")]
    pub fn diff_py(_py: Python, a: &PySequence, b: &PySequence) -> PyResult<(Diff, Diff)> {
        let a: Vec<_> = a.iter().unwrap().map(|x| x.unwrap()).collect();
        let b: Vec<_> = b.iter().unwrap().map(|x| x.unwrap()).collect();
        Ok(diff_by(&a, &b, |x, y| unsafe {
            let result = ffi::PyObject_RichCompare(x.as_ptr(), y.as_ptr(), ffi::Py_EQ);
            if result.is_null() {
                false
            } else {
                let ok = ffi::PyObject_IsTrue(result) == 1;
                ffi::Py_DECREF(result);
                ok
            }
        }))
    }
    Ok(())
}
