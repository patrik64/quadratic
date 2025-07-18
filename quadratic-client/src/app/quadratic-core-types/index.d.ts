// This file is automatically generated by quadratic-core/src/bin/export_types.rs
// Do not modify it manually.

export type A1Error = { "type": "InvalidCellReference", "error": string } | { "type": "InvalidSheetId", "error": string } | { "type": "InvalidSheetMap", "error": string } | { "type": "InvalidColumn", "error": string } | { "type": "InvalidSheetName", "error": string } | { "type": "InvalidSheetNameMissingQuotes", "error": string } | { "type": "InvalidRange", "error": string } | { "type": "InvalidRow", "error": string } | { "type": "SpuriousDollarSign", "error": string } | { "type": "TooManySheets", "error": string } | { "type": "MismatchedQuotes", "error": string } | { "type": "WrongCellCount", "error": string } | { "type": "InvalidExclusion", "error": string } | { "type": "TranslateInvalid", "error": string } | { "type": "SheetNotFound" } | { "type": "InvalidTableRef", "error": string } | { "type": "TableNotFound", "error": string } | { "type": "MultipleColumnDefinitions" } | { "type": "MultipleRowDefinitions" } | { "type": "UnexpectedRowNumber" } | { "type": "InvalidRowRange", "error": string } | { "type": "OutOfBounds", "error": RefError };
export type A1Selection = { 
/**
 * Current sheet.
 *
 * Selections can only span a single sheet.
 */
sheet_id: SheetId, 
/**
 * Cursor position, which is moved using the arrow keys (while not holding
 * shift).
 *
 * This always coincides with the start of the last range in `ranges`, but
 * in the case of an infinite selection it contains information that cannot
 * be inferred from `ranges`.
 */
cursor: Pos, 
/**
 * Selected ranges (union).
 *
 * The cursor selection must always contain at least one range, and the
 * last range can be manipulated using the arrow keys.
 *
 * The `start` of the last range is where the cursor outline is drawn, and
 * can be moved by pressing arrow keys without holding the shift key.
 *
 * The `end` of the last range can be moved by pressing arrow keys while
 * holding the shift key.
 */
ranges: Array<CellRefRange>, };
export type ArraySize = { 
/**
 * Width (number of columns)
 */
w: number, 
/**
 * Height (number of rows)
 */
h: number, };
export type Axis = "X" | "Y";
export type BorderSelection = "all" | "inner" | "outer" | "horizontal" | "vertical" | "left" | "top" | "right" | "bottom" | "clear";
export type BorderSide = "Top" | "Bottom" | "Left" | "Right";
export type BorderStyle = { color: Rgba, line: CellBorderLine, };
export type BorderStyleCell = { top: BorderStyleTimestamp | null, bottom: BorderStyleTimestamp | null, left: BorderStyleTimestamp | null, right: BorderStyleTimestamp | null, };
export type BorderStyleTimestamp = { color: Rgba, line: CellBorderLine, timestamp: SmallTimestamp, };
export type JsCellsA1Error = { core_error: string, };
export type JsCellsA1Response = { values: JsCellsA1Values | null, error: JsCellsA1Error | null, };
export type JsCellsA1Value = { x: number, y: number, v: string, t: number, };
export type JsCellsA1Values = { cells: Array<JsCellsA1Value>, x: number, y: number, w: number, h: number, one_dimensional: boolean, two_dimensional: boolean, has_headers: boolean, };
export type JsCellValueResult = [string, number];
export type CellAlign = "center" | "left" | "right";
export type CellBorderLine = "line1" | "line2" | "line3" | "dotted" | "dashed" | "double" | "clear";
export type CellFormatSummary = { bold: boolean | null, italic: boolean | null, commas: boolean | null, textColor: string | null, fillColor: string | null, align: CellAlign | null, verticalAlign: CellVerticalAlign | null, wrap: CellWrap | null, dateTime: string | null, cellType: CellType | null, underline: boolean | null, strikeThrough: boolean | null, };
export type CellRefCoord = { coord: bigint, is_absolute: boolean, };
export type CellRefRange = { range: RefRangeBounds, } | { range: TableRef, };
export type CellRefRangeEnd = { col: CellRefCoord, row: CellRefCoord, };
export type CellVerticalAlign = "top" | "middle" | "bottom";
export type CellWrap = "overflow" | "wrap" | "clip";
export type CodeCellLanguage = "Python" | "Formula" | { "Connection": { kind: ConnectionKind, id: string, } } | "Javascript" | "Import";
export type ColumnRow = { column: number, row: number, };
export type ConnectionKind = "POSTGRES" | "MYSQL" | "MSSQL" | "SNOWFLAKE";
export type DataTableSort = { column_index: number, direction: SortDirection, };
export type DateTimeRange = { "DateRange": [bigint | null, bigint | null] } | { "DateEqual": Array<bigint> } | { "DateNotEqual": Array<bigint> } | { "TimeRange": [number | null, number | null] } | { "TimeEqual": Array<number> } | { "TimeNotEqual": Array<number> };
export type Direction = "Up" | "Down" | "Left" | "Right";
export type Format = { align: CellAlign | null, vertical_align: CellVerticalAlign | null, wrap: CellWrap | null, numeric_format: NumericFormat | null, numeric_decimals: number | null, numeric_commas: boolean | null, bold: boolean | null, italic: boolean | null, text_color: string | null, fill_color: string | null, date_time: string | null, underline: boolean | null, strike_through: boolean | null, };
export type FormatUpdate = { align: CellAlign | null | null, vertical_align: CellVerticalAlign | null | null, wrap: CellWrap | null | null, numeric_format: NumericFormat | null | null, numeric_decimals: number | null | null, numeric_commas: boolean | null | null, bold: boolean | null | null, italic: boolean | null | null, text_color: string | null | null, fill_color: string | null | null, render_size: RenderSize | null | null, date_time: string | null | null, underline: boolean | null | null, strike_through: boolean | null | null, };
export type GridBounds = { "type": "empty" } | { "type": "nonEmpty" } & Rect;
export type JsBorderHorizontal = { color: Rgba, line: CellBorderLine, x: bigint, y: bigint, width: bigint | null, unbounded: boolean, };
export type JsBorderVertical = { color: Rgba, line: CellBorderLine, x: bigint, y: bigint, height: bigint | null, unbounded: boolean, };
export type JsBordersSheet = { horizontal: Array<JsBorderHorizontal> | null, vertical: Array<JsBorderVertical> | null, };
export type JsCellsAccessed = { sheetId: string, ranges: Array<CellRefRange>, };
export type JsCellValue = { value: string, kind: string, };
export type JsCellValuePos = { value: string, kind: string, pos: string, };
export type JsCellValuePosContext = { sheet_name: string, rect_origin: string, rect_width: number, rect_height: number, starting_rect_values: Array<Array<JsCellValuePos>>, };
export type JsChartContext = { sheet_name: string, chart_name: string, bounds: string, language: CodeCellLanguage, code_string: string, spill: boolean, };
export type JsClipboard = { plainText: string, html: string, };
export type JsCodeCell = { x: bigint, y: bigint, code_string: string, language: CodeCellLanguage, std_out: string | null, std_err: string | null, evaluation_result: string | null, spill_error: Array<Pos> | null, return_info: JsReturnInfo | null, cells_accessed: Array<JsCellsAccessed> | null, last_modified: bigint, };
export type JsCodeResult = { transaction_id: string, success: boolean, std_out: string | null, std_err: string | null, line_number: number | null, output_value: JsCellValueResult | null, output_array: Array<Array<JsCellValueResult>> | null, output_display_type: string | null, cancel_compute: boolean | null, chart_pixel_output: [number, number] | null, has_headers: boolean, };
export type JsCodeTableContext = { sheet_name: string, code_table_name: string, all_columns: Array<string>, visible_columns: Array<string>, first_row_visible_values: Array<JsCellValuePos>, last_row_visible_values: Array<JsCellValuePos>, bounds: string, show_name: boolean, show_columns: boolean, language: CodeCellLanguage, code_string: string, std_err: string | null, error: boolean, spill: boolean, };
export type JsColumnWidth = { column: bigint, width: number, };
export type JsCoordinate = { x: number, y: number, };
export type JsDataTableColumnHeader = { name: string, display: boolean, valueIndex: number, };
export type JsDataTableContext = { sheet_name: string, data_table_name: string, all_columns: Array<string>, visible_columns: Array<string>, first_row_visible_values: Array<JsCellValuePos>, last_row_visible_values: Array<JsCellValuePos>, bounds: string, show_name: boolean, show_columns: boolean, };
export type JsFormulaParseResult = { parse_error_msg: string | null, parse_error_span: Span | null, cells_accessed: Array<JsCellsAccessed>, spans: Array<Span>, };
export type JsHashesDirty = { sheet_id: SheetId, hashes: Array<Pos>, };
export type JsHashRenderCells = { sheet_id: SheetId, hash: Pos, cells: Array<JsRenderCell>, };
export type JsHashValidationWarnings = { sheet_id: SheetId, hash: Pos | null, warnings: Array<JsValidationWarning>, };
export type JsHtmlOutput = { sheet_id: string, x: number, y: number, w: number, h: number, html: string | null, name: string, show_name: boolean, };
export type JsNumber = { decimals: number | null, commas: boolean | null, format: NumericFormat | null, };
export type JsOffset = { column: number | null, row: number | null, size: number, };
export type JsRenderCell = { x: bigint, y: bigint, value: string, 
/**
 * Code language, set only for the top left cell of a code output.
 */
language: CodeCellLanguage | null, align: CellAlign | null, verticalAlign: CellVerticalAlign | null, wrap: CellWrap | null, bold: boolean | null, italic: boolean | null, textColor: string | null, special: JsRenderCellSpecial | null, number: JsNumber | null, underline: boolean | null, strikeThrough: boolean | null, tableName: boolean | null, columnHeader: boolean | null, };
export type JsRenderCellSpecial = "Chart" | "SpillError" | "RunError" | "Logical" | "Checkbox" | "List";
export type JsRenderCodeCell = { x: number, y: number, w: number, h: number, language: CodeCellLanguage, state: JsRenderCodeCellState, spill_error: Array<Pos> | null, name: string, columns: Array<JsDataTableColumnHeader>, first_row_header: boolean, sort: Array<DataTableSort> | null, sort_dirty: boolean, alternating_colors: boolean, is_code: boolean, is_html: boolean, is_html_image: boolean, show_name: boolean, show_columns: boolean, last_modified: bigint, };
export type JsRenderCodeCellState = "NotYetRun" | "RunError" | "SpillError" | "Success" | "HTML" | "Image";
export type JsRenderFill = { x: bigint, y: bigint, w: number, h: number, color: string, };
export type JsResponse = { result: boolean, error: string | null, };
export type JsReturnInfo = { line_number: number | null, output_type: string | null, };
export type JsRowHeight = { row: bigint, height: number, };
export type JsSelectionContext = { sheet_name: string, data_rects: Array<JsCellValuePosContext>, errored_code_cells: Array<JsCodeCell> | null, tables_summary: Array<JsTableSummaryContext> | null, charts_summary: Array<JsChartSummaryContext> | null, };
export type JsSheetFill = { x: number, y: number, w: number | null, h: number | null, color: string, };
export type JsSnackbarSeverity = "error" | "warning" | "success";
export type JsSummarizeSelectionResult = { count: bigint, sum: number | null, average: number | null, };
export type JsTableInfo = { name: string, sheet_name: string, chart: boolean, language: CodeCellLanguage, };
export type JsTablesContext = { sheet_name: string, data_tables: Array<JsDataTableContext>, code_tables: Array<JsCodeTableContext>, charts: Array<JsChartContext>, };
export type JsUpdateCodeCell = { sheet_id: SheetId, pos: Pos, render_code_cell: JsRenderCodeCell | null, };
export type JsValidationWarning = { pos: Pos, validation: string | null, style: ValidationStyle | null, };
export type MinMax = { min: number, max: number, };
export type NumberRange = { "Range": [number | null, number | null] } | { "Equal": Array<number> } | { "NotEqual": Array<number> };
export type NumericFormat = { type: NumericFormatKind, symbol: string | null, };
export type NumericFormatKind = "NUMBER" | "CURRENCY" | "PERCENTAGE" | "EXPONENTIAL";
export type PasteSpecial = "None" | "Values" | "Formats";
export type Pos = { 
/**
 * Column
 */
x: bigint, 
/**
 * Row
 */
y: bigint, };
export type Rect = { 
/**
 * Upper-left corner.
 */
min: Pos, 
/**
 * Lower-right corner.
 */
max: Pos, };
export type RefRangeBounds = { start: CellRefRangeEnd, end: CellRefRangeEnd, };
export type Rgba = { red: number, green: number, blue: number, alpha: number, };
export type RunError = { 
/**
 * Location of the source code where the error occurred (if any).
 */
span: Span | null, 
/**
 * Type of error.
 */
msg: RunErrorMsg, };
export type RunErrorMsg = { "CodeRunError": string } | "Spill" | { "Unimplemented": string } | "UnknownError" | { "InternalError": string } | { "Unterminated": string } | { "Expected": { expected: string, got: string | null, } } | { "Unexpected": string } | { "TooManyArguments": { func_name: string, max_arg_count: number, } } | { "MissingRequiredArgument": { func_name: string, arg_name: string, } } | "BadFunctionName" | "BadCellReference" | "BadNumber" | { "BadOp": { op: string, ty1: string, ty2: string | null, use_duration_instead: boolean, } } | { "ExactArraySizeMismatch": { expected: ArraySize, got: ArraySize, } } | { "ExactArrayAxisMismatch": { axis: Axis, expected: number, got: number, } } | { "ArrayAxisMismatch": { axis: Axis, expected: number, got: number, } } | "EmptyArray" | "NonRectangularArray" | "NonLinearArray" | "ArrayTooBig" | "NotAvailable" | "Name" | "Null" | "Num" | "Value" | "CircularReference" | "Overflow" | "DivideByZero" | "NegativeExponent" | "NaN" | "IndexOutOfBounds" | "NoMatch" | "InvalidArgument" | "NotANumber" | "Infinity";
export type SearchOptions = { case_sensitive: boolean | null, whole_cell: boolean | null, search_code: boolean | null, sheet_id: string | null, };
export type SheetBounds = { sheet_id: string, bounds: GridBounds, bounds_without_formatting: GridBounds, format_bounds: GridBounds, };
export type SheetId = { id: string, };
export type SheetInfo = { sheet_id: string, name: string, order: string, color: string | null, offsets: string, bounds: GridBounds, bounds_without_formatting: GridBounds, format_bounds: GridBounds, };
export type SheetPos = { x: bigint, y: bigint, sheet_id: SheetId, };
export type SheetRect = { 
/**
 * Upper-left corner.
 */
min: Pos, 
/**
 * Lower-right corner.
 */
max: Pos, 
/**
 * The sheet that this region is on.
 */
sheet_id: SheetId, };
export type SmallTimestamp = number;
export type SortDirection = "Ascending" | "Descending" | "None";
export type Span = { 
/**
 * The byte index of the first character.
 */
start: number, 
/**
 * The byte index after the last character.
 */
end: number, };
export type TableRef = { table_name: string, data: boolean, headers: boolean, totals: boolean, col_range: ColRange, };
export type TextCase = { "CaseInsensitive": Array<string> } | { "CaseSensitive": Array<string> };
export type TextMatch = { "Exactly": TextCase } | { "Contains": TextCase } | { "NotContains": TextCase } | { "TextLength": { min: number | null, max: number | null, } };
export type TransactionName = "Unknown" | "ResizeColumn" | "ResizeRow" | "ResizeRows" | "ResizeColumns" | "Autocomplete" | "SetBorders" | "SetCells" | "SetFormats" | "SetDataTableAt" | "CutClipboard" | "PasteClipboard" | "SetCode" | "RunCode" | "FlattenDataTable" | "SwitchDataTableKind" | "GridToDataTable" | "DataTableMeta" | "DataTableMutations" | "DataTableFirstRowAsHeader" | "DataTableAddDataTable" | "Import" | "SetSheetMetadata" | "SheetAdd" | "SheetDelete" | "DuplicateSheet" | "MoveCells" | "Validation" | "ManipulateColumnRow";
export type TransientResize = { row: bigint | null, column: bigint | null, old_size: number, new_size: number, };
export type Validation = { id: string, selection: A1Selection, rule: ValidationRule, message: ValidationMessage, error: ValidationError, };
export type ValidationDateTime = { ignore_blank: boolean, require_date: boolean, require_time: boolean, prohibit_date: boolean, prohibit_time: boolean, ranges: Array<DateTimeRange>, };
export type ValidationError = { show: boolean, style: ValidationStyle, title: string | null, message: string | null, };
export type ValidationList = { source: ValidationListSource, ignore_blank: boolean, drop_down: boolean, };
export type ValidationListSource = { "Selection": A1Selection } | { "List": Array<string> };
export type ValidationLogical = { show_checkbox: boolean, ignore_blank: boolean, };
export type ValidationMessage = { show: boolean, title: string | null, message: string | null, };
export type ValidationNumber = { ignore_blank: boolean, ranges: Array<NumberRange>, };
export type ValidationRule = "None" | { "List": ValidationList } | { "Logical": ValidationLogical } | { "Text": ValidationText } | { "Number": ValidationNumber } | { "DateTime": ValidationDateTime };
export type ValidationStyle = "Stop" | "Warning" | "Information";
export type ValidationText = { ignore_blank: boolean, text_match: Array<TextMatch>, };
