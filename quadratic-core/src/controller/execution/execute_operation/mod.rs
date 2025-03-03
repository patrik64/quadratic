use crate::controller::active_transactions::pending_transaction::PendingTransaction;
use crate::controller::operations::operation::Operation;
use crate::controller::GridController;

/// Asserts that an operation is a particular type, and unpacks its contents
/// into the current scope.
///
/// # Examples
///
/// ```ignore
/// fn set_cell_values(transaction: &mut PendingTransaction, op: Operation) {
///     // panic if `op` is not `SetCellValues`
///     unwrap_op!(let SetCellValues { sheet_pos, values } = op);
///
///     println!("sheet_pos: {:?}", sheet_pos);
///     println!("values: {:?}", values);
/// }
/// ```
macro_rules! unwrap_op {
    (let $op_type:ident $contents:tt = $op:ident) => {
        let $crate::controller::operations::operation::Operation::$op_type $contents = $op else {
            unreachable!("expected {}; got {:?}", stringify!($op_type), $op);
        };
    };
}

mod execute_borders;
mod execute_borders_old;
mod execute_code;
mod execute_col_rows;
mod execute_cursor;
mod execute_formats;
mod execute_formats_old;
mod execute_move_cells;
mod execute_offsets;
mod execute_sheets;
mod execute_validation;
mod execute_values;

impl GridController {
    /// Executes an operation and updates the transaction accordingly.
    pub fn execute_operation(&mut self, transaction: &mut PendingTransaction, op: Operation) {
        #[cfg(feature = "show-operations")]
        dbgjs!(&format!("[Operation] {:?}", &op));

        match op {
            Operation::SetCellValues { .. } => self.execute_set_cell_values(transaction, op),
            Operation::SetCodeRun { .. } => self.execute_set_code_run(transaction, op),
            Operation::SetCodeRunVersion { .. } => {
                self.execute_set_code_run_version(transaction, op);
            }
            Operation::ComputeCode { .. } => self.execute_compute_code(transaction, op),
            Operation::SetCellFormats { .. } => (), //self.execute_set_cell_formats(transaction, op)),
            Operation::SetCellFormatsSelection { .. } => {
                self.execute_set_cell_formats_selection(transaction, op);
            }
            Operation::SetCellFormatsA1 { .. } => {
                self.execute_set_cell_formats_a1(transaction, op);
            }
            Operation::SetBorders { .. } => (), // we no longer support this (12/9/2024)
            Operation::SetBordersSelection { .. } => {
                self.execute_set_borders_selection(transaction, op);
            }
            Operation::SetBordersA1 { .. } => self.execute_set_borders_a1(transaction, op),

            Operation::MoveCells { .. } => self.execute_move_cells(transaction, op),

            Operation::AddSheet { .. } => self.execute_add_sheet(transaction, op),
            Operation::AddSheetSchema { .. } => self.execute_add_sheet_schema(transaction, op),

            Operation::DeleteSheet { .. } => self.execute_delete_sheet(transaction, op),
            Operation::ReorderSheet { .. } => self.execute_reorder_sheet(transaction, op),
            Operation::SetSheetName { .. } => self.execute_set_sheet_name(transaction, op),
            Operation::SetSheetColor { .. } => self.execute_set_sheet_color(transaction, op),
            Operation::DuplicateSheet { .. } => self.execute_duplicate_sheet(transaction, op),

            Operation::ResizeColumn { .. } => self.execute_resize_column(transaction, op),
            Operation::ResizeRow { .. } => self.execute_resize_row(transaction, op),
            Operation::ResizeRows { .. } => self.execute_resize_rows(transaction, op),

            Operation::SetCursor { .. } => self.execute_set_cursor(transaction, op),
            Operation::SetCursorSelection { .. } => {
                self.execute_set_cursor_selection(transaction, op);
            }
            Operation::SetCursorA1 { .. } => self.execute_set_cursor_a1(transaction, op),

            Operation::SetValidation { .. } => self.execute_set_validation(transaction, op),
            Operation::RemoveValidation { .. } => {
                self.execute_remove_validation(transaction, op);
            }
            Operation::SetValidationWarning { .. } => {
                self.execute_set_validation_warning(transaction, op);
            }

            Operation::DeleteColumn { .. } => self.execute_delete_column(transaction, op),
            Operation::DeleteRow { .. } => self.execute_delete_row(transaction, op),
            Operation::InsertColumn { .. } => self.execute_insert_column(transaction, op),
            Operation::InsertRow { .. } => self.execute_insert_row(transaction, op),
        }
    }
}
