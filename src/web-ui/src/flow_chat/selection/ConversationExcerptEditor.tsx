import type { RefObject } from 'react';
import { OverflowText, Textarea } from '@openbitfun/ui';
import { useI18n } from '@/infrastructure/i18n';
import type { ConversationExcerptContext } from '@/shared/types/context';
import { excerptText } from '@/shared/utils/conversationExcerpt';

export function ConversationExcerptQuote({ excerpt }: { excerpt: ConversationExcerptContext }) {
  const quote = excerptText(excerpt).replace(/\s+/g, ' ').trim();
  return <div className="conversation-excerpt__quote"
    data-openbitfun-product-component="conversation-excerpt" data-openbitfun-product-part="quote">
    <OverflowText as="span" behavior="fade" overflowStyle="ellipsis" className="conversation-excerpt__quote-text"
      data-openbitfun-product-component="conversation-excerpt" data-openbitfun-product-part="quoteText">{quote}</OverflowText>
  </div>;
}

/** Shared compact content for creating and editing an annotation. */
export function ConversationExcerptEditor({ excerpt, comment, onCommentChange, inputRef, onSubmit }: {
  excerpt: ConversationExcerptContext;
  comment: string;
  onCommentChange: (comment: string) => void;
  inputRef: RefObject<HTMLTextAreaElement>;
  onSubmit: () => void;
}) {
  const { t } = useI18n('flow-chat');
  return <>
    <ConversationExcerptQuote excerpt={excerpt} />
    <Textarea ref={inputRef} rows={3} resize="none" aria-label={t('selection.annotation')}
      value={comment} onValueChange={onCommentChange} onKeyDown={event => {
        if ((event.ctrlKey || event.metaKey) && event.key === 'Enter' && !event.nativeEvent.isComposing) {
          event.preventDefault(); onSubmit();
        }
      }} />
  </>;
}
