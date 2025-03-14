import { Action } from '@/app/actions/actions';
import { defaultActionSpec } from '@/app/actions/defaultActionsSpec';
import { editorInteractionStateShowSearchAtom } from '@/app/atoms/editorInteractionStateAtom';
import { events } from '@/app/events/events';
import { sheets } from '@/app/grid/controller/Sheets';
import { focusGrid } from '@/app/helpers/focusGrid';
import { matchShortcut } from '@/app/helpers/keyboardShortcuts';
import type { SearchOptions, SheetPos } from '@/app/quadratic-core-types';
import { quadraticCore } from '@/app/web-workers/quadraticCore/quadraticCore';
import { Button } from '@/shared/shadcn/ui/button';
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from '@/shared/shadcn/ui/dropdown-menu';
import { Input } from '@/shared/shadcn/ui/input';
import { Popover, PopoverAnchor, PopoverContent } from '@/shared/shadcn/ui/popover';
import { ChevronLeftIcon, ChevronRightIcon, Cross2Icon, DotsHorizontalIcon } from '@radix-ui/react-icons';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useRecoilState } from 'recoil';

const findInSheetActionSpec = defaultActionSpec[Action.FindInCurrentSheet];
const findInSheetsActionSpec = defaultActionSpec[Action.FindInAllSheets];

export function Search() {
  const [showSearch, setShowSearch] = useRecoilState(editorInteractionStateShowSearchAtom);

  const [searchOptions, setSearchOptions] = useState<SearchOptions>({
    case_sensitive: false,
    whole_cell: false,
    search_code: false,
    sheet_id: sheets.current,
  });
  const [results, setResults] = useState<SheetPos[]>([]);
  const [current, setCurrent] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const placeholder = !searchOptions.sheet_id ? findInSheetsActionSpec.label : findInSheetActionSpec.label;

  const moveCursor = useCallback((pos: SheetPos) => {
    if (sheets.current !== pos.sheet_id.id) {
      sheets.current = pos.sheet_id.id;
    }
    sheets.sheet.cursor.moveTo(Number(pos.x), Number(pos.y));
    inputRef.current?.focus();
  }, []);

  const onChange = useCallback(
    async (search: string, updatedSearchOptions = searchOptions) => {
      if (search.length > 0) {
        const found = await quadraticCore.search(search, updatedSearchOptions);
        if (found.length) {
          setResults(found);
          setCurrent(0);
          moveCursor(found[0]);
          events.emit(
            'search',
            found.map((found) => ({ x: Number(found.x), y: Number(found.y), sheetId: found.sheet_id.id }), 0)
          );
          return;
        }
      }
      setResults([]);
      events.emit('search');
    },
    [moveCursor, searchOptions]
  );

  const navigate = useCallback(
    (delta: 1 | -1) => {
      setCurrent((current) => {
        let next = (current + delta) % results.length;
        if (next < 0) next = results.length - 1;
        events.emit(
          'search',
          results.map((found) => ({ x: Number(found.x), y: Number(found.y), sheetId: found.sheet_id.id }), next)
        );
        const result = results[next];
        moveCursor(result);
        return next;
      });
    },
    [moveCursor, results]
  );

  const changeOptions = useCallback(
    (option: 'case_sensitive' | 'whole_cell' | 'search_code' | 'sheet') => {
      let updatedSearchOptions: SearchOptions;
      if (option === 'sheet') {
        if (searchOptions.sheet_id) {
          setSearchOptions((prev) => {
            updatedSearchOptions = { ...prev, sheet_id: undefined };
            return updatedSearchOptions;
          });
        } else {
          setSearchOptions((prev) => {
            updatedSearchOptions = { ...prev, sheet_id: sheets.current };
            return updatedSearchOptions;
          });
        }
      } else {
        setSearchOptions((prev) => {
          updatedSearchOptions = { ...prev, [option]: !prev[option] };
          return updatedSearchOptions;
        });
      }

      const search = (inputRef.current as HTMLInputElement).value;
      onChange(search, updatedSearchOptions!);
    },
    [onChange, searchOptions.sheet_id]
  );

  const closeSearch = useCallback(() => {
    events.emit('search');
    focusGrid();
  }, []);

  useEffect(() => {
    const changeSheet = () => {
      if (!showSearch || !searchOptions.sheet_id) {
        return;
      }
      const newSearchOptions = { ...searchOptions };
      if (searchOptions.sheet_id) {
        newSearchOptions.sheet_id = sheets.current;
      }
      setSearchOptions(newSearchOptions);
      onChange((inputRef.current as HTMLInputElement).value, newSearchOptions);
    };
    events.on('changeSheet', changeSheet);
    return () => {
      events.off('changeSheet', changeSheet);
    };
  }, [showSearch, searchOptions, onChange]);

  useEffect(() => {
    if (!showSearch) {
      closeSearch();
    } else {
      setResults([]);
      setSearchOptions({
        case_sensitive: false,
        whole_cell: false,
        search_code: false,
        sheet_id: sheets.current,
      });

      // if it's not true then it's of type SearchOptions
      if (showSearch !== true) {
        setSearchOptions(showSearch);
      }
    }
  }, [closeSearch, showSearch]);

  return (
    <Popover open={!!showSearch}>
      <PopoverAnchor />
      <PopoverContent
        align="end"
        className="m-2 flex w-[100vw] flex-col items-center gap-1 p-2 min-[400px]:w-[400px] min-[400px]:flex-row min-[400px]:p-3"
        onKeyDown={(e) => {
          e.stopPropagation();

          // close search
          if (e.key === 'Escape') {
            setShowSearch(false);
          }
          if (matchShortcut(Action.FindInCurrentSheet, e)) {
            e.preventDefault();
            inputRef.current?.focus();
            inputRef.current?.select();
            // shift+cmd+f let's you change to all sheets search mode while in the dialog box
            if (e.shiftKey) {
              setSearchOptions((prev) => {
                if (!prev.sheet_id) return prev;
                const updatedSearchOptions = { ...prev, sheet_id: undefined };
                const search = (inputRef.current as HTMLInputElement).value;
                onChange(search, { ...searchOptions, sheet_id: undefined });
                return updatedSearchOptions;
              });
            }
          }
          if (e.key === 'Enter') {
            // If other elements have focus, like the 'close' button, don't handle Enter
            if (document.activeElement !== inputRef.current) return;

            e.preventDefault();
            if (results.length > 1) {
              navigate(e.shiftKey ? -1 : 1);
            } else if (results.length === 1) {
              setShowSearch(false);
            }
          }
        }}
      >
        <div className="relative w-full">
          <Input
            id="search-input"
            type="text"
            ref={inputRef}
            placeholder={placeholder}
            onChange={(e) => onChange(e.target.value)}
            className={`pr-[4rem]`}
            autoComplete="off"
            autoCapitalize="off"
            autoCorrect="off"
          />
          {inputRef.current && inputRef.current.value.length !== 0 && (
            <div className="absolute right-3 top-[.625rem] text-nowrap text-xs text-muted-foreground">
              {results.length === 0 ? '0' : current + 1} of {results.length}
            </div>
          )}
        </div>
        <div className="flex w-full justify-between min-[400px]:w-auto">
          <Button variant="ghost" className="px-2" onClick={() => navigate(-1)} disabled={results.length === 0}>
            <ChevronLeftIcon />
          </Button>
          <Button variant="ghost" className="px-2" onClick={() => navigate(1)} disabled={results.length === 0}>
            <ChevronRightIcon />
          </Button>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" className="px-2">
                <DotsHorizontalIcon />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent
              onCloseAutoFocus={(e) => {
                e.preventDefault();
                inputRef.current?.focus();
              }}
            >
              <DropdownMenuCheckboxItem
                checked={!searchOptions.sheet_id}
                onCheckedChange={() => changeOptions('sheet')}
              >
                Search all sheets
              </DropdownMenuCheckboxItem>
              <DropdownMenuCheckboxItem
                checked={searchOptions.case_sensitive}
                onCheckedChange={() => changeOptions('case_sensitive')}
              >
                Case sensitive search
              </DropdownMenuCheckboxItem>
              <DropdownMenuCheckboxItem
                checked={searchOptions.whole_cell}
                onCheckedChange={() => changeOptions('whole_cell')}
              >
                Match entire cell contents
              </DropdownMenuCheckboxItem>
              <DropdownMenuCheckboxItem
                checked={searchOptions.search_code}
                onCheckedChange={() => changeOptions('search_code')}
              >
                Search within code
              </DropdownMenuCheckboxItem>
            </DropdownMenuContent>
          </DropdownMenu>
          <Button variant="ghost" className="px-2" onClick={() => setShowSearch(false)}>
            <Cross2Icon />
          </Button>
        </div>
      </PopoverContent>
    </Popover>
  );
}
