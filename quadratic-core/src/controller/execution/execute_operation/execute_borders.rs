use crate::controller::{
    GridController, active_transactions::pending_transaction::PendingTransaction,
    operations::operation::Operation,
};

impl GridController {
    pub fn execute_set_borders_a1(&mut self, transaction: &mut PendingTransaction, op: Operation) {
        unwrap_op!(let SetBordersA1 { sheet_id, borders } = op);

        transaction.generate_thumbnail |= self.thumbnail_dirty_borders(sheet_id, &borders);

        let Some(sheet) = self.try_sheet_mut(sheet_id) else {
            return; // sheet may have been deleted
        };

        let reverse_borders = sheet.borders.set_borders_a1(&borders);
        transaction
            .reverse_operations
            .push(Operation::SetBordersA1 {
                sheet_id,
                borders: reverse_borders,
            });

        transaction
            .forward_operations
            .push(Operation::SetBordersA1 { sheet_id, borders });

        transaction.sheet_borders.insert(sheet_id);
    }
}
