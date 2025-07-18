use std::str::FromStr;

use wasm_bindgen::prelude::*;

use crate::{Pos, a1::A1Context, grid::SheetId};

use super::{parse_and_check_formula, parse_formula::parse_formula_results};

#[wasm_bindgen(js_name = "parseFormula")]
pub fn parse_formula(
    formula_string: &str,
    context: &[u8],
    sheet_id: &str,
    x: i32,
    y: i32,
) -> Result<JsValue, String> {
    let ctx = serde_json::from_slice::<A1Context>(context).expect("invalid A1Context");
    let sheet_id = SheetId::from_str(sheet_id).map_err(|e| e.to_string())?;

    let results = parse_formula_results(formula_string, ctx, sheet_id, x, y);
    serde_wasm_bindgen::to_value(&results).map_err(|e| e.to_string())
}

#[wasm_bindgen(js_name = "checkFormula")]
pub fn check_formula(
    formula_string: &str,
    context: &[u8],
    sheet_id: &str,
    x: i32,
    y: i32,
) -> Result<bool, String> {
    let ctx = serde_json::from_slice::<A1Context>(context).map_err(|e| e.to_string())?;
    let sheet_id = SheetId::from_str(sheet_id).map_err(|e| e.to_string())?;
    let pos = Pos {
        x: x as i64,
        y: y as i64,
    }
    .to_sheet_pos(sheet_id);

    Ok(parse_and_check_formula(formula_string, &ctx, pos))
}

#[wasm_bindgen(js_name = "provideCompletionItems")]
pub fn provide_completion_items(
    _text_model: JsValue,
    _position: JsValue,
    _context: JsValue,
    _token: JsValue,
) -> Result<JsValue, JsValue> {
    Ok(serde_wasm_bindgen::to_value(
        &super::lsp::provide_completion_items(),
    )?)
}

#[wasm_bindgen(js_name = "provideHover")]
pub fn provide_hover(
    text_model: JsValue,
    position: JsValue,
    _token: JsValue,
) -> Result<JsValue, JsValue> {
    let partial_function_name = jsexpr!(text_model.getWordAtPosition(position).word)
        .as_string()
        .unwrap_or_default();
    let result = super::lsp::provide_hover(&partial_function_name);
    Ok(serde_wasm_bindgen::to_value(&result)?)
}
