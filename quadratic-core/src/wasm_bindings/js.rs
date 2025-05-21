use super::*;

#[cfg(not(test))]
use js_sys::SharedArrayBuffer;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = JSON)]
    pub(crate) fn stringify(value: &JsValue) -> String;
}

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub(crate) fn log(s: &str);
}

#[cfg(not(test))]
#[wasm_bindgen(
    module = "/../quadratic-client/src/app/web-workers/quadraticCore/worker/rustCallbacks.ts"
)]
extern "C" {
    pub fn jsTime(name: String);
    pub fn jsTimeEnd(name: String);

    pub fn jsRunPython(
        transactionId: String,
        x: i32,
        y: i32,
        sheet_id: String,
        code: String,
    ) -> JsValue;

    pub fn jsRunJavascript(
        transactionId: String,
        x: i32,
        y: i32,
        sheet_id: String,
        code: String,
    ) -> JsValue;

    // cells: Vec<JsRenderCell>
    pub fn jsRenderCellSheets(
        sheet_id: String,
        hash_x: i64,
        hash_y: i64,
        cells: String, /*Vec<JsRenderCell>*/
    );

    pub fn jsSheetInfo(sheets: String /* Vec<JsSheetInfo> */);
    pub fn jsSheetInfoUpdate(sheet: String /* JsSheetInfo */);

    // todo: there should be a jsSheetFillUpdate instead of constantly passing back all sheet fills
    pub fn jsSheetFills(sheet_id: String, fills: String /* JsRenderFill */);

    pub fn jsSheetMetaFills(sheet_id: String, fills: String /* JsSheetFill */);

    pub fn jsAddSheet(sheetInfo: String /*SheetInfo*/, user: bool);
    pub fn jsDeleteSheet(sheetId: String, user: bool);
    pub fn jsRequestTransactions(sequence_num: u64);
    pub fn jsUpdateCodeCell(
        sheet_id: String,
        x: i64,
        y: i64,
        code_cell: Option<String>,        /*JsCodeCell*/
        render_code_cell: Option<String>, /*JsRenderCodeCell*/
    );
    pub fn jsOffsetsModified(sheet_id: String, offsets: String /* Vec<JsOffset> */);
    pub fn jsSetCursor(cursor: String);
    pub fn jsUpdateHtml(html: String /*JsHtmlOutput*/);
    pub fn jsHtmlOutput(html: String /*Vec<JsHtmlOutput>*/);
    pub fn jsGenerateThumbnail();
    pub fn jsBordersSheet(sheet_id: String, borders: String /* JsBordersSheet */);
    pub fn jsSheetCodeCell(sheet_id: String, code_cells: String);
    pub fn jsSheetBoundsUpdate(bounds: String);

    pub fn jsImportProgress(
        file_name: &str,
        current: u32,
        total: u32,
        x: i64,
        y: i64,
        w: u32,
        h: u32,
    );
    pub fn jsTransactionStart(transaction_id: String, name: String);
    pub fn jsTransactionProgress(transaction_id: String, remaining_operations: i32);
    pub fn jsTransactionEnd(transaction_id: String, name: String);

    pub fn addUnsentTransaction(transaction_id: String, transaction: String, operations: u32);
    pub fn jsSendTransaction(transaction_id: String, transaction: Vec<u8>);

    pub fn jsUndoRedo(undo: bool, redo: bool);

    pub fn jsConnection(
        transactionId: String,
        x: i32,
        y: i32,
        sheet_id: String,
        query: String,
        connector_type: ConnectionKind,
        connection_id: String,
    );

    pub fn jsSendImage(sheet_id: String, x: i32, y: i32, w: i32, h: i32, image: Option<String>);

    // rows: Vec<i64>
    pub fn jsRequestRowHeights(transaction_id: String, sheet_id: String, rows: String);

    pub fn jsSheetValidations(sheet_id: String, validations: String /* Vec<Validation> */);
    pub fn jsValidationWarning(
        sheet_id: String,
        validations: String, /* Vec<(x, y, validation_id, failed) */
    );
    pub fn jsRenderValidationWarnings(
        sheet_id: String,
        hash_x: i64,
        hash_y: i64,
        validations: String, /* Vec<(x, y, id) */
    );

    pub fn jsMultiplayerSynced();

    // hashes: Vec<Pos>
    pub fn jsHashesDirty(sheet_id: String, hashes: String);

    pub fn jsSendViewportBuffer(buffer: SharedArrayBuffer);

    pub fn jsClientMessage(message: String, error: String);

    pub fn jsA1Context(context: String);
}

#[cfg(test)]
use std::sync::Mutex;

#[cfg(test)]
thread_local! {
    /// Mock for JS calls used in tests.
    ///
    /// This must be cleared at the beginning of each test by using
    /// [`clear_js_calls()`].
    static JS_CALLS: Mutex<Vec<TestFunction>> = const { Mutex::new(vec![]) };
}

#[cfg(test)]
fn js_call(s: &str, args: String) {
    let test_function = TestFunction::new(s, args);
    JS_CALLS.with(|js_calls| js_calls.lock().unwrap().push(test_function));
}

#[cfg(test)]
pub fn print_js_calls() {
    JS_CALLS.with(|js_calls| {
        let js_calls = js_calls.lock().unwrap();
        println!("JS calls:");
        for call in js_calls.iter() {
            println!("  {:?}", call);
        }
    });
}

#[cfg(test)]
#[track_caller]
pub fn expect_js_call(name: &str, args: String, clear: bool) {
    JS_CALLS.with(|js_calls| {
        let mut js_calls = js_calls.lock().unwrap();
        let result = TestFunction {
            name: name.to_string(),
            args,
        };
        let index = js_calls.iter().position(|x| *x == result);
        match index {
            Some(index) => {
                js_calls.remove(index);
            }
            None => {
                dbg!(&js_calls);
                panic!("Expected to find in TEST_ARRAY: {:?}", result)
            }
        }
        if clear {
            js_calls.clear();
        }
    });
}

#[cfg(test)]
#[track_caller]
pub fn expect_js_call_count(name: &str, count: usize, clear: bool) {
    JS_CALLS.with(|js_calls| {
        let mut js_calls = js_calls.lock().unwrap();
        let mut found = 0;
        js_calls.retain(|x| {
            if x.name == name {
                found += 1;
                return false;
            }
            true
        });
        assert_eq!(found, count);
        if clear {
            js_calls.clear();
        }
    });
}

#[cfg(test)]
use js_types::JsOffset;

#[cfg(test)]
use std::collections::HashMap;

#[cfg(test)]
#[track_caller]
pub fn expect_js_offsets(
    sheet_id: SheetId,
    offsets: HashMap<(Option<i64>, Option<i64>), f64>,
    clear: bool,
) {
    let mut offsets = offsets
        .iter()
        .map(|(&(column, row), &size)| JsOffset {
            column: column.map(|c| c as i32),
            row: row.map(|r| r as i32),
            size,
        })
        .collect::<Vec<JsOffset>>();

    offsets.sort_by(|a, b| a.row.cmp(&b.row).then(a.column.cmp(&b.column)));

    let offsets = serde_json::to_string(&offsets).unwrap();

    expect_js_call(
        "jsOffsetsModified",
        format!("{},{}", sheet_id, offsets),
        clear,
    );
}

#[cfg(test)]
pub fn clear_js_calls() {
    JS_CALLS.with(|js_calls| js_calls.lock().unwrap().clear());
}

#[cfg(test)]
#[derive(serde::Serialize, Debug, PartialEq)]
pub struct TestFunction {
    pub name: String,
    pub args: String,
}

#[cfg(test)]
impl TestFunction {
    pub fn new(name: &str, args: String) -> Self {
        Self {
            name: name.to_string(),
            args,
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsTime(name: String) {
    js_call("jsTime", name);
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsTimeEnd(name: String) {
    js_call("jsTimeEnd", name);
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsRunPython(
    transactionId: String,
    x: i32,
    y: i32,
    sheet_id: String,
    code: String,
) -> JsValue {
    js_call(
        "jsRunPython",
        format!("{},{},{},{},{}", transactionId, x, y, sheet_id, code),
    );
    JsValue::NULL
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsRunJavascript(
    transactionId: String,
    x: i32,
    y: i32,
    sheet_id: String,
    code: String,
) -> JsValue {
    js_call(
        "jsRunJavascript",
        format!("{},{},{},{},{}", transactionId, x, y, sheet_id, code),
    );
    JsValue::NULL
}

#[cfg(test)]
pub fn hash_test<T: std::hash::Hash>(value: &T) -> u64 {
    use std::hash::{DefaultHasher, Hasher};
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsRenderCellSheets(
    sheet_id: String,
    hash_x: i64,
    hash_y: i64,
    cells: String, /*Vec<JsRenderCell>*/
) {
    // we use a hash of cells to avoid storing too large test data
    js_call(
        "jsRenderCellSheets",
        format!("{},{},{},{}", sheet_id, hash_x, hash_y, hash_test(&cells)),
    );
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsSheetInfo(sheets: String /* Vec<JsSheetInfo> */) {
    js_call("jsSheetInfo", sheets);
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsSheetInfoUpdate(sheet: String /* JsSheetInfo */) {
    js_call("jsSheetInfoUpdate", sheet);
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsSheetFills(sheet_id: String, fills: String /* JsRenderFill */) {
    js_call("jsSheetFills", format!("{},{}", sheet_id, fills));
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsSheetMetaFills(sheet_id: String, fills: String /* JsSheetFill */) {
    js_call("jsSheetMetaFills", format!("{},{}", sheet_id, fills));
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsAddSheet(sheetInfo: String /*SheetInfo*/, user: bool) {
    js_call("jsAddSheet", format!("{},{}", sheetInfo, user));
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsDeleteSheet(sheetId: String, user: bool) {
    js_call("jsDeleteSheet", format!("{},{}", sheetId, user));
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsRequestTransactions(sequence_num: u64) {
    js_call("jsRequestTransactions", sequence_num.to_string());
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsUpdateCodeCell(
    sheet_id: String,
    x: i64,
    y: i64,
    code_cell: Option<String>,        /*JsCodeCell*/
    render_code_cell: Option<String>, /*JsRenderCodeCell*/
) {
    js_call(
        "jsUpdateCodeCell",
        format!(
            "{},{},{},{:?},{:?}",
            sheet_id, x, y, code_cell, render_code_cell
        ),
    );
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsOffsetsModified(sheet_id: String, offsets: String /*Vec<JsOffset>*/) {
    js_call("jsOffsetsModified", format!("{},{}", sheet_id, offsets));
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsSetCursor(cursor: String) {
    js_call("jsSetCursor", cursor);
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsUpdateHtml(html: String /*JsHtmlOutput*/) {
    js_call("jsUpdateHtml", html);
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsHtmlOutput(html: String /*Vec<JsHtmlOutput>*/) {
    js_call("jsHtmlOutput", html);
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsGenerateThumbnail() {
    js_call("jsGenerateThumbnail", "".to_string());
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsBordersSheet(sheet_id: String, borders: String /* JsBordersSheet */) {
    js_call("jsBordersSheet", format!("{},{}", sheet_id, borders));
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsSheetCodeCell(sheet_id: String, code_cells: String) {
    js_call("jsSheetCodeCell", format!("{},{}", sheet_id, code_cells));
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsSheetBoundsUpdate(bounds: String) {
    js_call("jsSheetBoundsUpdate", bounds);
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsImportProgress(file_name: &str, current: u32, total: u32, x: i64, y: i64, w: u32, h: u32) {
    js_call(
        "jsImportProgress",
        format!(
            "{},{},{},{},{},{},{}",
            file_name, current, total, x, y, w, h
        ),
    );
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsTransactionStart(transaction_id: String, name: String) {
    js_call(
        "jsTransactionStart",
        format!("{},{}", transaction_id, name,),
    );
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsTransactionProgress(transaction_id: String, remaining_operations: i32) {
    js_call(
        "jsTransactionProgress",
        format!("{},{}", transaction_id, remaining_operations),
    );
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsTransactionEnd(transaction_id: String, name: String) {
    js_call("jsTransactionEnd", format!("{},{}", transaction_id, name,));
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn addUnsentTransaction(transaction_id: String, transaction: String, operations: u32) {
    js_call(
        "addUnsentTransaction",
        format!("{},{},{}", transaction_id, transaction, operations),
    );
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsSendTransaction(transaction_id: String, _transaction: Vec<u8>) {
    // We do not include the actual transaction as we don't want to save that in
    // the TEST_ARRAY because of its potential size.
    js_call("jsSendTransaction", transaction_id.to_string());
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsUndoRedo(undo: bool, redo: bool) {
    js_call("jsUndoRedo", format!("{},{}", undo, redo));
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsConnection(
    transactionId: String,
    x: i32,
    y: i32,
    sheet_id: String,
    query: String,
    connector_type: ConnectionKind,
    connection_id: String,
) -> JsValue {
    js_call(
        "jsConnection",
        format!(
            "{},{},{},{},{},{},{}",
            transactionId, x, y, sheet_id, query, connector_type, connection_id
        ),
    );
    JsValue::NULL
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::too_many_arguments)]
pub fn jsSendImage(sheet_id: String, x: i32, y: i32, w: i32, h: i32, image: Option<String>) {
    js_call(
        "jsSendImage",
        format!(
            "{},{},{},{:?},{:?},{:?}",
            sheet_id,
            x,
            y,
            image.is_some(),
            w,
            h,
        ),
    );
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsSheetValidations(sheet_id: String, validations: String /* JsValidation */) {
    js_call(
        "jsSheetValidations",
        format!("{},{}", sheet_id, validations),
    );
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsRequestRowHeights(
    transaction_id: String,
    sheet_id: String,
    rows: String, /*Vec<i64>*/
) {
    js_call(
        "jsRequestRowHeights",
        format!("{},{},{}", transaction_id, sheet_id, rows),
    );
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsValidationWarning(
    sheet_id: String,
    validations: String, /* Vec<(x, y, validation_id, failed) */
) {
    js_call(
        "jsValidationWarning",
        format!("{},{}", sheet_id, validations),
    );
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsRenderValidationWarnings(
    sheet_id: String,
    hash_x: i64,
    hash_y: i64,
    validations: String, /* Vec(x, y, id) */
) {
    js_call(
        "jsRenderValidationWarnings",
        format!(
            "{},{},{},{}",
            sheet_id,
            hash_x,
            hash_y,
            hash_test(&validations)
        ),
    );
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsMultiplayerSynced() {
    js_call("jsMultiplayerSynced", "".into());
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsHashesDirty(sheet_id: String, hashes: String /*Vec<Pos>*/) {
    js_call("jsHashesDirty", format!("{},{}", sheet_id, hashes));
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsSendViewportBuffer(buffer: [u8; 112]) {
    js_call("jsSendViewportBuffer", format!("{:?}", buffer));
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsClientMessage(message: String, severity: String) {
    js_call("jsClientMessage", format!("{},{}", message, severity));
}

#[cfg(test)]
#[allow(non_snake_case)]
pub fn jsA1Context(context: String) {
    js_call("jsA1Context", context);
}
