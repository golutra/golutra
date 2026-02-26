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

const librarySkillSeed: SkillLibraryItem[] = [];

export const createLibrarySkills = (): SkillLibraryItem[] =>
  librarySkillSeed.map((skill) => ({ ...skill }));
