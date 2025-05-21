import React, { memo } from 'react';

export const FixedBottomAlert = memo(({ children }: { children: React.ReactNode }) => {
  return (
    <div className="fixed bottom-16 left-1/2 z-20 flex w-[95%] max-w-xl -translate-x-1/2 flex-row items-center justify-between gap-4 rounded border border-border bg-background px-4 py-3 shadow-lg">
      {children}
    </div>
  );
});
