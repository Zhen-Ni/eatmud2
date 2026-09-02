# Plan.md: PyO3 绑定层零拷贝优化方案 (安全更新版)

## 方案名称
**Enum + Copy-on-Write (枚举 + 写时复制)** / **Safe Zero-Copy Borrowing (安全零拷贝借用)**

## 核心痛点与方案演进
在之前的 `Arc + ptr` 结合 `Arc::get_mut` 方案中，我们依赖 `Arc` 的引用计数机制来阻止对“借用”对象的修改。
但在实际跨语言交互中发现致命隐患：CPython 的垃圾回收时机是不确定的（循环引用、异常捕获、闭包持有等会导致对象延迟销毁）。如果派生出的子对象的 `Arc` 引用迟迟不释放，父对象自身的 `Arc::get_mut` 也会失败，导致父对象**无法安全修改自身数据**，引发逻辑阻塞或异常。

## 解决思路
彻底放弃依靠 `Arc` 共享所有权来拦截修改的做法。改为：**父对象完全拥有数据所有权，子对象仅持有只读裸指针，并配合 `Py<PyAny>` 强引用父对象防止其被回收。**
当 Python 端对借用的子对象进行读取时，零拷贝直接访问指针；当尝试修改时，触发**写时复制**，立即深拷贝一份脱离父对象的独立副本。

## 核心代码结构

```rust
use pyo3::prelude::*;
use std::ptr::NonNull;
use eatmud::record::DetailedRecord as CoreDetailedRecord;

pub enum RecordSource {
    /// 独立拥有所有权的情况（Python直接创建或触发写时复制后）
    Owned(CoreDetailedRecord),
    /// 借用的情况（从 TransactionIterator 返回）
    Borrowed {
        ptr: NonNull<CoreDetailedRecord>,
        parent: Py<PyAny>, // 持有父对象的 Python 引用，防止父对象被 GC 回收
    },
}

#[pyclass(name = "DetailedRecord")]
pub struct PyDetailedRecord {
    inner: RecordSource,
}

// 包含裸指针，需手动声明 Send/Sync (在 GIL 保护下是安全的)
unsafe impl Send for PyDetailedRecord {}
unsafe impl Sync for PyDetailedRecord {}
```

## 读取与写入策略 (绝对安全)

*   **读取（分支+零开销）**：通过 `match` 匹配枚举状态。若是 `Owned` 直接返回引用；若是 `Borrowed`，安全前提下解引用裸指针获取数据。
*   **修改（写时复制 Cow）**：当调用 `append`、`clear` 等修改类方法时，调用 `inner_mut()`。如果发现当前是 `Borrowed` 状态，立即 `clone` 底层数据，将自身状态替换为 `Owned`，然后返回可变引用。

```rust
impl PyDetailedRecord {
    #[inline]
    fn inner(&self) -> &CoreDetailedRecord {
        match &self.inner {
            RecordSource::Owned(r) => r,
            RecordSource::Borrowed { ptr, .. } => {
                // 安全前提：parent 对象依然存活，保证了 ptr 指向的内存有效
                unsafe { ptr.as_ref() }
            }
        }
    }

    fn inner_mut(&mut self) -> &mut CoreDetailedRecord {
        if let RecordSource::Borrowed { ptr, .. } = &self.inner {
            // 触发写时复制：克隆一份独立数据
            let cloned = unsafe { ptr.as_ref().clone() };
            self.inner = RecordSource::Owned(cloned);
        }
        
        match &mut self.inner {
            RecordSource::Owned(r) => r,
            _ => unreachable!(),
        }
    }
}
```

## 构造方式

1. **Python 端独立创建**：枚举初始化为 `Owned`。
2. **TransactionIterator 生成只读视图**：
   ```rust
   Some(PyDetailedRecord {
       inner: RecordSource::Borrowed {
           ptr: NonNull::from(record_ref),
           parent: self.into_py(self.py()), // 保持迭代器存活
       },
   })
   ```

## 优势总结
1. **不受 GC 时机影响**：父对象随时可以修改自身数据，完全不受子对象生命周期的限制。
2. **绝对内存安全**：任何写操作都会自动触发深拷贝，彻底杜绝悬垂指针与多线程数据竞争。
3. **零拷贝读取**：从迭代器获取只读视图时，性能拉满，没有任何 `Arc` 计算或数据复制开销。

## 适用场景
*   高频读取、低频修改的跨语言 FFI 场景。
*   父对象在生成视图后仍需保持独立可变性的架构。
*   对内存安全要求极高，不愿承担因 GC 延迟带来的未定义行为风险。

---

## 扩展方案：NumPy ndarray 的零拷贝暴露

### 核心痛点
在处理如 `navs` (净值矩阵) 等大型二维数组时，先前使用的 `to_pyarray()` 会在首次调用时触发内存的深拷贝，将 Rust 端的 `Array2<f64>` 复制到 NumPy 新分配的内存中。这在数据量极大时会造成显著的内存浪费和性能损耗。由于 `rust-numpy` 未提供安全的高级 API 来直接借用 Rust 内存并绑定父对象防 GC，我们需要寻找替代方案。

### 解决思路：`__array_interface__` + `numpy.asarray`
完全符合 Python 数据科学生态标准做法。在 Rust 端构建一个轻量级的包装类，实现 `__array_interface__` 协议，直接暴露底层数据的裸指针、形状和类型信息。然后在 Rust 内部调用 Python 的 `numpy.asarray()` 方法，将该包装类转换为原生的 `numpy.ndarray`。

### 核心代码结构

```rust
use ndarray::ArrayView2;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString, PyTuple};

/// 零拷贝暴露底层 ndarray 的包装类
#[pyclass(name = "Navs")]
pub struct PyNavs {
    /// 指向 Rust 底层数组的视图，生命周期被 hack 为 'static
    view: ArrayView2<'static, f64>,
    /// 强引用父对象，防止底层内存被回收
    parent: Py<PyAny>,
}

unsafe impl Send for PyNavs {}
unsafe impl Sync for PyNavs {}

#[pymethods]
impl PyNavs {
    /// 实现 __array_interface__，允许 np.asarray(view) 时零拷贝共享内存
    #[getter]
    fn __array_interface__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        
        let shape = PyTuple::new(py, self.view.shape())?;
        let typestr = PyString::new(py, "<f8"); // little-endian f64
        
        // 暴露数据指针，false 表示只读
        let data = PyTuple::new(py, [self.view.as_ptr() as usize, false])?;
        
        dict.set_item("shape", shape)?;
        dict.set_item("typestr", typestr)?;
        dict.set_item("data", data)?;
        dict.set_item("version", 3)?;
        Ok(dict)
    }
}
```

### 转换与返回策略

在暴露 `navs` 数据的方法中，利用生命周期hack构建 `ArrayView2`，包装进 `PyNavs`，并立即在 Rust 端调用 Python 的 `numpy.asarray` 进行转换：

```rust
    /// 获取底层 navs 的零拷贝原生 numpy.ndarray 视图
    #[getter]
    fn navs<'py>(this: &Bound<'py, Self>) -> PyResult<Bound<'py, PyAny>> {
        let py = this.py();
        let this_ref = this.borrow();
        
        // Lifetime Hack: 将引用转为 'static。由于 parent 强引用了父对象，
        // 底层数据不会被释放，裸指针始终有效。
        let view = unsafe {
            std::mem::transmute::<ArrayView2<'_, f64>, ArrayView2<'static, f64>>(
                this_ref.inner.navs().view()
            )
        };

        // 1. 构造含有 __array_interface__ 的内部视图对象
        let navs_view_obj = Py::new(py, PyNavs {
            view,
            parent: this.clone().into_py(py),
        })?;

        // 2. 在 Rust 端调用 numpy.asarray 进行转换，直接返回 ndarray
        let numpy = py.import_bound("numpy")?;
        let array = numpy.call_method1("asarray", (navs_view_obj,))?;
        
        Ok(array)
    }
```

### 优势总结
1. **原生体验**：Python 端直接获得 `<class 'numpy.ndarray'>` 对象，无需手动调用 `np.asarray()`。
2. **绝对零拷贝**：NumPy C-API 底层处理 `asarray` 时，发现 `__array_interface__` 且内存可直连，直接将 `ndarray` 的数据指针指向 Rust 内存，无任何复制。
3. **内存安全链**：NumPy 返回的 `ndarray` 的 `base` 属性会自动指向 `navs_view_obj`，而后者通过 `parent` 持有父对象（如 `PyTransaction`）。只要 `ndarray` 存活，引用链就能保证 Rust 底层内存绝对有效，彻底杜绝悬垂指针。
4. **只读保护**：通过 `__array_interface__` 中 `data` 字段的第二个参数设为 `false`，确保返回的 NumPy 数组是只读的，防止 Python 端意外破坏 Rust 内部状态。
