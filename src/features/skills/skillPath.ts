export const formatSkillPath = (rawPath: string): string => {
  if (!rawPath) {
    return '';
  }
  const trimmed = rawPath.trim();
  if (!trimmed) {
    return '';
  }
  if (trimmed.startsWith('\\\\?\\UNC\\')) {
    return `\\\\${trimmed.slice('\\\\?\\UNC\\'.length)}`;
  }
  if (trimmed.startsWith('\\\\?\\')) {
    return trimmed.slice('\\\\?\\'.length);
  }
  if (trimmed.startsWith('//?/UNC/')) {
    return `//${trimmed.slice('//?/UNC/'.length)}`;
  }
  if (trimmed.startsWith('//?/')) {
    return trimmed.slice('//?/'.length);
  }
  return trimmed;
};
