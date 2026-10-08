import { Context } from '@kit.AbilityKit';
import { UIContext, window } from '@kit.ArkUI';
import { RemoteLogger } from './RemoteLogger';

/**
 * How much of the window the app has to keep clear of its own content, in px.
 *
 * Both numbers are read rather than assumed. The shell runs in immersive layout,
 * so the page draws from the top of the window and the status bar is the app's
 * own inset to reserve; the bottom strip is the navigation indicator, until the
 * soft keyboard takes its place.
 */
export interface WindowInsets {
  top: number;
  bottom: number;
}

/** One read of the insets, plus the listener that keeps them current. */
export type WindowInsetListener = (insets: WindowInsets) => void;
export type WindowInsetSubscription = () => void;

/** Platform window adapter for surfaces that need to draw behind system bars. */
export class WindowSystemBarService {
  /**
   * Immersive layout for the whole shell, not one page: the transcript has to be
   * able to reach the bottom edge of the screen, which only exists in the page
   * area once the window stops reserving the navigation-bar strip.
   *
   * Immersive layout also moves the page's own origin to the top of the window,
   * so every page that draws chrome against an edge reads the insets below
   * instead of assuming the framework avoided the bars for it.
   */
  static async setLayoutFullScreen(context: Context, enabled: boolean): Promise<void> {
    try {
      const appWindow = await window.getLastWindow(context);
      await appWindow.setWindowLayoutFullScreen(enabled);
    } catch (_err) {
      // Window layout is best effort and must not block page rendering.
    }
  }

  static async topSystemInsetPx(context: Context): Promise<number> {
    try {
      const appWindow = await window.getLastWindow(context);
      return appWindow.getWindowAvoidArea(window.AvoidAreaType.TYPE_SYSTEM).topRect.height;
    } catch (_err) {
      return 0;
    }
  }

  /**
   * Reads the insets once and re-reads them on every avoid-area change — folding,
   * rotation and the soft keyboard all move the avoid areas without recreating
   * the page, and a stale number either clips the composer or leaves the
   * transcript reserving space nothing is drawn in.
   *
   * The returned function unregisters the listener.
   */
  static async observeInsets(context: Context, onChange: WindowInsetListener): Promise<WindowInsetSubscription> {
    try {
      const appWindow = await window.getLastWindow(context);
      const read: () => void = (): void => { onChange(WindowSystemBarService.insetsOf(appWindow)); };
      const onAvoidAreaChange: (data: window.AvoidAreaOptions) => void = (_data: window.AvoidAreaOptions): void => {
        read();
      };
      const subscription: WindowInsetSubscription = (): void => {
        appWindow.off('avoidAreaChange', onAvoidAreaChange);
      };
      appWindow.on('avoidAreaChange', onAvoidAreaChange);
      read();
      return subscription;
    } catch (_err) {
      onChange({ top: 0, bottom: 0 });
      return (): void => {};
    }
  }

  private static insetsOf(appWindow: window.Window): WindowInsets {
    const top = appWindow.getWindowAvoidArea(window.AvoidAreaType.TYPE_SYSTEM).topRect.height;
    // The keyboard covers the navigation indicator while it is up, and
    // `KeyboardAvoidMode.RESIZE` has already lifted the page above it, so the
    // bottom strip belongs to the keyboard for as long as it is visible.
    const keyboard = appWindow.getWindowAvoidArea(window.AvoidAreaType.TYPE_KEYBOARD).bottomRect.height;
    if (keyboard > 0) {
      return { top, bottom: 0 };
    }
    const navigation = appWindow.getWindowAvoidArea(window.AvoidAreaType.TYPE_NAVIGATION_INDICATOR).bottomRect.height;
    if (navigation > 0) {
      return { top, bottom: navigation };
    }
    // Some builds fold the navigation bar into the system avoid area instead of
    // reporting an indicator of its own.
    return { top, bottom: appWindow.getWindowAvoidArea(window.AvoidAreaType.TYPE_SYSTEM).bottomRect.height };
  }
}

/**
 * One component's live view of the window insets, in vp.
 *
 * The shell is immersive, so every surface that puts chrome against a screen
 * edge reads these numbers instead of assuming the framework reserved the strip
 * for it: the chat page's fade fills the bottom edge, and the interactive rows
 * of every other page — drawers, sheets, covers, galleries — keep clear of it.
 * They mount and unmount independently, so the binding owns the subscription
 * rather than leaving each page to juggle one, and it holds vp because padding,
 * margins and radii in this app are vp.
 */
@ObservedV2
export class WindowInsetsBinding {
  @Trace top: number = 0;
  @Trace bottom: number = 0;
  private subscription?: WindowInsetSubscription;
  private disposed: boolean = false;

  bind(uiContext: UIContext, context: Context): void {
    this.disposed = false;
    void WindowSystemBarService.observeInsets(context, (insets: WindowInsets): void => {
      if (this.disposed) {
        return;
      }
      this.top = uiContext.px2vp(insets.top);
      this.bottom = uiContext.px2vp(insets.bottom);
      RemoteLogger.info(`window insets top=${this.top} bottom=${this.bottom}`);
    }).then((subscription: WindowInsetSubscription): void => {
      // The page can disappear before the window call resolves; releasing here
      // keeps a listener from outliving the component that asked for it.
      if (this.disposed) {
        subscription();
        return;
      }
      this.subscription = subscription;
    });
  }

  unbind(): void {
    this.disposed = true;
    if (this.subscription) {
      this.subscription();
      this.subscription = undefined;
    }
  }

  /**
   * The padding a bottom-anchored content layer needs: the design's own spacing
   * where it already clears the strip, and the strip itself where it does not.
   * The strip is a device fact, so a page never hard-codes it.
   */
  bottomPadding(designSpacing: number): number {
    return Math.max(designSpacing, this.bottom);
  }

  /**
   * The tail spacer a scrolling surface appends after its last row: the strip
   * itself plus the design's own breathing. The viewport then runs to the
   * screen edge — rows roll under the navigation indicator while they are
   * still moving — while the last row still comes to rest above the bar.
   * Padding on the scrolling container cannot do that: it shrinks the viewport
   * and leaves a band of bare page colour under the bar instead.
   */
  tailSpacing(designSpacing: number): number {
    return this.bottom + designSpacing;
  }
}
