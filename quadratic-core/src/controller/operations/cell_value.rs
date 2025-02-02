use std::str::FromStr;

use bigdecimal::BigDecimal;

use super::operation::Operation;
use crate::cell_values::CellValues;
use crate::controller::GridController;
use crate::grid::formats::{FormatUpdate, SheetFormatUpdates};
use crate::grid::{CodeCellLanguage, CodeCellValue, NumericFormat, NumericFormatKind};
use crate::{A1Selection, CellValue, SheetPos};

// when a number's decimal is larger than this value, then it will treat it as text (this avoids an attempt to allocate a huge vector)
// there is an unmerged alternative that might be interesting: https://github.com/declanvk/bigdecimal-rs/commit/b0a2ea3a403ddeeeaeef1ddfc41ff2ae4a4252d6
// see original issue here: https://github.com/akubera/bigdecimal-rs/issues/108
const MAX_BIG_DECIMAL_SIZE: usize = 10000000;

impl GridController {
    /// Convert string to a cell_value and generate necessary operations
    pub(super) fn string_to_cell_value(
        &mut self,
        sheet_pos: SheetPos,
        value: &str,
    ) -> (Vec<Operation>, CellValue) {
        let mut ops = vec![];
        let cell_value = if value.is_empty() {
            CellValue::Blank
        } else if let Some((currency, number)) = CellValue::unpack_currency(value) {
            let mut format_update = FormatUpdate {
                numeric_format: Some(Some(NumericFormat {
                    kind: NumericFormatKind::Currency,
                    symbol: Some(currency),
                })),
                ..Default::default()
            };

            if value.contains(',') {
                format_update.numeric_commas = Some(Some(true));
            }

            ops.push(Operation::SetCellFormatsA1 {
                sheet_id: sheet_pos.sheet_id,
                formats: SheetFormatUpdates::from_selection(
                    &A1Selection::from_single_cell(sheet_pos),
                    format_update,
                ),
            });

            // We no longer automatically set numeric decimals for
            // currency; instead, we handle changes in currency decimal
            // length by using 2 if currency is set by default.

            CellValue::Number(number)
        } else if let Some(bool) = CellValue::unpack_boolean(value) {
            bool
        } else if let Ok(bd) = BigDecimal::from_str(&CellValue::strip_commas(value)) {
            if (bd.fractional_digit_count().unsigned_abs() as usize) > MAX_BIG_DECIMAL_SIZE {
                CellValue::Text(value.into())
            } else {
                if value.contains(',') {
                    let format_update = FormatUpdate {
                        numeric_commas: Some(Some(true)),
                        ..Default::default()
                    };
                    ops.push(Operation::SetCellFormatsA1 {
                        sheet_id: sheet_pos.sheet_id,
                        formats: SheetFormatUpdates::from_selection(
                            &A1Selection::from_single_cell(sheet_pos),
                            format_update,
                        ),
                    });
                }
                CellValue::Number(bd)
            }
        } else if let Some(percent) = CellValue::unpack_percentage(value) {
            let format_update = FormatUpdate {
                numeric_format: Some(Some(NumericFormat {
                    kind: NumericFormatKind::Percentage,
                    symbol: None,
                })),
                ..Default::default()
            };
            ops.push(Operation::SetCellFormatsA1 {
                sheet_id: sheet_pos.sheet_id,
                formats: SheetFormatUpdates::from_selection(
                    &A1Selection::from_single_cell(sheet_pos),
                    format_update,
                ),
            });
            CellValue::Number(percent)
        } else if let Some(time) = CellValue::unpack_time(value) {
            time
        } else if let Some(date) = CellValue::unpack_date(value) {
            date
        } else if let Some(date_time) = CellValue::unpack_date_time(value) {
            date_time
        } else if let Some(duration) = CellValue::unpack_duration(value) {
            duration
        } else if let Some(code) = value.strip_prefix("=") {
            ops.push(Operation::ComputeCode { sheet_pos });
            CellValue::Code(CodeCellValue {
                language: CodeCellLanguage::Formula,
                code: code.to_string(),
            })
        } else {
            CellValue::Text(value.into())
        };
        (ops, cell_value)
    }

    /// Generate operations for a user-initiated change to a cell value
    pub fn set_cell_value_operations(
        &mut self,
        sheet_pos: SheetPos,
        value: String,
    ) -> Vec<Operation> {
        let mut ops = vec![];

        // strip whitespace
        let value = value.trim();

        // convert the string to a cell value and generate necessary operations
        let (operations, cell_value) = self.string_to_cell_value(sheet_pos, value);
        ops.push(Operation::SetCellValues {
            sheet_pos,
            values: CellValues::from(cell_value),
        });
        ops.extend(operations);
        ops
    }

    /// Generates and returns the set of operations to delete the values and code in a Selection
    /// Does not commit the operations or create a transaction.
    pub fn delete_cells_operations(&self, selection: &A1Selection) -> Vec<Operation> {
        let mut ops = vec![];
        if let Some(sheet) = self.try_sheet(selection.sheet_id) {
            let rects = sheet.selection_to_rects(selection);
            for rect in rects {
                let cell_values = CellValues::new(rect.width(), rect.height());
                ops.push(Operation::SetCellValues {
                    sheet_pos: (rect.min.x, rect.min.y, selection.sheet_id).into(),
                    values: cell_values,
                });
            }
        };
        ops
    }

    /// Generates and returns the set of operations to clear the formatting in a sheet_rect
    pub fn delete_values_and_formatting_operations(
        &mut self,
        selection: &A1Selection,
    ) -> Vec<Operation> {
        let mut ops = self.delete_cells_operations(selection);
        ops.extend(self.clear_format_borders_operations(selection));
        ops
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use bigdecimal::BigDecimal;
    use serial_test::parallel;

    use crate::cell_values::CellValues;
    use crate::controller::operations::operation::Operation;
    use crate::controller::GridController;
    use crate::grid::{CodeCellLanguage, CodeCellValue, SheetId};
    use crate::{A1Selection, CellValue, SheetPos, SheetRect};

    #[test]
    #[parallel]
    fn test() {
        let mut client = GridController::test();
        let sheet_id = SheetId::test();
        client.sheet_mut(client.sheet_ids()[0]).id = sheet_id;
        client.set_cell_value(
            SheetPos {
                x: 1,
                y: 2,
                sheet_id,
            },
            "hello".to_string(),
            None,
        );
        let operations = client.last_transaction().unwrap().operations.clone();

        let values = CellValues::from(CellValue::Text("hello".to_string()));
        assert_eq!(
            operations,
            vec![Operation::SetCellValues {
                sheet_pos: SheetPos {
                    x: 1,
                    y: 2,
                    sheet_id: SheetId::test()
                },
                values
            }]
        );
    }

    #[test]
    #[parallel]
    fn boolean_to_cell_value() {
        let mut gc = GridController::test();
        let sheet_pos = SheetPos {
            x: 1,
            y: 2,
            sheet_id: SheetId::test(),
        };
        let (ops, value) = gc.string_to_cell_value(sheet_pos, "true");
        assert_eq!(ops.len(), 0);
        assert_eq!(value, true.into());

        let (ops, value) = gc.string_to_cell_value(sheet_pos, "false");
        assert_eq!(ops.len(), 0);
        assert_eq!(value, false.into());

        let (ops, value) = gc.string_to_cell_value(sheet_pos, "TRUE");
        assert_eq!(ops.len(), 0);
        assert_eq!(value, true.into());

        let (ops, value) = gc.string_to_cell_value(sheet_pos, "FALSE");
        assert_eq!(ops.len(), 0);
        assert_eq!(value, false.into());

        let (ops, value) = gc.string_to_cell_value(sheet_pos, "tRue");
        assert_eq!(ops.len(), 0);
        assert_eq!(value, true.into());

        let (ops, value) = gc.string_to_cell_value(sheet_pos, "FaLse");
        assert_eq!(ops.len(), 0);
        assert_eq!(value, false.into());
    }

    #[test]
    #[parallel]
    fn number_to_cell_value() {
        let mut gc = GridController::test();
        let sheet_pos = SheetPos {
            x: 1,
            y: 2,
            sheet_id: SheetId::test(),
        };
        let (ops, value) = gc.string_to_cell_value(sheet_pos, "123");
        assert_eq!(ops.len(), 0);
        assert_eq!(value, 123.into());

        let (ops, value) = gc.string_to_cell_value(sheet_pos, "123.45");
        assert_eq!(ops.len(), 0);
        assert_eq!(
            value,
            CellValue::Number(BigDecimal::from_str("123.45").unwrap())
        );

        let (ops, value) = gc.string_to_cell_value(sheet_pos, "123,456.78");
        assert_eq!(ops.len(), 1);
        assert_eq!(
            value,
            CellValue::Number(BigDecimal::from_str("123456.78").unwrap())
        );

        let (ops, value) = gc.string_to_cell_value(sheet_pos, "123,456,789.01");
        assert_eq!(ops.len(), 1);
        assert_eq!(
            value,
            CellValue::Number(BigDecimal::from_str("123456789.01").unwrap())
        );
    }

    #[test]
    #[parallel]
    fn formula_to_cell_value() {
        let mut gc = GridController::test();
        let sheet_pos = SheetPos {
            x: 1,
            y: 2,
            sheet_id: SheetId::test(),
        };

        let (ops, value) = gc.string_to_cell_value(sheet_pos, "=1+1");
        assert_eq!(ops.len(), 1);
        assert_eq!(
            value,
            CellValue::Code(CodeCellValue {
                language: CodeCellLanguage::Formula,
                code: "1+1".to_string(),
            })
        );

        let (ops, value) = gc.string_to_cell_value(sheet_pos, "=1/0");
        assert_eq!(ops.len(), 1);
        assert_eq!(
            value,
            CellValue::Code(CodeCellValue {
                language: CodeCellLanguage::Formula,
                code: "1/0".to_string(),
            })
        );

        let (ops, value) = gc.string_to_cell_value(sheet_pos, "=A1+A2");
        assert_eq!(ops.len(), 1);
        assert_eq!(
            value,
            CellValue::Code(CodeCellValue {
                language: CodeCellLanguage::Formula,
                code: "A1+A2".to_string(),
            })
        );
    }

    #[test]
    #[parallel]
    fn problematic_number() {
        let mut gc = GridController::test();
        let value = "980E92207901934";
        let (_, cell_value) = gc.string_to_cell_value(
            SheetPos {
                sheet_id: gc.sheet_ids()[0],
                x: 0,
                y: 0,
            },
            value,
        );
        assert_eq!(cell_value.to_string(), value.to_string());
    }

    #[test]
    #[parallel]
    fn delete_cells_operations() {
        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];
        let sheet_pos = SheetPos {
            x: 1,
            y: 2,
            sheet_id,
        };
        gc.set_cell_value(sheet_pos, "hello".to_string(), None);

        let sheet_pos_2 = SheetPos {
            x: 2,
            y: 2,
            sheet_id,
        };
        gc.set_code_cell(
            sheet_pos_2,
            CodeCellLanguage::Formula,
            "5 + 5".to_string(),
            None,
        );
        let selection = A1Selection::from_rect(SheetRect::from_numbers(1, 2, 2, 1, sheet_id));
        let operations = gc.delete_cells_operations(&selection);
        assert_eq!(operations.len(), 1);
        assert_eq!(
            operations,
            vec![Operation::SetCellValues {
                sheet_pos: SheetPos {
                    x: 1,
                    y: 2,
                    sheet_id
                },
                values: CellValues::new(2, 1)
            }]
        );
    }

    #[test]
    #[parallel]
    fn delete_columns() {
        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];
        let sheet_pos = SheetPos {
            x: 1,
            y: 2,
            sheet_id,
        };
        gc.set_cell_value(sheet_pos, "hello".to_string(), None);

        let sheet_pos_2 = SheetPos {
            x: 2,
            y: 2,
            sheet_id,
        };
        gc.set_code_cell(
            sheet_pos_2,
            CodeCellLanguage::Formula,
            "5 + 5".to_string(),
            None,
        );
        let selection = A1Selection::test_a1("A2:,B");
        let operations = gc.delete_cells_operations(&selection);
        assert_eq!(operations.len(), 2);
        assert_eq!(
            operations,
            vec![
                Operation::SetCellValues {
                    sheet_pos: SheetPos {
                        x: 1,
                        y: 2,
                        sheet_id
                    },
                    values: CellValues::new(2, 1)
                },
                Operation::SetCellValues {
                    sheet_pos: SheetPos {
                        x: 2,
                        y: 1,
                        sheet_id
                    },
                    values: CellValues::new(1, 2)
                }
            ]
        );
    }
}
