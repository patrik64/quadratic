use std::fmt;
use std::num::NonZeroU32;

use anyhow::Result;
use bigdecimal::BigDecimal;
use itertools::Itertools;
use rand::Rng;
use serde::{Deserialize, Serialize};
use smallvec::{smallvec, SmallVec};

use super::{ArraySize, Axis, CellValue, Spanned, Value};
use crate::controller::operations::operation::Operation;
use crate::grid::Sheet;
use crate::{CodeResult, Pos, RunError, RunErrorMsg, Span};

#[macro_export]
macro_rules! array {
    ($( $( $value:expr ),+ );+ $(;)?) => {{
        let values = [$( [$( $crate::CellValue::from($value) ),+] ),+];
        let height = values.len();
        let width = values[0].len(); // This will generate a compile-time error if there are no values.
        let size = $crate::ArraySize::new(width as u32, height as u32)
            .expect("empty array is not allowed");
        $crate::Array::new_row_major(size, values.into_iter().flatten().collect()).unwrap()
    }};
}

/// 2D array of values in the formula language. The array may be a single value
/// (1x1) but must not be degenerate (zero width or zero height).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Array {
    /// Width and height.
    size: ArraySize,
    /// Flattened array of `width * height` many values, stored in row-major
    /// order.
    values: SmallVec<[CellValue; 1]>,
}
impl fmt::Display for Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        let mut is_first_row = true;
        for row in self.rows() {
            if is_first_row {
                is_first_row = false;
            } else {
                write!(f, "; ")?;
            }
            let mut is_first_value = true;
            for value in row {
                if is_first_value {
                    is_first_value = false;
                } else {
                    write!(f, ", ")?;
                }
                write!(f, "{value}")?; // TODO: consider replacing this with `value.repr()`
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl From<CellValue> for Array {
    fn from(value: CellValue) -> Self {
        Array {
            size: ArraySize::_1X1,
            values: smallvec![value],
        }
    }
}
impl TryFrom<Value> for Array {
    type Error = RunErrorMsg;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.into_array()
    }
}
impl TryFrom<Value> for Vec<Array> {
    type Error = RunErrorMsg;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(value.into_arrays())
    }
}

impl From<Vec<Vec<String>>> for Array {
    fn from(v: Vec<Vec<String>>) -> Self {
        let w = v[0].len();
        let h = v.len();
        Array {
            size: ArraySize::new(w as u32, h as u32).unwrap(),
            values: v
                .iter()
                .flatten()
                .map(|s| CellValue::from(s.as_ref()))
                .collect(),
        }
    }
}

impl From<Vec<Vec<&str>>> for Array {
    fn from(v: Vec<Vec<&str>>) -> Self {
        let w = v[0].len();
        let h = v.len();
        Array {
            size: ArraySize::new(w as u32, h as u32).unwrap(),
            values: v.iter().flatten().map(|s| (*s).into()).collect(),
        }
    }
}

impl From<Vec<Vec<CellValue>>> for Array {
    fn from(v: Vec<Vec<CellValue>>) -> Self {
        if v.is_empty() {
            return Array::new_empty(ArraySize::_1X1);
        }

        let w = v[0].len();
        let h = v.len();
        Array {
            size: ArraySize::new(w as u32, h as u32).unwrap(),
            values: v.into_iter().flatten().collect(),
        }
    }
}

impl Array {
    /// Constructs an array of blank values.
    pub fn new_empty(size: ArraySize) -> Self {
        let values = smallvec![CellValue::Blank; size.len() ];
        Self::new_row_major(size, values).expect("error constructing empty array")
    }
    /// Constructs an array of random float values.
    pub fn from_random_floats(size: ArraySize) -> Self {
        let mut rng = rand::rng();
        let values = std::iter::from_fn(|| {
            Some(CellValue::Number(BigDecimal::from(
                &rng.random_range(-100..=100),
            )))
        })
        .take(size.len())
        .collect();
        Self::new_row_major(size, values).expect("error constructing random float array")
    }
    /// Constructs an array from a list of values in row-major order.
    pub fn new_row_major(size: ArraySize, values: SmallVec<[CellValue; 1]>) -> CodeResult<Self> {
        if values.len() == size.len() {
            Ok(Self { size, values })
        } else {
            internal_error!(
                "bad array dimensions: {size} needs {} values, but got {}",
                size.len(),
                values.len(),
            )
        }
    }
    /// Returns a formula-source-code representation of the value.
    pub fn repr(&self) -> String {
        format!(
            "{{{}}}",
            self.rows()
                .map(|row| row.iter().map(|v| v.repr()).join(", "))
                .join("; "),
        )
    }

    /// Iterates over rows (if `axis` is `Axis::Y`) or columns (if `axis` is
    /// `Axis::X`).
    pub fn slices(&self, axis: Axis) -> impl Iterator<Item = Vec<&CellValue>> {
        (0..self.size()[axis].get()).map(move |i| {
            (0..self.size()[axis.other_axis()].get())
                .filter_map(|j| match axis {
                    Axis::X => self.get(i, j).ok(),
                    Axis::Y => self.get(j, i).ok(),
                })
                .collect()
        })
    }
    /// Constructs an array from rows (if `axis` is `Axis::Y`) or columns (if
    /// `axis` is `Axis::X`). All rows/columns must have the same length, or
    /// else the result is undefined. Returns `None` if `slices` is empty or if
    /// each slice is empty.
    pub fn from_slices<'a>(
        span: Span,
        axis: Axis,
        slices: impl IntoIterator<Item = Vec<&'a CellValue>>,
    ) -> CodeResult<Self> {
        Self::try_from_slices(axis, slices).ok_or(RunErrorMsg::EmptyArray.with_span(span))
    }
    fn try_from_slices<'a>(
        axis: Axis,
        slices: impl IntoIterator<Item = Vec<&'a CellValue>>,
    ) -> Option<Self> {
        let slices = slices.into_iter().collect_vec();
        let main_len = slices.len() as u32;
        let other_len = slices.first()?.len() as u32;
        let size = ArraySize::new(other_len, main_len)?;
        let a = Self::new_row_major(size, slices.into_iter().flatten().cloned().collect()).ok();
        match axis {
            Axis::X => a.map(|a| a.transpose()),
            Axis::Y => a,
        }
    }

    /// Transposes an array (swaps rows and columns). This is an expensive
    /// operation for large arrays.
    pub fn transpose(&self) -> Array {
        let new_size = self.size.transpose();
        let values = new_size
            .iter()
            .map(|(x, y)| self.get(y, x).unwrap().clone())
            .collect();
        Self::new_row_major(new_size, values).unwrap()
    }
    /// Flips an array horizontally. This is an expensive operation for large
    /// arrays.
    pub fn flip_horizontally(&self) -> Array {
        Self::new_row_major(
            self.size(),
            self.rows()
                .flat_map(|row| row.iter().rev().cloned())
                .collect(),
        )
        .unwrap()
    }
    /// Flips an array vertically. This is an expensive operation for large
    /// arrays.
    pub fn flip_vertically(&self) -> Array {
        Self::new_row_major(self.size, self.rows().rev().flatten().cloned().collect()).unwrap()
    }

    /// Returns the width of an array.
    pub fn width(&self) -> u32 {
        self.size.w.get()
    }
    /// Returns the height of an array.
    pub fn height(&self) -> u32 {
        self.size.h.get()
    }
    /// Returns the width and height of an array.
    pub fn size(&self) -> ArraySize {
        self.size
    }
    /// Returns an iterator over the rows of the array.
    pub fn rows(&self) -> std::slice::Chunks<'_, CellValue> {
        self.values.chunks(self.width() as usize)
    }

    /// Returns the only cell value in a 1x1 array, or an error if this is not a
    /// 1x1 array.
    pub fn into_cell_value(self) -> Result<CellValue, Self> {
        if self.values.len() == 1 {
            Ok(self.values.into_iter().next().unwrap())
        } else {
            Err(self)
        }
    }
    /// Returns a reference to the only cell value in a 1x1 array, or an error
    /// if this is not a 1x1 array.
    pub fn cell_value(&self) -> Option<&CellValue> {
        if self.values.len() == 1 {
            self.values.first()
        } else {
            None
        }
    }
    /// Returns the value at a given 0-indexed position in an array. If the
    /// width is 1, then `x` is ignored. If the height is 1, then `y` is
    /// ignored. Otherwise, returns an error if a coordinate is out of bounds.
    pub fn get(&self, x: u32, y: u32) -> Result<&CellValue, RunErrorMsg> {
        let i = self.size().flatten_index(x, y)?;
        Ok(&self.values[i])
    }
    /// Sets the value at a given 0-indexed position in an array. Returns an
    /// error if `x` or `y` is out of range.
    pub fn set(&mut self, x: u32, y: u32, value: CellValue) -> Result<(), RunErrorMsg> {
        let i = self.size().flatten_index(x, y)?;
        self.values[i] = value;
        Ok(())
    }
    /// Returns a flat slice of cell values in the array.
    pub fn cell_values_slice(&self) -> &[CellValue] {
        &self.values
    }
    pub fn cell_values_slice_mut(&mut self) -> &mut [CellValue] {
        &mut self.values
    }
    /// Returns a flat `SmallVec` of cell values in the array.
    pub fn into_cell_values_vec(self) -> SmallVec<[CellValue; 1]> {
        self.values
    }

    /// Returns a human-friendly string describing the type of value.
    pub fn type_name(&self) -> &'static str {
        match self.cell_value() {
            Some(v) => v.type_name(),
            None => "array",
        }
    }
    /// Returns the unique length that fits all `values` along `axis`. See
    /// `common_array_size()` for more.
    pub fn common_len<'a>(
        axis: Axis,
        arrays: impl IntoIterator<Item = Spanned<&'a Array>>,
    ) -> CodeResult<NonZeroU32> {
        let mut common_len = 1;

        for array in arrays {
            let new_array_len = array.inner.size()[axis].get();
            match (common_len, new_array_len) {
                (a, b) if a == b => continue,
                (_, 1) => continue,
                (1, l) => common_len = l,
                _ => {
                    return Err(RunErrorMsg::ArrayAxisMismatch {
                        axis,
                        expected: common_len,
                        got: new_array_len,
                    }
                    .with_span(array.span))
                }
            }
        }

        Ok(NonZeroU32::new(common_len).expect("bad array size"))
    }

    /// Returns the first error in the array if there is one.
    pub fn first_error(&self) -> Option<&RunError> {
        self.values.iter().find_map(|v| v.error())
    }
    /// Iterates over errors in the array.
    pub fn errors(&self) -> impl Iterator<Item = &RunError> {
        self.values.iter().filter_map(|v| v.error())
    }
    /// Returns the first error in the array if there is one; otherwise returns
    /// the original array.
    pub fn into_non_error_array(self) -> CodeResult<Self> {
        match self.first_error() {
            Some(e) => Err(e.clone()),
            None => Ok(self),
        }
    }

    pub fn from_string_list(
        start: Pos,
        sheet: &mut Sheet,
        v: Vec<Vec<Vec<String>>>,
    ) -> (Option<Array>, Vec<Operation>) {
        // catch the unlikely case where we receive an array of empty arrays
        if v[0].is_empty() {
            return (None, vec![]);
        }
        let size = ArraySize::new(v[0].len() as u32, v.len() as u32).unwrap();
        let mut ops = vec![];
        let Pos { mut x, mut y } = start;
        let values = v
            .iter()
            .flatten()
            .map(|s| {
                x += 1;
                if x == v[0].len() as i64 + start.x {
                    x = start.x;
                    y += 1;
                }
                match CellValue::from_js(&s[0], &s[1], start, sheet) {
                    Ok(value) => value,
                    Err(_) => (CellValue::Blank, vec![]),
                }
            })
            // .flatten_ok()
            .map(|(value, updated_ops)| {
                ops.extend(updated_ops);
                value
            })
            .collect::<SmallVec<[CellValue; 1]>>();

        (Some(Array { size, values }), ops)
    }
}

impl Spanned<Array> {
    /// Checks that an array is linear (width=1 or height=1), then returns which
    /// is the long axis. Returns `None` in the case of a 1x1 array.
    pub fn array_linear_axis(&self) -> CodeResult<Option<Axis>> {
        match (self.inner.width(), self.inner.height()) {
            (1, 1) => Ok(None),
            (_, 1) => Ok(Some(Axis::X)), // height = 1
            (1, _) => Ok(Some(Axis::Y)), // width = 1
            _ => Err(RunErrorMsg::NonLinearArray.with_span(self.span)),
        }
    }
    /// Checks that an array is linear along a particular axis, then returns the
    /// length along that axis.
    pub fn array_linear_length(&self, axis: Axis) -> CodeResult<NonZeroU32> {
        self.check_array_size_on(axis.other_axis(), 1)?;
        Ok(self.inner.size()[axis])
    }
    /// Checks that an array is linear (width=1 or height=1), then returns it if
    /// it is.
    pub fn try_as_linear_array(&self) -> CodeResult<&[CellValue]> {
        self.array_linear_axis()?; // Check that the array is linear.
        Ok(&self.inner.values)
    }

    /// Checks the size of the array on one axis, returning an error if it does
    /// not match exactly.
    pub fn check_array_size_on(&self, axis: Axis, len: u32) -> CodeResult<()> {
        let expected = len;
        let got = self.inner.size()[axis].get();
        if expected == got {
            Ok(())
        } else {
            Err(RunErrorMsg::ExactArrayAxisMismatch {
                axis,
                expected,
                got,
            }
            .with_span(self.span))
        }
    }

    /// Checks the size of the array, returning an error if it does not match
    /// exactly.
    pub fn check_array_size_exact(&self, size: ArraySize) -> CodeResult<()> {
        let expected = size;
        let got = self.inner.size();
        if expected == got {
            Ok(())
        } else {
            Err(RunErrorMsg::ExactArraySizeMismatch { expected, got }.with_span(self.span))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_string_list_empty() {
        let mut sheet = Sheet::test();
        assert_eq!(
            Array::from_string_list(Pos { x: 0, y: 0 }, &mut sheet, vec![vec![], vec![]]),
            (None, vec![])
        );
    }
}
