pub use bounds::GridBounds;
pub use cells_accessed::*;
pub use code_cell::*;
pub use code_run::*;
pub use column::{Column, ColumnData};
pub use contiguous::{Block, Contiguous2D, ContiguousBlocks};
pub use data_table::*;
pub use formats::Format;
pub use formatting::{
    Bold, CellAlign, CellVerticalAlign, CellWrap, FillColor, Italic, NumericCommas,
    NumericDecimals, NumericFormat, NumericFormatKind, StrikeThrough, TextColor, Underline,
};
pub use ids::*;
use indexmap::IndexMap;
pub use region_map::RegionMap;
use serde::{Deserialize, Serialize};
pub use sheet::Sheet;
pub use sheet_formatting::SheetFormatting;
pub use sheet_region_map::SheetRegionMap;

use crate::CellValue;
#[cfg(test)]
use crate::{Array, Pos};

mod a1_context;
pub mod ai;
mod block;
mod bounds;
mod cells_accessed;
mod cells_accessed_cache;
mod code_cell;
mod code_run;
pub mod column;
pub mod contiguous;
pub mod data_table;
pub mod file;
pub mod formats;
pub mod formatting;
mod ids;
pub mod js_types;
mod region_map;
pub mod resize;
pub mod search;
pub mod selection;
pub mod series;
pub mod sheet;
pub mod sheet_formatting;
mod sheet_region_map;
pub mod sheets;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Grid {
    pub sheets: IndexMap<SheetId, Sheet>,
}
impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}
impl Grid {
    pub fn new() -> Self {
        let mut ret = Grid::new_blank();
        ret.add_sheet(None);
        ret
    }
    pub fn new_blank() -> Self {
        Grid {
            sheets: IndexMap::new(),
        }
    }

    /// Creates a grid for testing.
    pub fn test() -> Self {
        let mut ret = Grid::new_blank();
        let sheet = Sheet::test();
        ret.add_sheet(Some(sheet));
        ret
    }

    #[cfg(test)]
    pub fn from_array(base_pos: Pos, array: &Array) -> Self {
        let mut ret = Grid::new();
        let sheet = ret.first_sheet_mut();
        for ((x, y), value) in array.size().iter().zip(array.cell_values_slice()) {
            let x = base_pos.x + x as i64;
            let y = base_pos.y + y as i64;
            let _ = sheet.set_cell_value(Pos { x, y }, value.clone());
        }
        ret
    }

    #[cfg(test)]
    pub fn origin_in_first_sheet(&self) -> crate::SheetPos {
        crate::Pos::ORIGIN.to_sheet_pos(self.sheets()[0].id)
    }
}
