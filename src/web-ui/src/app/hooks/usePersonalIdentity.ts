import { useEffect, useState } from 'react';
import {
  aiExperienceConfigService,
  type AIExperienceSettings,
  type AgentCompanionPetSelection,
} from '@/infrastructure/config/services/AIExperienceConfigService';
import { DEFAULT_AGENT_COMPANION_PET } from '@/infrastructure/config/services/AgentCompanionPetService';

export const DEFAULT_PERSONAL_IDENTITY_NAME = 'OpenBitFun';

export interface PersonalIdentity {
  /** Owner title for the identity badge; falls back to the product brand. */
  displayName: string;
  /** Companion pet selected in Settings > Companion pet (built-in pet fallback). */
  pet: AgentCompanionPetSelection;
}

/** Reactive personal-identity facts from the AI experience settings. */
export function usePersonalIdentity(): PersonalIdentity {
  const [settings, setSettings] = useState<AIExperienceSettings>(
    () => aiExperienceConfigService.getSettings(),
  );
  useEffect(() => {
    let disposed = false;
    const unsubscribe = aiExperienceConfigService.addChangeListener((next) => {
      if (!disposed) setSettings(next);
    });
    void aiExperienceConfigService.getSettingsAsync().then((next) => {
      if (!disposed) setSettings(next);
    });
    return () => {
      disposed = true;
      unsubscribe();
    };
  }, []);
  return {
    displayName: settings.personal_title?.trim() || DEFAULT_PERSONAL_IDENTITY_NAME,
    pet: settings.agent_companion_pet ?? DEFAULT_AGENT_COMPANION_PET,
  };
}
