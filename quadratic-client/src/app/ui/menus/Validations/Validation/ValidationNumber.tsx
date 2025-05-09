import type { NumberRange } from '@/app/quadratic-core-types';
import type { ValidationData } from '@/app/ui/menus/Validations/Validation/useValidationData';
import type { ValidationUndefined } from '@/app/ui/menus/Validations/Validation/validationType';
import { ValidationInput } from '@/app/ui/menus/Validations/Validation/ValidationUI/ValidationInput';
import {
  ValidationMoreOptions,
  ValidationUICheckbox,
} from '@/app/ui/menus/Validations/Validation/ValidationUI/ValidationUI';
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '@/shared/shadcn/ui/accordion';
import { Button } from '@/shared/shadcn/ui/button';
import { cn } from '@/shared/shadcn/utils';
import DeleteIcon from '@mui/icons-material/Delete';
import { Tooltip } from '@mui/material';
import { InfoCircledIcon } from '@radix-ui/react-icons';
import { useCallback, useMemo, useState } from 'react';

interface Props {
  validationData: ValidationData;
  onEnter: () => void;
}

export const ValidationNumber = (props: Props) => {
  const { onEnter, validationData } = props;
  const { ignoreBlank, changeIgnoreBlank, readOnly, validation, setValidation } = validationData;

  const [equalsError, setEqualsError] = useState(false);

  //#region Equals

  const equals = useMemo(() => {
    if (validation && 'rule' in validation && validation.rule && validation.rule !== 'None') {
      if ('Number' in validation.rule) {
        const equals = validation.rule.Number.ranges.find((r) => 'Equal' in r);
        if (equals && 'Equal' in equals) {
          return equals.Equal;
        }
      }
    }
  }, [validation]);

  const changeEquals = useCallback(
    (values: string) => {
      const numbers = values.split(',').flatMap((v) => {
        if (!v.trim()) {
          return [];
        } else {
          return [parseFloat(v)];
        }
      });
      if (numbers.some((n) => isNaN(n))) {
        setEqualsError(true);
        return;
      }
      setEqualsError(false);

      setValidation((validation) => {
        if (!validation || !('rule' in validation) || validation.rule === 'None' || !('Number' in validation.rule)) {
          return;
        }

        if (!numbers.length) {
          return {
            ...validation,
            rule: {
              Number: {
                ...validation.rule.Number,
                ranges: [],
              },
            },
          };
        }

        const rules = validation.rule.Number.ranges.filter((m) => !('Equal' in m));
        if (values.length) {
          rules.push({
            Equal: numbers,
          });
        }
        return {
          ...validation,
          rule: {
            Number: {
              ...validation.rule.Number,
              ranges: [
                {
                  Equal: numbers,
                },
              ],
            },
          },
        };
      });
    },
    [setValidation]
  );

  //#endregion

  //#region Not Equals

  const notEquals = useMemo(() => {
    if (validation && 'rule' in validation && validation.rule && validation.rule !== 'None') {
      if ('Number' in validation.rule) {
        const notEquals = validation.rule.Number.ranges.find((r) => 'NotEqual' in r);
        if (notEquals && 'NotEqual' in notEquals) {
          return notEquals.NotEqual;
        }
      }
    }
  }, [validation]);

  const changeNotEquals = useCallback(
    (values: string) => {
      const numbers = values.split(',').flatMap((v) => {
        if (!v.trim()) {
          return [];
        } else {
          return [parseFloat(v)];
        }
      });
      if (numbers.some((n) => isNaN(n))) {
        setEqualsError(true);
        return;
      }
      setEqualsError(false);

      setValidation((validation) => {
        if (!validation || !('rule' in validation) || validation.rule === 'None' || !('Number' in validation.rule)) {
          return;
        }

        if (!numbers.length) {
          return {
            ...validation,
            rule: {
              Number: {
                ...validation.rule.Number,
                ranges: [],
              },
            },
          };
        }

        const rules = validation.rule.Number.ranges.filter((m) => !('NotEqual' in m));
        if (values.length) {
          rules.push({
            NotEqual: numbers,
          });
        }
        return {
          ...validation,
          rule: {
            Number: {
              ...validation.rule.Number,
              ranges: [
                {
                  NotEqual: numbers,
                },
              ],
            },
          },
        };
      });
    },
    [setValidation]
  );

  //#endregion

  //#region Ranges

  const ranges: NumberRange[] = useMemo(() => {
    const ranges: NumberRange[] = [];
    if (validation && 'rule' in validation && validation.rule && validation.rule !== 'None') {
      if ('Number' in validation.rule) {
        validation.rule.Number.ranges.forEach((r) => {
          ranges.push(r);
        });
      }
    }

    // always add an empty range to the bottom of the list
    const last = ranges[ranges.length - 1];
    if (!last || ('Range' in last && (last.Range[0] !== null || last.Range[1] !== null))) {
      ranges.push({ Range: [null, null] });
    }
    return ranges;
  }, [validation]);

  const [rangeError, setRangeError] = useState<number[]>([]);
  const changeRange = useCallback(
    (index: number, value: string, type: 'min' | 'max') => {
      value = value.trim();
      setValidation((validation): ValidationUndefined => {
        if (!validation || !('rule' in validation) || validation.rule === 'None' || !('Number' in validation.rule)) {
          return;
        }
        const newRanges = [...ranges];
        const current = newRanges[index];
        if (!('Range' in current)) throw new Error('Expected Range in changeRange');
        if (type === 'min') {
          // check for error (min > max)
          if (current.Range[1] !== null && parseFloat(value) > current.Range[1]) {
            setRangeError((rangeError) => {
              const r = rangeError.filter((r) => r !== index);
              r.push(index);
              return r;
            });
            return validation;
          }
          current.Range[0] = value ? parseFloat(value) : null;
          setRangeError((rangeError) => {
            return rangeError.filter((r) => r !== index);
          });
        } else {
          // check for error (max < min)
          if (current.Range[0] !== null && parseFloat(value) < current.Range[0]) {
            setRangeError((rangeError) => {
              const r = rangeError.filter((r) => r !== index);
              r.push(index);
              return r;
            });
            return validation;
          }
          current.Range[1] = value ? parseFloat(value) : null;
          setRangeError((rangeError) => {
            return rangeError.filter((r) => r !== index);
          });
        }

        const filteredRanges = newRanges.filter((r) => 'Range' in r && (r.Range[0] !== null || r.Range[1] !== null));
        return {
          ...validation,
          rule: {
            Number: {
              ...validation.rule.Number,
              ranges: filteredRanges,
            },
          },
        };
      });
    },
    [setValidation, ranges]
  );

  const removeRange = useCallback(
    (index: number) => {
      setValidation((validation) => {
        if (!validation || !('rule' in validation) || validation.rule === 'None' || !('Number' in validation.rule)) {
          return;
        }
        const newRanges = [...ranges];
        newRanges.splice(index, 1);
        return {
          ...validation,
          rule: {
            Number: {
              ...validation.rule.Number,
              ranges: newRanges,
            },
          },
        };
      });
    },
    [setValidation, ranges]
  );

  //#endregion

  const equalsOverrides = useMemo(() => {
    if (equals) {
      return (
        <Tooltip title="'Number equals' cannot be combined with other rules">
          <InfoCircledIcon />
        </Tooltip>
      );
    }
    return null;
  }, [equals]);

  return (
    <div className="flex flex-col gap-5">
      <ValidationUICheckbox
        label="Allow blank values"
        value={ignoreBlank}
        changeValue={changeIgnoreBlank}
        readOnly={readOnly}
      />
      <Accordion type="single" collapsible className="w-full" defaultValue={equals ? 'number-equals' : undefined}>
        <AccordionItem value="number-equals">
          <AccordionTrigger>Number equals</AccordionTrigger>
          <AccordionContent className="px-1 pt-1">
            <div className="flex w-full flex-col gap-1">
              <ValidationInput
                placeholder="Enter numbers separated by commas"
                disabled={readOnly}
                value={equals ? equals.join(', ') : ''}
                onInput={changeEquals}
                readOnly={readOnly}
                error={equalsError ? 'Please enter valid numbers separated by commas' : undefined}
                onEnter={onEnter}
              />
            </div>
          </AccordionContent>
        </AccordionItem>
      </Accordion>

      <Accordion
        type="single"
        collapsible
        className="w-full"
        defaultValue={notEquals ? 'number-not-equals' : undefined}
        value={equals ? '' : undefined}
      >
        <AccordionItem value="number-not-equals">
          <AccordionTrigger className={equals ? 'opacity-50' : ''} disabled={!!equals}>
            <div className="flex">Number does not equal{equalsOverrides}</div>
          </AccordionTrigger>
          <AccordionContent className="px-1 pt-1">
            <div className="flex w-full flex-col gap-1">
              <ValidationInput
                placeholder="Enter numbers separated by commas"
                disabled={readOnly}
                value={notEquals ? notEquals.join(', ') : ''}
                onInput={changeNotEquals}
                readOnly={readOnly}
                error={equalsError ? 'Please enter valid numbers separated by commas' : undefined}
                onEnter={onEnter}
              />
            </div>
          </AccordionContent>
        </AccordionItem>
      </Accordion>

      <Accordion
        type="single"
        collapsible
        className="w-full"
        defaultValue={ranges.length > 1 ? 'number-range' : undefined}
        value={equals ? '' : undefined}
      >
        <AccordionItem value="number-range">
          <AccordionTrigger className={equals ? 'opacity-50' : ''} disabled={!!equals}>
            <div className="flex">Number ranges{equalsOverrides}</div>
          </AccordionTrigger>
          <AccordionContent className="px-1 pt-1">
            {ranges.map((range, i) => {
              const r = 'Range' in range ? range.Range : [null, null];
              const min = r[0]?.toString() ?? '';
              const max = r[1]?.toString() ?? '';
              return (
                <div className="mb-2 flex w-full flex-col gap-1" key={i}>
                  <div className="flex items-center gap-1">
                    <ValidationInput
                      type="number"
                      placeholder="Minimum"
                      disabled={readOnly}
                      value={min}
                      onChange={(value) => changeRange(i, value, 'min')}
                      onEnter={onEnter}
                    />
                    <ValidationInput
                      type="number"
                      placeholder="Maximum"
                      disabled={readOnly}
                      value={max}
                      onChange={(value) => changeRange(i, value, 'max')}
                      onEnter={onEnter}
                    />
                    <Button
                      className={cn('grow-0 px-2', i !== ranges.length - 1 ? '' : 'invisible')}
                      onClick={() => removeRange(i)}
                    >
                      <DeleteIcon />
                    </Button>
                  </div>
                  {rangeError.includes(i) && (
                    <div className="mb-2 text-xs text-red-500">Range minimum must be less than maximum</div>
                  )}
                </div>
              );
            })}
          </AccordionContent>
        </AccordionItem>
      </Accordion>

      <ValidationMoreOptions validationData={validationData} />
    </div>
  );
};
