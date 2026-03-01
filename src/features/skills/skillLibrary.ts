// 技能库种子数据：用于已安装列表与展示的基础配置。
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

// [TODO/SkillLibrary, 2026-01-23] 补充技能库种子数据或接入远端资源。
const librarySkillSeed: SkillLibraryItem[] = [];

/**
 * 创建技能库数据副本，避免直接修改种子数据。
 * 输入：无。
 * 输出：技能库条目数组。
 */
export const createLibrarySkills = (): SkillLibraryItem[] =>
  librarySkillSeed.map((skill) => ({ ...skill }));
