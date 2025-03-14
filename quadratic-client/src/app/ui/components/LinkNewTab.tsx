import type { LinkProps } from '@mui/material';
import { Link } from '@mui/material';

export function LinkNewTab({ href, children, ...rest }: LinkProps) {
  return (
    <Link {...rest} href={href} target="_blank" rel="noopener">
      {children}
    </Link>
  );
}
