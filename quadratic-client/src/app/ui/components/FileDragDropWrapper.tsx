import { hasPermissionToEditFile } from '@/app/actions';
import { userMessageAtom } from '@/app/atoms/userMessageAtom';
import { sheets } from '@/app/grid/controller/Sheets';
import { pixiApp } from '@/app/gridGL/pixiApp/PixiApp';
import { pixiAppSettings } from '@/app/gridGL/pixiApp/PixiAppSettings';
import { Coordinate } from '@/app/gridGL/types/size';
import { isExcelMimeType } from '@/app/helpers/files';
import { useFileImport } from '@/app/ui/hooks/useFileImport';
import { DragEvent, PropsWithChildren, useCallback, useRef, useState } from 'react';
import { useSetRecoilState } from 'recoil';

export const FileDragDropWrapper = (props: PropsWithChildren) => {
  // drag state
  const [dragActive, setDragActive] = useState(false);
  const divRef = useRef<HTMLDivElement>(null);

  const setUserMessageState = useSetRecoilState(userMessageAtom);
  const handleFileImport = useFileImport();

  const getColumnRowFromScreen = useCallback((e: DragEvent<HTMLDivElement>) => {
    const clientBoundingRect = divRef?.current?.getBoundingClientRect();
    const world = pixiApp.viewport.toWorld(
      e.pageX - (clientBoundingRect?.left || 0),
      e.pageY - (clientBoundingRect?.top || 0)
    );
    return sheets.sheet.getColumnRowFromScreen(world.x, world.y);
  }, []);

  const moveCursor = useCallback(
    (e: DragEvent<HTMLDivElement>) => {
      const { column, row } = getColumnRowFromScreen(e);
      const cursor = sheets.sheet.cursor;
      const hasMoved =
        cursor.cursorPosition.x !== column ||
        cursor.cursorPosition.y !== row ||
        cursor.keyboardMovePosition.x !== column ||
        cursor.keyboardMovePosition.y !== row;
      if (hasMoved) {
        cursor.changePosition({
          cursorPosition: { x: column, y: row },
          keyboardMovePosition: { x: column, y: row },
        });
      }
    },
    [getColumnRowFromScreen]
  );

  // handle drag events
  const handleDrag = useCallback(
    (e: DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      e.stopPropagation();

      if (!hasPermissionToEditFile(pixiAppSettings.permissions)) return;

      if (e.type === 'dragenter' && e.dataTransfer.types.includes('Files')) {
        setDragActive(true);
      } else if (e.type === 'dragover') {
        const mimeType = e.dataTransfer.items[0].type;
        if (isExcelMimeType(mimeType)) {
          setUserMessageState({ message: 'Dropped Excel file(s) will be imported as new sheet(s) in this file.' });
        } else {
          setUserMessageState({ message: undefined });
          moveCursor(e);
        }
      } else if (e.type === 'dragleave') {
        setDragActive(false);
        setUserMessageState({ message: undefined });
      }
    },
    [moveCursor, setUserMessageState]
  );

  // triggers when file is dropped
  const handleDrop = useCallback(
    async (e: DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      e.stopPropagation();

      if (!hasPermissionToEditFile(pixiAppSettings.permissions)) return;

      setDragActive(false);
      setUserMessageState({ message: undefined });

      const files = e.dataTransfer.files;
      if (files && files[0]) {
        const sheetId = sheets.sheet.id;
        const cursor = sheets.getCursorPosition();
        const { column, row } = getColumnRowFromScreen(e);
        const insertAt = { x: column, y: row } as Coordinate;
        handleFileImport({ files, insertAt, sheetId, cursor });
      }
    },
    [getColumnRowFromScreen, handleFileImport, setUserMessageState]
  );

  return (
    <div
      ref={divRef}
      onDragEnter={handleDrag}
      style={{
        display: 'flex',
        flexDirection: 'column',
        width: '100%',
        height: '100%',
        position: 'relative',
        minWidth: 0,
      }}
    >
      {props.children}
      {dragActive && (
        <div
          onDragLeave={handleDrag}
          onDragOver={handleDrag}
          onDrop={handleDrop}
          style={{
            position: 'absolute',
            width: '100%',
            height: '100%',
            top: '0px',
            right: '0px',
            bottom: '0px',
            left: '0px',
            opacity: '0',
          }}
        ></div>
      )}
    </div>
  );
};