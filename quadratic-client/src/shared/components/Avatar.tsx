import { getAuth0AvatarSrc } from '@/app/helpers/links';
import { cn } from '@/shared/shadcn/utils';
import type { ImgHTMLAttributes } from 'react';
import React, { forwardRef } from 'react';

interface AvatarProps extends ImgHTMLAttributes<HTMLImageElement> {
  size?: 'xs' | 'small' | 'medium' | 'large';
  children?: string | React.ReactNode;
}

export const Avatar = forwardRef<HTMLImageElement, AvatarProps>(
  ({ src, alt, size, style, className, children, ...rest }, ref) => {
    const [error, setError] = React.useState(false);

    const stylePreset = {
      width:
        size === 'xs'
          ? '20px'
          : size === 'small'
            ? '24px'
            : size === 'medium'
              ? '32px'
              : size === 'large'
                ? '40px'
                : '24px',
      height:
        size === 'xs'
          ? '20px'
          : size === 'small'
            ? '24px'
            : size === 'medium'
              ? '32px'
              : size === 'large'
                ? '40px'
                : '24px',
      fontSize:
        size === 'xs'
          ? '0.625rem'
          : size === 'small'
            ? '0.75rem'
            : size === 'medium'
              ? '1rem'
              : size === 'large'
                ? '1.125rem'
                : '0.8125rem',
      borderRadius: '50%',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
    };

    return (
      <>
        {error ? (
          <span
            ref={ref}
            className={cn(className, 'bg-muted-foreground text-background')}
            style={{ ...stylePreset, ...style }}
            {...rest}
          >
            {typeof children === 'string' ? getLettersFromString(children) : children}
          </span>
        ) : (
          <img
            alt={alt}
            ref={ref}
            src={getAuth0AvatarSrc(src) ?? ''}
            crossOrigin="anonymous"
            onError={() => setError(true)}
            style={{ ...stylePreset, ...style }}
            className={className}
            {...rest}
          />
        )}
      </>
    );
  }
);

function getLettersFromString(str: string) {
  let [first, last] = str.split(' ');

  if (first && last) {
    return first[0].toUpperCase() + last[0].toUpperCase();
  } else if (first) {
    return first[0].toUpperCase();
  } else {
    return '?';
  }
}
