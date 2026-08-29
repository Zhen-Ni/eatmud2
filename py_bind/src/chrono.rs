use eatmud::Weekday as CoreWeekday;
use pyo3::prelude::*;

use crate::common::map_err;

#[pyclass(name = "Weekday", from_py_object)]
#[derive(Clone, Copy)]
pub struct PyWeekday {
    pub inner: CoreWeekday,
}

#[pymethods]
#[allow(non_snake_case)]
impl PyWeekday {
    #[new]
    fn new(num: u8) -> PyResult<Self> {
        let weekday = CoreWeekday::try_from(num).map_err(map_err)?;
        Ok(PyWeekday { inner: weekday })
    }

    #[classattr]
    fn Mon() -> Self {
        Self {
            inner: CoreWeekday::Mon,
        }
    }
    #[classattr]
    fn Tue() -> Self {
        Self {
            inner: CoreWeekday::Tue,
        }
    }
    #[classattr]
    fn Wed() -> Self {
        Self {
            inner: CoreWeekday::Wed,
        }
    }
    #[classattr]
    fn Thu() -> Self {
        Self {
            inner: CoreWeekday::Thu,
        }
    }
    #[classattr]
    fn Fri() -> Self {
        Self {
            inner: CoreWeekday::Fri,
        }
    }
    #[classattr]
    fn Sat() -> Self {
        Self {
            inner: CoreWeekday::Sat,
        }
    }
    #[classattr]
    fn Sun() -> Self {
        Self {
            inner: CoreWeekday::Sun,
        }
    }
}
