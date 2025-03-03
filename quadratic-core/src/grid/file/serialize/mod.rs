#![allow(unused)] // TODO: remove this

use anyhow::Result;
use sheets::{export_sheet, import_sheet};

pub use crate::grid::file::v1_7_1 as current;
use crate::grid::Grid;

use super::CURRENT_VERSION;

pub(crate) mod borders;
pub(crate) mod cell_value;
pub(crate) mod code_cell;
pub(crate) mod column;
pub(crate) mod contiguous_2d;
pub(crate) mod formats;
pub(crate) mod row_resizes;
pub(crate) mod selection;
pub mod sheets;
pub(crate) mod validations;

pub fn import(file: current::GridSchema) -> Result<Grid> {
    Ok(Grid {
        sheets: file
            .sheets
            .into_iter()
            .map(import_sheet)
            .collect::<Result<_>>()?,
    })
}

pub fn export(grid: Grid) -> Result<current::GridSchema> {
    Ok(current::GridSchema {
        version: CURRENT_VERSION.into(),
        sheets: grid.sheets.into_iter().map(export_sheet).collect(),
    })
}
