import { useFileImport } from '@/app/ui/hooks/useFileImport';
import { fileDragDropModalAtom } from '@/dashboard/atoms/fileDragDropModalAtom';
import { cn } from '@/shared/shadcn/utils';
import type { DragEvent } from 'react';
import { useCallback } from 'react';
import { useRecoilState } from 'recoil';

interface FileDragDropProps {
  className?: string;
}

export function FileDragDrop({ className }: FileDragDropProps) {
  const [fileDragDropModal, setFileDragDropModal] = useRecoilState(fileDragDropModalAtom);
  const handleFileImport = useFileImport();

  const handleDrag = useCallback(
    (e: DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      e.stopPropagation();

      if (e.type === 'dragleave') {
        setFileDragDropModal({ show: false, teamUuid: undefined, isPrivate: undefined });
      }
    },
    [setFileDragDropModal]
  );

  const handleDrop = useCallback(
    async (e: DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      e.stopPropagation();

      setFileDragDropModal({ show: false, teamUuid: undefined, isPrivate: undefined });

      const files = Array.from(e.dataTransfer.files);
      const { isPrivate, teamUuid } = fileDragDropModal;
      handleFileImport({ files, isPrivate, teamUuid });
    },
    [fileDragDropModal, handleFileImport, setFileDragDropModal]
  );

  if (!fileDragDropModal.show) return null;

  return (
    <div
      id="file-drag-drop"
      className={cn(
        'fixed left-0 top-0 z-20 flex h-full w-full flex-col items-center justify-center bg-background opacity-90',
        className
      )}
      onDragEnter={handleDrag}
      onDragLeave={handleDrag}
      onDragOver={handleDrag}
    >
      <div
        className="relative z-10 h-[90%] w-[90%] select-none rounded-lg border-4 border-dashed border-border bg-background opacity-90"
        onDrop={handleDrop}
        onDragLeave={handleDrag}
      >
        <div className="pointer-events-none flex h-full w-full flex-col items-center justify-center gap-2">
          <span className="text-2xl font-bold text-foreground">Drop file here</span>

          <span className="pl-4 pr-4 text-center text-base font-medium text-muted-foreground">
            Start a new spreadsheet by importing a CSV, Parquet, Excel or Grid file(s)
          </span>
        </div>
      </div>
    </div>
  );
}
