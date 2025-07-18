import { ToolCardQuery } from '@/app/ai/toolCards/ToolCardQuery';
import {
  aiAnalystCurrentChatAtom,
  aiAnalystCurrentChatMessagesAtom,
  aiAnalystCurrentChatMessagesCountAtom,
  aiAnalystLoadingAtom,
  aiAnalystPDFImportLoadingAtom,
  aiAnalystPromptSuggestionsAtom,
  aiAnalystPromptSuggestionsCountAtom,
  aiAnalystWaitingOnMessageIndexAtom,
  aiAnalystWebSearchLoadingAtom,
} from '@/app/atoms/aiAnalystAtom';
import { debug, debugShowAIInternalContext } from '@/app/debugFlags';
import { AILoading } from '@/app/ui/components/AILoading';
import { Markdown } from '@/app/ui/components/Markdown';
import { AIAnalystExamplePrompts } from '@/app/ui/menus/AIAnalyst/AIAnalystExamplePrompts';
import { AIAnalystToolCard } from '@/app/ui/menus/AIAnalyst/AIAnalystToolCard';
import { AIAnalystUserMessageForm } from '@/app/ui/menus/AIAnalyst/AIAnalystUserMessageForm';
import { ThinkingBlock } from '@/app/ui/menus/AIAnalyst/AIThinkingBlock';
import { defaultAIAnalystContext } from '@/app/ui/menus/AIAnalyst/const/defaultAIAnalystContext';
import { useSubmitAIAnalystPrompt } from '@/app/ui/menus/AIAnalyst/hooks/useSubmitAIAnalystPrompt';
import { GoogleSearchSources } from '@/app/ui/menus/CodeEditor/AIAssistant/GoogleSearchSources';
import { apiClient } from '@/shared/api/apiClient';
import { ThumbDownIcon, ThumbUpIcon } from '@/shared/components/Icons';
import { Button } from '@/shared/shadcn/ui/button';
import { TooltipPopover } from '@/shared/shadcn/ui/tooltip';
import { cn } from '@/shared/shadcn/utils';
import mixpanel from 'mixpanel-browser';
import {
  getLastAIPromptMessageIndex,
  getUserPromptMessages,
  isContentGoogleSearchInternal,
  isInternalMessage,
  isToolResultMessage,
} from 'quadratic-shared/ai/helpers/message.helper';
import { memo, useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useRecoilCallback, useRecoilValue } from 'recoil';

type AIAnalystMessagesProps = {
  textareaRef: React.RefObject<HTMLTextAreaElement | null>;
};

export const AIAnalystMessages = memo(({ textareaRef }: AIAnalystMessagesProps) => {
  const messages = useRecoilValue(aiAnalystCurrentChatMessagesAtom);
  const messagesCount = useRecoilValue(aiAnalystCurrentChatMessagesCountAtom);
  const loading = useRecoilValue(aiAnalystLoadingAtom);
  const waitingOnMessageIndex = useRecoilValue(aiAnalystWaitingOnMessageIndexAtom);
  const promptSuggestionsCount = useRecoilValue(aiAnalystPromptSuggestionsCountAtom);

  const [div, setDiv] = useState<HTMLDivElement | null>(null);
  const ref = useCallback((div: HTMLDivElement | null) => {
    setDiv(div);
    div?.scrollTo({
      top: div.scrollHeight,
      behavior: 'smooth',
    });
  }, []);

  const shouldAutoScroll = useRef(true);
  const handleScroll = useCallback((e: Event) => {
    const div = e.target as HTMLDivElement;
    // Add a small buffer (5px) to account for rounding errors and tiny scroll differences
    const isScrolledToBottom = div.scrollHeight - div.scrollTop - div.clientHeight < 5;
    shouldAutoScroll.current = isScrolledToBottom;
  }, []);

  useEffect(() => {
    // Use both scroll and scrollend events for better cross-browser support
    div?.addEventListener('scroll', handleScroll);
    div?.addEventListener('scrollend', handleScroll);
    return () => {
      div?.removeEventListener('scroll', handleScroll);
      div?.removeEventListener('scrollend', handleScroll);
    };
  }, [div, handleScroll]);

  const scrollToBottom = useCallback(
    (force = false) => {
      if (force || shouldAutoScroll.current) {
        // Use requestAnimationFrame to ensure scrolling happens in the next frame
        // This helps prevent race conditions with React re-renders
        requestAnimationFrame(() => {
          if (div) {
            div.scrollTo({
              top: div.scrollHeight,
              // Use auto for rapid text updates to avoid falling behind
              // Smooth scrolling can't keep up with fast text generation
              behavior: force ? 'auto' : 'smooth',
            });
          }
        });
      }
    },
    [div]
  );

  useEffect(() => {
    if (loading) {
      // Don't force shouldAutoScroll to true - respect current value
      // Only force initial scroll if already at bottom
      if (shouldAutoScroll.current) {
        scrollToBottom(true);
      }

      // Only observe mutations while loading
      if (div) {
        const observer = new MutationObserver(() => {
          // Only scroll if user was already at bottom
          if (shouldAutoScroll.current) {
            scrollToBottom(true);
          }
        });

        observer.observe(div, {
          childList: true,
          subtree: true,
          characterData: true,
        });

        return () => observer.disconnect();
      }
    }
  }, [loading, scrollToBottom, div]);

  useEffect(() => {
    if (messagesCount === 0) {
      shouldAutoScroll.current = true;
    }
  }, [messagesCount]);

  // Only scroll on message changes if we're loading and user was already at bottom
  useEffect(() => {
    if (loading && shouldAutoScroll.current) {
      scrollToBottom();
    }
  }, [messages, scrollToBottom, loading, shouldAutoScroll]);

  // Scroll to bottom when prompt suggestions are available
  useEffect(() => {
    if (promptSuggestionsCount > 0) {
      scrollToBottom();
    }
  }, [promptSuggestionsCount, scrollToBottom]);

  if (messagesCount === 0) {
    return <AIAnalystExamplePrompts />;
  }

  return (
    <div
      ref={ref}
      className="flex select-text flex-col gap-3 overflow-y-auto px-2 pb-8 outline-none"
      spellCheck={false}
      onKeyDown={(e) => {
        if (((e.metaKey || e.ctrlKey) && e.key === 'a') || ((e.metaKey || e.ctrlKey) && e.key === 'c')) {
          // Allow a few commands, but nothing else
        } else {
          e.preventDefault();
        }
      }}
      // Disable Grammarly
      data-gramm="false"
      data-gramm_editor="false"
      data-enable-grammarly="false"
    >
      {messages.map((message, index) => {
        if (!debugShowAIInternalContext && !['userPrompt', 'webSearchInternal'].includes(message.contextType)) {
          return null;
        }

        const isCurrentMessage = index === messagesCount - 1;
        const modelKey = 'modelKey' in message ? message.modelKey : undefined;

        return (
          <div
            key={`${index}-${message.role}-${message.contextType}-${message.content}`}
            className={cn(
              'flex flex-col gap-3',
              message.role === 'assistant' ? 'px-2' : '',
              // For debugging internal context
              ['userPrompt', 'webSearchInternal'].includes(message.contextType) ? '' : 'rounded-lg bg-gray-500 p-2'
            )}
          >
            {debug && !!modelKey && <span className="text-xs text-muted-foreground">{modelKey}</span>}

            {isInternalMessage(message) ? (
              isContentGoogleSearchInternal(message.content) ? (
                <GoogleSearchSources content={message.content} />
              ) : null
            ) : message.role === 'user' && message.contextType === 'userPrompt' ? (
              <AIAnalystUserMessageForm
                initialContent={message.content}
                initialContext={message.context}
                textareaRef={textareaRef}
                messageIndex={index}
              />
            ) : isToolResultMessage(message) ? (
              message.content.map((result) => (
                <AIAnalystUserMessageForm
                  key={`${index}-${result.id}`}
                  initialContent={result.content}
                  textareaRef={textareaRef}
                  messageIndex={index}
                />
              ))
            ) : (
              <>
                {message.content.map((item, contentIndex) =>
                  item.type === 'anthropic_thinking' && !!item.text ? (
                    <ThinkingBlock
                      key={item.text}
                      isCurrentMessage={isCurrentMessage && contentIndex === message.content.length - 1}
                      isLoading={loading}
                      thinkingContent={item}
                      expandedDefault={true}
                    />
                  ) : item.type === 'text' && !!item.text ? (
                    <Markdown key={item.text}>{item.text}</Markdown>
                  ) : null
                )}

                {message.contextType === 'userPrompt' &&
                  message.toolCalls.map((toolCall, index) => (
                    <AIAnalystToolCard
                      key={`${index}-${toolCall.id}-${toolCall.arguments}`}
                      name={toolCall.name}
                      args={toolCall.arguments}
                      loading={toolCall.loading}
                    />
                  ))}
              </>
            )}
          </div>
        );
      })}

      {messagesCount > 1 && !loading && waitingOnMessageIndex === undefined && <FeedbackButtons />}

      {messagesCount > 1 && !loading && waitingOnMessageIndex === undefined && <PromptSuggestions />}

      <PDFImportLoading />

      <WebSearchLoading />

      <AILoading loading={loading} />
    </div>
  );
});

const FeedbackButtons = memo(() => {
  // true=positive, false=negative, null=neutral
  const [like, setLike] = useState<boolean | null>(null);

  const logFeedback = useRecoilCallback(
    ({ snapshot }) =>
      (newLike: boolean | null) => {
        // Log it to mixpanel
        mixpanel.track('[AIAnalyst].feedback', { like: newLike });

        // Otherwise, log it to our DB
        const messages = snapshot.getLoadable(aiAnalystCurrentChatMessagesAtom).getValue();
        const messageIndex = getLastAIPromptMessageIndex(messages);
        if (messageIndex < 0) return;

        const chatId = snapshot.getLoadable(aiAnalystCurrentChatAtom).getValue().id;
        apiClient.ai.feedback({
          chatId,
          messageIndex,
          like: newLike,
        });
      },
    [apiClient]
  );

  return (
    <div className="relative flex flex-row items-center px-2">
      <TooltipPopover label="Good response">
        <Button
          onClick={() => {
            setLike((prev) => {
              const newLike = prev === true ? null : true;
              logFeedback(newLike);
              return newLike;
            });
          }}
          variant="ghost"
          size="icon-sm"
          className={cn('select-none hover:text-success', like === true ? 'text-success' : 'text-muted-foreground')}
          disabled={like === false}
        >
          <ThumbUpIcon className="scale-75" />
        </Button>
      </TooltipPopover>

      <TooltipPopover label="Bad response">
        <Button
          onClick={() => {
            setLike((prev) => {
              const newLike = prev === false ? null : false;
              logFeedback(newLike);
              return newLike;
            });
          }}
          variant="ghost"
          size="icon-sm"
          className={cn(
            'select-none hover:text-destructive',
            like === false ? 'text-destructive' : 'text-muted-foreground'
          )}
          disabled={like === true}
        >
          <ThumbDownIcon className="scale-75" />
        </Button>
      </TooltipPopover>
    </div>
  );
});

const PromptSuggestions = memo(() => {
  const { submitPrompt } = useSubmitAIAnalystPrompt();
  const promptSuggestions = useRecoilValue(aiAnalystPromptSuggestionsAtom);
  const messages = useRecoilValue(aiAnalystCurrentChatMessagesAtom);
  const lastContext = useMemo(
    () =>
      getUserPromptMessages(messages)
        .filter((message) => message.contextType === 'userPrompt')
        .at(-1)?.context,
    [messages]
  );

  if (!messages.length || !promptSuggestions.suggestions.length) {
    return null;
  }

  return (
    <div className="flex flex-col gap-2 px-2">
      {promptSuggestions.suggestions.map((suggestion, index) => (
        <div
          key={`${index}-${suggestion.label}`}
          className="flex h-8 cursor-pointer items-center justify-between rounded-md bg-accent p-2 text-sm hover:bg-accent/80"
          onClick={() =>
            submitPrompt({
              content: [
                {
                  type: 'text',
                  text: suggestion.prompt,
                },
              ],
              context: {
                ...(lastContext ?? defaultAIAnalystContext),
              },
              messageIndex: messages.length,
            })
          }
        >
          <span className="truncate">{suggestion.label}</span>
        </div>
      ))}
    </div>
  );
});

const PDFImportLoading = memo(() => {
  const pdfImportLoading = useRecoilValue(aiAnalystPDFImportLoadingAtom);

  if (!pdfImportLoading) {
    return null;
  }

  return (
    <ToolCardQuery
      className="px-2"
      label="Reading file. Large files may take a few minutes."
      isLoading={pdfImportLoading}
    />
  );
});

const WebSearchLoading = memo(() => {
  const webSearchLoading = useRecoilValue(aiAnalystWebSearchLoadingAtom);

  if (!webSearchLoading) {
    return null;
  }

  return <ToolCardQuery className="px-2" label="Searching the web." isLoading={webSearchLoading} />;
});
