import { getActiveSurfaceScope } from '@/infrastructure/peer-device/deviceSurface';
import { useContextStore } from '@/shared/stores/contextStore';
import type { ContextItem, ConversationExcerptContext } from '@/shared/types/context';
import { formatConversationExcerpt, isConversationExcerpt } from '@/shared/utils/conversationExcerpt';
import { flowChatStore } from '../store/FlowChatStore';
import { sessionComposerStore } from '../store/sessionComposerStore';
import { pendingQueueManager, type QueuedMessagePayloadUpdate } from '../services/flow-chat-manager/PendingQueueModule';
import type { QueuedMessage } from '../types/flow-chat';
import { parseComposerPresentation } from '../utils/composerPresentation';
import { queuedConversationExcerpts } from './conversationExcerptInventory';

export type ExcerptSaveResult = 'saved' | 'unavailable' | 'queue-unavailable';
interface ExcerptDialogBase {
  excerpt: ConversationExcerptContext;
  isCurrent: () => boolean;
}
export type ExcerptDialogTarget = ExcerptDialogBase & (
  | { mode: 'view' }
  | { mode: 'edit'; save: (comment: string) => ExcerptSaveResult }
);

function matches(item: ContextItem, excerpt: ConversationExcerptContext): item is ConversationExcerptContext {
  return isConversationExcerpt(item) && item.id === excerpt.id
    && item.source.surfaceId === excerpt.source.surfaceId && item.source.sessionId === excerpt.source.sessionId;
}

function replaceUnique(text: string, before: string, after: string): string | null {
  const index = text.indexOf(before);
  if (index < 0 || text.indexOf(before, index + before.length) >= 0) return null;
  return text.slice(0, index) + after + text.slice(index + before.length);
}

/** Preserve the already-expanded prompt and every unrelated attachment. */
export function reviseQueuedExcerpt(message: QueuedMessage, excerpt: ConversationExcerptContext, comment: string): QueuedMessagePayloadUpdate | null {
  const previous = queuedConversationExcerpts([message]).find(item => matches(item, excerpt));
  if (!previous) return null;
  const revised = { ...previous, comment: comment.trim() };
  const before = formatConversationExcerpt(previous);
  const after = formatConversationExcerpt(revised);
  const content = replaceUnique(message.content, before, after);
  if (content === null) return null;
  const displayMessage = message.displayMessage?.includes(before)
    ? replaceUnique(message.displayMessage, before, after) : message.displayMessage;
  if (displayMessage === null) return null;
  const presentation = parseComposerPresentation(message.userMessageMetadata?.composerPresentation);
  return {
    content, displayMessage,
    composerDraft: message.composerDraft ? { ...message.composerDraft,
      contexts: message.composerDraft.contexts.map(item => matches(item, previous) ? revised : item),
    } : undefined,
    userMessageMetadata: presentation ? { ...message.userMessageMetadata, composerPresentation: {
      ...presentation, segments: presentation.segments.map(segment => segment.kind === 'context' && matches(segment.context, previous)
        ? { ...segment, context: revised } : segment),
    } } : message.userMessageMetadata,
  };
}

function pendingOwners(excerpt: ConversationExcerptContext) {
  const scope = getActiveSurfaceScope();
  const composer = sessionComposerStore.getState();
  const sessionIds = [...flowChatStore.getState().sessions.keys()];
  const drafts = sessionIds.flatMap(sessionId => {
    const draft = composer.getDraft(sessionId, scope.surfaceId);
    const item = draft.contexts.find(item => matches(item, excerpt));
    return item ? [{ sessionId, draft, excerpt: item as ConversationExcerptContext }] : [];
  });
  const queues = sessionIds.flatMap(sessionId => pendingQueueManager.listForSurface(scope.surfaceId, sessionId)
    .flatMap(message => {
      const item = queuedConversationExcerpts([message]).find(item => matches(item, excerpt));
      return item ? [{ sessionId, message, excerpt: item }] : [];
    }));
  const visible = useContextStore.getState().contexts.find(item => matches(item, excerpt)) as ConversationExcerptContext | undefined;
  return { drafts, queues, visible };
}

/** Sent-message entries always view their snapshot; only pending source marks can edit. */
export function conversationExcerptDialogTarget(
  excerpt: ConversationExcerptContext,
  origin: 'source' | 'sent',
): ExcerptDialogTarget {
  const scope = getActiveSurfaceScope();
  const isCurrent = () => scope.isCurrent() && scope.surfaceId === excerpt.source.surfaceId;
  if (origin === 'sent' || !isCurrent()) return { mode: 'view', excerpt, isCurrent };
  const owners = pendingOwners(excerpt);
  const current = owners.visible ?? owners.drafts[0]?.excerpt ?? owners.queues[0]?.excerpt ?? excerpt;
  const wasPending = Boolean(owners.visible || owners.drafts.length || owners.queues.length);
  if (!wasPending || owners.queues.some(owner => owner.message.status === 'sending')) {
    return { mode: 'view', excerpt: current, isCurrent };
  }
  return {
    mode: 'edit',
    excerpt: current,
    isCurrent,
    save: comment => {
      if (!scope.isCurrent() || scope.surfaceId !== excerpt.source.surfaceId) return 'unavailable';
      const latest = pendingOwners(excerpt);
      const trimmed = comment.trim();
      // Re-read ownership on Save: a queue can start sending while the editor is open.
      if (latest.queues.length) {
        if (latest.queues.some(owner => !['queued', 'failed'].includes(owner.message.status)
          || !reviseQueuedExcerpt(owner.message, excerpt, trimmed))) return 'queue-unavailable';
        for (const owner of latest.queues) {
          if (!pendingQueueManager.updatePayloadForSurface(scope.surfaceId, owner.sessionId, owner.message.id,
            message => reviseQueuedExcerpt(message, excerpt, trimmed))) return 'queue-unavailable';
        }
      }
      for (const owner of latest.drafts) {
        sessionComposerStore.getState().setContexts(owner.sessionId,
          owner.draft.contexts.map(item => matches(item, excerpt) ? { ...item, comment: trimmed } : item), scope.surfaceId);
      }
      if (latest.visible) useContextStore.getState().updateContext(excerpt.id, { comment: trimmed });
      if (latest.visible || latest.drafts.length || latest.queues.length) return 'saved';
      return 'unavailable';
    },
  };
}
