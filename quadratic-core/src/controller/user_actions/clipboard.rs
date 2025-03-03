use crate::controller::active_transactions::transaction_name::TransactionName;
use crate::controller::operations::clipboard::PasteSpecial;
use crate::controller::GridController;
use crate::grid::js_types::JsClipboard;
use crate::grid::{GridBounds, SheetId};
use crate::{A1Selection, Pos, Rect, SheetPos, SheetRect};

// To view you clipboard contents, go to https://evercoder.github.io/clipboard-inspector/
// To decode the html, use https://codebeautify.org/html-decode-string

impl GridController {
    pub fn cut_to_clipboard(
        &mut self,
        selection: &A1Selection,
        cursor: Option<String>,
    ) -> Result<JsClipboard, String> {
        let (ops, js_clipboard) = self.cut_to_clipboard_operations(selection)?;
        self.start_user_transaction(ops, cursor, TransactionName::CutClipboard);
        Ok(js_clipboard)
    }

    pub fn paste_from_clipboard(
        &mut self,
        selection: &A1Selection,
        plain_text: Option<String>,
        html: Option<String>,
        special: PasteSpecial,
        cursor: Option<String>,
    ) {
        // first try html
        if let Some(html) = html {
            if let Ok(ops) = self.paste_html_operations(selection, html, special) {
                return self.start_user_transaction(ops, cursor, TransactionName::PasteClipboard);
            }
        }
        // if not quadratic html, then use the plain text
        if let Some(plain_text) = plain_text {
            let dest_pos = selection.to_cursor_sheet_pos();
            let ops = self.paste_plain_text_operations(dest_pos, plain_text, special);
            self.start_user_transaction(ops, cursor, TransactionName::PasteClipboard);
        }
    }

    pub fn move_cells(&mut self, source: SheetRect, dest: SheetPos, cursor: Option<String>) {
        let ops = self.move_cells_operations(source, dest);
        self.start_user_transaction(ops, cursor, TransactionName::PasteClipboard);
    }

    pub fn move_code_cell_vertically(
        &mut self,
        sheet_id: SheetId,
        x: i64,
        y: i64,
        sheet_end: bool,
        reverse: bool,
        cursor: Option<String>,
    ) -> Pos {
        if let Some(sheet) = self.try_sheet(sheet_id) {
            let source = SheetRect::from_numbers(x, y, 1, 1, sheet_id);
            let mut dest = SheetPos::new(sheet_id, x, y);
            if let Some(code_cell) = sheet.get_render_code_cell((x, y).into()) {
                if sheet_end {
                    if let GridBounds::NonEmpty(rect) = sheet.bounds(true) {
                        dest = if !reverse {
                            SheetPos::new(sheet_id, x, rect.max.y + 1)
                        } else {
                            SheetPos::new(sheet_id, x, rect.min.y - code_cell.h as i64)
                        };
                    }
                } else {
                    let rect = Rect::from_numbers(
                        code_cell.x as i64,
                        code_cell.y as i64,
                        code_cell.w as i64,
                        code_cell.h as i64,
                    );
                    let row = sheet.find_next_row_for_rect(
                        y + if !reverse { 1 } else { -1 },
                        x,
                        reverse,
                        rect,
                    );
                    dest = SheetPos::new(sheet_id, x, row);
                }
                let dest_js_pos = dest.into();
                let ops = self.move_cells_operations(source, dest);
                self.start_user_transaction(ops, cursor, TransactionName::PasteClipboard);
                return dest_js_pos;
            }
        }
        Pos { x, y }
    }

    pub fn move_code_cell_horizontally(
        &mut self,
        sheet_id: SheetId,
        x: i64,
        y: i64,
        sheet_end: bool,
        reverse: bool,
        cursor: Option<String>,
    ) -> Pos {
        if let Some(sheet) = self.try_sheet(sheet_id) {
            let source = SheetRect::from_numbers(x, y, 1, 1, sheet_id);
            let mut dest = SheetPos::new(sheet_id, x, y);
            if let Some(code_cell) = sheet.get_render_code_cell((x, y).into()) {
                if sheet_end {
                    if let GridBounds::NonEmpty(rect) = sheet.bounds(true) {
                        dest = if !reverse {
                            SheetPos::new(sheet_id, rect.max.x + 1, y)
                        } else {
                            SheetPos::new(sheet_id, rect.min.x - code_cell.w as i64, y)
                        }
                    }
                } else {
                    let rect = Rect::from_numbers(
                        code_cell.x as i64,
                        code_cell.y as i64,
                        code_cell.w as i64,
                        code_cell.h as i64,
                    );
                    let col = sheet.find_next_column_for_rect(
                        x + if !reverse { 1 } else { -1 },
                        y,
                        reverse,
                        rect,
                    );
                    dest = SheetPos::new(sheet_id, col, y);
                }
                let dest_js_pos = dest.into();
                let ops = self.move_cells_operations(source, dest);
                self.start_user_transaction(ops, cursor, TransactionName::PasteClipboard);
                return dest_js_pos;
            }
        }
        Pos { x, y }
    }
}

#[cfg(test)]
mod test {
    use bigdecimal::BigDecimal;
    use serial_test::parallel;

    use super::*;
    use crate::grid::js_types::JsClipboard;
    use crate::grid::sheet::borders::{BorderSelection, BorderSide, BorderStyle, CellBorderLine};
    use crate::{
        controller::GridController,
        grid::{js_types::CellFormatSummary, CodeCellLanguage, CodeCellValue, SheetId},
        CellValue, Pos, SheetPos, SheetRect,
    };

    fn set_cell_value(gc: &mut GridController, sheet_id: SheetId, value: &str, x: i64, y: i64) {
        gc.set_cell_value(SheetPos { x, y, sheet_id }, value.into(), None);
    }

    fn set_code_cell(
        gc: &mut GridController,
        sheet_id: SheetId,
        language: CodeCellLanguage,
        code: &str,
        x: i64,
        y: i64,
    ) {
        gc.set_code_cell(SheetPos { x, y, sheet_id }, language, code.into(), None);
    }

    fn set_formula_code_cell(
        gc: &mut GridController,
        sheet_id: SheetId,
        code: &str,
        x: i64,
        y: i64,
    ) {
        set_code_cell(gc, sheet_id, CodeCellLanguage::Formula, code, x, y);
    }

    fn assert_cell_values(gc: &GridController, sheet_id: SheetId, values: &[(i64, i64, i64)]) {
        let sheet = gc.sheet(sheet_id);
        for &(x, y, expected) in values {
            assert_eq!(
                sheet.display_value(Pos { x, y }),
                Some(CellValue::Number(BigDecimal::from(expected)))
            );
        }
    }

    fn assert_empty_cells(gc: &GridController, sheet_id: SheetId, cells: &[(i64, i64)]) {
        let sheet = gc.sheet(sheet_id);
        for &(x, y) in cells {
            assert_eq!(sheet.display_value(Pos { x, y }), None);
        }
    }

    #[test]
    #[parallel]
    fn test_copy_to_clipboard() {
        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];

        set_cell_value(&mut gc, sheet_id, "1, 1", 1, 1);
        gc.set_bold(&A1Selection::test_a1("A1"), true, None)
            .unwrap();
        set_cell_value(&mut gc, sheet_id, "12", 3, 2);
        gc.set_italic(&A1Selection::test_a1("C2"), true, None)
            .unwrap();
        set_cell_value(&mut gc, sheet_id, "underline", 5, 3);
        gc.set_underline(&A1Selection::test_a1("E3"), true, None)
            .unwrap();
        set_cell_value(&mut gc, sheet_id, "strike through", 7, 4);
        gc.set_strike_through(&A1Selection::test_a1("G4"), true, None)
            .unwrap();

        let selection = A1Selection::from_rect(SheetRect::new(1, 1, 7, 4, sheet_id));
        let sheet = gc.sheet(sheet_id);
        let JsClipboard { plain_text, .. } = sheet.copy_to_clipboard(&selection).unwrap();
        assert_eq!(plain_text, String::from("1, 1\t\t\t\t\t\t\n\t\t12\t\t\t\t\n\t\t\t\tunderline\t\t\n\t\t\t\t\t\tstrike through"));

        let selection = A1Selection::from_rect(SheetRect::new(0, 0, 7, 5, sheet_id));
        let JsClipboard { plain_text, html } = sheet.copy_to_clipboard(&selection).unwrap();

        // paste using plain_text
        let mut gc = GridController::default();
        let sheet_id = gc.sheet_ids()[0];
        gc.paste_from_clipboard(
            &A1Selection::from_xy(0, 0, sheet_id),
            Some(plain_text),
            None,
            PasteSpecial::None,
            None,
        );

        let sheet = gc.sheet(sheet_id);
        assert_eq!(
            sheet.display_value(Pos { x: 1, y: 1 }),
            Some(CellValue::Text(String::from("1, 1")))
        );
        assert_eq!(
            sheet.display_value(Pos { x: 3, y: 2 }),
            Some(CellValue::Number(BigDecimal::from(12)))
        );

        // paste using html
        let mut gc = GridController::default();
        let sheet_id = gc.sheet_ids()[0];
        gc.paste_from_clipboard(
            &A1Selection::from_xy(0, 0, sheet_id),
            Some(String::from("")),
            Some(html),
            PasteSpecial::None,
            None,
        );
        let sheet = gc.sheet(sheet_id);
        assert_eq!(
            sheet.display_value(Pos { x: 1, y: 1 }),
            Some(CellValue::Text(String::from("1, 1")))
        );
        assert_eq!(
            sheet.cell_format_summary(Pos { x: 1, y: 1 }),
            CellFormatSummary {
                bold: Some(true),
                italic: None,
                text_color: None,
                fill_color: None,
                commas: None,
                align: None,
                vertical_align: None,
                wrap: None,
                date_time: None,
                cell_type: None,
                underline: None,
                strike_through: None,
            }
        );
        assert_eq!(
            sheet.display_value(Pos { x: 3, y: 2 }),
            Some(CellValue::Number(BigDecimal::from(12)))
        );
        assert_eq!(
            sheet.cell_format_summary(Pos { x: 3, y: 2 }),
            CellFormatSummary {
                bold: None,
                italic: Some(true),
                text_color: None,
                fill_color: None,
                commas: None,
                align: None,
                vertical_align: None,
                wrap: None,
                date_time: None,
                cell_type: None,
                underline: None,
                strike_through: None
            }
        );
        assert_eq!(
            sheet.display_value(Pos { x: 5, y: 3 }),
            Some(CellValue::Text(String::from("underline")))
        );
        assert_eq!(
            sheet.cell_format_summary(Pos { x: 5, y: 3 }),
            CellFormatSummary {
                bold: None,
                italic: None,
                text_color: None,
                fill_color: None,
                commas: None,
                align: None,
                vertical_align: None,
                wrap: None,
                date_time: None,
                cell_type: None,
                underline: Some(true),
                strike_through: None
            }
        );
        assert_eq!(
            sheet.display_value(Pos { x: 7, y: 4 }),
            Some(CellValue::Text(String::from("strike through")))
        );
        assert_eq!(
            sheet.cell_format_summary(Pos { x: 7, y: 4 }),
            CellFormatSummary {
                bold: None,
                italic: None,
                text_color: None,
                fill_color: None,
                commas: None,
                align: None,
                vertical_align: None,
                wrap: None,
                date_time: None,
                cell_type: None,
                underline: None,
                strike_through: Some(true),
            }
        );
    }

    #[test]
    #[parallel]
    fn test_copy_code_to_clipboard() {
        let mut gc = GridController::default();
        let sheet_id = gc.sheet_ids()[0];

        set_formula_code_cell(&mut gc, sheet_id, "1 + 1", 1, 1);

        assert_eq!(gc.undo_stack.len(), 1);
        let sheet = gc.sheet(sheet_id);
        assert_eq!(
            sheet.display_value(Pos { x: 1, y: 1 }),
            Some(CellValue::Number(BigDecimal::from(2)))
        );

        let selection = A1Selection::from_rect(SheetRect::new(1, 1, 1, 1, sheet_id));
        let JsClipboard { html, .. } = sheet.copy_to_clipboard(&selection).unwrap();

        // paste using html on a new grid controller
        let mut gc = GridController::default();
        let sheet_id = gc.sheet_ids()[0];

        // ensure the grid controller is empty
        assert_eq!(gc.undo_stack.len(), 0);

        gc.paste_from_clipboard(
            &A1Selection::from_xy(0, 0, sheet_id),
            None,
            Some(html.clone()),
            PasteSpecial::None,
            None,
        );
        let sheet = gc.sheet(sheet_id);
        assert_eq!(
            sheet.display_value(Pos { x: 0, y: 0 }),
            Some(CellValue::Number(BigDecimal::from(2)))
        );
        gc.undo(None);
        let sheet = gc.sheet(sheet_id);
        assert_eq!(sheet.display_value(Pos { x: 0, y: 0 }), None);

        // prepare a cell to be overwritten
        set_formula_code_cell(&mut gc, sheet_id, "2 + 2", 0, 0);

        assert_eq!(gc.undo_stack.len(), 1);

        let sheet = gc.sheet(sheet_id);
        assert_eq!(
            sheet.display_value(Pos { x: 0, y: 0 }),
            Some(CellValue::Number(BigDecimal::from(4)))
        );

        gc.paste_from_clipboard(
            &A1Selection::from_xy(0, 0, sheet_id),
            Some(String::from("")),
            Some(html),
            PasteSpecial::None,
            None,
        );
        let sheet = gc.sheet(sheet_id);
        assert_eq!(
            sheet.display_value(Pos { x: 0, y: 0 }),
            Some(CellValue::Number(BigDecimal::from(2)))
        );

        assert_eq!(gc.undo_stack.len(), 2);

        // undo to original code cell value
        gc.undo(None);

        let sheet = gc.sheet(sheet_id);
        assert_eq!(
            sheet.display_value(Pos { x: 0, y: 0 }),
            Some(CellValue::Number(BigDecimal::from(4)))
        );

        assert_eq!(gc.undo_stack.len(), 1);

        // empty code cell
        gc.undo(None);
        let sheet = gc.sheet(sheet_id);
        assert_eq!(sheet.display_value(Pos { x: 0, y: 0 }), None);

        assert_eq!(gc.undo_stack.len(), 0);
    }

    #[test]
    #[parallel]
    fn test_copy_code_to_clipboard_with_array_output() {
        let mut gc = GridController::default();
        let sheet_id = gc.sheet_ids()[0];

        set_formula_code_cell(&mut gc, sheet_id, "{1, 2, 3}", 1, 1);

        assert_eq!(gc.undo_stack.len(), 1);
        let sheet = gc.sheet(sheet_id);
        assert_eq!(
            sheet.display_value(Pos { x: 1, y: 1 }),
            Some(CellValue::Number(BigDecimal::from(1)))
        );
        assert_eq!(
            sheet.display_value(Pos { x: 2, y: 1 }),
            Some(CellValue::Number(BigDecimal::from(2)))
        );
        assert_eq!(
            sheet.display_value(Pos { x: 3, y: 1 }),
            Some(CellValue::Number(BigDecimal::from(3)))
        );
        let selection = A1Selection::from_rect(SheetRect::new(1, 1, 3, 1, sheet_id));
        let JsClipboard { html, .. } = sheet.copy_to_clipboard(&selection).unwrap();

        // paste using html on a new grid controller
        let mut gc = GridController::default();
        let sheet_id = gc.sheet_ids()[0];

        // ensure the grid controller is empty
        assert_eq!(gc.undo_stack.len(), 0);

        gc.paste_from_clipboard(
            &A1Selection::from_xy(0, 0, sheet_id),
            None,
            Some(html),
            PasteSpecial::None,
            None,
        );
        let sheet = gc.sheet(sheet_id);
        assert_eq!(
            sheet.display_value(Pos { x: 0, y: 0 }),
            Some(CellValue::Number(BigDecimal::from(1)))
        );
        assert_eq!(
            sheet.display_value(Pos { x: 1, y: 0 }),
            Some(CellValue::Number(BigDecimal::from(2)))
        );
        assert_eq!(
            sheet.display_value(Pos { x: 2, y: 0 }),
            Some(CellValue::Number(BigDecimal::from(3)))
        );

        gc.undo(None);
        let sheet = gc.sheet(sheet_id);
        assert_eq!(sheet.display_value(Pos { x: 0, y: 0 }), None);
        assert_eq!(sheet.display_value(Pos { x: 1, y: 0 }), None);
        assert_eq!(sheet.display_value(Pos { x: 2, y: 0 }), None);
    }

    #[test]
    #[parallel]
    fn test_copy_borders_to_clipboard() {
        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];
        let selection = A1Selection::from_rect(SheetRect::new(1, 1, 1, 1, sheet_id));

        gc.set_borders(
            selection.clone(),
            BorderSelection::All,
            Some(BorderStyle::default()),
            None,
        );

        let sheet = gc.sheet(sheet_id);
        let JsClipboard { html, .. } = sheet.copy_to_clipboard(&selection).unwrap();

        gc.paste_from_clipboard(
            &A1Selection::from_xy(3, 3, sheet_id),
            Some(String::from("")),
            Some(html),
            PasteSpecial::None,
            None,
        );

        let sheet = gc.sheet(sheet_id);
        assert_eq!(
            sheet
                .borders
                .get(BorderSide::Top, Pos { x: 3, y: 3 })
                .unwrap()
                .line,
            CellBorderLine::default()
        );
    }

    #[test]
    #[parallel]
    fn test_copy_borders_inside() {
        let mut gc = GridController::test();
        let sheet_id_1 = gc.sheet_ids()[0];

        gc.add_sheet(None);
        let sheet_id_2 = gc.sheet_ids()[1];

        gc.set_borders(
            A1Selection::from_rect(SheetRect::new(1, 1, 5, 5, sheet_id_1)),
            BorderSelection::Outer,
            Some(BorderStyle::default()),
            None,
        );

        let sheet = gc.sheet(sheet_id_1);
        let borders = sheet.borders.borders_in_sheet().unwrap();
        let mut horizontal_borders = borders.horizontal.as_ref().unwrap().iter();
        let mut vertical_borders = borders.vertical.as_ref().unwrap().iter();

        let border = horizontal_borders.next().unwrap();
        assert_eq!(border.x, 1);
        assert_eq!(border.y, 1);
        assert_eq!(border.width, Some(5));
        assert_eq!(border.line, CellBorderLine::default());

        let border = horizontal_borders.next().unwrap();
        assert_eq!(border.x, 1);
        assert_eq!(border.y, 6);
        assert_eq!(border.width, Some(5));
        assert_eq!(border.line, CellBorderLine::default());

        let border = vertical_borders.next().unwrap();
        assert_eq!(border.x, 1);
        assert_eq!(border.y, 1);
        assert_eq!(border.height, Some(5));
        assert_eq!(border.line, CellBorderLine::default());

        let border = vertical_borders.next().unwrap();
        assert_eq!(border.x, 6);
        assert_eq!(border.y, 1);
        assert_eq!(border.height, Some(5));
        assert_eq!(border.line, CellBorderLine::default());

        let selection = A1Selection::from_rect(SheetRect::new(1, 1, 5, 5, sheet_id_1));
        let JsClipboard { html, .. } = sheet.copy_to_clipboard(&selection).unwrap();
        gc.paste_from_clipboard(
            &A1Selection::from_xy(1, 11, sheet_id_2),
            None,
            Some(html),
            PasteSpecial::None,
            None,
        );

        let sheet = gc.sheet(sheet_id_2);
        let borders = sheet.borders.borders_in_sheet().unwrap();
        let mut horizontal_borders = borders.horizontal.as_ref().unwrap().iter();
        let mut vertical_borders = borders.vertical.as_ref().unwrap().iter();

        let border = horizontal_borders.next().unwrap();
        assert_eq!(border.x, 1);
        assert_eq!(border.y, 11);
        assert_eq!(border.width, Some(5));
        assert_eq!(border.line, CellBorderLine::default());

        let border = horizontal_borders.next().unwrap();
        assert_eq!(border.x, 1);
        assert_eq!(border.y, 16);
        assert_eq!(border.width, Some(5));
        assert_eq!(border.line, CellBorderLine::default());

        let border = vertical_borders.next().unwrap();
        assert_eq!(border.x, 1);
        assert_eq!(border.y, 11);
        assert_eq!(border.height, Some(5));
        assert_eq!(border.line, CellBorderLine::default());

        let border = vertical_borders.next().unwrap();
        assert_eq!(border.x, 6);
        assert_eq!(border.y, 11);
        assert_eq!(border.height, Some(5));
        assert_eq!(border.line, CellBorderLine::default());
    }

    #[test]
    #[parallel]
    fn test_paste_from_quadratic_clipboard() {
        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];

        set_cell_value(&mut gc, sheet_id, "1, 1", 1, 1);
        gc.set_bold(&A1Selection::test_a1("A1"), true, None)
            .unwrap();
        set_cell_value(&mut gc, sheet_id, "12", 3, 2);
        gc.set_italic(&A1Selection::test_a1("C2"), true, None)
            .unwrap();

        let sheet = gc.sheet(sheet_id);
        let selection = A1Selection::from_rect(SheetRect::new(1, 1, 3, 2, sheet_id));
        let JsClipboard { plain_text, .. } = sheet.copy_to_clipboard(&selection).unwrap();
        assert_eq!(plain_text, String::from("1, 1\t\t\n\t\t12"));

        let selection = A1Selection::from_rect(SheetRect::new(0, 0, 3, 2, sheet_id));
        let JsClipboard { html, .. } = sheet.copy_to_clipboard(&selection).unwrap();

        gc.paste_from_clipboard(
            &A1Selection::from_xy(1, 2, sheet_id),
            None,
            Some(html),
            PasteSpecial::None,
            None,
        );

        let sheet = gc.sheet(sheet_id);
        let cell11 = sheet.display_value(Pos { x: 2, y: 3 });
        assert_eq!(cell11.unwrap(), CellValue::Text(String::from("1, 1")));
        let cell21 = sheet.display_value(Pos { x: 4, y: 4 });
        assert_eq!(cell21.unwrap(), CellValue::Number(BigDecimal::from(12)));
    }

    // | 1 | A1           |
    // | 2 | [paste here] |
    //
    // paste the code cell (2,1) => A1 from the clipboard to (2,2),
    // expect value to change to 2
    #[test]
    #[parallel]
    fn test_paste_relative_code_from_quadratic_clipboard() {
        let mut gc = GridController::default();
        let sheet_id = gc.sheet_ids()[0];
        let src_pos: Pos = (2, 1).into();

        set_formula_code_cell(&mut gc, sheet_id, "SUM(A1)", src_pos.x, src_pos.y);
        set_cell_value(&mut gc, sheet_id, "1", 1, 1);
        set_cell_value(&mut gc, sheet_id, "2", 1, 2);
        set_cell_value(&mut gc, sheet_id, "3", 1, 3);

        // generate the html from the values above
        let sheet = gc.sheet(sheet_id);
        let selection = A1Selection::from_rect(SheetRect::single_pos(src_pos, sheet_id));
        let JsClipboard { html, .. } = sheet.copy_to_clipboard(&selection).unwrap();

        let get_value = |gc: &GridController, x, y| {
            let sheet = gc.sheet(sheet_id);
            let cell_value = sheet.cell_value(Pos { x, y });
            let display_value = sheet.display_value(Pos { x, y });
            (cell_value, display_value)
        };

        let assert_code_cell =
            |gc: &mut GridController, dest_pos: SheetPos, code: &str, value: i32| {
                gc.paste_from_clipboard(
                    &A1Selection::from_xy(dest_pos.x, dest_pos.y, sheet_id),
                    None,
                    Some(html.clone()),
                    PasteSpecial::None,
                    None,
                );

                let cell_value = get_value(gc, dest_pos.x, dest_pos.y);
                let expected_cell_value = Some(CellValue::Code(CodeCellValue {
                    language: CodeCellLanguage::Formula,
                    code: code.into(),
                }));
                let expected_display_value = Some(CellValue::Number(BigDecimal::from(value)));

                assert_eq!(cell_value, (expected_cell_value, expected_display_value));
            };

        // paste code cell (2,1) from the clipboard to (2,2)
        let dest_pos: SheetPos = (2, 2, sheet_id).into();
        assert_code_cell(&mut gc, dest_pos, "SUM(A2)", 2);

        // paste code cell (2,1) from the clipboard to (2,3)
        let dest_pos: SheetPos = (2, 3, sheet_id).into();
        assert_code_cell(&mut gc, dest_pos, "SUM(A3)", 3);
    }

    #[test]
    #[parallel]
    fn paste_special_values() {
        let mut gc = GridController::default();
        let sheet_id = gc.sheet_ids()[0];

        set_formula_code_cell(&mut gc, sheet_id, "{1, 2, 3}", 1, 1);

        let sheet = gc.sheet(sheet_id);
        assert_eq!(
            sheet.display_value(Pos { x: 1, y: 1 }),
            Some(CellValue::Number(BigDecimal::from(1)))
        );
        assert_eq!(
            sheet.display_value(Pos { x: 3, y: 1 }),
            Some(CellValue::Number(BigDecimal::from(3)))
        );

        let sheet = gc.sheet(sheet_id);
        let selection = A1Selection::from_rect(SheetRect::new(1, 1, 3, 1, sheet_id));
        let JsClipboard { plain_text, html } = sheet.copy_to_clipboard(&selection).unwrap();

        gc.paste_from_clipboard(
            &A1Selection::from_xy(0, 2, sheet_id),
            Some(plain_text),
            Some(html),
            PasteSpecial::Values,
            None,
        );

        let sheet = gc.sheet(sheet_id);
        assert_eq!(
            sheet.cell_value(Pos { x: 0, y: 2 }),
            Some(CellValue::Number(BigDecimal::from(1)))
        );
        assert_eq!(
            sheet.cell_value(Pos { x: 1, y: 2 }),
            Some(CellValue::Number(BigDecimal::from(2)))
        );
        assert_eq!(
            sheet.cell_value(Pos { x: 2, y: 2 }),
            Some(CellValue::Number(BigDecimal::from(3)))
        );
    }

    #[test]
    #[parallel]
    fn paste_special_formats() {
        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];

        set_cell_value(&mut gc, sheet_id, "1", 1, 1);
        gc.set_bold(&A1Selection::test_a1("A1"), true, None)
            .unwrap();

        set_cell_value(&mut gc, sheet_id, "12", 2, 2);
        gc.set_italic(&A1Selection::test_a1("B2"), true, None)
            .unwrap();

        let sheet = gc.sheet(sheet_id);
        let selection = A1Selection::from_rect(SheetRect::new(0, 0, 2, 2, sheet_id));
        let JsClipboard { plain_text, html } = sheet.copy_to_clipboard(&selection).unwrap();
        assert_eq!(plain_text, String::from("\t\t\n\t1\t\n\t\t12"));

        let mut gc = GridController::default();
        let sheet_id = gc.sheet_ids()[0];
        gc.paste_from_clipboard(
            &A1Selection::from_xy(0, 0, sheet_id),
            Some(plain_text),
            Some(html),
            PasteSpecial::Formats,
            None,
        );
        let sheet = gc.sheet(sheet_id);
        assert_eq!(sheet.display_value(Pos { x: 1, y: 1 }), None);
        assert_eq!(sheet.display_value(Pos { x: 2, y: 2 }), None);
        assert_eq!(
            sheet.cell_format_summary(Pos { x: 1, y: 1 }),
            CellFormatSummary {
                bold: Some(true),
                italic: None,
                text_color: None,
                fill_color: None,
                commas: None,
                align: None,
                vertical_align: None,
                wrap: None,
                date_time: None,
                cell_type: None,
                underline: None,
                strike_through: None
            }
        );
        assert_eq!(
            sheet.cell_format_summary(Pos { x: 2, y: 2 }),
            CellFormatSummary {
                bold: None,
                italic: Some(true),
                text_color: None,
                fill_color: None,
                commas: None,
                align: None,
                vertical_align: None,
                wrap: None,
                date_time: None,
                cell_type: None,
                underline: None,
                strike_through: None
            }
        );
    }

    #[test]
    #[parallel]
    fn copy_part_of_code_run() {
        let mut gc = GridController::default();
        let sheet_id = gc.sheet_ids()[0];

        set_formula_code_cell(&mut gc, sheet_id, "{1, 2, 3; 4, 5, 6}", 1, 1);

        let sheet = gc.sheet(sheet_id);
        assert_eq!(
            sheet.display_value(Pos { x: 1, y: 1 }),
            Some(CellValue::Number(BigDecimal::from(1)))
        );

        // don't copy the origin point
        let sheet = gc.sheet(sheet_id);
        let selection = A1Selection::from_rect(SheetRect::new(2, 1, 3, 2, sheet_id));
        let JsClipboard { plain_text, html } = sheet.copy_to_clipboard(&selection).unwrap();

        // paste using html on a new grid controller
        let mut gc = GridController::default();
        let sheet_id = gc.sheet_ids()[0];

        // ensure the grid controller is empty
        assert_eq!(gc.undo_stack.len(), 0);

        gc.paste_from_clipboard(
            &A1Selection::from_xy(0, 0, sheet_id),
            Some(plain_text),
            Some(html),
            PasteSpecial::None,
            None,
        );
        let sheet = gc.sheet(sheet_id);
        assert_eq!(
            sheet.display_value(Pos { x: 0, y: 0 }),
            Some(CellValue::Number(BigDecimal::from(2)))
        );

        gc.undo(None);
        let sheet = gc.sheet(sheet_id);
        assert_eq!(sheet.display_value(Pos { x: 0, y: 0 }), None);
    }

    #[test]
    #[parallel]
    fn move_cells() {
        let mut gc = GridController::default();
        let sheet_id = gc.sheet_ids()[0];

        set_formula_code_cell(&mut gc, sheet_id, "{1, 2, 3; 4, 5, 6}", 0, 0);
        set_cell_value(&mut gc, sheet_id, "100", 0, 2);

        gc.move_cells(
            SheetRect::new_pos_span(Pos { x: 0, y: 0 }, Pos { x: 3, y: 2 }, sheet_id),
            (10, 10, sheet_id).into(),
            None,
        );

        let sheet = gc.sheet(sheet_id);
        assert_eq!(
            sheet.display_value((10, 10).into()),
            Some(CellValue::Number(BigDecimal::from(1)))
        );
        assert_eq!(
            sheet.display_value((10, 12).into()),
            Some(CellValue::Number(BigDecimal::from(100)))
        );
        assert_eq!(sheet.display_value((0, 0).into()), None);
        assert_eq!(sheet.display_value((0, 2).into()), None);

        gc.undo(None);

        let sheet = gc.sheet(sheet_id);
        assert_eq!(sheet.display_value((10, 10).into()), None);
        assert_eq!(sheet.display_value((10, 12).into()), None);
        assert_eq!(
            sheet.display_value((0, 0).into()),
            Some(CellValue::Number(BigDecimal::from(1)))
        );
        assert_eq!(
            sheet.display_value((0, 2).into()),
            Some(CellValue::Number(BigDecimal::from(100)))
        );
    }

    #[test]
    #[parallel]
    fn copy_cell_formats() {
        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];
        let sheet = gc.sheet_mut(sheet_id);
        sheet.formats.bold.set(Pos { x: 1, y: 2 }, Some(true));
        sheet
            .formats
            .fill_color
            .set(Pos { x: 3, y: 4 }, Some("red".to_string()));
        let selection = A1Selection::from_rect(SheetRect::new(0, 0, 4, 4, sheet_id));
        let JsClipboard { plain_text, html } = sheet.copy_to_clipboard(&selection).unwrap();

        let mut gc = GridController::test();
        let sheet_id = gc.sheet_ids()[0];
        gc.paste_from_clipboard(
            &A1Selection::from_xy(0, 0, sheet_id),
            Some(plain_text),
            Some(html),
            PasteSpecial::None,
            None,
        );

        let sheet = gc.sheet(sheet_id);
        assert_eq!(
            sheet.cell_format_summary(Pos { x: 1, y: 2 }),
            CellFormatSummary {
                bold: Some(true),
                italic: None,
                text_color: None,
                fill_color: None,
                commas: None,
                align: None,
                vertical_align: None,
                wrap: None,
                date_time: None,
                cell_type: None,
                underline: None,
                strike_through: None
            }
        );
        assert_eq!(
            sheet.cell_format_summary(Pos { x: 3, y: 4 }),
            CellFormatSummary {
                bold: None,
                italic: None,
                text_color: None,
                fill_color: Some("red".to_string()),
                commas: None,
                align: None,
                vertical_align: None,
                wrap: None,
                date_time: None,
                cell_type: None,
                underline: None,
                strike_through: None
            }
        );
    }

    #[test]
    #[parallel]
    fn test_move_code_cell_vertically() {
        let mut gc = GridController::default();
        let sheet_id = gc.sheet_ids()[0];

        // Set up a code cell
        set_formula_code_cell(&mut gc, sheet_id, "{1; 2; 3}", 1, 1);
        assert_cell_values(&gc, sheet_id, &[(1, 1, 1), (1, 2, 2), (1, 3, 3)]);

        // Move down
        let result = gc.move_code_cell_vertically(sheet_id, 1, 1, false, false, None);
        assert_eq!(result, Pos { x: 1, y: 4 });
        assert_cell_values(&gc, sheet_id, &[(1, 4, 1), (1, 5, 2), (1, 6, 3)]);
        assert_empty_cells(&gc, sheet_id, &[(1, 1), (1, 2), (1, 3)]);

        // Move up
        let result = gc.move_code_cell_vertically(sheet_id, 1, 4, false, true, None);
        assert_eq!(result, Pos { x: 1, y: 1 });
        assert_cell_values(&gc, sheet_id, &[(1, 1, 1), (1, 2, 2), (1, 3, 3)]);
        assert_empty_cells(&gc, sheet_id, &[(1, 4), (1, 5), (1, 6)]);

        // Move to sheet end (down)
        set_cell_value(&mut gc, sheet_id, "obstacle", 1, 10);
        let result = gc.move_code_cell_vertically(sheet_id, 1, 1, true, false, None);
        assert_eq!(result, Pos { x: 1, y: 11 });
        assert_cell_values(&gc, sheet_id, &[(1, 11, 1), (1, 12, 2), (1, 13, 3)]);
        assert_empty_cells(&gc, sheet_id, &[(1, 1), (1, 2), (1, 3)]);

        // Move to sheet start (up)
        set_cell_value(&mut gc, sheet_id, "obstacle1", 1, 3);
        let result = gc.move_code_cell_vertically(sheet_id, 1, 11, true, true, None);
        assert_eq!(result, Pos { x: 1, y: 0 });
        assert_cell_values(&gc, sheet_id, &[(1, 0, 1), (1, 1, 2), (1, 2, 3)]);
        assert_empty_cells(&gc, sheet_id, &[(1, 11), (1, 12), (1, 13)]);

        // Move when there's no code cell
        let result = gc.move_code_cell_vertically(sheet_id, 20, 20, false, false, None);
        assert_eq!(result, Pos { x: 20, y: 20 });

        // Move down with obstacles
        set_cell_value(&mut gc, sheet_id, "obstacle2", 1, 4);
        set_cell_value(&mut gc, sheet_id, "obstacle3", 1, 5);
        let result = gc.move_code_cell_vertically(sheet_id, 1, 0, false, false, None);
        assert_eq!(result, Pos { x: 1, y: 6 });
        assert_cell_values(&gc, sheet_id, &[(1, 6, 1), (1, 7, 2), (1, 8, 3)]);

        // Move up with obstacles
        let result = gc.move_code_cell_vertically(sheet_id, 1, 6, false, true, None);
        assert_eq!(result, Pos { x: 1, y: 0 });
        assert_cell_values(&gc, sheet_id, &[(1, 0, 1), (1, 1, 2), (1, 2, 3)]);

        // Move a single-cell code output
        set_formula_code_cell(&mut gc, sheet_id, "42", 1, 15);
        let result = gc.move_code_cell_vertically(sheet_id, 1, 15, false, false, None);
        assert_eq!(result, Pos { x: 1, y: 16 });
        assert_cell_values(&gc, sheet_id, &[(1, 16, 42)]);
        assert_empty_cells(&gc, sheet_id, &[(1, 15)]);

        // Undo and redo
        gc.undo(None);
        assert_cell_values(&gc, sheet_id, &[(1, 15, 42)]);
        gc.redo(None);
        assert_cell_values(&gc, sheet_id, &[(1, result.y, 42)]);
    }

    #[test]
    #[parallel]
    fn test_move_code_cell_horizontally() {
        let mut gc = GridController::default();
        let sheet_id = gc.sheet_ids()[0];

        // Set up a code cell
        set_formula_code_cell(&mut gc, sheet_id, "{1, 2, 3}", 1, 1);
        assert_cell_values(&gc, sheet_id, &[(1, 1, 1), (2, 1, 2), (3, 1, 3)]);

        // Move right
        let result = gc.move_code_cell_horizontally(sheet_id, 1, 1, false, false, None);
        assert_eq!(result, Pos { x: 4, y: 1 });
        assert_cell_values(&gc, sheet_id, &[(4, 1, 1), (5, 1, 2), (6, 1, 3)]);
        assert_empty_cells(&gc, sheet_id, &[(1, 1), (2, 1), (3, 1)]);

        // Move left
        let result = gc.move_code_cell_horizontally(sheet_id, 4, 1, false, true, None);
        assert_eq!(result, Pos { x: 1, y: 1 });
        assert_cell_values(&gc, sheet_id, &[(1, 1, 1), (2, 1, 2), (3, 1, 3)]);
        assert_empty_cells(&gc, sheet_id, &[(4, 1), (5, 1), (6, 1)]);

        // Move to sheet end (right)
        set_cell_value(&mut gc, sheet_id, "obstacle", 10, 1);
        let result = gc.move_code_cell_horizontally(sheet_id, 1, 1, true, false, None);
        assert_eq!(result, Pos { x: 11, y: 1 });
        assert_cell_values(&gc, sheet_id, &[(11, 1, 1), (12, 1, 2), (13, 1, 3)]);
        assert_empty_cells(&gc, sheet_id, &[(1, 1), (2, 1), (3, 1)]);

        // Move to sheet start (left)
        set_cell_value(&mut gc, sheet_id, "obstacle1", 3, 1);
        let result = gc.move_code_cell_horizontally(sheet_id, 11, 1, true, true, None);
        assert_eq!(result, Pos { x: 0, y: 1 });
        assert_cell_values(&gc, sheet_id, &[(0, 1, 1), (1, 1, 2), (2, 1, 3)]);
        assert_empty_cells(&gc, sheet_id, &[(11, 1), (12, 1), (13, 1)]);

        // Move when there's no code cell
        let result = gc.move_code_cell_horizontally(sheet_id, 20, 20, false, false, None);
        assert_eq!(result, Pos { x: 20, y: 20 });

        // Move right with obstacles
        set_cell_value(&mut gc, sheet_id, "obstacle2", 4, 1);
        set_cell_value(&mut gc, sheet_id, "obstacle3", 5, 1);
        let result = gc.move_code_cell_horizontally(sheet_id, 0, 1, false, false, None);
        assert_eq!(result, Pos { x: 6, y: 1 });
        assert_cell_values(&gc, sheet_id, &[(6, 1, 1), (7, 1, 2), (8, 1, 3)]);

        // Move left with obstacles
        let result = gc.move_code_cell_horizontally(sheet_id, 6, 1, false, true, None);
        assert_eq!(result, Pos { x: 0, y: 1 });
        assert_cell_values(&gc, sheet_id, &[(0, 1, 1), (1, 1, 2), (2, 1, 3)]);

        // Move a single-cell code output
        set_formula_code_cell(&mut gc, sheet_id, "42", 15, 1);
        let result = gc.move_code_cell_horizontally(sheet_id, 15, 1, false, false, None);
        assert_eq!(result, Pos { x: 16, y: 1 });
        assert_cell_values(&gc, sheet_id, &[(16, 1, 42)]);
        assert_empty_cells(&gc, sheet_id, &[(15, 1)]);

        // Undo and redo
        gc.undo(None);
        assert_cell_values(&gc, sheet_id, &[(15, 1, 42)]);
        gc.redo(None);
        assert_cell_values(&gc, sheet_id, &[(result.x, 1, 42)]);
    }
}
