import { api } from '@/infrastructure/api/service-api/ApiClient';
import { agentAPI } from '@/infrastructure/api/service-api/AgentAPI';
import { getActiveSurfaceScope } from '@/infrastructure/peer-device/deviceSurface';
import { peerConnectionManager } from '@/infrastructure/peer-device/PeerConnectionManager';
import { isTauriRuntime } from '@/infrastructure/runtime';
import { flowChatStore } from '../store/FlowChatStore';
import { resolveSessionDriverId } from '../session-drivers/resolve';
import { isAcpFlowSession } from '../utils/acpSession';

export interface ControlConversation { sessionId: string; workspacePath: string }
const pending = new Map<number, Promise<ControlConversation>>();

export function supportsControlConversation(): boolean {
  const { surfaceId } = getActiveSurfaceScope();
  return surfaceId === 'local'
    ? isTauriRuntime()
    : peerConnectionManager.get(surfaceId)?.getState().capabilities.controlConversationV1 === true;
}

export function supportsNewControlConversation(): boolean {
  const { surfaceId } = getActiveSurfaceScope();
  return supportsControlConversation() && (surfaceId === 'local'
    || peerConnectionManager.get(surfaceId)?.getState().capabilities.controlConversationResetV1 === true);
}

export function assertVoiceConversationSupported(sessionId: string) {
  if (!supportsControlConversation()) throw new Error('This host does not support shared voice history. Update the target host.');
  const session = flowChatStore.getState().sessions.get(sessionId);
  if (!session) throw new Error('The voice conversation is unavailable');
  if (isAcpFlowSession(session) || resolveSessionDriverId(sessionId, session) !== 'local') {
    throw new Error('Shared voice history is not supported for this session provider. Continue in text mode.');
  }
}

export function ensureControlConversation(): Promise<ControlConversation> {
  return requestControlConversation();
}

export function createControlConversation(expectedSessionId: string): Promise<ControlConversation> {
  if (!supportsNewControlConversation()) return Promise.reject(new Error('This host does not support new control conversations. Update the target host.'));
  return requestControlConversation(expectedSessionId);
}

function requestControlConversation(expectedSessionId?: string): Promise<ControlConversation> {
  const scope = getActiveSurfaceScope();
  if (!supportsControlConversation()) return Promise.reject(new Error(scope.surfaceId === 'local'
    ? 'Persistent control conversations are not available on the web server. Use Desktop or a supported peer host.'
    : 'This host does not support persistent control conversations. Update the target host.'));
  const previous = pending.get(scope.epoch);
  if (previous && expectedSessionId === undefined) return previous;
  const request = (async () => {
    // Selection is serialized with initial hydration. A late initial read must
    // not replace a newly created conversation in the dock.
    if (previous) await previous;
    scope.assertCurrent('select control conversation');
    const result = expectedSessionId === undefined
      ? await api.invoke<ControlConversation>('ensure_control_conversation')
      : await api.invoke<ControlConversation>('create_control_conversation', { request: { expectedSessionId } });
    scope.assertCurrent('ensure control conversation');
    const existing = flowChatStore.getState().sessions.get(result.sessionId);
    if (!existing) {
      flowChatStore.addExternalSession(result.sessionId, 'OpenBitFun', 'OpenBitFun', result.workspacePath,
        { isTransient: true, agentBackedTransient: true });
    }
    if (!existing || existing.historyState === 'failed') {
      await flowChatStore.loadSessionHistory(result.sessionId, result.workspacePath, undefined, undefined, undefined, { includeInternal: true });
    }
    scope.assertCurrent('hydrate control conversation');
    return result;
  })().finally(() => { if (pending.get(scope.epoch) === request) pending.delete(scope.epoch); });
  pending.set(scope.epoch, request);
  return request;
}

export interface VoiceExchange {
  surfaceId: string; sessionId: string; workspacePath: string;
  remoteConnectionId?: string; remoteSshHost?: string;
  exchangeId: string; userText: string; assistantText: string;
}
const OUTBOX = 'openbitfun-voice-exchange:';
const volatileOutbox = new Map<string, VoiceExchange>();
const outboxKey = (request: VoiceExchange) => OUTBOX + JSON.stringify([request.surfaceId, request.sessionId, request.exchangeId]);
const outboxStorage = () => typeof localStorage === 'undefined' ? undefined : localStorage;

/** Keep ordinary text submissions synchronous when no native history needs recovery. */
export function hasPendingVoiceExchanges(sessionId: string): boolean {
  const { surfaceId } = getActiveSurfaceScope();
  if ([...volatileOutbox.values()].some(request => request.surfaceId === surfaceId && request.sessionId === sessionId)) return true;
  const storage = outboxStorage();
  return storage !== undefined && Object.keys(storage).some(key => {
    if (!key.startsWith(OUTBOX)) return false;
    try {
      const [surface, session] = JSON.parse(key.slice(OUTBOX.length));
      return surface === surfaceId && session === sessionId;
    } catch { return false; }
  });
}

export function stageVoiceExchange(request: VoiceExchange) {
  volatileOutbox.set(outboxKey(request), request);
  const storage = outboxStorage();
  if (!storage) throw new Error('Voice history is retained in memory; persistent storage is unavailable');
  storage.setItem(outboxKey(request), JSON.stringify(request));
}

export async function recordVoiceExchange(request: VoiceExchange) {
  const scope = getActiveSurfaceScope();
  if (scope.surfaceId !== request.surfaceId) throw new Error('Voice conversation belongs to another device');
  // The host can evict a session or restart while an outbox entry is pending.
  // Restore through the existing workspace adapter, including SSH identity.
  await agentAPI.ensureCoordinatorSession({
    sessionId: request.sessionId, workspacePath: request.workspacePath,
    remoteConnectionId: request.remoteConnectionId, remoteSshHost: request.remoteSshHost,
    includeInternal: true,
  });
  scope.assertCurrent('restore voice history session');
  await api.invoke('record_voice_exchange', { request: {
    sessionId: request.sessionId, exchangeId: request.exchangeId,
    userText: request.userText, assistantText: request.assistantText,
  } });
  volatileOutbox.delete(outboxKey(request));
  outboxStorage()?.removeItem(outboxKey(request));
  scope.assertCurrent('record voice exchange');
  if (flowChatStore.getState().sessions.has(request.sessionId)) {
    await flowChatStore.loadSessionHistory(request.sessionId, request.workspacePath, undefined, request.remoteConnectionId, request.remoteSshHost, { includeInternal: true });
  }
}

const replays = new Map<string, Promise<void>>();
/** Unacknowledged exchanges survive reload/disconnect and retry on their own device. */
export function replayVoiceExchanges(sessionId?: string): Promise<void> {
  const scope = getActiveSurfaceScope();
  const replayKey = JSON.stringify([scope.epoch, sessionId]);
  const existing = replays.get(replayKey);
  if (existing) return existing;
  const replay = (async () => {
    const requests = new Map(volatileOutbox);
    const failures: unknown[] = [];
    const storage = outboxStorage();
    for (const key of Object.keys(storage ?? {}).filter(key => key.startsWith(OUTBOX))) {
      let selected = !sessionId;
      try {
        const [surface, session] = JSON.parse(key.slice(OUTBOX.length));
        if (surface !== scope.surfaceId || sessionId && sessionId !== session) continue;
        selected = true;
        const request = JSON.parse(storage!.getItem(key)!) as VoiceExchange;
        if (!request || outboxKey(request) !== key
          || !['surfaceId', 'sessionId', 'workspacePath', 'exchangeId', 'userText', 'assistantText']
            .every(field => typeof request[field as keyof VoiceExchange] === 'string')) {
          throw new Error('Invalid voice history record retained for recovery');
        }
        requests.set(key, request);
      } catch (error) { if (selected) failures.push(error); }
    }
    for (const request of requests.values()) {
      scope.assertCurrent('recover voice history');
      const session = flowChatStore.getState().sessions.get(request.sessionId);
      if (request.surfaceId !== scope.surfaceId || sessionId && sessionId !== request.sessionId) continue;
      if (session?.dialogTurns.some(turn => ['pending', 'processing', 'finishing', 'image_analyzing', 'cancelling'].includes(turn.status))) continue;
      try { await recordVoiceExchange(request); } catch (error) { failures.push(error); }
    }
    if (failures.length) throw Object.assign(new Error('Some voice history is awaiting recovery'), { errors: failures });
  })().finally(() => { replays.delete(replayKey); });
  replays.set(replayKey, replay);
  return replay;
}
