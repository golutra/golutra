export type SkillLibraryItem = {
  id: number;
  nameKey: string;
  descKey: string;
  icon: string;
  color: string;
  bg: string;
  ring: string;
  gradient: string;
  ver: string;
  assetsKey: string;
  checked: boolean;
};

const librarySkillSeed: SkillLibraryItem[] = [
  {
    id: 1,
    nameKey: 'skills.items.frontendToolkit.name',
    descKey: 'skills.items.frontendToolkit.desc',
    icon: 'code',
    color: 'text-blue-400',
    bg: 'bg-blue-500/10',
    ring: 'ring-blue-500/20',
    gradient: 'from-blue-500/10',
    ver: 'v2.4.0',
    assetsKey: 'skills.assets.frontendToolkit',
    checked: true
  },
  {
    id: 2,
    nameKey: 'skills.items.iconAssetPack.name',
    descKey: 'skills.items.iconAssetPack.desc',
    icon: 'category',
    color: 'text-purple-400',
    bg: 'bg-purple-500/10',
    ring: 'ring-purple-500/20',
    gradient: 'from-purple-500/10',
    ver: 'v1.1.0',
    assetsKey: 'skills.assets.iconAssetPack',
    checked: false
  },
  {
    id: 3,
    nameKey: 'skills.items.motionPresets.name',
    descKey: 'skills.items.motionPresets.desc',
    icon: 'motion_photos_on',
    color: 'text-orange-400',
    bg: 'bg-orange-500/10',
    ring: 'ring-orange-500/20',
    gradient: 'from-orange-500/10',
    ver: 'v3.0.0',
    assetsKey: 'skills.assets.motionPresets',
    checked: true
  },
  {
    id: 4,
    nameKey: 'skills.items.shellCommands.name',
    descKey: 'skills.items.shellCommands.desc',
    icon: 'terminal',
    color: 'text-emerald-400',
    bg: 'bg-emerald-500/10',
    ring: 'ring-emerald-500/20',
    gradient: 'from-emerald-500/10',
    ver: 'v0.9.1',
    assetsKey: 'skills.assets.shellCommands',
    checked: false
  },
  {
    id: 5,
    nameKey: 'skills.items.brandColors.name',
    descKey: 'skills.items.brandColors.desc',
    icon: 'palette',
    color: 'text-pink-400',
    bg: 'bg-pink-500/10',
    ring: 'ring-pink-500/20',
    gradient: 'from-pink-500/10',
    ver: 'v4.2',
    assetsKey: 'skills.assets.brandColors',
    checked: false
  }
];

export const createLibrarySkills = (): SkillLibraryItem[] =>
  librarySkillSeed.map((skill) => ({ ...skill }));
