import type { AppearanceSurfaceDescriptor } from '@/infrastructure/appearance/types';
export const welcomeAppearanceDescriptor: AppearanceSurfaceDescriptor = {
  id: 'welcome', parts: [
    { id: 'root', propertyProfile: 'layout', visualRole: 'workspace' },
    { id: 'content', visualRole: 'card' },
    { id: 'greeting', visualRole: 'content' },
    { id: 'identity', visualRole: 'toolbar' },
    { id: 'identityAvatar', propertyProfile: 'paint', visualRole: 'decoration' },
    { id: 'identityLabel', propertyProfile: 'paint', visualRole: 'content' },
    { id: 'title', propertyProfile: 'paint', visualRole: 'content' },
    { id: 'subtitle', propertyProfile: 'paint', visualRole: 'content' },
  ],
};
