import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..');
const etsRoot = path.join(repoRoot, 'src/apps/mobile/harmonyos/entry/src/main/ets');
const pagesRoot = path.join(etsRoot, 'pages');

function walkEts(root) {
  const entries = fs.readdirSync(root, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      files.push(...walkEts(entryPath));
    } else if (entry.isFile() && entry.name.endsWith('.ets')) {
      files.push(entryPath);
    }
  }
  return files;
}

function relative(file) {
  return path.relative(repoRoot, file).split(path.sep).join('/');
}

function imports(file) {
  const source = fs.readFileSync(file, 'utf8');
  const specs = [...source.matchAll(/from\s+['"]([^'"]+)['"]/g)].map((match) => match[1]);
  return specs.map((spec) => {
    if (!spec.startsWith('.')) {
      return spec;
    }
    return path.relative(etsRoot, path.resolve(path.dirname(file), spec)).split(path.sep).join('/');
  });
}

function filesUnder(root) {
  return walkEts(root).sort();
}

const allPages = filesUnder(pagesRoot);
const services = filesUnder(path.join(etsRoot, 'services'));
const components = allPages.filter((file) => file.includes(`${path.sep}pages${path.sep}components${path.sep}`));
const viewmodels = allPages.filter((file) => file.includes(`${path.sep}pages${path.sep}viewmodel${path.sep}`));
const stateFiles = allPages.filter((file) => file.includes(`${path.sep}pages${path.sep}state${path.sep}`));
const forbiddenComponentPlatformImports = new Set([
  '@kit.AbilityKit',
  '@kit.IMEKit',
  '@kit.ScanKit'
]);

const serviceToPages = services
  .filter((file) => imports(file).some((spec) => spec === 'pages' || spec.startsWith('pages/')))
  .map(relative);
const componentToViewmodel = components
  .filter((file) => imports(file).some((spec) => spec === 'pages/viewmodel' || spec.startsWith('pages/viewmodel/')))
  .map(relative);
const viewmodelToComponents = viewmodels
  .filter((file) => imports(file).some((spec) => spec === 'pages/components' || spec.startsWith('pages/components/')))
  .map(relative);
const stateToComponents = stateFiles
  .filter((file) => imports(file).some((spec) => spec === 'pages/components' || spec.startsWith('pages/components/')))
  .map(relative);
const componentToRawPlatformServices = components
  .filter((file) => imports(file).some((spec) => forbiddenComponentPlatformImports.has(spec)))
  .map(relative);
const hardcodedTextFontSizes = components.flatMap((file) => {
  const source = fs.readFileSync(file, 'utf8');
  const builderMatcher = /\b(SymbolGlyph|Text|Button|TextInput|TextArea|Search|Span|RichEditor)\s*\(/g;
  const builders = [...source.matchAll(builderMatcher)];
  return [...source.matchAll(/\.fontSize\((\d+(?:\.\d+)?)\)/g)].flatMap((match) => {
    const builder = builders.filter((candidate) => candidate.index < match.index).at(-1);
    if (builder?.[1] === 'SymbolGlyph') {
      return [];
    }
    const line = source.slice(0, match.index).split('\n').length;
    return [`${relative(file)}:${line}:${match[1]}`];
  });
});
const indirectHardcodedTypography = components.flatMap((file) => {
  const source = fs.readFileSync(file, 'utf8');
  return [...source.matchAll(/this\.InlineText\([^;]*?,\s*(\d+(?:\.\d+)?)\s*,\s*(\d+(?:\.\d+)?)\s*,/gs)]
    .map((match) => {
      const line = source.slice(0, match.index).split('\n').length;
      return `${relative(file)}:${line}:${match[1]}/${match[2]}`;
    });
});
const v1Components = allPages
  .filter((file) => /^\s*@Component\s*$/m.test(fs.readFileSync(file, 'utf8')))
  .map(relative);
const positionalActionConstructors = allPages
  .filter((file) => /export\s+class\s+\w+(?:Actions|Hooks)\b/.test(fs.readFileSync(file, 'utf8')) &&
    /\bconstructor\s*\(/.test(fs.readFileSync(file, 'utf8')))
  .map(relative);
const sharedConversationFields = [
  'sessions',
  'activeSession',
  'persistedMessages',
  'optimisticMessages',
  'activeTurnMessage',
  'hasMoreMessages',
  'timelineItems',
  'timelineRevision',
  'isBusy',
  'modelCatalog',
  'selectedModelId',
  'statusText',
  'chatInput',
  'selectedImages',
  'isVoiceListening'
];
const conversationPageStateFiles = [
  path.join(pagesRoot, 'state/GeneralChatPageState.ets'),
  path.join(pagesRoot, 'state/RemotePageState.ets')
];
const duplicatedConversationTraceFields = conversationPageStateFiles.flatMap((file) => {
  const source = fs.readFileSync(file, 'utf8');
  return sharedConversationFields
    .filter((field) => new RegExp(`@Trace\\s+${field}\\s*:`).test(source))
    .map((field) => `${relative(file)}:${field}`);
});
const appRootRuntimeFile = path.join(pagesRoot, 'runtime/AppRootRuntime.ets');
const appRootRuntimeSource = fs.readFileSync(appRootRuntimeFile, 'utf8');
const requiredPresentationFiles = [
  'components/AppRootOverlaySurfaces.ets',
  'components/ChatMessageChrome.ets',
  'components/RemoteSessionRow.ets',
  'components/ConnectManualPairingOverlay.ets',
  'components/ConversationHeader.ets',
  'components/ConversationRouteSurface.ets',
  'components/ToolInteractionPanels.ets',
  'components/WideConversationHost.ets',
  'components/platform/InlineQrScanner.ets',
  'components/remote/RemoteSurfaceHost.ets',
  'policy/ConversationHeaderPolicy.ets',
  'policy/ChatMessageStructurePolicy.ets',
  'policy/ToolStatusPresentationPolicy.ets',
  'viewmodel/ConversationRuntime.ets',
  'viewmodel/RemoteTranscriptController.ets',
  'viewmodel/RemoteCreateFlowController.ets',
  'viewmodel/VisibleConversationController.ets'
];
const missingPresentationFiles = requiredPresentationFiles
  .filter((file) => !fs.existsSync(path.join(pagesRoot, file)));
const extractedFilePreviewMethods = [
  'openFilePreview',
  'closeFilePreview',
  'refreshFilePreview',
  'openFilePreviewLink',
  'invalidateFilePreviewTarget'
].filter((method) => new RegExp(`^\\s{2}${method}\\s*\\(`, 'm').test(appRootRuntimeSource));
const extractedSettingsMethods = [
  'saveGeneralChatConfig',
  'testGeneralChatConfig',
  'validateGeneralChatConfig',
  'probeGeneralChatConfig',
  'effectiveGeneralChatApiKey',
  'applyGeneralChatConfig',
  'refreshGeneralChatModelCatalog'
].filter((method) => new RegExp(`^\\s{2}(?:private\\s+)?(?:async\\s+)?${method}\\s*\\(`, 'm')
  .test(appRootRuntimeSource));
const extractedCloudAccountMethods = [
  'persistDelegatedAccountSession',
  'loginCloudAccount',
  'restoreCloudAccountSession',
  'loadGeneralChatAccountModels',
  'applyCloudAccountSession',
  'logoutCloudAccount',
  'listCloudAccountDevices',
  'getRemotePermissionMode',
  'setRemotePermissionMode',
  'restoreCloudTarget',
  'expireCloudAccountSession',
  'handleRemoteConnectionError',
  'selectCloudAccountDevice'
].filter((method) => new RegExp(`^\\s{2}(?:private\\s+|protected\\s+)?(?:async\\s+)?${method}\\s*\\(`, 'm')
  .test(appRootRuntimeSource));
const extractedConversationMethods = [
  'isGeneralComposerRoute',
  'visibleChatInput',
  'visibleSelectedImages',
  'visibleVoiceListening',
  'setChatInputForRoute',
  'setSelectedImagesForRoute',
  'addSelectedImagesForRoute',
  'removeSelectedImageForRoute',
  'clearComposerForRoute',
  'setVoiceListeningForRoute',
  'setAllVoiceListening',
  'voiceInputSnapshot',
  'visibleChatBusy',
  'visibleStatusText',
  'setVisibleStatusText'
].filter((method) => new RegExp(`^\\s{2}(?:private\\s+)?(?:async\\s+)?${method}\\s*\\(`, 'm')
  .test(appRootRuntimeSource));
const extractedRemoteConversationMethods = [
  'sendChatMessage',
  'stopActiveTask',
  'renameActiveSession',
  'copyMessage',
  'downloadFile',
  'retryMessage',
  'approveTool',
  'rejectTool',
  'cancelTool',
  'answerQuestion',
  'resetChatTimeline',
  'syncChatTimelineFromStore',
  'startPolling',
  'currentChatPollingCursor',
  'updateChatPollingCursor',
  'applyChatSessionSnapshot',
  'hasRunningActiveTurn',
  'projectedTimelineItems',
  'syncAfterTurnEnded'
].filter((method) => new RegExp(`^\\s{2}(?:private\\s+)?(?:async\\s+)?${method}\\s*\\(`, 'm')
  .test(appRootRuntimeSource));
const extractedRemoteCreateMethods = [
  'createSession',
  'openRemoteCreateSession',
  'closeRemoteCreateSession',
  'loadRemoteCreateChoices',
  'loadRemoteCreateModelCatalog',
  'loadRemoteCreateDevices',
  'loadRemoteCreateWorkspaces',
  'toggleRemoteCreateDevices',
  'toggleRemoteCreateWorkspaces',
  'selectRemoteCreateDevice',
  'selectRemoteCreateWorkspace',
  'submitRemoteCreateSession',
  'createSessionInWorkspace',
  'openSession',
  'applyRemoteActiveSession',
  'deleteSession'
].filter((method) => new RegExp(`^\\s{2}(?:private\\s+)?(?:async\\s+)?${method}\\s*\\(`, 'm')
  .test(appRootRuntimeSource));
const extractedGeneralConversationMethods = [
  'openHomeSession',
  'openHomeSessionInPlace',
  'deleteHomeSession',
  'activeGeneralChatAsRemoteSession',
  'activeGeneralUploadedFileCount',
  'archiveHomeSession',
  'exportHomeSession',
  'openGeneralSession',
  'startGeneralChat',
  'sendVisibleChatMessage',
  'stopActiveChatTask',
  'closeActiveChat',
  'renameVisibleSession',
  'retryVisibleMessage',
  'downloadVisibleFile',
  'selectModel',
  'sendGeneralChatMessage',
  'stopGeneralChatStream',
  'startVisibleGeneralChat',
  'generalChatHomeStatusText',
  'prepareNewGeneralChat',
  'onVisibleChatInputChange',
  'visibleGeneralChatDraftId',
  'restoreGeneralChatDraft',
  'latestUserMessageText',
  'showHomeToast',
  'resetGeneralChatTimeline',
  'syncGeneralChatTimelineFromStore'
].filter((method) => new RegExp(`^\\s{2}(?:private\\s+)?(?:async\\s+)?${method}\\s*\\(`, 'm')
  .test(appRootRuntimeSource));
const extractedRemoteConnectionForwards = [
  'applyWorkspace',
  'applyRemotePairingProjection',
  'ensureRemoteAvailable',
  'setRemoteConnectionState',
  'setRemoteUrl',
  'setRemoteUserId',
  'setRemoteAuthenticatedUserId',
  'setRemoteStatusText',
  'setRemoteConnectionFailureKind',
  'setRemoteBusy',
  'setRemoteUrlInputVisible'
].filter((method) => new RegExp(`^\\s{2}(?:private\\s+)?(?:async\\s+)?${method}\\s*\\(`, 'm')
  .test(appRootRuntimeSource));
const appRootRuntimeStateGetters = [
  'remoteUrl', 'userId', 'authenticatedUserId', 'statusText', 'connectionState',
  'connectionFailureKind', 'isBusy', 'showRemoteUrlInput', 'workspaceName', 'workspacePath',
  'workspaceBranch', 'workspaceKind', 'assistantId', 'desktopName', 'desktopId', 'activeSession',
  'messages', 'pendingMessages', 'activeTurnMessage', 'timelineItems', 'hasMoreMessages'
].filter((getter) => new RegExp(`^\\s{2}get\\s+${getter}\\s*\\(`, 'm').test(appRootRuntimeSource));
const extractedOwnerForwards = [
  'currentRoute', 'isRoute', 'isGeneralChatVisible', 'pushRoute', 'replaceRoute', 'popRoute',
  'handleConversationIntent', 'pasteRemoteUrl', 'scanRemoteUrl', 'handleDetectedRemoteUrl',
  'showRecentWorkspaces', 'showAssistants', 'refreshSessions', 'loadMoreSessions', 'setSessionFilter',
  'openAddConnection', 'selectRemoteCreateModel', 'loadRecentWorkspacesInBackground',
  'loadOlderMessages', 'removeSelectedImage', 'persistVisibleGeneralChatDraft', 'stopPolling',
  'nudgeChatPolling', 'pollActiveSession', 'startHeartbeat', 'stopHeartbeat',
  'checkConnectionHealth', 'resumeRemoteActivity'
].filter((method) => new RegExp(`^\\s{2}(?:private\\s+)?(?:async\\s+)?${method}\\s*\\(`, 'm')
  .test(appRootRuntimeSource));

const expected = {
  serviceToPages: [],
  componentToViewmodel: [],
  viewmodelToComponents: [],
  stateToComponents: [],
  componentToRawPlatformServices: [],
  hardcodedTextFontSizes: [],
  indirectHardcodedTypography: [],
  v1Components: [],
  positionalActionConstructors: [],
  duplicatedConversationTraceFields: [],
  extractedFilePreviewMethods: [],
  extractedSettingsMethods: [],
  extractedCloudAccountMethods: [],
  extractedConversationMethods: [],
  extractedRemoteConversationMethods: [],
  extractedRemoteCreateMethods: [],
  extractedGeneralConversationMethods: [],
  extractedRemoteConnectionForwards: [],
  appRootRuntimeStateGetters: [],
  extractedOwnerForwards: [],
  missingPresentationFiles: [],
  hiddenLocalConversationSections: [],
  eagerChatTimeline: [],
  missingTimelineReuse: [],
  snapshottedTimelineRepeatItem: [],
  snapshottedMessageBuilderInput: [],
  missingObservableTimelineRows: [],
  wideConversationViewContract: [],
  duplicateSheetGeometry: [],
  synchronousArgon2Declaration: [],
  synchronousNativeArgon2: [],
  misplacedPresentationPolicies: []
};

function sameSet(actual, wanted) {
  return actual.length === wanted.length && actual.every((item, index) => item === wanted[index]);
}

const actual = {
  serviceToPages,
  componentToViewmodel,
  viewmodelToComponents,
  stateToComponents,
  componentToRawPlatformServices,
  hardcodedTextFontSizes,
  indirectHardcodedTypography,
  v1Components,
  positionalActionConstructors,
  duplicatedConversationTraceFields,
  extractedFilePreviewMethods,
  extractedSettingsMethods,
  extractedCloudAccountMethods,
  extractedConversationMethods,
  extractedRemoteConversationMethods,
  extractedRemoteCreateMethods,
  extractedGeneralConversationMethods,
  extractedRemoteConnectionForwards,
  appRootRuntimeStateGetters,
  extractedOwnerForwards,
  missingPresentationFiles,
  hiddenLocalConversationSections,
  eagerChatTimeline,
  missingTimelineReuse,
  snapshottedTimelineRepeatItem,
  snapshottedMessageBuilderInput,
  missingObservableTimelineRows,
  wideConversationViewContract,
  duplicateSheetGeometry,
  synchronousArgon2Declaration,
  synchronousNativeArgon2,
  misplacedPresentationPolicies
};
let failed = false;
for (const [name, wanted] of Object.entries(expected)) {
  if (!sameSet(actual[name], wanted)) {
    failed = true;
    console.error(`${name} mismatch`);
    console.error(`expected: ${JSON.stringify(wanted)}`);
    console.error(`actual:   ${JSON.stringify(actual[name])}`);
  }
}
if (failed) {
  process.exitCode = 1;
} else {
  console.log('HarmonyOS architecture contracts are satisfied.');
}
