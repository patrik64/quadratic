import { hasPermissionToEditFile } from '@/app/actions';
import { Action } from '@/app/actions/actions';
import { debug } from '@/app/debugFlags';
import { sheets } from '@/app/grid/controller/Sheets.js';
import { zoomIn, zoomOut, zoomTo100, zoomToFit, zoomToSelection } from '@/app/gridGL/helpers/zoom';
import { pixiApp } from '@/app/gridGL/pixiApp/PixiApp';
import { pixiAppSettings } from '@/app/gridGL/pixiApp/PixiAppSettings';
import { matchShortcut } from '@/app/helpers/keyboardShortcuts.js';
import {
  clearFormattingAndBorders,
  setBold,
  setItalic,
  setStrikeThrough,
  setUnderline,
} from '@/app/ui/helpers/formatCells';
import { javascriptWebWorker } from '@/app/web-workers/javascriptWebWorker/javascriptWebWorker.js';
import { pythonWebWorker } from '@/app/web-workers/pythonWebWorker/pythonWebWorker';
import { quadraticCore } from '@/app/web-workers/quadraticCore/quadraticCore.js';

export function keyboardViewport(event: React.KeyboardEvent<HTMLElement>): boolean {
  const { pointer } = pixiApp;
  const {
    editorInteractionState,
    setEditorInteractionState,
    codeEditorState,
    setCodeEditorState,
    gridSettings,
    setGridSettings,
  } = pixiAppSettings;

  if (!setEditorInteractionState) {
    throw new Error('Expected setEditorInteractionState to be defined in keyboardViewport');
  }

  if (!setCodeEditorState) {
    throw new Error('Expected setCodeEditorState to be defined in keyboardViewport');
  }

  if (!setGridSettings) {
    throw new Error('Expected d to be defined in keyboardViewport');
  }

  // Show command palette
  if (matchShortcut(Action.ShowCommandPalette, event)) {
    setEditorInteractionState({
      ...editorInteractionState,
      showFeedbackMenu: false,
      showCellTypeMenu: false,
      showGoToMenu: false,
      showShareFileMenu: false,
      showCommandPalette: !editorInteractionState.showCommandPalette,
    });
    return true;
  }

  // Toggle presentation mode
  if (matchShortcut(Action.TogglePresentationMode, event)) {
    setGridSettings({ ...gridSettings, presentationMode: !gridSettings.presentationMode });
    return true;
  }

  // Close overlay
  if (matchShortcut(Action.CloseOverlay, event)) {
    if (gridSettings.presentationMode) {
      setGridSettings({ ...gridSettings, presentationMode: false });
      return true;
    } else if (codeEditorState.showCodeEditor) {
      setCodeEditorState({
        ...codeEditorState,
        escapePressed: true,
      });
      return true;
    } else if (editorInteractionState.showValidation) {
      // todo: this should check for changes first!!!
      setEditorInteractionState({
        ...editorInteractionState,
        showValidation: false,
      });
      return true;
    }
    return pointer.handleEscape();
  }

  // Show go to menu
  if (matchShortcut(Action.ShowGoToMenu, event)) {
    setEditorInteractionState({
      ...editorInteractionState,
      showFeedbackMenu: false,
      showCellTypeMenu: false,
      showCommandPalette: false,
      showGoToMenu: !editorInteractionState.showGoToMenu,
    });
    return true;
  }

  // Zoom in
  if (matchShortcut(Action.ZoomIn, event)) {
    zoomIn();
    return true;
  }

  // Zoom out
  if (matchShortcut(Action.ZoomOut, event)) {
    zoomOut();
    return true;
  }

  // Zoom to selection
  if (matchShortcut(Action.ZoomToSelection, event)) {
    zoomToSelection();
    return true;
  }

  // Zoom to fit
  if (matchShortcut(Action.ZoomToFit, event)) {
    zoomToFit();
    return true;
  }

  // Zoom to 100%
  if (matchShortcut(Action.ZoomTo100, event)) {
    zoomTo100();
    return true;
  }

  // Save
  if (matchShortcut(Action.Save, event)) {
    // don't do anything on Command+S
    return true;
  }

  // Switch to next sheet
  if (matchShortcut(Action.SwitchSheetNext, event)) {
    if (sheets.size > 1) {
      const nextSheet = sheets.getNext(sheets.sheet.order) ?? sheets.getFirst();
      sheets.current = nextSheet.id;
    }
    return true;
  }

  // Switch to previous sheet
  if (matchShortcut(Action.SwitchSheetPrevious, event)) {
    if (sheets.size > 1) {
      const previousSheet = sheets.getPrevious(sheets.sheet.order) ?? sheets.getLast();
      sheets.current = previousSheet.id;
    }
    return true;
  }

  // All formatting options past here are only available for people with rights
  if (!hasPermissionToEditFile(editorInteractionState.permissions)) {
    return false;
  }

  // Clear formatting and borders
  if (matchShortcut(Action.ClearFormattingBorders, event)) {
    clearFormattingAndBorders();
    return true;
  }

  // Toggle bold
  if (matchShortcut(Action.ToggleBold, event)) {
    setBold();
    return true;
  }

  // Toggle italic
  if (matchShortcut(Action.ToggleItalic, event)) {
    setItalic();
    return true;
  }

  // Toggle underline
  if (matchShortcut(Action.ToggleUnderline, event)) {
    setUnderline();
    return true;
  }

  // Toggle strike-through
  if (matchShortcut(Action.ToggleStrikeThrough, event)) {
    setStrikeThrough();
    return true;
  }

  // Fill right
  // Disabled in debug mode, to allow page reload
  if (!debug && matchShortcut(Action.FillRight, event)) {
    const cursor = sheets.sheet.cursor;
    if (cursor.columnRow?.all || cursor.columnRow?.rows) return true;
    if (cursor.columnRow?.columns && cursor.multiCursor) return true;
    if (cursor.columnRow?.columns) {
      if (cursor.columnRow.columns.length > 1) return true;
      const column = cursor.columnRow.columns[0];
      const bounds = sheets.sheet.getBounds(false);
      if (!bounds) return true;
      quadraticCore.autocomplete(
        sheets.current,
        column - 1,
        bounds.top,
        column - 1,
        bounds.bottom,
        column - 1,
        bounds.top,
        column,
        bounds.bottom
      );
    } else if (cursor.multiCursor) {
      if (cursor.multiCursor.length > 1) return true;
      const rectangle = cursor.multiCursor[0];
      if (rectangle.width > 1) return true;
      quadraticCore.autocomplete(
        sheets.current,
        rectangle.x - 1,
        rectangle.top,
        rectangle.x - 1,
        rectangle.bottom,
        rectangle.x - 1,
        rectangle.top,
        rectangle.x,
        rectangle.bottom
      );
    } else {
      const position = cursor.cursorPosition;
      quadraticCore.autocomplete(
        sheets.current,
        position.x - 1,
        position.y,
        position.x - 1,
        position.y,
        position.x - 1,
        position.y,
        position.x,
        position.y
      );
    }

    return true;
  }

  // Fill down
  if (matchShortcut(Action.FillDown, event)) {
    const cursor = sheets.sheet.cursor;
    if (cursor.columnRow?.all || cursor.columnRow?.columns) return true;
    if (cursor.columnRow?.rows && cursor.multiCursor) return true;
    if (cursor.columnRow?.rows) {
      if (cursor.columnRow.rows.length > 1) return true;
      const row = cursor.columnRow.rows[0];
      const bounds = sheets.sheet.getBounds(false);
      if (!bounds) return true;
      quadraticCore.autocomplete(
        sheets.current,
        bounds.left,
        row - 1,
        bounds.right,
        row - 1,
        bounds.left,
        row - 1,
        bounds.right,
        row
      );
    } else if (cursor.multiCursor) {
      if (cursor.multiCursor.length > 1) return true;
      const rectangle = cursor.multiCursor[0];
      if (rectangle.height > 1) return true;
      quadraticCore.autocomplete(
        sheets.current,
        rectangle.left,
        rectangle.top - 1,
        rectangle.right,
        rectangle.top - 1,
        rectangle.left,
        rectangle.top - 1,
        rectangle.right,
        rectangle.top
      );
    } else {
      const position = cursor.cursorPosition;
      quadraticCore.autocomplete(
        sheets.current,
        position.x,
        position.y - 1,
        position.x,
        position.y - 1,
        position.x,
        position.y - 1,
        position.x,
        position.y
      );
    }

    return true;
  }

  // Cancel execution
  if (matchShortcut(Action.CancelExecution, event)) {
    pythonWebWorker.cancelExecution();
    javascriptWebWorker.cancelExecution();
  }

  return false;
}
