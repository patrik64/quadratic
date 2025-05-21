import { DROPDOWN_SIZE } from '@/app/gridGL/cells/cellsLabel/drawSpecial';
import { getLanguageSymbol } from '@/app/gridGL/cells/CellsMarkers';
import type { Table } from '@/app/gridGL/cells/tables/Table';
import type { TablePointerDownResult } from '@/app/gridGL/cells/tables/Tables';
import { intersects } from '@/app/gridGL/helpers/intersects';
import { getCSSVariableTint } from '@/app/helpers/convertColor';
import { OPEN_SANS_FIX } from '@/app/web-workers/renderWebWorker/worker/cellsLabel/CellLabel';
import { CELL_HEIGHT } from '@/shared/constants/gridConstants';
import { sharedEvents } from '@/shared/sharedEvents';
import type { Point } from 'pixi.js';
import { Assets, BitmapText, Container, Graphics, Rectangle, Sprite } from 'pixi.js';

export const TABLE_NAME_FONT_SIZE = 14;
export const TABLE_NAME_PADDING = [4, 2];

const DROPDOWN_PADDING = 10;
const SYMBOL_SCALE = 0.5;
const SYMBOL_PADDING = 5;

export class TableName extends Container {
  // height of the table name's row
  private h = 0;

  private table: Table;
  private background: Graphics;
  private symbol: Sprite | undefined;
  private text: BitmapText;
  private dropdown: Sprite;
  private backgroundWidth = 0;

  tableNameBounds = new Rectangle();

  constructor(table: Table) {
    super();
    this.table = table;
    this.background = this.addChild(new Graphics());
    this.text = this.addChild(new BitmapText('', { fontSize: TABLE_NAME_FONT_SIZE, fontName: 'OpenSans-Bold' }));
    this.symbol = this.addChild(new Sprite());
    const dropdownWhiteIconTexture = Assets.get('dropdown-white-icon');
    this.dropdown = this.addChild(new Sprite(dropdownWhiteIconTexture));
    this.dropdown.anchor.set(0.5, 0);
    this.dropdown.width = DROPDOWN_SIZE[0];
    this.dropdown.height = DROPDOWN_SIZE[1];

    sharedEvents.on('changeThemeAccentColor', this.drawBackground);
  }

  destroy() {
    sharedEvents.off('changeThemeAccentColor', this.drawBackground);
    super.destroy();
  }

  private drawBackground = () => {
    this.backgroundWidth = this.table.tableBounds.width;
    this.background.clear();
    const color = this.table.active ? 'primary' : 'muted-foreground';
    this.background.beginFill(getCSSVariableTint(color));
    this.background.drawShape(new Rectangle(0, 0, this.backgroundWidth, this.h));
    this.background.endFill();
  };

  private drawSymbol() {
    if (this.symbol) {
      this.removeChild(this.symbol);
      this.symbol = undefined;
    }
    if (this.table.codeCell.language !== 'Import') {
      this.symbol = getLanguageSymbol(this.table.codeCell.language, false);
      if (this.symbol) {
        this.addChild(this.symbol);
        this.symbol.width = CELL_HEIGHT * SYMBOL_SCALE;
        this.symbol.scale.y = this.symbol.scale.x;
        this.symbol.anchor.set(0, 0.5);
        this.symbol.y = this.h / 2;
        this.symbol.x = SYMBOL_PADDING;
        if (
          this.table.codeCell.language === 'Formula' ||
          this.table.codeCell.language === 'Python' ||
          this.table.codeCell.language === 'Javascript'
        ) {
          this.symbol.tint = 0xffffff;
        }
      }
    }
  }

  private drawText() {
    this.text.text = this.table.codeCell.name;
    this.text.anchor.set(0, 0.5);
    this.text.position.set(
      TABLE_NAME_PADDING[0] + (this.symbol ? SYMBOL_PADDING + this.symbol.width : 0),
      OPEN_SANS_FIX.y + this.h / 2
    );

    // truncate the name if it's too long
    let name = this.table.codeCell.name;
    while (
      name &&
      this.text.x + this.text.width + TABLE_NAME_PADDING[0] + (this.symbol ? SYMBOL_PADDING + this.symbol.width : 0) >
        this.table.tableBounds.width
    ) {
      name = name.slice(0, -1);
      this.text.text = name + '…';
    }
  }

  private drawDropdown() {
    this.dropdown.position.set(
      this.text.width +
        OPEN_SANS_FIX.x +
        DROPDOWN_PADDING +
        TABLE_NAME_PADDING[0] +
        (this.symbol ? SYMBOL_PADDING + this.symbol.width : 0),
      this.text.y - this.dropdown.height / 4 // the 4 is b/c the icon is saved with the top in the middle of the texture
    );
  }

  update() {
    this.h = this.table.sheet.offsets.getRowHeight(this.table.codeCell.y);
    this.drawSymbol();
    this.drawText();
    if (this.table.active) {
      this.dropdown.visible = true;
      this.drawDropdown();
    } else {
      this.dropdown.visible = false;
    }
    this.drawBackground();
    this.tableNameBounds = new Rectangle(
      this.table.tableBounds.x,
      this.table.tableBounds.y,
      this.backgroundWidth,
      this.h
    );
  }

  intersects(world: Point): TablePointerDownResult | undefined {
    if (this.visible && intersects.rectanglePoint(this.tableNameBounds, world)) {
      if (world.x <= this.tableNameBounds.x + this.text.x + this.text.width + DROPDOWN_PADDING) {
        return { table: this.table.codeCell, type: 'table-name' };
      }
      if (
        this.table.active &&
        world.x <= this.tableNameBounds.x + this.text.x + this.text.width + this.dropdown.width + DROPDOWN_PADDING
      ) {
        return { table: this.table.codeCell, type: 'dropdown' };
      }
      return { table: this.table.codeCell, type: 'table-name' };
    }
    return undefined;
  }

  hide() {
    this.visible = false;
  }

  show() {
    this.visible = true;
  }

  get hidden(): boolean {
    return !this.visible;
  }

  toHover(y: number) {
    this.tableNameBounds.y = y;
  }

  toGrid() {
    this.tableNameBounds.y = this.table.tableBounds.y;
  }
}
