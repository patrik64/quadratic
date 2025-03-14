import { PanMode } from '@/app/atoms/gridPanModeAtom';
import { events } from '@/app/events/events';
import { sheets } from '@/app/grid/controller/Sheets';
import { intersects } from '@/app/gridGL/helpers/intersects';
import { htmlCellsHandler } from '@/app/gridGL/HTMLGrid/htmlCells/htmlCellsHandler';
import { pixiApp } from '@/app/gridGL/pixiApp/PixiApp';
import { pixiAppSettings } from '@/app/gridGL/pixiApp/PixiAppSettings';
import { quadraticCore } from '@/app/web-workers/quadraticCore/quadraticCore';
import { rectToSheetRect } from '@/app/web-workers/quadraticCore/worker/rustConversions';
import { Point, Rectangle } from 'pixi.js';
import { isMobile } from 'react-device-detect';

// Distance from top left corner to trigger a cell move.
const TOP_LEFT_CORNER_THRESHOLD_SQUARED = 50;
const BORDER_THRESHOLD = 8;

interface MoveCells {
  column: number;
  row: number;
  width: number;
  height: number;
  toColumn: number;
  toRow: number;
  offset: { x: number; y: number };
  original?: Rectangle;
}

export class PointerCellMoving {
  private startCell?: Point;
  movingCells?: MoveCells;
  state?: 'hover' | 'move';

  get cursor(): string | undefined {
    switch (this.state) {
      case 'move':
        return 'grabbing';
      case 'hover':
        return 'grab';
      default:
        return undefined;
    }
  }

  private startMove = () => {
    this.state = 'move';
    events.emit('cellMoving', true);
    pixiApp.viewport.enableMouseEdges();
    htmlCellsHandler.disable();
  };

  // Starts a table move.
  tableMove = (column: number, row: number, point: Point, width: number, height: number) => {
    if (this.state) return false;
    this.startCell = new Point(column, row);
    const offset = sheets.sheet.getColumnRowFromScreen(point.x, point.y);
    this.movingCells = {
      column,
      row,
      width,
      height,
      toColumn: column,
      toRow: row,
      offset: { x: column - offset.column, y: row - offset.row },
      original: new Rectangle(column, row, width, height),
    };
    this.startMove();
  };

  pointerDown = (event: PointerEvent): boolean => {
    if (isMobile || pixiAppSettings.panMode !== PanMode.Disabled || event.button === 1) return false;

    if (this.state === 'hover' && this.movingCells && event.button === 0) {
      this.startCell = new Point(this.movingCells.column, this.movingCells.row);
      this.startMove();
      return true;
    }
    return false;
  };

  private reset = () => {
    this.movingCells = undefined;
    if (this.state === 'move') {
      pixiApp.cellMoving.dirty = true;
      events.emit('cellMoving', false);
      pixiApp.viewport.disableMouseEdges();
    }
    this.state = undefined;
    this.startCell = undefined;
    htmlCellsHandler.enable();
  };

  private pointerMoveMoving = (world: Point) => {
    if (this.state !== 'move' || !this.movingCells) {
      throw new Error('Expected moving to be defined in pointerMoveMoving');
    }
    pixiApp.viewport.enableMouseEdges();
    const sheet = sheets.sheet;
    const position = sheet.getColumnRowFromScreen(world.x, world.y);
    this.movingCells.toColumn = Math.max(1, position.column + this.movingCells.offset.x);
    this.movingCells.toRow = Math.max(1, position.row + this.movingCells.offset.y);
    pixiApp.cellMoving.dirty = true;
  };

  private moveOverlaps = (world: Point): false | 'corner' | 'top' | 'bottom' | 'left' | 'right' => {
    const cursorRectangle = pixiApp.cursor.cursorRectangle;
    if (!cursorRectangle) return false;

    // top-left corner + threshold
    if (
      Math.pow(cursorRectangle.x - world.x, 2) + Math.pow(cursorRectangle.y - world.y, 2) <=
      TOP_LEFT_CORNER_THRESHOLD_SQUARED
    ) {
      return 'corner';
    }

    // if overlap indicator (autocomplete), then return false
    const indicator = pixiApp.cursor.indicator;
    if (intersects.rectanglePoint(indicator, world)) {
      return false;
    }

    // if overlaps any of the borders (with threshold), then return true
    const left = new Rectangle(
      cursorRectangle.x - BORDER_THRESHOLD / 2,
      cursorRectangle.y,
      BORDER_THRESHOLD,
      cursorRectangle.height
    );
    if (intersects.rectanglePoint(left, world)) {
      return 'left';
    }

    const right = new Rectangle(
      cursorRectangle.x + cursorRectangle.width - BORDER_THRESHOLD / 2,
      cursorRectangle.y,
      BORDER_THRESHOLD,
      cursorRectangle.height
    );
    if (intersects.rectanglePoint(right, world)) {
      return 'right';
    }

    const top = new Rectangle(
      cursorRectangle.x,
      cursorRectangle.y - BORDER_THRESHOLD / 2,
      cursorRectangle.width,
      BORDER_THRESHOLD
    );
    if (intersects.rectanglePoint(top, world)) {
      return 'top';
    }
    const bottom = new Rectangle(
      cursorRectangle.x,
      cursorRectangle.y + cursorRectangle.height - BORDER_THRESHOLD / 2,
      cursorRectangle.width,
      BORDER_THRESHOLD
    );

    if (intersects.rectanglePoint(bottom, world)) {
      return 'bottom';
    }

    return false;
  };

  private pointerMoveHover = (world: Point): boolean => {
    // we do not move if there are multiple rectangles (for now)
    const rectangle = sheets.sheet.cursor.getSingleRectangleOrCursor();
    if (!rectangle) return false;
    const origin = sheets.sheet.cursor.position;
    const column = origin.x;
    const row = origin.y;

    const overlap = this.moveOverlaps(world);
    if (overlap) {
      this.state = 'hover';
      const screenRectangle = pixiApp.cursor.cursorRectangle;
      if (!screenRectangle) return false;

      // the offset is the clamped value of the rectangle based on where the user clicks
      const offset = sheets.sheet.getColumnRowFromScreen(world.x, world.y);
      offset.column = Math.min(Math.max(offset.column, rectangle.left), rectangle.right - 1);
      offset.row = Math.min(Math.max(offset.row, rectangle.top), rectangle.bottom - 1);
      this.movingCells = {
        column,
        row,
        width: rectangle.width,
        height: rectangle.height,
        toColumn: column,
        toRow: row,
        offset: {
          x: rectangle.left - offset.column,
          y: rectangle.top - offset.row,
        },
      };
      return true;
    }
    this.reset();
    return false;
  };

  pointerMove = (event: PointerEvent, world: Point): boolean => {
    if (isMobile || pixiAppSettings.panMode !== PanMode.Disabled || event.button === 1) return false;

    if (this.state === 'move') {
      this.pointerMoveMoving(world);
      return true;
    } else if (event.buttons === 0) {
      return this.pointerMoveHover(world);
    }
    return false;
  };

  pointerUp = (): boolean => {
    if (this.state === 'move') {
      if (this.startCell === undefined) {
        throw new Error('[PointerCellMoving] Expected startCell to be defined in pointerUp');
      }
      if (
        this.movingCells &&
        (this.startCell.x !== this.movingCells.toColumn || this.startCell.y !== this.movingCells.toRow)
      ) {
        const rectangle = sheets.sheet.cursor.getLargestRectangle();
        quadraticCore.moveCells(
          rectToSheetRect(rectangle, sheets.current),
          this.movingCells.toColumn,
          this.movingCells.toRow,
          sheets.current
        );

        const { showCodeEditor, codeCell } = pixiAppSettings.codeEditorState;
        if (
          showCodeEditor &&
          codeCell.sheetId === sheets.current &&
          intersects.rectanglePoint(rectangle, new Point(codeCell.pos.x, codeCell.pos.y))
        ) {
          pixiAppSettings.setCodeEditorState?.({
            ...pixiAppSettings.codeEditorState,
            codeCell: {
              ...codeCell,
              pos: {
                x: codeCell.pos.x + this.movingCells.toColumn - this.movingCells.column,
                y: codeCell.pos.y + this.movingCells.toRow - this.movingCells.row,
              },
            },
          });
        }
      }
      this.reset();
      return true;
    }
    return false;
  };

  handleEscape = (): boolean => {
    if (this.state === 'move') {
      this.reset();
      return true;
    }
    return false;
  };
}
