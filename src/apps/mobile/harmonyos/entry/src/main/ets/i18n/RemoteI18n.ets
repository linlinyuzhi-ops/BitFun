import { AppLocale, APP_LOCALE_EN_US, APP_LOCALE_ZH_CN } from './AppLocale';
import { HostTextRetranslator } from './HostTextRetranslator';
import { intl } from '@kit.LocalizationKit';

@ObservedV2
export class AppLocaleState {
  @Trace language: string = AppLocale.DEFAULT;
  @Trace revision: number = 0;
}

export class RemoteI18n {
  static readonly state: AppLocaleState = new AppLocaleState();
  private static retranslator: HostTextRetranslator | null = null;
  private static retranslatorLocale: string = '';

  static language(): string {
    return RemoteI18n.state.language;
  }

  static revision(): number {
    return RemoteI18n.state.revision;
  }

  static setLanguage(language: string): string {
    const resolved = AppLocale.resolve(language) || AppLocale.DEFAULT;
    if (RemoteI18n.state.language !== resolved) {
      RemoteI18n.state.language = resolved;
      RemoteI18n.state.revision += 1;
      RemoteI18n.retranslator = null;
      RemoteI18n.retranslatorLocale = '';
    }
    return resolved;
  }

  static t(key: string): string {
    try {
      const resourceManager = getContext(RemoteI18n).resourceManager;
      // HarmonyOS resource names only allow [a-zA-Z0-9_]; map dotted code keys
      // (e.g. "workspaceTools.browse") to underscore resource names.
      const resourceName = key.replace(/\./g, '_');
      return resourceManager.getStringByNameSync(resourceName);
    } catch (error) {
      return key;
    }
  }

  static f(key: string, value: string): string {
    return RemoteI18n.t(key).replace('%1$s', value);
  }

  static f2(key: string, first: string, second: string): string {
    return RemoteI18n.t(key).replace('%1$s', first).replace('%2$s', second);
  }

  static translate(language: string, key: string): string {
    return RemoteI18n.t(key);
  }

  static textEquals(key: string, message: string): boolean {
    const supported = AppLocale.supported();
    for (let index = 0; index < supported.length; index += 1) {
      if (RemoteI18n.t(key) === message) {
        return true;
      }
    }
    return false;
  }

  static textContains(key: string, message: string): boolean {
    const supported = AppLocale.supported();
    for (let index = 0; index < supported.length; index += 1) {
      const value = RemoteI18n.t(key);
      if (value.length > 0 && message.indexOf(value) >= 0) {
        return true;
      }
    }
    return false;
  }

  static textHasTemplatePrefix(key: string, message: string): boolean {
    const supported = AppLocale.supported();
    for (let index = 0; index < supported.length; index += 1) {
      const prefix = RemoteI18n.t(key).split('%1$s')[0];
      if (prefix.length > 0 && message.indexOf(prefix) >= 0) {
        return true;
      }
    }
    return false;
  }

  static keys(language: string = AppLocale.DEFAULT): string[] {
    // Return empty array - keys are now managed by the system resource manager
    return [];
  }

  static formatAbsoluteDate(timestamp: number): string {
    const locale = RemoteI18n.language() === APP_LOCALE_EN_US ? 'en-US' : 'zh-CN';
    const formatter = new intl.DateTimeFormat(locale, {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit'
    });
    return formatter.format(new Date(timestamp));
  }

  static speechLanguage(): string {
    return AppLocale.speechLanguage(RemoteI18n.language());
  }

  static speechLocate(): string {
    return AppLocale.speechLocate(RemoteI18n.language());
  }

  static nativeName(language: string = RemoteI18n.language()): string {
    return RemoteI18n.t(AppLocale.nativeNameKey(language));
  }

  static retranslateKnown(message: string, keys: string[]): string {
    for (let index = 0; index < keys.length; index += 1) {
      if (RemoteI18n.textEquals(keys[index], message)) {
        return RemoteI18n.t(keys[index]);
      }
    }
    return RemoteI18n.retranslate(message);
  }

  static retranslate(message: string): string {
    if (message.length === 0) {
      return message;
    }
    const locale = RemoteI18n.language();
    if (!RemoteI18n.retranslator || RemoteI18n.retranslatorLocale !== locale) {
      RemoteI18n.retranslator = new HostTextRetranslator(locale);
      RemoteI18n.retranslatorLocale = locale;
    }
    return RemoteI18n.retranslator!.retranslate(message);
  }
}
