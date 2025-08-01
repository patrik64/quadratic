import { ConnectionsIcon } from '@/dashboard/components/CustomRadixIcons';
import { useConfirmDialog } from '@/shared/components/ConfirmProvider';
import { EmptyState } from '@/shared/components/EmptyState';
import { AddIcon, CloseIcon, ExploreSchemaIcon } from '@/shared/components/Icons';
import { LanguageIcon } from '@/shared/components/LanguageIcon';
import { Type } from '@/shared/components/Type';
import type {
  ConnectionsListConnection,
  NavigateToCreateView,
  NavigateToView,
} from '@/shared/components/connections/Connections';
import { connectionsByType } from '@/shared/components/connections/connectionsByType';
import { Button } from '@/shared/shadcn/ui/button';
import { Input } from '@/shared/shadcn/ui/input';
import { Skeleton } from '@/shared/shadcn/ui/skeleton';
import { TooltipPopover } from '@/shared/shadcn/ui/tooltip';
import { cn } from '@/shared/shadcn/utils';
import { timeAgo } from '@/shared/utils/timeAgo';
import { Cross2Icon } from '@radix-ui/react-icons';
import type { ConnectionType } from 'quadratic-shared/typesAndSchemasConnections';
import { useState } from 'react';
import { useLocation } from 'react-router';

type Props = {
  connections: ConnectionsListConnection[];
  connectionsAreLoading?: boolean;
  handleNavigateToCreateView: NavigateToCreateView;
  handleNavigateToDetailsView: NavigateToView;
  handleNavigateToEditView: NavigateToView;
  handleShowConnectionDemo: (showConnectionDemo: boolean) => void;
};

export const ConnectionsList = ({
  connections,
  connectionsAreLoading,
  handleNavigateToCreateView,
  handleNavigateToDetailsView,
  handleNavigateToEditView,
  handleShowConnectionDemo,
}: Props) => {
  const [filterQuery, setFilterQuery] = useState<string>('');

  return (
    <>
      <div className="flex flex-col gap-4">
        <div className="grid grid-cols-2 gap-4">
          {Object.entries(connectionsByType).map(([type, { Logo }], i) => (
            <Button
              key={type}
              variant="outline"
              className="group relative h-auto w-full"
              onClick={() => {
                handleNavigateToCreateView(type as ConnectionType);
              }}
            >
              <AddIcon className="absolute bottom-1 right-1 opacity-30 group-hover:opacity-100" />
              <Logo className="h-[40px] w-[160px]" />
            </Button>
          ))}
        </div>

        {connectionsAreLoading && (
          <div className="flex flex-col gap-2">
            <Skeleton className="h-[20px] w-full rounded" />
            <Skeleton className="h-[20px] w-full rounded" />
          </div>
        )}

        {!connectionsAreLoading && connections.length ? (
          <>
            <form
              className="grid gap-4"
              onSubmit={(e) => {
                e.preventDefault();
              }}
            >
              <div className="relative">
                <Input
                  placeholder="Filter by name"
                  value={filterQuery}
                  onChange={(e) => setFilterQuery(e.target.value)}
                  autoFocus
                />
                {filterQuery.length > 0 && (
                  <Button
                    type="button"
                    variant="link"
                    aria-label="Clear"
                    onClick={() => setFilterQuery('')}
                    className="group absolute right-0 top-0"
                  >
                    <Cross2Icon className="h-4 w-4 text-muted-foreground group-hover:text-foreground" />
                  </Button>
                )}
              </div>
            </form>
            <ListItems
              filterQuery={filterQuery}
              items={connections}
              handleNavigateToDetailsView={handleNavigateToDetailsView}
              handleNavigateToEditView={handleNavigateToEditView}
              handleShowConnectionDemo={handleShowConnectionDemo}
            />
          </>
        ) : (
          <EmptyState
            title="No connections"
            className={'my-8'}
            description={
              <>
                <p>Create a connection from the options above, then open a spreadsheet and pull in data from it.</p>
                <p className="mt-2">
                  Or,{' '}
                  <button
                    className="relative font-semibold text-primary"
                    onClick={() => {
                      handleShowConnectionDemo(true);
                    }}
                  >
                    add a demo connection
                  </button>
                  .
                </p>
              </>
            }
            Icon={ConnectionsIcon}
          />
        )}
      </div>
    </>
  );
};

function ListItems({
  filterQuery,
  handleNavigateToDetailsView,
  handleNavigateToEditView,
  handleShowConnectionDemo,
  items,
}: {
  filterQuery: string;
  handleNavigateToDetailsView: Props['handleNavigateToDetailsView'];
  handleNavigateToEditView: Props['handleNavigateToEditView'];
  handleShowConnectionDemo: Props['handleShowConnectionDemo'];
  items: ConnectionsListConnection[];
}) {
  const confirmFn = useConfirmDialog('deleteDemoConnection', undefined);

  const filteredItems = filterQuery
    ? items.filter(({ name, type }) => name.toLowerCase().includes(filterQuery.toLowerCase()))
    : items;
  const location = useLocation();
  const isApp = location.pathname.startsWith('/file/');

  return filteredItems.length > 0 ? (
    <div className="relative -mt-3">
      {filteredItems.map(({ uuid, name, type, createdDate, disabled, isDemo }, i) => {
        const isNavigable = !(disabled || isDemo);
        const showSecondaryAction = !isApp && !disabled;
        const showIconHideDemo = !disabled && isDemo;
        const showIconBrowseSchema = !isApp && !disabled && !isDemo;
        return (
          <div className="group" key={uuid}>
            <div
              className={cn(
                'relative flex w-full items-center gap-1',
                disabled && 'cursor-not-allowed opacity-50',
                isNavigable && 'group-hover:bg-accent',
                showSecondaryAction && 'pr-12'
              )}
            >
              <button
                onClick={() => {
                  handleNavigateToEditView({ connectionUuid: uuid, connectionType: type });
                }}
                disabled={!isNavigable}
                key={uuid}
                className={cn('flex w-full items-center gap-4 rounded px-1 py-2')}
              >
                <div className="flex h-6 w-6 items-center justify-center">
                  <LanguageIcon language={type} />
                </div>

                <div className="flex w-full min-w-0 flex-grow flex-col text-left">
                  <span className="truncate text-sm">{name}</span>

                  {isDemo ? (
                    <span className="text-xs text-muted-foreground">Maintained by the Quadratic team</span>
                  ) : (
                    <time dateTime={createdDate} className="text-xs text-muted-foreground">
                      Created {timeAgo(createdDate)}
                    </time>
                  )}
                </div>
              </button>

              {showIconHideDemo && (
                <TooltipPopover label="Remove connection">
                  <Button
                    variant="ghost"
                    size="icon"
                    className="absolute right-2 top-2 flex items-center gap-1 text-muted-foreground hover:bg-background"
                    onClick={async () => {
                      if (await confirmFn()) {
                        handleShowConnectionDemo(false);
                      }
                    }}
                  >
                    <CloseIcon />
                  </Button>
                </TooltipPopover>
              )}

              {showIconBrowseSchema && (
                <TooltipPopover label="Browse schema">
                  <Button
                    variant="ghost"
                    size="icon"
                    className="absolute right-2 top-2 rounded p-2 text-muted-foreground hover:bg-background"
                    onClick={() => handleNavigateToDetailsView({ connectionUuid: uuid, connectionType: type })}
                  >
                    <ExploreSchemaIcon />
                  </Button>
                </TooltipPopover>
              )}
            </div>
          </div>
        );
      })}
    </div>
  ) : (
    <Type className="py-2 text-center">No matches.</Type>
  );
}
