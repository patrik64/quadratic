import type { Request, Response } from 'express';
import type { ApiTypes } from 'quadratic-shared/typesAndSchemas';
import { z } from 'zod';
import dbClient from '../../dbClient';
import { getFile } from '../../middleware/getFile';
import { userMiddleware } from '../../middleware/user';
import { validateAccessToken } from '../../middleware/validateAccessToken';
import { validateRequestSchema } from '../../middleware/validateRequestSchema';
import { getFileUrl } from '../../storage/storage';
import type { RequestWithUser } from '../../types/Request';
import { ApiError } from '../../utils/ApiError';

export default [
  validateRequestSchema(
    z.object({
      params: z.object({
        uuid: z.string().uuid(),
        checkpointId: z.string(),
      }),
    })
  ),
  validateAccessToken,
  userMiddleware,
  handler,
];

async function handler(
  req: Request,
  res: Response<ApiTypes['/v0/files/:uuid/checkpoints/:checkpointId.GET.response']>
) {
  const {
    user: { id: userId },
    params: { uuid, checkpointId },
  } = req as RequestWithUser;
  const {
    userMakingRequest: { filePermissions, teamPermissions },
  } = await getFile({ uuid, userId });

  if (!(filePermissions.includes('FILE_EDIT') && teamPermissions?.includes('TEAM_VIEW'))) {
    throw new ApiError(403, 'You do not have permission to access the version history of this file');
  }

  const checkpoint = await dbClient.fileCheckpoint.findFirstOrThrow({
    where: {
      id: Number(checkpointId),
    },
  });

  const dataUrl = await getFileUrl(checkpoint.s3Key);

  return res.status(200).json({
    dataUrl,
    id: checkpoint.id,
    sequenceNumber: checkpoint.sequenceNumber,
    timestamp: checkpoint.timestamp.toISOString(),
    version: checkpoint.version,
  });
}
