import { RemoteSession, SessionSummary } from '../../model/RemoteModels';
import { CloudAccountDevice } from '../../services/CloudAccountClient';
import { RemoteI18n } from '../../i18n/RemoteI18n';
import { RemoteLogger } from '../../services/RemoteLogger';
import { RemotePageState } from '../state/RemotePageState';
import {
  RemoteConversationDependencies,
  requireRemoteRuntime,
  routeCreatedRemoteSession,
  routeRemoteSessionInPlace
} from './ConversationRuntime';

/** Creates and opens remote sessions without an intermediate creation page. */
export class RemoteCreateFlowController {
  private readonly remote: RemotePageState;
  private readonly remoteRuntime?: RemoteConversationDependencies;

  constructor(
    remote: RemotePageState,
    remoteRuntime: RemoteConversationDependencies | undefined
  ) {
    this.remote = remote;
    this.remoteRuntime = remoteRuntime;
  }

  private navigateCreated(sessionId: string): void {
    routeCreatedRemoteSession(sessionId, requireRemoteRuntime(this.remoteRuntime), (dismissed: boolean) => {
      this.remote.setConversationDismissed(dismissed);
    });
  }

  private navigateInPlace(sessionId: string): void {
    routeRemoteSessionInPlace(sessionId, requireRemoteRuntime(this.remoteRuntime), (dismissed: boolean) => {
      this.remote.setConversationDismissed(dismissed);
    });
  }

  async createRemoteSession(agentType: string, inPlace: boolean = false): Promise<void> {
    const runtime = requireRemoteRuntime(this.remoteRuntime);
    runtime.filePreview.close();
    await runtime.sessions.createSession(
      agentType,
      '',
      inPlace ? (sessionId: string): void => this.navigateInPlace(sessionId) :
        (sessionId: string): void => this.navigateCreated(sessionId)
    );
  }

  async ensureRemoteControlTarget(deviceId?: string): Promise<void> {
    const target = (deviceId || '').trim();
    if (target.length === 0) {
      return;
    }
    const runtime = requireRemoteRuntime(this.remoteRuntime);
    const activeId = this.remote.controlTargetDeviceId || this.remote.desktopId;
    if (target === activeId && this.remote.connectionState === 'connected') {
      return;
    }
    this.remote.setStatusText(RemoteI18n.t('remote.settings.deviceConnecting'));
    try {
      let device = runtime.settings.knownAccountDevice(target);
      if (!device) {
        device = (await runtime.settings.listCloudAccountDevices())
          .find((item: CloudAccountDevice): boolean => item.deviceId === target);
      }
      if (!device) {
        throw new Error(RemoteI18n.t('remote.settings.deviceUnavailable'));
      }
      await runtime.settings.selectCloudAccountDevice(device, false);
    } catch (err) {
      this.remoteCreate.isLoadingWorkspaces = false;
      this.remoteCreate.errorText = err instanceof Error ? err.message :
        RemoteI18n.t('remote.settings.deviceSwitchFailed');
    }
  }

  selectRemoteCreateWorkspace(path: string): void {
    const workspace = this.remoteCreate.workspaces
      .find((item: RecentWorkspaceEntry): boolean => item.path === path);
    this.remoteCreate.selectWorkspace(workspace);
  }

  async submitRemoteCreateSession(): Promise<void> {
    const runtime = requireRemoteRuntime(this.remoteRuntime);
    const instruction = this.remoteCreate.draft.trim();
    if (instruction.length === 0 || this.remoteCreate.isSubmitting || !runtime.connection.ensureAvailable()) {
      return;
    }
    const context = this.remoteCreate.submissionContext();
    const activeDeviceId = this.remote.controlTargetDeviceId || this.remote.desktopId;
    if (context.deviceId.length === 0 || context.deviceId !== activeDeviceId) {
      this.remoteCreate.errorText = RemoteI18n.t('remote.create.deviceMismatch');
      return;
    }
    this.remoteCreate.isSubmitting = true;
    this.remoteCreate.errorText = '';
    this.remoteCreate.closeMenu();
    try {
      if (context.workspacePath.length > 0) {
        await runtime.sessions.createSessionInWorkspace(
          context.workspacePath,
          instruction,
          context.agentType,
          undefined,
          this.remoteCreate.selectedModelId
        );
      } else {
        await this.bindAssistantWorkspace();
        await runtime.sessions.createSession(
          context.agentType,
          instruction,
          undefined,
          this.remoteCreate.selectedModelId
        );
      }
      if (runtime.appShell.isRoute(AppRoute.RemoteCreate)) {
        this.remoteCreate.errorText = this.remote.statusText || RemoteI18n.t('remote.create.submitFailed');
      }
    } catch (err) {
      this.remoteCreate.errorText = err instanceof Error ? err.message :
        RemoteI18n.t('remote.create.submitFailed');
    } finally {
      this.remoteCreate.isSubmitting = false;
    }
  }

  /**
   * The chat option creates a Claw session, and the desktop always binds those
   * to its assistant workspace. Follow it there first, otherwise the app stays
   * bound to the code workspace it was on and the new chat is listed, titled
   * and file-scoped as if it had been created inside that workspace.
   */
  private async bindAssistantWorkspace(): Promise<void> {
    const runtime = requireRemoteRuntime(this.remoteRuntime);
    if (this.remote.workspaceKind === 'assistant') {
      return;
    }
    try {
      const assistants = await runtime.workspace.assistants();
      if (assistants.length === 0) {
        return;
      }
      await runtime.hooks.selectAssistantWorkspace(assistants[0].path);
    } catch (err) {
      RemoteLogger.warn(`assistant workspace bind failed: ${String(err)}`);
    }
  }

  async ensureRemoteControlTarget(deviceId?: string): Promise<void> {
    const target = (deviceId || '').trim();
    if (target.length === 0) {
      return;
    }
    const runtime = requireRemoteRuntime(this.remoteRuntime);
    const activeId = this.remote.controlTargetDeviceId || this.remote.desktopId;
    if (target === activeId && this.remote.connectionState === 'connected') {
      return;
    }
    this.remote.setStatusText(RemoteI18n.t('remote.settings.deviceConnecting'));
    try {
      let device = runtime.settings.knownAccountDevice(target);
      if (!device) {
        device = (await runtime.settings.listCloudAccountDevices())
          .find((item: CloudAccountDevice): boolean => item.deviceId === target);
      }
      if (!device) {
        throw new Error(RemoteI18n.t('remote.settings.deviceUnavailable'));
      }
      // The only visible sign that anything is happening. This switch is
      // implicit — the user tapped a session, not a device — and it runs behind
      // whatever surface the sidebar just closed over, which has no place to
      // show a connection state. `isLoadingHome` looks like the answer and is
      // not: the only thing that reads it is the RemoteHome route, and this
      // path never routes there.
      runtime.hooks.showToast(RemoteI18n.f('remote.settings.deviceSwitching', device.deviceName || target));
      await runtime.settings.selectCloudAccountDevice(device, false);
    } catch (err) {
      this.remote.setLoadingHome(false);
      throw new Error(err instanceof Error ? err.message : String(err));
    }
  }

  async createRemoteSessionInWorkspace(
    path: string,
    agentType: string = 'code',
    inPlace: boolean = false,
    deviceId: string = ''
  ): Promise<void> {
    const runtime = requireRemoteRuntime(this.remoteRuntime);
    runtime.filePreview.close();
    await this.ensureRemoteControlTarget(deviceId);
    await runtime.sessions.createSessionInWorkspace(
      path,
      '',
      agentType,
      inPlace ? (sessionId: string): void => this.navigateInPlace(sessionId) :
        (sessionId: string): void => this.navigateCreated(sessionId)
    );
  }

  async openRemoteSession(item: RemoteSession, inPlace: boolean = false): Promise<void> {
    const runtime = requireRemoteRuntime(this.remoteRuntime);
    runtime.filePreview.close();
    this.remote.setPendingSessionId(item.id);
    try {
      await this.ensureRemoteControlTarget(item.deviceId);
      await runtime.sessions.openSession(
        item,
        this.remote.workspacePath,
        inPlace ? (sessionId: string): void => this.navigateInPlace(sessionId) :
          (sessionId: string): void => this.navigateCreated(sessionId)
      );
    } catch (err) {
      this.remote.setPendingSessionId('');
      this.remote.setLoadingHome(false);
      RemoteLogger.warn(`open remote session failed session=${item.id}: ${String(err)}`);
      throw new Error(err instanceof Error ? err.message : String(err));
    }
  }

  applyRemoteActiveSession(session: SessionSummary): void {
    const runtime = requireRemoteRuntime(this.remoteRuntime);
    const current = this.remote.activeSession;
    if (runtime.hooks.isFilePreviewVisible() &&
      (current.sessionId !== session.sessionId || current.workspacePath !== session.workspacePath)) {
      runtime.filePreview.close();
    }
    this.remote.setActiveSession(session);
  }

  async deleteRemoteSession(item: RemoteSession): Promise<void> {
    await requireRemoteRuntime(this.remoteRuntime).sessions.deleteSession(item, this.remote.workspacePath);
  }

  routeCreatedRemoteSession(sessionId: string): void {
    this.navigateCreated(sessionId);
  }
}
