use std::collections::HashSet;

use anyhow::{Result, anyhow};
use bigdecimal::RoundingMode;
use borders::Borders;
use columns::SheetColumns;
use data_tables::SheetDataTables;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use validations::Validations;

use super::bounds::GridBounds;
use super::column::Column;
use super::ids::SheetId;
use super::js_types::{CellFormatSummary, CellType, JsCellValue, JsCellValuePos};
use super::resize::ResizeMap;
use super::{CellWrap, Format, NumericFormatKind, SheetFormatting};
use crate::a1::{A1Context, A1Selection, CellRefRange};
use crate::sheet_offsets::SheetOffsets;
use crate::{CellValue, Pos, Rect};

pub mod a1_context;
pub mod a1_selection;
pub mod ai_context;
pub mod borders;
pub mod bounds;
pub mod cell_array;
pub mod cell_values;
pub mod cells_accessed_cache;
pub mod clipboard;
pub mod code;
pub mod col_row;
pub mod columns;
pub mod data_table;
pub mod data_tables;
pub mod formats;
pub mod keyboard;
pub mod rendering;
pub mod rendering_date_time;
pub mod row_resize;
pub mod search;
#[cfg(test)]
pub mod sheet_test;
pub mod summarize;
pub mod validations;

const SHEET_NAME_VALID_CHARS: &str = r#"^[a-zA-Z0-9_\-(][a-zA-Z0-9_\- .()\p{Pd}]*[a-zA-Z0-9_\-)]$"#;
lazy_static! {
    static ref SHEET_NAME_VALID_CHARS_COMPILED: Regex =
        Regex::new(SHEET_NAME_VALID_CHARS).expect("Failed to compile SHEET_NAME_VALID_CHARS");
}

/// Sheet in a file.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Sheet {
    pub id: SheetId,
    pub name: String,
    pub color: Option<String>,
    pub order: String,

    pub offsets: SheetOffsets,

    pub columns: SheetColumns,

    pub data_tables: SheetDataTables,

    /// Formatting for the entire sheet.
    pub formats: SheetFormatting,

    pub validations: Validations,

    // bounds for the grid with only data
    pub(super) data_bounds: GridBounds,

    // bounds for the grid with only formatting
    pub(super) format_bounds: GridBounds,

    pub(super) rows_resize: ResizeMap,

    pub borders: Borders,
}
impl Sheet {
    /// Constructs a new empty sheet.
    pub fn new(id: SheetId, name: String, order: String) -> Self {
        Sheet {
            id,
            name,
            color: None,
            order,
            offsets: SheetOffsets::default(),
            columns: SheetColumns::new(),
            data_tables: SheetDataTables::new(),
            formats: SheetFormatting::default(),
            data_bounds: GridBounds::Empty,
            format_bounds: GridBounds::Empty,
            validations: Validations::default(),
            rows_resize: ResizeMap::default(),
            borders: Borders::default(),
        }
    }

    /// Creates a sheet for testing.
    pub fn test() -> Self {
        Sheet::new(SheetId::TEST, String::from("Sheet 1"), String::from("a0"))
    }

    /// Returns an error if a sheet name would be invalid to add.
    pub fn validate_sheet_name(
        name: &str,
        sheet_id: SheetId,
        a1_context: &A1Context,
    ) -> Result<(), String> {
        // Check length limit
        if name.is_empty() || name.len() > 31 {
            return Err("Sheet name must be between 1 and 31 characters".to_string());
        }

        // Validate characters using regex pattern
        if !SHEET_NAME_VALID_CHARS_COMPILED.is_match(name) {
            return Err("Sheet name contains invalid characters".to_string());
        }

        // Check if sheet name already exists
        if let Some(existing_sheet_id) = a1_context.sheet_map.try_sheet_name(name) {
            if existing_sheet_id != sheet_id {
                return Err("Sheet name must be unique".to_string());
            }
        }

        Ok(())
    }

    /// Replaces a sheet name when referenced in code cells.
    pub fn replace_sheet_name_in_code_cells(&mut self, old_name: &str, new_name: &str) {
        let sheet_id = SheetId::new();
        let old_a1_context = A1Context::with_single_sheet(old_name, sheet_id);
        let new_a1_context = A1Context::with_single_sheet(new_name, sheet_id);
        self.replace_names_in_code_cells(&old_a1_context, &new_a1_context);
    }

    /// Replaces any number of sheet names and table names when referenced in
    /// code cells.
    pub fn replace_names_in_code_cells(
        &mut self,
        old_a1_context: &A1Context,
        new_a1_context: &A1Context,
    ) {
        self.update_code_cells(|code_cell_value, pos| {
            code_cell_value.replace_sheet_name_in_cell_references(
                old_a1_context,
                new_a1_context,
                pos,
            );
        });
    }

    /// Returns true if the cell at Pos has content (ie, not blank). Also checks
    /// tables. Ignores Blanks except in tables.
    pub fn has_content(&self, pos: Pos) -> bool {
        if self
            .get_column(pos.x)
            .and_then(|column| column.values.get(&pos.y))
            .is_some_and(|cell_value| !cell_value.is_blank_or_empty_string())
        {
            return true;
        }
        self.has_table_content(pos, false)
    }

    /// Returns true if the cell at Pos has content (ie, not blank). Ignores
    /// Blanks in tables.
    pub fn has_content_ignore_blank_table(&self, pos: Pos) -> bool {
        if self
            .get_column(pos.x)
            .and_then(|column| column.values.get(&pos.y))
            .is_some_and(|cell_value| !cell_value.is_blank_or_empty_string())
        {
            return true;
        }
        self.has_table_content_ignore_blanks(pos)
    }

    /// Returns true if the cell at Pos is at a vertical edge of a table.
    pub fn is_at_table_edge_col(&self, pos: &Pos) -> bool {
        if let Some((dt_pos, dt)) = self.data_table_that_contains(pos) {
            // we handle charts separately in find_next_*;
            // we ignore single_value tables
            if dt.is_html_or_image() || dt.is_single_value() {
                return false;
            }
            let bounds = dt.output_rect(dt_pos, false);
            if bounds.min.x == pos.x || bounds.max.x == pos.x {
                return true;
            }
        }
        false
    }

    /// Returns true if the cell at Pos is at a horizontal edge of a table.
    pub fn is_at_table_edge_row(&self, pos: &Pos) -> bool {
        if let Some((dt_pos, dt)) = self.data_table_that_contains(pos) {
            // we handle charts separately in find_next_*;
            // we ignore single_value tables
            if dt.is_html_or_image() || dt.is_single_value() {
                return false;
            }
            let bounds = dt.output_rect(dt_pos, false);
            if bounds.min.y == pos.y {
                // table name, or column header if no table name, or top of data if no column header or table name
                return true;
            }

            let show_name = dt.get_show_name();
            let show_columns = dt.get_show_columns();

            if bounds.min.y + (if show_name { 1 } else { 0 }) + (if show_columns { 1 } else { 0 })
                == pos.y
            {
                // ignore column header--just go to first line of data or table name
                return true;
            } else if bounds.max.y == pos.y {
                return true;
            }
        }
        false
    }

    /// Returns the cell_value at a Pos using both column.values and data_tables (i.e., what would be returned if code asked
    /// for it).
    pub fn display_value(&self, pos: Pos) -> Option<CellValue> {
        let cell_value = self
            .get_column(pos.x)
            .and_then(|column| column.values.get(&pos.y));

        // if CellValue::Code or CellValue::Import, then we need to get the value from data_tables
        if let Some(cell_value) = cell_value {
            if !matches!(
                cell_value,
                CellValue::Code(_) | CellValue::Import(_) | CellValue::Blank
            ) {
                return Some(cell_value.clone());
            }
        }

        // if there is no CellValue at Pos, then we still need to check data_tables
        self.get_code_cell_value(pos)
    }

    /// Returns the JsCellValue at a position
    pub fn js_cell_value(&self, pos: Pos) -> Option<JsCellValue> {
        self.display_value(pos).map(|value| JsCellValue {
            value: value.to_string(),
            kind: value.type_name().to_string(),
        })
    }

    /// Returns the JsCellValuePos at a position
    pub fn js_cell_value_pos(&self, pos: Pos) -> Option<JsCellValuePos> {
        self.display_value(pos).map(|cell_value| match cell_value {
            CellValue::Image(_) => {
                CellValue::Image("Javascript chart".into()).to_cell_value_pos(pos)
            }
            CellValue::Html(_) => CellValue::Html("Python chart".into()).to_cell_value_pos(pos),
            _ => cell_value.to_cell_value_pos(pos),
        })
    }

    /// Returns the JsCellValuePos in a rect
    pub fn js_cell_value_pos_in_rect(
        &self,
        rect: Rect,
        max_rows: Option<u32>,
    ) -> Vec<Vec<JsCellValuePos>> {
        let mut rect_values = Vec::new();
        for y in rect
            .y_range()
            .take(max_rows.unwrap_or(rect.height()) as usize)
        {
            let mut row_values = Vec::new();
            for x in rect.x_range() {
                if let Some(cell_value_pos) = self.js_cell_value_pos((x, y).into()) {
                    row_values.push(cell_value_pos);
                }
            }
            if !row_values.is_empty() {
                rect_values.push(row_values);
            }
        }
        rect_values
    }

    /// Returns the cell_value at the Pos in column.values. This does not check or return results within code_runs.
    pub fn cell_value(&self, pos: Pos) -> Option<CellValue> {
        let column = self.get_column(pos.x)?;
        column.values.get(&pos.y).cloned()
    }

    /// Returns the ref of the cell_value at the Pos in column.values. This does
    /// not check or return results within data_tables.
    pub fn cell_value_ref(&self, pos: Pos) -> Option<&CellValue> {
        let column = self.get_column(pos.x)?;
        column.values.get(&pos.y)
    }

    /// Returns the cell value at a position, or an error if the cell value is not found.
    pub fn cell_value_result(&self, pos: Pos) -> Result<CellValue> {
        self.cell_value(pos)
            .ok_or_else(|| anyhow!("Cell value not found at {:?}", pos))
    }

    /// Returns a mutable reference to the cell value at the Pos in column.values.
    pub fn cell_value_mut(&mut self, pos: Pos) -> Option<&mut CellValue> {
        self.columns.get_value_mut(&pos)
    }

    /// Returns the cell value at a position using both `column.values` and
    /// `data_tables`, for use when a formula references a cell.
    pub fn get_cell_for_formula(&self, pos: Pos) -> CellValue {
        let cell_value = self
            .get_column(pos.x)
            .and_then(|column| column.values.get(&pos.y));

        if let Some(cell_value) = cell_value {
            match cell_value {
                CellValue::Blank | CellValue::Code(_) | CellValue::Import(_) => {
                    match self.data_tables.get_at(&pos) {
                        Some(data_table) => data_table.get_cell_for_formula(
                            0,
                            if data_table.header_is_first_row { 1 } else { 0 },
                        ),
                        None => CellValue::Blank,
                    }
                }
                other => other.clone(),
            }
        } else if let Some(value) = self.get_code_cell_value(pos) {
            if matches!(value, CellValue::Html(_) | CellValue::Image(_)) {
                CellValue::Blank
            } else {
                value
            }
        } else {
            CellValue::Blank
        }
    }

    /// Returns the type of number (defaulting to NumericFormatKind::Number) for a cell.
    pub fn cell_numeric_format_kind(&self, pos: Pos) -> NumericFormatKind {
        match self.formats.numeric_format.get(pos) {
            Some(format) => format.kind,
            None => NumericFormatKind::Number,
        }
    }

    /// Returns the format of a cell taking into account the sheet and data_tables formatting.
    pub fn cell_format(&self, pos: Pos) -> Format {
        let sheet_format = self.formats.try_format(pos).unwrap_or_default();

        if let Ok(data_table_pos) = self.data_table_pos_that_contains(&pos) {
            if let Some(data_table) = self.data_table_at(&data_table_pos) {
                if !data_table.has_spill() && !data_table.has_error() {
                    // pos relative to data table pos (top left pos)
                    let format_pos = pos.translate(-data_table_pos.x, -data_table_pos.y, 0, 0);
                    let table_format = data_table.get_format(format_pos);
                    let combined_format = table_format.combine(&sheet_format);
                    return combined_format;
                }
            }
        }

        sheet_format
    }

    /// Returns a string representation of the format of a cell for use by AI.
    pub fn cell_text_format_as_string(&self, pos: Pos) -> Option<String> {
        let format = self.cell_format(pos);
        if format.is_default() {
            None
        } else {
            let mut values = vec![];
            if format.bold.is_some_and(|b| b) {
                values.push("bold".to_string());
            }
            if format.italic.is_some_and(|i| i) {
                values.push("italic".to_string());
            }
            if format.underline.is_some_and(|u| u) {
                values.push("underline".to_string());
            }
            if format.strike_through.is_some_and(|s| s) {
                values.push("strike through".to_string());
            }
            if let Some(text_color) = format.text_color {
                if !text_color.is_empty() {
                    values.push(format!("text color is {}", text_color.clone()));
                }
            }
            if let Some(fill_color) = format.fill_color {
                if !fill_color.is_empty() {
                    values.push(format!("fill color is {}", fill_color.clone()));
                }
            }
            if let Some(align) = format.align {
                values.push(format!("horizontal align is {}", align.clone()));
            }
            if let Some(vertical_align) = format.vertical_align {
                values.push(format!("vertical align is {}", vertical_align.clone()));
            }
            if let Some(wrap) = format.wrap {
                values.push(format!("wrap is {}", wrap.clone()));
            }
            if let Some(numeric_format) = format.numeric_format {
                values.push(format!("numeric kind is {}", numeric_format.kind));
                if let Some(symbol) = numeric_format.symbol {
                    values.push(format!("numeric symbol is {}", symbol));
                }
            }
            if let Some(numeric_decimals) = format.numeric_decimals {
                values.push(format!("numeric decimals is {}", numeric_decimals));
            }
            if let Some(numeric_commas) = format.numeric_commas {
                values.push(format!("numeric commas is {}", numeric_commas));
            }
            if let Some(date_time) = format.date_time {
                values.push(format!("date time is {}", date_time.clone()));
            }

            Some(values.join(", "))
        }
    }

    /// Returns a summary of formatting in a region.
    pub fn cell_format_summary(&self, pos: Pos) -> CellFormatSummary {
        let format = self.cell_format(pos);
        let cell_type = self
            .display_value(pos)
            .and_then(|cell_value| match cell_value {
                CellValue::Date(_) => Some(CellType::Date),
                CellValue::DateTime(_) => Some(CellType::DateTime),
                _ => None,
            });
        CellFormatSummary {
            bold: format.bold,
            italic: format.italic,
            text_color: format.text_color,
            fill_color: format.fill_color,
            commas: format.numeric_commas,
            align: format.align,
            vertical_align: format.vertical_align,
            wrap: format.wrap,
            date_time: format.date_time,
            cell_type,
            underline: format.underline,
            strike_through: format.strike_through,
        }
    }

    /// Returns a column of a sheet from the column index.
    pub(crate) fn get_column(&self, index: i64) -> Option<&Column> {
        self.columns.get_column(index)
    }

    pub fn id_to_string(&self) -> String {
        self.id.to_string()
    }

    /// get or calculate decimal places for a cell
    pub fn calculate_decimal_places(&self, pos: Pos, kind: NumericFormatKind) -> Option<i16> {
        // first check if numeric_decimals already exists for this cell
        if let Some(decimals) = self.cell_format(pos).numeric_decimals {
            return Some(decimals);
        }

        // if currency and percentage, then use the default 2 decimal places
        if kind == NumericFormatKind::Currency || kind == NumericFormatKind::Percentage {
            return Some(2);
        }

        // otherwise check value to see if it has a decimal and use that length
        if let Some(value) = self.display_value(pos) {
            match value {
                CellValue::Number(n) => {
                    if kind == NumericFormatKind::Exponential {
                        return Some(n.to_string().len() as i16 - 1);
                    }

                    let exponent = n.as_bigint_and_exponent().1;
                    let max_decimals = 9;
                    let mut decimals = n
                        .with_scale_round(exponent.min(max_decimals), RoundingMode::HalfUp)
                        .normalized()
                        .as_bigint_and_exponent()
                        .1 as i16;

                    if kind == NumericFormatKind::Percentage {
                        decimals -= 2;
                    }

                    Some(decimals)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    /// Returns true if the cell at Pos has wrap formatting.
    pub fn check_if_wrap_in_cell(&self, pos: Pos) -> bool {
        if !self.has_content(pos) {
            return false;
        }
        self.formats.wrap.get(pos) == Some(CellWrap::Wrap)
    }

    pub fn check_if_wrap_in_row(&self, y: i64) -> bool {
        self.formats.wrap.any_in_row(y, |wrap| {
            let pos = Pos { x: 1, y };
            self.has_content(pos) && *wrap == Some(CellWrap::Wrap)
        })
    }

    pub fn get_rows_with_wrap_in_column(&self, x: i64) -> Vec<i64> {
        let mut rows = vec![];
        if let Some((start, end)) = self.column_bounds(x, true) {
            for y in start..=end {
                if self.has_content(Pos { x, y })
                    && self
                        .formats
                        .wrap
                        .get(Pos { x, y })
                        .is_some_and(|wrap| wrap == CellWrap::Wrap)
                {
                    rows.push(y);
                }
            }
        }
        rows
    }

    pub fn get_rows_with_wrap_in_rect(&self, rect: &Rect, include_blanks: bool) -> Vec<i64> {
        let mut rows = vec![];
        for y in rect.y_range() {
            for x in rect.x_range() {
                if (include_blanks || self.has_content(Pos { x, y }))
                    && self
                        .formats
                        .wrap
                        .get((x, y).into())
                        .is_some_and(|wrap| wrap == CellWrap::Wrap)
                {
                    rows.push(y);
                    break;
                }
            }
        }
        rows
    }

    pub fn get_rows_with_wrap_in_selection(
        &self,
        selection: &A1Selection,
        include_blanks: bool,
        ignore_formatting: bool,
        a1_context: &A1Context,
    ) -> Vec<i64> {
        let mut rows_set = HashSet::<i64>::new();
        selection.ranges.iter().for_each(|range| {
            if let Some(rect) = match range {
                CellRefRange::Sheet { range } => {
                    Some(self.ref_range_bounds_to_rect(range, ignore_formatting))
                }
                CellRefRange::Table { range } => {
                    self.table_ref_to_rect(range, false, false, a1_context)
                }
            } {
                let rows = self.get_rows_with_wrap_in_rect(&rect, include_blanks);
                rows_set.extend(rows);
            }
        });
        rows_set.into_iter().collect()
    }

    /// Sets a cell value and returns the old cell value. Returns `None` if the cell was deleted
    /// and did not previously exist (so no change is needed).
    #[cfg(test)]
    pub fn set_cell_value(&mut self, pos: Pos, value: impl Into<CellValue>) -> Option<CellValue> {
        self.columns.set_value(&pos, value)
    }

    /// Populates the current sheet with random values
    /// Should only be used for testing (as it will not propagate in multiplayer)
    #[cfg(test)]
    pub fn random_numbers(&mut self, rect: &Rect, a1_context: &A1Context) {
        use std::str::FromStr;

        use bigdecimal::BigDecimal;
        use rand::Rng;

        self.columns.clear();
        let mut rng = rand::rng();
        for x in rect.x_range() {
            for y in rect.y_range() {
                let value = rng.random_range(-10000..=10000).to_string();
                self.set_cell_value(
                    (x, y).into(),
                    CellValue::Number(BigDecimal::from_str(&value).unwrap()),
                );
            }
        }
        self.recalculate_bounds(a1_context);
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use bigdecimal::BigDecimal;
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

    use super::*;
    use crate::a1::A1Selection;
    use crate::controller::GridController;
    use crate::grid::{
        CodeCellLanguage, CodeCellValue, CodeRun, DataTable, DataTableKind, NumericFormat,
    };
    use crate::test_util::print_table_in_rect;
    use crate::{Array, SheetPos, SheetRect, Value};

    fn test_setup(selection: &Rect, vals: &[&str]) -> (GridController, SheetId) {
        let mut grid_controller = GridController::test();
        let sheet_id = grid_controller.grid().sheets()[0].id;
        let mut count = 0;

        for y in selection.y_range() {
            for x in selection.x_range() {
                let sheet_pos = SheetPos { x, y, sheet_id };
                grid_controller.set_cell_value(sheet_pos, vals[count].to_string(), None);
                count += 1;
            }
        }

        (grid_controller, sheet_id)
    }

    fn test_setup_basic() -> (GridController, SheetId, Rect) {
        let vals = vec!["1", "2", "3", "4", "5", "6", "7", "8"];
        let selected = Rect::new_span(Pos { x: 2, y: 1 }, Pos { x: 5, y: 2 });
        let (grid_controller, sheet_id) = test_setup(&selected, &vals);

        (grid_controller, sheet_id, selected)
    }

    // assert decimal places after a set_cell_value
    fn assert_decimal_places_for_number(
        sheet: &mut Sheet,
        x: i64,
        y: i64,
        value: &str,
        kind: NumericFormatKind,
        expected: Option<i16>,
    ) {
        let pos = Pos { x, y };
        let _ = sheet.set_cell_value(pos, CellValue::Number(BigDecimal::from_str(value).unwrap()));
        assert_eq!(sheet.calculate_decimal_places(pos, kind), expected);
    }

    #[test]
    fn test_current_decimal_places_value() {
        let mut sheet = Sheet::new(SheetId::new(), String::from(""), String::from(""));

        // validate simple decimal places
        assert_decimal_places_for_number(
            &mut sheet,
            1,
            2,
            "12.23",
            NumericFormatKind::Number,
            Some(2),
        );

        // validate percentage
        assert_decimal_places_for_number(
            &mut sheet,
            2,
            2,
            "0.23",
            NumericFormatKind::Percentage,
            Some(2),
        );

        // validate rounding
        assert_decimal_places_for_number(
            &mut sheet,
            3,
            2,
            "9.1234567891",
            NumericFormatKind::Number,
            Some(9),
        );

        // validate percentage rounding
        assert_decimal_places_for_number(
            &mut sheet,
            3,
            2,
            "9.1234567891",
            NumericFormatKind::Percentage,
            Some(2),
        );

        assert_decimal_places_for_number(
            &mut sheet,
            3,
            2,
            "9.1234567891",
            NumericFormatKind::Currency,
            Some(2),
        );

        assert_decimal_places_for_number(
            &mut sheet,
            3,
            2,
            "91234567891",
            NumericFormatKind::Exponential,
            Some(10),
        );
    }

    #[test]
    fn decimal_places() {
        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];

        let sheet = gc.sheet_mut(sheet_id);
        sheet
            .formats
            .numeric_decimals
            .set_rect(3, 1, Some(3), None, Some(2));
        assert_eq!(
            sheet.calculate_decimal_places(Pos { x: 3, y: 3 }, NumericFormatKind::Number),
            Some(2)
        );

        sheet
            .formats
            .numeric_decimals
            .set(Pos { x: 3, y: 3 }, Some(3));
        assert_eq!(
            sheet.calculate_decimal_places(Pos { x: 3, y: 3 }, NumericFormatKind::Number),
            Some(3)
        );
        assert_eq!(
            sheet.calculate_decimal_places(Pos { x: 3, y: 3 }, NumericFormatKind::Percentage),
            Some(3)
        );
        assert_eq!(
            sheet.calculate_decimal_places(Pos { x: 3, y: 3 }, NumericFormatKind::Currency),
            Some(3)
        );
    }

    #[test]
    fn test_current_decimal_places_text() {
        let mut sheet = Sheet::new(SheetId::new(), String::from(""), String::from(""));

        let _ = sheet.set_cell_value(
            crate::Pos { x: 1, y: 2 },
            CellValue::Text(String::from("abc")),
        );

        assert_eq!(
            sheet.calculate_decimal_places(Pos { x: 1, y: 2 }, NumericFormatKind::Number),
            None
        );
    }

    #[test]
    fn test_current_decimal_places_float() {
        let mut sheet = Sheet::new(SheetId::new(), String::from(""), String::from(""));

        sheet.set_cell_value(
            crate::Pos { x: 1, y: 2 },
            CellValue::Number(BigDecimal::from_str("11.100000000000000000").unwrap()),
        );

        // expect a single decimal place
        assert_eq!(
            sheet.calculate_decimal_places(Pos { x: 1, y: 2 }, NumericFormatKind::Number),
            Some(1)
        );
    }

    #[test]
    fn test_cell_numeric_format_kind() {
        let mut sheet = Sheet::test();

        sheet.formats.numeric_format.set(
            pos![A1],
            Some(NumericFormat {
                kind: NumericFormatKind::Percentage,
                symbol: None,
            }),
        );

        assert_eq!(
            sheet.cell_numeric_format_kind(pos![A1]),
            NumericFormatKind::Percentage
        );
    }

    #[test]
    fn test_set_cell_values() {
        let selected: Rect = Rect::new_span(Pos { x: 2, y: 1 }, Pos { x: 4, y: 1 });
        let vals = vec!["a", "1", "$1.11"];
        let expected = [
            CellValue::Text("a".into()),
            CellValue::Number(BigDecimal::from_str("1").unwrap()),
            CellValue::Number(BigDecimal::from_str("1.11").unwrap()),
        ];
        let (grid, sheet_id) = test_setup(&selected, &vals);

        print_table_in_rect(&grid, sheet_id, selected);

        let sheet = grid.sheet(sheet_id);
        let values = sheet.cell_values_in_rect(&selected, false).unwrap();
        values
            .into_cell_values_vec()
            .into_iter()
            .enumerate()
            .for_each(|(index, val)| assert_eq!(val, *expected.get(index).unwrap()));
    }

    #[test]
    fn delete_cell_values() {
        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];
        let sheet = gc.sheet_mut(sheet_id);
        sheet.test_set_values(0, 0, 2, 2, vec!["1", "2", "a", "b"]);

        let rect = SheetRect::from_numbers(0, 0, 2, 2, sheet_id);
        let selection = A1Selection::from_rect(rect);
        gc.delete_cells(&selection, None);

        let sheet = gc.sheet(sheet_id);
        assert!(sheet.cell_value(Pos { x: 0, y: 0 }).is_none());
        assert!(sheet.cell_value(Pos { x: 0, y: 1 }).is_none());
        assert!(sheet.cell_value(Pos { x: 1, y: 0 }).is_none());
        assert!(sheet.cell_value(Pos { x: 1, y: 1 }).is_none());
    }

    #[test]
    fn delete_cell_values_code() {
        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];
        let sheet = gc.sheet_mut(sheet_id);
        sheet.set_cell_value(
            Pos { x: 0, y: 0 },
            CellValue::Code(CodeCellValue {
                code: "test".to_string(),
                language: CodeCellLanguage::Formula,
            }),
        );
        gc.delete_cells(&A1Selection::from_xy(0, 0, sheet_id), None);

        let sheet = gc.sheet(sheet_id);
        assert!(sheet.cell_value(Pos { x: 0, y: 0 }).is_none());
    }

    #[test]
    fn test_get_cell_value() {
        let (grid, sheet_id, _) = test_setup_basic();
        let sheet = grid.sheet(sheet_id);
        let value = sheet.display_value((2, 1).into());

        assert_eq!(value, Some(CellValue::Number(BigDecimal::from(1))));
    }

    #[test]
    fn cell_format_summary() {
        let (grid, sheet_id, _) = test_setup_basic();
        let mut sheet = grid.sheet(sheet_id).clone();

        let format_summary = sheet.cell_format_summary((2, 1).into());
        assert_eq!(format_summary, CellFormatSummary::default());

        // just set a bold value
        sheet.formats.bold.set(Pos { x: 2, y: 1 }, Some(true));
        let value = sheet.cell_format_summary((2, 1).into());
        let mut cell_format_summary = CellFormatSummary {
            bold: Some(true),
            ..Default::default()
        };
        assert_eq!(value, cell_format_summary);

        let format_summary = sheet.cell_format_summary((2, 1).into());
        assert_eq!(cell_format_summary.clone(), format_summary);

        // now set a italic value
        sheet.formats.italic.set(Pos { x: 2, y: 1 }, Some(true));
        let value = sheet.cell_format_summary((2, 1).into());
        cell_format_summary.italic = Some(true);
        assert_eq!(value, cell_format_summary);

        let existing_cell_format_summary = sheet.cell_format_summary((2, 1).into());
        assert_eq!(cell_format_summary.clone(), existing_cell_format_summary);

        sheet.set_cell_value(
            Pos { x: 0, y: 0 },
            CellValue::Date(NaiveDate::from_str("2024-12-21").unwrap()),
        );
        let format_summary = sheet.cell_format_summary((0, 0).into());
        assert_eq!(format_summary.cell_type, Some(CellType::Date));

        sheet.set_cell_value(
            Pos { x: 1, y: 0 },
            CellValue::DateTime(
                NaiveDateTime::parse_from_str("2024-12-21 1:23 PM", "%Y-%m-%d %-I:%M %p").unwrap(),
            ),
        );
        let format_summary = sheet.cell_format_summary((1, 0).into());
        assert_eq!(format_summary.cell_type, Some(CellType::DateTime));

        sheet.set_cell_value(
            Pos { x: 2, y: 0 },
            CellValue::Time(NaiveTime::parse_from_str("1:23 pm", "%-I:%M %p").unwrap()),
        );
        let format_summary = sheet.cell_format_summary((2, 0).into());
        assert_eq!(format_summary.cell_type, None);
    }

    #[test]
    fn display_value_blanks() {
        let mut sheet = Sheet::test();
        let pos = pos![A1];
        assert_eq!(sheet.display_value(pos), None);
        sheet.set_cell_value(pos, CellValue::Blank);
        assert_eq!(sheet.display_value(pos), None);
    }

    #[test]
    fn test_check_if_wrap_in_cell() {
        let mut sheet = Sheet::test();
        let pos = pos![A1];
        sheet.set_cell_value(pos, "test");
        assert!(!sheet.check_if_wrap_in_cell(pos));
        sheet.formats.wrap.set(pos, Some(CellWrap::Wrap));
        assert!(sheet.check_if_wrap_in_cell(pos));
        sheet.formats.wrap.set(pos, Some(CellWrap::Overflow));
        assert!(!sheet.check_if_wrap_in_cell(pos));
        sheet.formats.wrap.set(pos, Some(CellWrap::Wrap));
        assert!(sheet.check_if_wrap_in_cell(pos));
        sheet.formats.wrap.set(pos, Some(CellWrap::Clip));
        assert!(!sheet.check_if_wrap_in_cell(pos));
    }

    #[test]
    fn test_check_if_wrap_in_row() {
        let mut sheet = Sheet::test();
        let pos = pos![A1];
        sheet.set_cell_value(pos, "test");
        assert!(!sheet.check_if_wrap_in_row(1));
        sheet.formats.wrap.set(pos, Some(CellWrap::Wrap));
        assert!(sheet.check_if_wrap_in_row(1));
        sheet.formats.wrap.set(pos, Some(CellWrap::Overflow));
        assert!(!sheet.check_if_wrap_in_row(1));
        sheet.formats.wrap.set(pos, Some(CellWrap::Clip));
        assert!(!sheet.check_if_wrap_in_row(1));
    }

    #[test]
    fn test_get_rows_with_wrap_in_column() {
        let mut sheet = Sheet::test();
        sheet.set_cell_value(pos![A1], "test");
        sheet.set_cell_value(pos![A3], "test");
        assert_eq!(sheet.get_rows_with_wrap_in_column(1), Vec::<i64>::new());
        sheet
            .formats
            .wrap
            .set_rect(1, 1, Some(1), Some(5), Some(CellWrap::Wrap));
        assert_eq!(sheet.get_rows_with_wrap_in_column(1), vec![1, 3]);
    }

    #[test]
    fn test_get_rows_with_wrap_in_rect() {
        let mut sheet = Sheet::test();
        sheet.set_cell_value(pos![A1], "test");
        sheet.set_cell_value(pos![A3], "test");
        let rect = Rect {
            min: pos![A1],
            max: pos![A4],
        };
        assert_eq!(
            sheet.get_rows_with_wrap_in_rect(&rect, false),
            Vec::<i64>::new()
        );
        sheet
            .formats
            .wrap
            .set_rect(1, 1, Some(1), Some(5), Some(CellWrap::Wrap));
        assert_eq!(sheet.get_rows_with_wrap_in_rect(&rect, false), vec![1, 3]);
    }

    #[test]
    fn test_get_rows_with_wrap_in_selection() {
        let mut sheet = Sheet::test();
        sheet.set_cell_value(pos![A1], "test");
        sheet.set_cell_value(pos![A3], "test");
        let selection = A1Selection::test_a1("A1:A4");
        let a1_context = sheet.expensive_make_a1_context();
        assert_eq!(
            sheet.get_rows_with_wrap_in_selection(&selection, false, false, &a1_context),
            Vec::<i64>::new()
        );
        sheet
            .formats
            .wrap
            .set_rect(1, 1, Some(1), Some(5), Some(CellWrap::Wrap));
        let mut rows = sheet.get_rows_with_wrap_in_selection(&selection, false, false, &a1_context);
        rows.sort();
        assert_eq!(rows, vec![1, 3]);
    }

    #[test]
    fn js_cell_value() {
        let mut sheet = Sheet::test();
        sheet.set_cell_value(Pos { x: 0, y: 0 }, "test");
        let js_cell_value = sheet.js_cell_value(Pos { x: 0, y: 0 });
        assert_eq!(
            js_cell_value,
            Some(JsCellValue {
                value: "test".to_string(),
                kind: "text".to_string()
            })
        );
    }

    #[test]
    fn test_has_content() {
        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];
        let sheet = gc.sheet_mut(sheet_id);
        let pos = Pos { x: 1, y: 1 };

        // Empty cell should have no content
        assert!(!sheet.has_content(pos));

        // Text content
        sheet.set_cell_value(pos, "test");
        assert!(sheet.has_content(pos));

        // Blank value should count as no content
        sheet.set_cell_value(pos, CellValue::Blank);
        assert!(!sheet.has_content(pos));

        // Empty string should count as no content
        sheet.set_cell_value(pos, "");
        assert!(!sheet.has_content(pos));

        // Number content
        sheet.set_cell_value(pos, CellValue::Text("test".to_string()));
        assert!(sheet.has_content(pos));

        // Table content
        let dt = DataTable::new(
            DataTableKind::CodeRun(CodeRun::default()),
            "test",
            Value::Array(Array::from(vec![vec!["test", "test"]])),
            false,
            Some(true),
            Some(true),
            None,
        );
        sheet.data_table_insert_full(&pos, dt.clone());
        assert!(sheet.has_content(pos));
        assert!(sheet.has_content(Pos { x: 2, y: 2 }));
        assert!(!sheet.has_content(Pos { x: 3, y: 2 }));

        let mut dt = dt.clone();
        dt.chart_output = Some((5, 5));
        let pos2 = Pos { x: 10, y: 10 };
        sheet.data_table_insert_full(&pos2, dt);
        assert!(sheet.has_content(pos2));
        assert!(sheet.has_content(Pos { x: 14, y: 10 }));
        assert!(!sheet.has_content(Pos { x: 15, y: 10 }));
    }

    #[test]
    fn js_cell_value_pos() {
        let mut sheet = Sheet::test();
        let pos = pos![A1];
        sheet.set_cell_value(pos, "test");
        let js_cell_value_pos = sheet.js_cell_value_pos(pos);
        assert_eq!(
            js_cell_value_pos,
            Some(JsCellValuePos {
                value: "test".to_string(),
                kind: "text".to_string(),
                pos: pos.a1_string(),
            })
        );

        let pos = pos![B2];
        sheet.set_cell_value(pos, CellValue::Image("image string".to_string()));
        let js_cell_value_pos = sheet.js_cell_value_pos(pos);
        assert_eq!(
            js_cell_value_pos,
            Some(JsCellValuePos {
                value: "Javascript chart".to_string(),
                kind: "image".to_string(),
                pos: pos.a1_string(),
            })
        );

        let pos = pos![C3];
        sheet.set_cell_value(pos, CellValue::Html("html string".to_string()));
        let js_cell_value_pos = sheet.js_cell_value_pos(pos);
        assert_eq!(
            js_cell_value_pos,
            Some(JsCellValuePos {
                value: "Python chart".to_string(),
                kind: "html".to_string(),
                pos: pos.a1_string(),
            })
        );
    }

    #[test]
    fn test_js_cell_value_pos_in_rect() {
        let mut sheet = Sheet::test();
        sheet.set_cell_values(
            Rect {
                min: Pos { x: 1, y: 1 },
                max: Pos { x: 10, y: 1000 },
            },
            Array::from(
                (1..=1000)
                    .map(|row| {
                        (1..=10)
                            .map(|_| {
                                if row == 1 {
                                    "heading".to_string()
                                } else {
                                    "value".to_string()
                                }
                            })
                            .collect::<Vec<String>>()
                    })
                    .collect::<Vec<Vec<String>>>(),
            ),
        );

        let max_rows = 3;

        let js_cell_value_pos_in_rect = sheet.js_cell_value_pos_in_rect(
            Rect {
                min: Pos { x: 1, y: 1 },
                max: Pos { x: 10, y: 1000 },
            },
            Some(max_rows),
        );

        assert_eq!(js_cell_value_pos_in_rect.len(), max_rows as usize);

        let expected_js_cell_value_pos_in_rect: Vec<Vec<JsCellValuePos>> = (1..=max_rows)
            .map(|row| {
                (1..=10)
                    .map(|col| {
                        if row == 1 {
                            JsCellValuePos {
                                value: "heading".to_string(),
                                kind: "text".to_string(),
                                pos: Pos {
                                    x: col,
                                    y: row as i64,
                                }
                                .a1_string(),
                            }
                        } else {
                            JsCellValuePos {
                                value: "value".to_string(),
                                kind: "text".to_string(),
                                pos: Pos {
                                    x: col,
                                    y: row as i64,
                                }
                                .a1_string(),
                            }
                        }
                    })
                    .collect::<Vec<JsCellValuePos>>()
            })
            .collect::<Vec<Vec<JsCellValuePos>>>();

        assert_eq!(
            js_cell_value_pos_in_rect,
            expected_js_cell_value_pos_in_rect
        );
    }

    #[test]
    fn test_validate_sheet_name() {
        // Setup test context with an existing sheet
        let context = A1Context::test(&[("ExistingSheet", SheetId::TEST)], &[]);

        // Test valid sheet names
        let longest_name = "a".repeat(31);
        let valid_names = vec![
            "Sheet1",
            "MySheet",
            "Test_Sheet",
            "Sheet-1",
            "Sheet (1)",
            "Sheet.1",
            "1Sheet",
            "Sheet_with_underscore",
            "Sheet-with-dashes",
            "Sheet With Spaces",
            "Sheet.With.Dots",
            "Sheet(With)Parentheses",
            "_hidden_sheet",
            "Sheet-with–en—dash", // Testing various dash characters
            longest_name.as_str(),
        ];

        for name in valid_names {
            assert!(
                Sheet::validate_sheet_name(name, SheetId::TEST, &context).is_ok(),
                "Expected '{}' to be valid",
                name
            );
        }

        // Test invalid sheet names
        let long_name = "a".repeat(32);
        let test_cases = vec![
            ("", "Sheet name must be between 1 and 31 characters"),
            (
                long_name.as_str(),
                "Sheet name must be between 1 and 31 characters",
            ),
            ("#Invalid", "Sheet name contains invalid characters"),
            ("@Sheet", "Sheet name contains invalid characters"),
            ("Sheet!", "Sheet name contains invalid characters"),
            ("Sheet?", "Sheet name contains invalid characters"),
            ("Sheet*", "Sheet name contains invalid characters"),
            ("Sheet/", "Sheet name contains invalid characters"),
            ("Sheet\\", "Sheet name contains invalid characters"),
            ("Sheet$", "Sheet name contains invalid characters"),
            ("Sheet%", "Sheet name contains invalid characters"),
            ("Sheet^", "Sheet name contains invalid characters"),
            ("Sheet&", "Sheet name contains invalid characters"),
            ("Sheet+", "Sheet name contains invalid characters"),
            ("Sheet=", "Sheet name contains invalid characters"),
            ("Sheet;", "Sheet name contains invalid characters"),
            ("Sheet,", "Sheet name contains invalid characters"),
            ("Sheet<", "Sheet name contains invalid characters"),
            ("Sheet>", "Sheet name contains invalid characters"),
            ("Sheet[", "Sheet name contains invalid characters"),
            ("Sheet]", "Sheet name contains invalid characters"),
            ("Sheet{", "Sheet name contains invalid characters"),
            ("Sheet}", "Sheet name contains invalid characters"),
            ("Sheet|", "Sheet name contains invalid characters"),
            ("Sheet`", "Sheet name contains invalid characters"),
            ("Sheet~", "Sheet name contains invalid characters"),
            ("Sheet'", "Sheet name contains invalid characters"),
            ("Sheet\"", "Sheet name contains invalid characters"),
            // Test names with leading/trailing spaces
            (" Sheet", "Sheet name contains invalid characters"),
            ("Sheet ", "Sheet name contains invalid characters"),
            // Test names with other invalid patterns
            ("\tSheet", "Sheet name contains invalid characters"),
            ("\nSheet", "Sheet name contains invalid characters"),
            ("Sheet\r", "Sheet name contains invalid characters"),
        ];

        for (name, expected_error) in test_cases {
            let result = Sheet::validate_sheet_name(name, SheetId::TEST, &context);
            assert!(
                result.is_err(),
                "Expected '{}' to be invalid, but it was valid",
                name
            );
            assert_eq!(
                result.unwrap_err(),
                expected_error,
                "Unexpected error message for '{}'",
                name
            );
        }

        // Test duplicate sheet name
        let result = Sheet::validate_sheet_name("ExistingSheet", SheetId::new(), &context);
        assert_eq!(result.unwrap_err(), "Sheet name must be unique");

        // Test same sheet name with different case
        let result = Sheet::validate_sheet_name("EXISTINGSHEET", SheetId::TEST, &context);
        result.unwrap();
    }

    #[test]
    fn test_is_at_table_edge() {
        let mut sheet = Sheet::test();
        let anchor_pos = pos![B2];

        // Create a test data table with 3x3 dimensions
        let dt = DataTable::new(
            DataTableKind::CodeRun(CodeRun::default()),
            "test",
            Value::Array(Array::from(vec![
                vec!["a", "b", "c"],
                vec!["d", "e", "f"],
                vec!["g", "h", "i"],
                vec!["j", "k", "l"],
            ])),
            false,
            Some(true),
            Some(true),
            None,
        );
        sheet.data_table_insert_full(&anchor_pos, dt);

        // Test row edges
        assert!(sheet.is_at_table_edge_row(&pos![B2])); // Table name
        assert!(!sheet.is_at_table_edge_row(&pos![B3])); // Column header
        assert!(sheet.is_at_table_edge_row(&pos![B4])); // first line of data
        assert!(sheet.is_at_table_edge_row(&pos![C7])); // Bottom edge
        assert!(!sheet.is_at_table_edge_row(&pos![C5])); // Middle row

        // Test column edges
        assert!(sheet.is_at_table_edge_col(&pos![B5])); // Left edge
        assert!(sheet.is_at_table_edge_col(&pos![D5])); // Right edge
        assert!(!sheet.is_at_table_edge_col(&pos![C5])); // Middle column

        // Test position outside table
        assert!(!sheet.is_at_table_edge_row(&pos![E5]));
        assert!(!sheet.is_at_table_edge_col(&pos![E5]));

        // Test with show_ui = false
        let mut dt_no_ui = DataTable::new(
            DataTableKind::CodeRun(CodeRun::default()),
            "test",
            Value::Array(Array::from(vec![vec!["a", "b"], vec!["c", "d"]])),
            false,
            Some(false),
            Some(false),
            None,
        );
        dt_no_ui.show_name = Some(false);
        sheet.data_table_insert_full(&pos![E5], dt_no_ui);

        // Test edges without UI
        assert!(sheet.is_at_table_edge_row(&pos![E5])); // Top edge
        assert!(sheet.is_at_table_edge_row(&pos![E6])); // Bottom edge
        assert!(sheet.is_at_table_edge_col(&pos![E5])); // Left edge
        assert!(sheet.is_at_table_edge_col(&pos![F5])); // Right edge
    }

    #[test]
    fn test_has_content_ignore_blank_table() {
        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];
        let sheet = gc.sheet_mut(sheet_id);
        let pos = pos![A1];

        // Empty cell should have no content
        assert!(!sheet.has_content_ignore_blank_table(pos));

        // Text content
        sheet.set_cell_value(pos, "test");
        assert!(sheet.has_content_ignore_blank_table(pos));

        // Blank value should count as no content
        sheet.set_cell_value(pos, CellValue::Blank);
        assert!(!sheet.has_content_ignore_blank_table(pos));

        // Empty string should count as no content
        sheet.set_cell_value(pos, "");
        assert!(!sheet.has_content_ignore_blank_table(pos));

        // Table with non-blank content
        let dt = DataTable::new(
            DataTableKind::CodeRun(CodeRun::default()),
            "test",
            Value::Array(Array::from(vec![vec!["test", "test"]])),
            false,
            Some(true),
            Some(true),
            None,
        );
        sheet.data_table_insert_full(&pos, dt.clone());
        assert!(sheet.has_content_ignore_blank_table(pos));
        assert!(sheet.has_content_ignore_blank_table(Pos { x: 2, y: 2 }));
        assert!(!sheet.has_content_ignore_blank_table(Pos { x: 3, y: 2 }));

        // Table with blank content should be ignored
        sheet.test_set_code_run_array(10, 10, vec!["1", "", "", "4"], false);

        let a1_context = gc.a1_context().clone();
        gc.sheet_mut(sheet_id).recalculate_bounds(&a1_context);

        let sheet = gc.sheet(sheet_id);
        assert!(sheet.has_content_ignore_blank_table(Pos { x: 10, y: 10 }));
        assert!(!sheet.has_content_ignore_blank_table(Pos { x: 11, y: 10 }));
        assert!(sheet.has_content_ignore_blank_table(Pos { x: 13, y: 10 }));

        // Chart output should still count as content
        let mut dt_chart = dt.clone();
        dt_chart.chart_output = Some((5, 5));
        let pos3 = Pos { x: 20, y: 20 };
        let sheet = gc.sheet_mut(sheet_id);
        sheet.data_table_insert_full(&pos3, dt_chart);

        let a1_context = gc.a1_context().clone();
        gc.sheet_mut(sheet_id).recalculate_bounds(&a1_context);

        let sheet = gc.sheet(sheet_id);
        assert!(sheet.has_content_ignore_blank_table(pos3));
        assert!(sheet.has_content_ignore_blank_table(Pos { x: 24, y: 20 }));
        assert!(!sheet.has_content_ignore_blank_table(Pos { x: 25, y: 20 }));
    }
}
