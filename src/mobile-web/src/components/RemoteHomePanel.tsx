import { Menu as LucideMenu } from 'lucide-react';
import React from 'react';
import { MobileIconButton } from '@openbitfun/ui/mobile';
import { useI18n } from '../i18n';

interface RemoteHomePanelProps {
  onOpenSidebar?: () => void;
}

const RemoteHomePanel: React.FC<RemoteHomePanelProps> = ({
  onOpenSidebar,
}) => {
  const { t } = useI18n();
  const openNavigation = onOpenSidebar;

  return (
    <main className="remote-home" aria-labelledby="remote-home-title">
      <header className="remote-home__header">
        {openNavigation ? (
          <MobileIconButton
            appearance="floating"
            className="remote-home__menu"
            icon={(
              <LucideMenu aria-hidden="true" />
            )}
            onClick={openNavigation}
            aria-label={t('sessions.sessionHistory')}
          />
        ) : (
          <span className="remote-home__menu remote-home__menu--placeholder" aria-hidden="true" />
        )}
        <div className="remote-home__heading">
          <h2 id="remote-home-title">OpenBitFun</h2>
        </div>
        <span className="remote-home__header-spacer" aria-hidden="true" />
      </header>
      <div className="remote-home__content">
        <p className="remote-home__message">{t('shell.selectConversation')}</p>
      </div>
    </main>
  );
};

export default RemoteHomePanel;
