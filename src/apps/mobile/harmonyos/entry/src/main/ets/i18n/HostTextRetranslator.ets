import { resourceManager } from '@kit.LocalizationKit';

interface TemplateEntry {
  key: string;
  template: string;
}

interface StringEntry {
  name: string;
  value: string;
}

interface StringCatalog {
  string: StringEntry[];
}

export class HostTextRetranslator {
  private forward: Map<string, string> = new Map();
  private reverse: Map<string, string> = new Map();
  private templates: TemplateEntry[] = [];

  constructor(locale: string) {
    this.loadCatalog(locale);
  }

  private loadCatalog(locale: string): void {
    try {
      const resourceManager = getContext(this).resourceManager;
      const rawResource = resourceManager.getRawFileContentSync('resources/rawfile/string.json');
      let jsonString = '';
      for (let i = 0; i < rawResource.length; i++) {
        jsonString += String.fromCharCode(rawResource[i]);
      }
      const catalog = JSON.parse(jsonString) as StringCatalog;
      const strings = catalog.string || [];
      
      for (let index = 0; index < strings.length; index += 1) {
        const entry = strings[index];
        const key = entry.name;
        const value = entry.value;
        this.forward.set(key, value);
        this.reverse.set(value, key);
        if (value.indexOf('%1$s') >= 0 || value.indexOf('%2$s') >= 0) {
          this.templates.push({ key, template: value });
        }
      }
    } catch (error) {
      // Fallback: catalog unavailable, retranslate returns original message
    }
  }

  retranslate(message: string): string {
    if (message.length === 0) {
      return message;
    }

    // 1. Exact match O(1)
    const exactKey = this.reverse.get(message);
    if (exactKey) {
      return this.forward.get(exactKey) || message;
    }

    // 2. Template match O(M)
    for (let index = 0; index < this.templates.length; index += 1) {
      const entry = this.templates[index];
      const matched = this.matchTemplate(entry.template, message);
      if (matched !== null) {
        return this.fillTemplate(entry.key, matched);
      }
    }

    return message;
  }

  private matchTemplate(template: string, message: string): string | null {
    // Parse %1$s and %2$s placeholders
    const firstSlot = template.indexOf('%1$s');
    if (firstSlot < 0) {
      return null;
    }

    const prefix = template.slice(0, firstSlot);
    const afterFirst = template.slice(firstSlot + 4);
    const secondSlot = afterFirst.indexOf('%2$s');

    if (secondSlot < 0) {
      // Single placeholder: %1$s
      const suffix = afterFirst;
      if (!message.startsWith(prefix) || !message.endsWith(suffix)) {
        return null;
      }
      const value = message.slice(prefix.length, message.length - suffix.length);
      return value;
    }

    // Two placeholders: %1$s and %2$s
    const middle = afterFirst.slice(0, secondSlot);
    const suffix = afterFirst.slice(secondSlot + 4);
    if (!message.startsWith(prefix) || !message.endsWith(suffix)) {
      return null;
    }
    const body = message.slice(prefix.length, message.length - suffix.length);
    const middleAt = body.indexOf(middle);
    if (middleAt < 0) {
      return null;
    }
    const first = body.slice(0, middleAt);
    const second = body.slice(middleAt + middle.length);
    return `${first}${second}`;
  }

  private fillTemplate(key: string, matched: string): string {
    const template = this.forward.get(key) || '';
    // Replace %1$s and %2$s with the matched value
    // For single placeholder, matched is the value
    // For two placeholders, matched is concatenated value
    return template.replace('%1$s', matched).replace('%2$s', matched);
  }
}
