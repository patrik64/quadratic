import * as z from 'zod';

// Helper to turn empty string into undefined, so JSON.stringify() will remove empty values
const transformEmptyStringToUndefined = (val: string | undefined) => (val === '' ? undefined : val);

/**
 * =============================================================================
 * Shared schemas
 * =============================================================================
 */

export const ConnectionNameSchema = z.string().min(1, { message: 'Required' });
export const ConnectionTypeSchema = z.enum(['POSTGRES', 'MYSQL', 'MSSQL', 'SNOWFLAKE']);
const ConnectionHostSchema = z
  .string()
  .min(1, { message: 'Required' })
  .refine(
    (host) => {
      // If we're running locally, allow localhost
      if (window?.location?.hostname === 'localhost') return true;

      // Otherwise, disallow specific hosts
      host = host.trim();

      // Check for localhost variations
      if (host.includes('localhost')) return false;

      // Check for local IP ranges
      if (host.startsWith('127.')) return false; // Loopback addresses
      if (host.includes('0.0.0.0')) return false; // Default route
      if (host.startsWith('169.254.')) return false; // Link-local addresses

      return true;
    },
    {
      message:
        'Quadratic runs in the cloud and can’t connect to a local database. Please use a publicly-accessible host.',
    }
  );
const ConnectionTypeDetailsSchema = z.record(z.string(), z.any());
const ConnectionSchema = z.object({
  createdDate: z.string().datetime(),
  updatedDate: z.string().datetime(),
  name: ConnectionNameSchema,
  uuid: z.string().uuid(),

  type: ConnectionTypeSchema,
  typeDetails: ConnectionTypeDetailsSchema,
});

export type ConnectionTypeDetails = z.infer<typeof ConnectionTypeDetailsSchema>;
export type ConnectionType = z.infer<typeof ConnectionTypeSchema>;
export type Connection = z.infer<typeof ConnectionSchema>;

/**
 * =============================================================================
 * Schemas for individual connections
 * =============================================================================
 */
export const ConnectionTypeDetailsPostgresSchema = z.object({
  host: ConnectionHostSchema,
  port: z
    .string()
    .min(1, { message: 'Required' })
    .refine(
      (port) => {
        const portNumber = Number(port);
        if (isNaN(portNumber)) return false;
        return portNumber >= 0 && portNumber <= 65535;
      },
      {
        message: 'Port must be a valid number between 0 and 65535',
      }
    ),
  database: z.string().min(1, { message: 'Required' }),
  username: z.string().min(1, { message: 'Required' }),
  password: z.string().optional().transform(transformEmptyStringToUndefined),
});
export const ConnectionTypeDetailsMysqlSchema = ConnectionTypeDetailsPostgresSchema;
export const ConnectionTypeDetailsMssqlSchema = z.object({
  host: ConnectionHostSchema,
  port: z
    .string()
    .min(1, { message: 'Required' })
    .refine(
      (port) => {
        const portNumber = Number(port);
        if (isNaN(portNumber)) return false;
        return portNumber >= 0 && portNumber <= 65535;
      },
      {
        message: 'Port must be a valid number between 0 and 65535',
      }
    ),
  database: z.string().optional(),
  username: z.string().min(1, { message: 'Required' }),
  password: z.string().min(1, { message: 'Required' }),
});
export const ConnectionTypeDetailsSnowflakeSchema = z.object({
  account_identifier: z.string().min(1, { message: 'Required' }),
  database: z.string().min(1, { message: 'Required' }),
  username: z.string().min(1, { message: 'Required' }),
  password: z.string().min(1, { message: 'Required' }),
  warehouse: z.string().optional().transform(transformEmptyStringToUndefined),
  role: z.string().optional().transform(transformEmptyStringToUndefined),
});

/**
 * =============================================================================
 * Export
 * =============================================================================
 */

export const ConnectionListSchema = z.array(
  ConnectionSchema.pick({ uuid: true, name: true, createdDate: true, type: true })
);
export type ConnectionList = z.infer<typeof ConnectionListSchema>;

export const ApiSchemasConnections = {
  // List connections
  '/v0/teams/:uuid/connections.GET.response': ConnectionListSchema,

  // Create connection
  '/v0/teams/:uuid/connections.POST.request': ConnectionSchema.pick({
    name: true,
    type: true,
    typeDetails: true,
  }),
  '/v0/teams/:uuid/connections.POST.response': ConnectionSchema.pick({ uuid: true }),

  // Get connection
  '/v0/teams/:uuid/connections/:connectionUuid.GET.response': ConnectionSchema,

  // Update connection
  '/v0/teams/:uuid/connections/:connectionUuid.PUT.request': ConnectionSchema.pick({ name: true, typeDetails: true }),
  '/v0/teams/:uuid/connections/:connectionUuid.PUT.response': ConnectionSchema,

  // Delete connection
  '/v0/teams/:uuid/connections/:connectionUuid.DELETE.response': z.object({ message: z.string() }),
};
