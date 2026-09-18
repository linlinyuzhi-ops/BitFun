import { describe, expect, it } from 'vitest';
import { surfaceScopedKey } from '@/infrastructure/peer-device/deviceSurface';
import type { ConversationExcerptContext } from '@/shared/types/context';
import type { SessionComposerDraft } from '../store/sessionComposerStore';
import type { QueuedMessage, Session } from '../types/flow-chat';
import { withConversationExcerpts } from '../utils/composerPresentation';
import { createConversationExcerptInventory } from './conversationExcerptInventory';

const excerpt: ConversationExcerptContext = {
  id: 'annotation', type: 'conversation-excerpt', timestamp: 1, annotationNumber: 1,
  source: { surfaceId: 'local', sessionId: 'source', sessionName: 'Source' },
  fragments: [{ turnId: 'turn', flowItemId: 'text', text: 'quote', start: 0, end: 5, prefix: '', suffix: '' }],
  comment: 'Comment',
};
function session(sessionId: string, excerpts: ConversationExcerptContext[] = []): Session {
  return { sessionId, dialogTurns: excerpts.length ? [{ id: 'sent', userMessage: {
    id: 'sent-message', content: 'Question', timestamp: 2,
    metadata: { composerPresentation: withConversationExcerpts(null, excerpts, 'Question') },
  }, modelRounds: [] }] : [] } as unknown as Session;
}
function draft(contexts: ConversationExcerptContext[]): SessionComposerDraft {
  return { contexts, value: '', pendingLargePastes: {}, updatedAt: 1 };
}
const sourceKey = surfaceScopedKey('local', 'source');
const draftKey = surfaceScopedKey('local', 'destination');

describe('shared annotation source inventory', () => {
  it('keeps queued marks until their message is sent or removed', () => {
    const read = createConversationExcerptInventory();
    const sessions = new Map([['destination', session('destination')]]);
    const queued: QueuedMessage = {
      id: 'queued', sessionId: 'destination', content: 'Question', timestamp: 1, status: 'queued', retryCount: 0,
      userMessageMetadata: { composerPresentation: withConversationExcerpts(null, [excerpt], 'Question') },
      composerDraft: { value: 'Question', contexts: [excerpt], pendingLargePastes: {} },
    };
    const snapshot = read(sessions, {}, [queued]).get(sourceKey);
    expect(snapshot?.get('turn')).toEqual([excerpt]);
    expect(read(sessions, {}, [{ ...queued, status: 'sending' }]).get(sourceKey)).toBe(snapshot);
    expect(read(new Map([['destination', session('destination', [excerpt])]]), {}, []).get(sourceKey)?.get('turn')).toEqual([excerpt]);
    expect(read(sessions, {}, []).has(sourceKey)).toBe(false);
  });

  it('removes a pending marker with its attachment and retains sent markers after draft clearing', () => {
    const read = createConversationExcerptInventory();
    let sessions = new Map([['destination', session('destination')]]);
    expect(read(sessions, { [draftKey]: draft([excerpt]) }).get(sourceKey)?.get('turn')).toEqual([excerpt]);
    expect(read(sessions, {}).has(sourceKey)).toBe(false);
    sessions = new Map([['destination', session('destination', [excerpt])]]);
    const sent = read(sessions, {}).get(sourceKey)!;
    expect(sent.get('turn')).toEqual([excerpt]);
    const revised = { ...excerpt, comment: 'Revised' };
    expect(read(sessions, { [draftKey]: draft([revised]) }).get(sourceKey)?.get('turn')).toEqual([revised]);
    expect(read(sessions, {}).get(sourceKey)?.get('turn')).toEqual([excerpt]);
  });

  it('indexes by source device and session instead of the destination transcript', () => {
    const read = createConversationExcerptInventory();
    const peer = { ...excerpt, source: { ...excerpt.source, surfaceId: 'peer' } };
    const child = { ...excerpt, id: 'child', source: { ...excerpt.source, sessionId: 'child' } };
    const sessions = new Map([
      ['destination', session('destination', [excerpt, child])],
      ['peer-destination', session('peer-destination', [peer])],
    ]);
    const index = read(sessions, {});
    expect(index.get(sourceKey)?.get('turn')).toEqual([excerpt]);
    expect(index.get(surfaceScopedKey('peer', 'source'))?.get('turn')).toEqual([peer]);
    expect(index.get(surfaceScopedKey('local', 'child'))?.get('turn')).toEqual([child]);
    expect(index.has(surfaceScopedKey('local', 'destination'))).toBe(false);
  });

  it('preserves source snapshots through parent streaming and unrelated draft typing', () => {
    const read = createConversationExcerptInventory();
    const original = session('parent', [excerpt]);
    const sessions = new Map([['parent', original]]);
    const first = read(sessions, {}).get(sourceKey);
    const streaming = { ...original, dialogTurns: original.dialogTurns.map(turn => ({ ...turn, modelRounds: [] })) };
    expect(read(new Map([['parent', streaming]]), { [draftKey]: draft([]) }).get(sourceKey)).toBe(first);
    expect(read(new Map(), {}).has(sourceKey)).toBe(false);
  });
});
