// @vitest-environment jsdom

import React, { act } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createRoot, type Root } from 'react-dom/client';

vi.mock('./settingsRegistry', () => {
  const pages = {
    'application.general': {
      id: 'application.general',
      categoryId: 'application',
      component: ({ viewId, isActive }: { viewId?: string; isActive?: boolean }) => (
        <div
          data-testid="general-page"
          data-view={viewId}
          data-settings-scene-active={isActive ? 'true' : 'false'}
        />
      ),
    },
    'application.appearance': {
      id: 'application.appearance',
      categoryId: 'application',
      component: () => <div data-testid="appearance-page" />,
    },
    'tools.automation': {
      id: 'tools.automation',
      categoryId: 'tools',
      component: ({ viewId }: { viewId?: string }) => <div data-testid="automation-page" data-view={viewId} />,
    },
  };
  return {
    DEFAULT_SETTINGS_PAGE_ID: 'application.general',
    getSettingsPageManifest: (pageId: keyof typeof pages) => pages[pageId] ?? pages['application.general'],
    isSettingsPageId: (value: string) => value in pages,
    isSettingsPageReady: () => true,
    preloadSettingsPage: vi.fn(async () => undefined),
  };
});

import SettingsScene from './SettingsScene';
import { useSettingsStore } from './settingsStore';
import {
  registerSettingsDraft,
  resetSettingsDraftRegistryForTests,
} from '@/infrastructure/config/settingsDraftRegistry';

vi.mock('../../../infrastructure/config/components/AIModelConfig', () => ({
  default: () => <div data-testid="models-config" />,
}));

vi.mock('../../../infrastructure/config/components/McpToolsConfig', () => ({
  default: () => <div data-testid="mcp-tools-config" />,
}));

vi.mock('../../../infrastructure/config/components/AcpAgentsConfig', () => ({
  default: () => <div data-testid="acp-agents-config" />,
}));

vi.mock('../../../infrastructure/config/components/ExternalSourcesConfig', () => ({
  default: ({
    initialFocus,
    focusRequestId,
  }: {
    initialFocus?: 'hooks';
    focusRequestId?: number;
  }) => (
    <div
      data-testid="external-sources-config"
      data-initial-focus={initialFocus}
      data-focus-request-id={focusRequestId}
    />
  ),
}));

vi.mock('../../../infrastructure/config/components/EditorConfig', () => ({
  default: () => <div data-testid="editor-config" />,
}));

vi.mock('../../../infrastructure/config/components/BasicsConfig', () => ({
  default: () => <div data-testid="basics-config" />,
}));

vi.mock('../../../infrastructure/config/components/AppearanceConfig', () => ({
  default: () => <div data-testid="appearance-config" />,
}));

vi.mock('../../../infrastructure/config/components/ReviewConfig', () => ({
  default: () => <div data-testid="review-config" />,
}));

vi.mock('../../../infrastructure/config/components/QuickActionsConfig', () => ({
  default: () => <div data-testid="quick-actions-config" />,
}));

vi.mock('../../../infrastructure/config/components/VoiceInputConfig', () => ({
  default: () => <div data-testid="voice-input-config" />,
}));

vi.mock('../../../infrastructure/config/components/SessionConfig', () => ({
  SessionPersonalizationConfig: () => <div data-testid="session-personalization-config" />,
  SessionPermissionsConfig: () => <div data-testid="session-permissions-config" />,
}));

/**
 * The scene runs its own external-application lookup on mount, which reaches
 * the real transport: outside Tauri that is the WebSocket adapter, and once its
 * connection to a server nobody started fails it keeps retrying — and logging —
 * on a timer that outlives this file. A console write from that timer can land
 * in a worker whose rpc is already closing, failing the run with an
 * EnvironmentTeardownError attributed here. Tab routing is what this file
 * covers; the lookup is chrome it does not assert on.
 */
vi.mock('../../../infrastructure/config/components/external-sources/useExternalAppAwareness', () => ({
  useExternalAppAwareness: () => {},
}));

vi.mock('./components/ArchivedSessionsConfig', () => ({
  default: () => <div data-testid="archived-sessions-config" />,
}));

vi.mock('./components/KeyboardShortcutsTab', () => ({
  default: () => <div data-testid="keyboard-shortcuts-config" />,
}));

describe('SettingsScene lazy tab routing', () => {
  let container: HTMLDivElement;
  let root: Root;

  beforeEach(() => {
    resetSettingsDraftRegistryForTests();
    container = document.createElement('div');
    document.body.appendChild(container);
    root = createRoot(container);
    useSettingsStore.setState({
      activePageId: 'application.general',
      activeViewId: null,
      navigationRequestId: 0,
      pageTransitionTarget: null,
      pageTransitionMotion: 'instant',
      pageTransitionSequence: 0,
      searchQuery: '',
    });
  });

  afterEach(() => {
    act(() => root.unmount());
    container.remove();
    resetSettingsDraftRegistryForTests();
  });

  it('renders the active canonical page', async () => {
    await act(async () => root.render(<SettingsScene />));
    expect(container.querySelector('[data-testid="general-page"]')).not.toBeNull();
    expect(container.querySelector('[data-settings-page="application.general"]')).not.toBeNull();
  });

  it('passes an internal view destination without creating a sidebar page', async () => {
    useSettingsStore.getState().openDestination({
      pageId: 'tools.automation',
      viewId: 'hooks',
    });
    await act(async () => root.render(<SettingsScene />));
    expect(container.querySelector('[data-testid="automation-page"]')?.getAttribute('data-view')).toBe('hooks');
  });

  it('switches pages without retaining the outgoing page for instant navigation', async () => {
    await act(async () => root.render(<SettingsScene />));
    await act(async () => useSettingsStore.getState().openPage('application.appearance'));
    expect(container.querySelector('[data-testid="appearance-page"]')).not.toBeNull();
    expect(container.querySelector('[data-testid="general-page"]')).toBeNull();
  });

  it('passes scene activation changes to the active settings page', async () => {
    await act(async () => root.render(<SettingsScene isActive={false} />));
    expect(container.querySelector('[data-testid="general-page"]')?.getAttribute(
      'data-settings-scene-active',
    )).toBe('false');

    await act(async () => root.render(<SettingsScene isActive />));
    expect(container.querySelector('[data-testid="general-page"]')?.getAttribute(
      'data-settings-scene-active',
    )).toBe('true');
  });

  it('saves registered drafts before committing a page change', async () => {
    const save = vi.fn(async () => true);
    registerSettingsDraft({
      id: 'general-form',
      pageId: 'application.general',
      label: 'General form',
      dirty: true,
      save,
      discard: vi.fn(),
    });
    await act(async () => root.render(<SettingsScene />));

    await act(async () => useSettingsStore.getState().openPage('application.appearance'));
    expect(useSettingsStore.getState().activePageId).toBe('application.general');
    const dialog = document.querySelector<HTMLElement>(
      '[data-testid="settings-unsaved-navigation-dialog"]',
    );
    expect(dialog?.textContent).toContain('General form');

    const confirmButton = dialog?.querySelectorAll<HTMLButtonElement>('button').item(2);
    await act(async () => {
      confirmButton?.click();
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(save).toHaveBeenCalledOnce();
    expect(useSettingsStore.getState().activePageId).toBe('application.appearance');
  });
});
