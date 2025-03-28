//! Supports Old borders types used by Operation::SetBordersSelection. Delete
//! when that Operation is removed.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::Pos;
use crate::grid::{ColumnData, block::SameValue};
use crate::{RunLengthEncoding, grid::sheet::borders::BorderStyleTimestamp};

use super::{BorderStyle, BorderStyleCell};

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OldBorders {
    // sheet-wide formatting
    pub(crate) all: BorderStyleCell,
    pub(crate) columns: HashMap<i64, BorderStyleCell>,
    pub(crate) rows: HashMap<i64, BorderStyleCell>,

    // cell-specific formatting (vertical) first key = x-coordinate; column-data key is y-coordinate
    pub(crate) left: HashMap<i64, ColumnData<SameValue<BorderStyleTimestamp>>>,
    pub(crate) right: HashMap<i64, ColumnData<SameValue<BorderStyleTimestamp>>>,

    // cell-specific formatting (horizontal); first key = y-coordinate; column-data key is x-coordinate
    pub(crate) top: HashMap<i64, ColumnData<SameValue<BorderStyleTimestamp>>>,
    pub(crate) bottom: HashMap<i64, ColumnData<SameValue<BorderStyleTimestamp>>>,
}

impl OldBorders {
    /// Sets the border for a cell. This is used in the upgrade_border for going
    /// from v1_6 to v1_7.
    pub fn set(
        &mut self,
        x: i64,
        y: i64,
        top: Option<BorderStyle>,
        bottom: Option<BorderStyle>,
        left: Option<BorderStyle>,
        right: Option<BorderStyle>,
    ) {
        if let Some(top) = top {
            self.top.entry(y).or_default().set(x, Some(top.into()));
        }
        if let Some(bottom) = bottom {
            self.bottom
                .entry(y)
                .or_default()
                .set(x, Some(bottom.into()));
        }
        if let Some(left) = left {
            self.left.entry(x).or_default().set(y, Some(left.into()));
        }
        if let Some(right) = right {
            self.right.entry(x).or_default().set(y, Some(right.into()));
        }
    }
}

pub type BorderStyleCellUpdates = RunLengthEncoding<BorderStyleCellUpdate>;

#[derive(Default, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorderStyleCellUpdate {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub top: Option<Option<BorderStyleTimestamp>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub bottom: Option<Option<BorderStyleTimestamp>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub left: Option<Option<BorderStyleTimestamp>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub right: Option<Option<BorderStyleTimestamp>>,
}

impl BorderStyleCellUpdate {
    /// Create a update that will clear the border. If force_clear is true, then
    /// the border is set to BorderLineStyle::Clear, otherwise the border is set
    /// to None (ie, removed).
    pub fn clear(force_clear: bool) -> Self {
        if force_clear {
            BorderStyleCellUpdate {
                top: Some(Some(BorderStyleTimestamp::clear())),
                bottom: Some(Some(BorderStyleTimestamp::clear())),
                left: Some(Some(BorderStyleTimestamp::clear())),
                right: Some(Some(BorderStyleTimestamp::clear())),
            }
        } else {
            BorderStyleCellUpdate {
                top: Some(None),
                bottom: Some(None),
                left: Some(None),
                right: Some(None),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct SheetBorders {
    pub per_cell: IdSpaceBorders,
    pub render_lookup: GridSpaceBorders,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct IdSpaceBorders {
    pub borders: HashMap<i64, ColumnData<SameValue<CellBorders>>>,
}

impl Serialize for IdSpaceBorders {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let map: HashMap<String, ColumnData<SameValue<CellBorders>>> = self
            .borders
            .iter()
            .map(|(id, idx)| (id.to_string(), idx.to_owned()))
            .collect();
        map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for IdSpaceBorders {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map =
            HashMap::<&'de str, ColumnData<SameValue<CellBorders>>>::deserialize(deserializer)?;
        let mut ret = IdSpaceBorders {
            borders: HashMap::new(),
        };
        for (k, v) in map {
            ret.borders.insert(k.parse::<i64>().unwrap(), v);
        }
        Ok(ret)
    }
}

impl IdSpaceBorders {
    pub fn set_cell_border(&mut self, pos: Pos, side: CellSide, style: Option<BorderStyle>) {
        let column_borders = self.borders.entry(pos.x).or_default();
        let new_borders = CellBorders::combine(column_borders.get(pos.y), side, style);

        if new_borders.is_empty() {
            column_borders.set(pos.y, None);
        } else {
            column_borders.set(pos.y, Some(new_borders));
        }
    }

    pub fn try_get_cell_border(&self, pos: Pos) -> Option<CellBorders> {
        let column_borders = self.borders.get(&pos.x)?;
        column_borders.get(pos.y)
    }

    pub fn get_cell_border(&mut self, pos: Pos) -> Option<CellBorders> {
        let column_borders = self.borders.entry(pos.x).or_default();
        column_borders.get(pos.y)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct GridSpaceBorders {
    pub(super) vertical: HashMap<i64, ColumnData<SameValue<BorderStyle>>>,
    pub(super) horizontal: HashMap<i64, ColumnData<SameValue<BorderStyle>>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "js", derive(ts_rs::TS))]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum CellSide {
    Left = 0,
    Top = 1,
    Right = 2,
    Bottom = 3,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default, Copy)]
pub struct CellBorders {
    pub borders: [Option<BorderStyle>; 4],
}

impl CellBorders {
    pub fn new(borders: &[(CellSide, BorderStyle)]) -> Self {
        let mut as_array = [None; 4];
        for (side, style) in borders {
            as_array[*side as usize] = Some(*style);
        }
        Self { borders: as_array }
    }

    pub(super) fn combine(
        maybe_existing: Option<Self>,
        side: CellSide,
        style: Option<BorderStyle>,
    ) -> Self {
        if let Some(existing) = maybe_existing {
            existing.with_side(side, style)
        } else {
            Self::default().with_side(side, style)
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.borders.iter().all(|style| style.is_none())
    }

    fn with_side(&self, side: CellSide, style: Option<BorderStyle>) -> Self {
        let mut cloned = *self;
        cloned.borders[side as usize] = style;
        cloned
    }
}
