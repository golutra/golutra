// 时区选项常量：用于设置 UI。
export const TIME_ZONE_OPTIONS = [
  { id: 'utc', labelKey: 'settings.timeZones.utc' },
  { id: 'pacificMidway', labelKey: 'settings.timeZones.pacificMidway' },
  { id: 'pacificHonolulu', labelKey: 'settings.timeZones.pacificHonolulu' },
  { id: 'americaAnchorage', labelKey: 'settings.timeZones.americaAnchorage' },
  { id: 'americaLosAngeles', labelKey: 'settings.timeZones.americaLosAngeles' },
  { id: 'americaDenver', labelKey: 'settings.timeZones.americaDenver' },
  { id: 'americaChicago', labelKey: 'settings.timeZones.americaChicago' },
  { id: 'americaNewYork', labelKey: 'settings.timeZones.americaNewYork' },
  { id: 'americaHalifax', labelKey: 'settings.timeZones.americaHalifax' },
  { id: 'americaSaoPaulo', labelKey: 'settings.timeZones.americaSaoPaulo' },
  { id: 'atlanticAzores', labelKey: 'settings.timeZones.atlanticAzores' },
  { id: 'europeLondon', labelKey: 'settings.timeZones.europeLondon' },
  { id: 'europeParis', labelKey: 'settings.timeZones.europeParis' },
  { id: 'europeHelsinki', labelKey: 'settings.timeZones.europeHelsinki' },
  { id: 'europeMoscow', labelKey: 'settings.timeZones.europeMoscow' },
  { id: 'asiaDubai', labelKey: 'settings.timeZones.asiaDubai' },
  { id: 'asiaKarachi', labelKey: 'settings.timeZones.asiaKarachi' },
  { id: 'asiaDhaka', labelKey: 'settings.timeZones.asiaDhaka' },
  { id: 'asiaBangkok', labelKey: 'settings.timeZones.asiaBangkok' },
  { id: 'asiaShanghai', labelKey: 'settings.timeZones.asiaShanghai' },
  { id: 'asiaTokyo', labelKey: 'settings.timeZones.asiaTokyo' },
  { id: 'australiaSydney', labelKey: 'settings.timeZones.australiaSydney' },
  { id: 'pacificNoumea', labelKey: 'settings.timeZones.pacificNoumea' },
  { id: 'pacificAuckland', labelKey: 'settings.timeZones.pacificAuckland' }
] as const;

export type TimeZoneOption = (typeof TIME_ZONE_OPTIONS)[number];
export type TimeZoneId = TimeZoneOption['id'];

// 快速校验用集合，避免重复遍历列表。
export const TIME_ZONE_IDS = new Set<TimeZoneId>(TIME_ZONE_OPTIONS.map((option) => option.id));
