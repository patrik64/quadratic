use anyhow::Result;

use super::DataTable;
use crate::{CellValue, CopyFormats};

impl DataTable {
    /// Get the values of a row (does not include the header)
    pub fn get_row(&self, row_index: usize) -> Result<Vec<CellValue>> {
        let data_row_index = usize::try_from(row_index as i64 - self.y_adjustment(true))?;

        let row = self
            .value_ref()?
            .iter()
            .skip(data_row_index * self.width())
            .take(self.width())
            .map(|value| value.to_owned().to_owned())
            .collect();

        Ok(row)
    }

    /// Get the values of a row taking into account sorted columns.
    ///
    /// Maps the display row index to the actual row index
    pub fn get_row_sorted(&self, display_row_index: usize) -> Result<Vec<CellValue>> {
        let row_index = display_row_index as i64 - self.y_adjustment(true);

        let actual_row_index = self.get_row_index_from_display_index(row_index as u64) as i64;

        self.get_row(usize::try_from(actual_row_index + self.y_adjustment(true))?)
    }

    /// Insert a new row at the given index.
    pub fn insert_row(&mut self, row_index: usize, values: Option<Vec<CellValue>>) -> Result<()> {
        let row_index = row_index as i64 - self.y_adjustment(true);

        let array = self.mut_value_as_array()?;
        array.insert_row(usize::try_from(row_index)?, values)?;

        // formats and borders are 1 indexed
        self.formats.insert_row(row_index + 1, CopyFormats::None);
        self.borders.insert_row(row_index + 1, CopyFormats::None);

        let row_index = u64::try_from(row_index)?;
        // add the row to the display buffer
        if let Some(display_buffer) = &mut self.display_buffer {
            for y in display_buffer.iter_mut() {
                if *y >= row_index {
                    *y += 1;
                }
            }
            display_buffer.insert(usize::try_from(row_index)?, row_index);
        }

        Ok(())
    }

    /// Remove a row at the given index.
    pub fn delete_row(&mut self, row_index: usize) -> Result<()> {
        let row_index = row_index as i64 - self.y_adjustment(true);

        let array = self.mut_value_as_array()?;
        array.delete_row(usize::try_from(row_index)?)?;

        // formats and borders are 1 indexed
        self.formats.remove_row(row_index + 1);
        self.borders.remove_row(row_index + 1);

        Ok(())
    }

    /// Remove a row at the given index, for table having sorted columns.
    ///
    /// Removes the row and calls sort_all to update the display buffer.
    pub fn delete_row_sorted(
        &mut self,
        display_row_index: usize,
    ) -> Result<(u32, Option<Vec<CellValue>>)> {
        let old_values = self.get_row_sorted(display_row_index)?;

        let row_index = display_row_index as i64 - self.y_adjustment(true);

        let actual_row_index = self.get_row_index_from_display_index(row_index as u64);

        self.delete_row(usize::try_from(
            actual_row_index as i64 + self.y_adjustment(true),
        )?)?;

        // remove the row from the display buffer
        if let Some(display_buffer) = &mut self.display_buffer {
            display_buffer.remove(usize::try_from(row_index)?);
            for y in display_buffer.iter_mut() {
                if *y > actual_row_index {
                    *y -= 1;
                }
            }
        }

        let reverse_row_index = u32::try_from(actual_row_index as i64 + self.y_adjustment(true))?;

        Ok((reverse_row_index, Some(old_values)))
    }
}

#[cfg(test)]
pub mod test {
    use crate::{
        ArraySize, CellValue,
        grid::test::{new_data_table, pretty_print_data_table},
    };

    #[test]
    fn test_data_table_insert_row() {
        let (_, mut data_table) = new_data_table();
        data_table.apply_first_row_as_header();

        pretty_print_data_table(&data_table, Some("Original Data Table"), None);

        data_table.insert_row(4, None).unwrap();
        pretty_print_data_table(&data_table, Some("Data Table with New Row"), None);

        // this should be a 5x4 array
        let expected_size = ArraySize::new(4, 6).unwrap();
        assert_eq!(data_table.output_size(), expected_size);
    }

    #[test]
    fn test_data_table_delete_row() {
        let (_, mut source_data_table) = new_data_table();
        source_data_table.apply_first_row_as_header();

        pretty_print_data_table(&source_data_table, Some("Original Data Table"), None);

        let mut data_table = source_data_table.clone();
        data_table.delete_row(5).unwrap();
        pretty_print_data_table(&data_table, Some("Data Table without Seattle row"), None);

        // this should be a 4x3 array, includes the header row
        let expected_size = ArraySize::new(4, 4).unwrap();
        assert_eq!(data_table.output_size(), expected_size);

        let mut data_table = source_data_table.clone();
        data_table.delete_row(4).unwrap();
        pretty_print_data_table(&data_table, Some("Data Table without Denver row"), None);

        // this should be a 4x3 array, includes the header row
        let expected_size = ArraySize::new(4, 4).unwrap();
        assert_eq!(data_table.output_size(), expected_size);

        // Denver should no longer be at (0, 2)
        assert_eq!(
            data_table.cell_value_at(0, 2),
            Some(CellValue::Text("Southborough".to_string()))
        );
    }
}
