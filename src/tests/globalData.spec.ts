import { describe, expect, it } from 'vitest';

import {
  normalizeImportedSkillFolders,
  reconcileImportedSkillFolders,
  sameImportedSkillFolders
} from '@/features/global/globalData';

describe('reconcileImportedSkillFolders', () => {
  it('adds folders found on disk but missing from the registry', () => {
    const reconciled = reconcileImportedSkillFolders(
      [],
      [{ name: 'token-compact', path: 'C:\\skills\\token-compact' }],
      123
    );

    expect(reconciled).toEqual([
      {
        id: 'C:\\skills\\token-compact',
        name: 'token-compact',
        path: 'C:\\skills\\token-compact',
        addedAt: 123
      }
    ]);
  });

  it('preserves metadata by name while updating stale paths', () => {
    const reconciled = reconcileImportedSkillFolders(
      [
        {
          id: '\\\\?\\E:\\old\\token-compact',
          name: 'token-compact',
          path: '\\\\?\\E:\\old\\token-compact',
          addedAt: 456
        }
      ],
      [
        {
          name: 'token-compact',
          path: 'C:\\Users\\will\\AppData\\Roaming\\com.golutra\\skills\\token-compact'
        }
      ],
      999
    );

    expect(reconciled).toEqual([
      {
        id: 'C:\\Users\\will\\AppData\\Roaming\\com.golutra\\skills\\token-compact',
        name: 'token-compact',
        path: 'C:\\Users\\will\\AppData\\Roaming\\com.golutra\\skills\\token-compact',
        addedAt: 456
      }
    ]);
  });

  it('drops registry entries that no longer exist on disk', () => {
    const reconciled = reconcileImportedSkillFolders(
      [
        {
          id: 'C:\\skills\\old-skill',
          name: 'old-skill',
          path: 'C:\\skills\\old-skill',
          addedAt: 1
        }
      ],
      [],
      999
    );

    expect(reconciled).toEqual([]);
  });
});

describe('global data helpers', () => {
  it('normalizes imported folders and keeps equality strict', () => {
    const normalized = normalizeImportedSkillFolders([
      {
        name: 'token-compact',
        path: 'C:\\skills\\token-compact',
        addedAt: 1
      },
      {
        name: 'token-compact',
        path: 'C:\\skills\\token-compact',
        addedAt: 2
      }
    ]);

    expect(normalized).toEqual([
      {
        id: 'C:\\skills\\token-compact',
        name: 'token-compact',
        path: 'C:\\skills\\token-compact',
        addedAt: 1
      }
    ]);
    expect(sameImportedSkillFolders(normalized, normalized)).toBe(true);
    expect(sameImportedSkillFolders(normalized, [{ ...normalized[0], addedAt: 2 }])).toBe(false);
  });
});
