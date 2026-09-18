import type { MouseEvent, ReactNode } from 'react';
import { Icon } from '@openbitfun/ui';
import './ChatInputAttachment.scss';

/** Images and annotations share the same composer attachment shell. */
export function ChatInputAttachment({ children, label, removeLabel, onRemove, kind = 'image' }: {
  children: ReactNode;
  label: string;
  removeLabel: string;
  onRemove: () => void;
  kind?: 'image' | 'annotation';
}) {
  const remove = (event: MouseEvent<HTMLButtonElement>) => { event.stopPropagation(); onRemove(); };
  // Keep the existing image Appearance parts while sharing layout and behavior.
  if (kind === 'image') return <div className="openbitfun-chat-input__attachment" title={label}
    data-openbitfun-component="chat-input" data-openbitfun-part="image">
    {children}
    <button type="button" className="openbitfun-chat-input__attachment-remove"
      data-openbitfun-component="chat-input" data-openbitfun-part="imageRemove" aria-label={removeLabel} onClick={remove}>
      <Icon name="xmark" size="xs" />
    </button>
  </div>;
  return <div className="openbitfun-chat-input__attachment" title={label}
    data-openbitfun-component="chat-input" data-openbitfun-part="attachment">
    {children}
    <button type="button" className="openbitfun-chat-input__attachment-remove"
      data-openbitfun-component="chat-input" data-openbitfun-part="attachmentRemove" aria-label={removeLabel} onClick={remove}>
      <Icon name="xmark" size="xs" />
    </button>
  </div>;
}
