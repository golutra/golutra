export const TIME_ZONE_OPTIONS = [
  { id: 'utc', labelKey: 'settings.timeZones.utc', legacyId: 'UTC' },
  { id: 'pacificMidway', labelKey: 'settings.timeZones.pacificMidway', legacyId: 'Pacific/Midway' },
  { id: 'pacificHonolulu', labelKey: 'settings.timeZones.pacificHonolulu', legacyId: 'Pacific/Honolulu' },
  { id: 'americaAnchorage', labelKey: 'settings.timeZones.americaAnchorage', legacyId: 'America/Anchorage' },
  { id: 'americaLosAngeles', labelKey: 'settings.timeZones.americaLosAngeles', legacyId: 'America/Los_Angeles' },
  { id: 'americaDenver', labelKey: 'settings.timeZones.americaDenver', legacyId: 'America/Denver' },
  { id: 'americaChicago', labelKey: 'settings.timeZones.americaChicago', legacyId: 'America/Chicago' },
  { id: 'americaNewYork', labelKey: 'settings.timeZones.americaNewYork', legacyId: 'America/New_York' },
  { id: 'americaHalifax', labelKey: 'settings.timeZones.americaHalifax', legacyId: 'America/Halifax' },
  { id: 'americaSaoPaulo', labelKey: 'settings.timeZones.americaSaoPaulo', legacyId: 'America/Sao_Paulo' },
  { id: 'atlanticAzores', labelKey: 'settings.timeZones.atlanticAzores', legacyId: 'Atlantic/Azores' },
  { id: 'europeLondon', labelKey: 'settings.timeZones.europeLondon', legacyId: 'Europe/London' },
  { id: 'europeParis', labelKey: 'settings.timeZones.europeParis', legacyId: 'Europe/Paris' },
  { id: 'europeHelsinki', labelKey: 'settings.timeZones.europeHelsinki', legacyId: 'Europe/Helsinki' },
  { id: 'europeMoscow', labelKey: 'settings.timeZones.europeMoscow', legacyId: 'Europe/Moscow' },
  { id: 'asiaDubai', labelKey: 'settings.timeZones.asiaDubai', legacyId: 'Asia/Dubai' },
  { id: 'asiaKarachi', labelKey: 'settings.timeZones.asiaKarachi', legacyId: 'Asia/Karachi' },
  { id: 'asiaDhaka', labelKey: 'settings.timeZones.asiaDhaka', legacyId: 'Asia/Dhaka' },
  { id: 'asiaBangkok', labelKey: 'settings.timeZones.asiaBangkok', legacyId: 'Asia/Bangkok' },
  { id: 'asiaShanghai', labelKey: 'settings.timeZones.asiaShanghai', legacyId: 'Asia/Shanghai' },
  { id: 'asiaTokyo', labelKey: 'settings.timeZones.asiaTokyo', legacyId: 'Asia/Tokyo' },
  { id: 'australiaSydney', labelKey: 'settings.timeZones.australiaSydney', legacyId: 'Australia/Sydney' },
  { id: 'pacificNoumea', labelKey: 'settings.timeZones.pacificNoumea', legacyId: 'Pacific/Noumea' },
  { id: 'pacificAuckland', labelKey: 'settings.timeZones.pacificAuckland', legacyId: 'Pacific/Auckland' }
] as const;

export type TimeZoneOption = (typeof TIME_ZONE_OPTIONS)[number];
export type TimeZoneId = TimeZoneOption['id'];

export const TIME_ZONE_IDS = new Set<TimeZoneId>(TIME_ZONE_OPTIONS.map((option) => option.id));

export const LEGACY_TIME_ZONE_ALIASES: Record<string, TimeZoneId> = TIME_ZONE_OPTIONS.reduce(
  (acc, option) => {
    acc[option.legacyId] = option.id;
    return acc;
  },
  {} as Record<string, TimeZoneId>
);
