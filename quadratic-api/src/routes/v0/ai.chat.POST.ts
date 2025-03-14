import type { Response } from 'express';
import { getLastUserPromptMessageIndex } from 'quadratic-shared/ai/helpers/message.helper';
import {
  getModelFromModelKey,
  isAnthropicModel,
  isBedrockAnthropicModel,
  isBedrockModel,
  isOpenAIModel,
  isXAIModel,
} from 'quadratic-shared/ai/helpers/model.helper';
import type { ApiTypes } from 'quadratic-shared/typesAndSchemas';
import { ApiSchemas } from 'quadratic-shared/typesAndSchemas';
import { type AIMessagePrompt } from 'quadratic-shared/typesAndSchemasAI';
import { z } from 'zod';
import { handleAnthropicRequest } from '../../ai/handler/anthropic';
import { handleBedrockRequest } from '../../ai/handler/bedrock';
import { handleOpenAIRequest } from '../../ai/handler/openai';
import { getQuadraticContext, getToolUseContext } from '../../ai/helpers/context.helper';
import { ai_rate_limiter } from '../../ai/middleware/aiRateLimiter';
import { anthropic, bedrock, bedrock_anthropic, openai, xai } from '../../ai/providers';
import dbClient from '../../dbClient';
import { STORAGE_TYPE } from '../../env-vars';
import { getFile } from '../../middleware/getFile';
import { userMiddleware } from '../../middleware/user';
import { validateAccessToken } from '../../middleware/validateAccessToken';
import { parseRequest } from '../../middleware/validateRequestSchema';
import { getBucketName, S3Bucket } from '../../storage/s3';
import { uploadFile } from '../../storage/storage';
import type { RequestWithUser } from '../../types/Request';

export default [validateAccessToken, ai_rate_limiter, userMiddleware, handler];

const schema = z.object({
  body: ApiSchemas['/v0/ai/chat.POST.request'],
});

async function handler(req: RequestWithUser, res: Response<ApiTypes['/v0/ai/chat.POST.response']>) {
  const {
    user: { id: userId },
  } = req;

  const { body } = parseRequest(req, schema);
  const { chatId, fileUuid, modelKey, ...args } = body;
  const source = args.source;

  if (args.useToolsPrompt) {
    const toolUseContext = getToolUseContext(source);
    args.messages.unshift(...toolUseContext);
  }

  if (args.useQuadraticContext) {
    const quadraticContext = getQuadraticContext(args.language);
    args.messages.unshift(...quadraticContext);
  }

  let responseMessage: AIMessagePrompt | undefined;
  if (isBedrockModel(modelKey)) {
    responseMessage = await handleBedrockRequest(modelKey, args, res, bedrock);
  } else if (isBedrockAnthropicModel(modelKey)) {
    responseMessage = await handleAnthropicRequest(modelKey, args, res, bedrock_anthropic);
  } else if (isAnthropicModel(modelKey)) {
    responseMessage = await handleAnthropicRequest(modelKey, args, res, anthropic);
  } else if (isOpenAIModel(modelKey)) {
    responseMessage = await handleOpenAIRequest(modelKey, args, res, openai);
  } else if (isXAIModel(modelKey)) {
    responseMessage = await handleOpenAIRequest(modelKey, args, res, xai);
  } else {
    throw new Error(`Model not supported: ${modelKey}`);
  }

  if (responseMessage) {
    args.messages.push(responseMessage);
  }

  const {
    file: { id: fileId, ownerTeam },
  } = await getFile({ uuid: fileUuid, userId });

  if (!ownerTeam.settingAnalyticsAi || STORAGE_TYPE !== 's3' || !getBucketName(S3Bucket.ANALYTICS)) {
    return;
  }

  const jwt = req.header('Authorization');
  if (!jwt) {
    return;
  }

  try {
    // key: <fileUuid>-<source>_<chatUuid>_<messageIndex>.json
    const messageIndex = getLastUserPromptMessageIndex(args.messages);
    const key = `${fileUuid}-${source}_${chatId.replace(/-/g, '_')}_${messageIndex}.json`;

    const contents = Buffer.from(JSON.stringify(args)).toString('base64');
    const response = await uploadFile(key, contents, jwt, S3Bucket.ANALYTICS);

    const model = getModelFromModelKey(modelKey);

    await dbClient.analyticsAIChat.upsert({
      where: {
        chatId,
      },
      create: {
        userId,
        fileId,
        chatId,
        source,
        messages: {
          create: {
            model,
            messageIndex,
            s3Key: response.key,
          },
        },
      },
      update: {
        messages: {
          create: {
            model,
            messageIndex,
            s3Key: response.key,
          },
        },
        updatedDate: new Date(),
      },
    });
  } catch (e) {
    console.error(e);
  }
}
