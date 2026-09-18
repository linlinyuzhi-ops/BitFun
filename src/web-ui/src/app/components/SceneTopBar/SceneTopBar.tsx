import React, { useCallback } from 'react';
import { Toolbar, ToolbarSeparator } from '@openbitfun/ui';
import { WindowControls } from '@/app/components/WindowControls';
import { startNativeWindowDragging, supportsNativeWindowDragging } from '@/infrastructure/runtime';
import { createLogger } from '@/shared/utils/logger';
import { useSceneStore } from '../../stores/sceneStore';
import SceneBar from '../SceneBar/SceneBar';
import { SceneChromeHost } from './SceneChrome';
import './SceneTopBar.scss';

const log = createLogger('SceneTopBar');

const INTERACTIVE_SELECTOR =
  'button, input, textarea, select, a, [role="button"], [role="tab"], [role="menu"], [contenteditable]:not([contenteditable="false"]), [draggable="true"], .window-controls';

function blocksWindowChromeInteraction(
  event: React.MouseEvent<HTMLDivElement>,
): boolean {
  const target = event.target;
  if (event.defaultPrevented || !(target instanceof Element) || !event.currentTarget.contains(target)) {
    return true;
  }

  // The whole tab, including item padding, belongs to tab interaction.
  if (target.closest('[data-openbitfun-component="tab-group"] [data-openbitfun-part="item"]')) {
    return true;
  }
  return target.closest(INTERACTIVE_SELECTOR) !== null;
}

interface SceneTopBarProps {
  className?: string;
  onMinimize?: () => void;
  onMaximize?: () => void;
  onClose?: () => void;
  isMaximized?: boolean;
}

const SceneTopBar: React.FC<SceneTopBarProps> = ({
  className = '',
  onMinimize,
  onMaximize,
  onClose,
  isMaximized = false,
}) => {
  const hasTabs = useSceneStore(state => state.openTabs.length > 0);
  const canDragWindow = supportsNativeWindowDragging();
  const hasWindowControls = Boolean(onMinimize && onMaximize && onClose);

  const handleMouseDown = useCallback((event: React.MouseEvent<HTMLDivElement>) => {
    if (!canDragWindow || event.button !== 0 || event.detail > 1) return;
    if (blocksWindowChromeInteraction(event)) return;

    void startNativeWindowDragging().catch(error => {
      log.debug('startDragging failed', { error });
    });
  }, [canDragWindow]);

  const handleDoubleClick = useCallback((event: React.MouseEvent<HTMLDivElement>) => {
    if (!canDragWindow || event.button !== 0 || blocksWindowChromeInteraction(event)) return;
    onMaximize?.();
  }, [canDragWindow, onMaximize]);

  return (
    <Toolbar
      bordered={hasTabs}
      className={`openbitfun-scene-top-bar ${className}`.trim()}
      onMouseDown={handleMouseDown}
      onDoubleClick={handleDoubleClick}
      data-openbitfun-scene="workbench"
      data-openbitfun-part="topBar"
      leading={<SceneBar />}
      size="md"
      trailing={<>
        <SceneChromeHost
          className="openbitfun-scene-top-bar__actions"
          data-openbitfun-scene="workbench"
          data-openbitfun-part="sceneActions"
        />
        {hasWindowControls ? (
          <>
            <ToolbarSeparator className="openbitfun-scene-top-bar__actions-divider" />
            <div
              className="openbitfun-scene-top-bar__window-controls"
              data-openbitfun-component="scene-bar"
              data-openbitfun-part="controls"
            >
              <WindowControls
                onMinimize={onMinimize!}
                onToggleMaximize={onMaximize!}
                onClose={onClose!}
                maximized={isMaximized}
              />
            </div>
          </>
        ) : null}
      </>}
    />
  );
};

export default SceneTopBar;
