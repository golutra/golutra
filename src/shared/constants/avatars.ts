export type AvatarPreset = {
  id: string;
  labelKey: string;
  vars: Record<string, string>;
};

export const CSS_AVATAR_PREFIX = 'css:';

export const AVATAR_PRESETS: AvatarPreset[] = [
  {
    id: 'orbit',
    labelKey: 'settings.avatarOptions.orbit',
    vars: {
      '--avatar-bg': 'linear-gradient(135deg, #0b1220 0%, #1f2937 100%)',
      '--avatar-spot': 'radial-gradient(circle at 30% 30%, rgba(56, 189, 248, 0.95), rgba(56, 189, 248, 0))',
      '--avatar-spot-2': 'radial-gradient(circle at 70% 75%, rgba(14, 165, 233, 0.85), rgba(14, 165, 233, 0))',
      '--avatar-ring': 'rgba(125, 211, 252, 0.7)',
      '--avatar-glow': 'rgba(56, 189, 248, 0.35)',
      '--avatar-spot-size': '78%',
      '--avatar-spot-2-size': '48%',
      '--avatar-spot-x': '-18%',
      '--avatar-spot-y': '-12%',
      '--avatar-spot-2-x': '20%',
      '--avatar-spot-2-y': '18%',
      '--avatar-spot-rotate': '12deg',
      '--avatar-spot-2-rotate': '-6deg'
    }
  },
  {
    id: 'ember',
    labelKey: 'settings.avatarOptions.ember',
    vars: {
      '--avatar-bg': 'linear-gradient(135deg, #1f1308 0%, #3a2011 100%)',
      '--avatar-spot': 'radial-gradient(circle at 25% 30%, rgba(251, 146, 60, 0.95), rgba(251, 146, 60, 0))',
      '--avatar-spot-2': 'radial-gradient(circle at 70% 70%, rgba(244, 63, 94, 0.8), rgba(244, 63, 94, 0))',
      '--avatar-ring': 'rgba(253, 186, 116, 0.7)',
      '--avatar-glow': 'rgba(251, 146, 60, 0.35)',
      '--avatar-spot-size': '74%',
      '--avatar-spot-2-size': '50%',
      '--avatar-spot-x': '-20%',
      '--avatar-spot-y': '-14%',
      '--avatar-spot-2-x': '18%',
      '--avatar-spot-2-y': '20%',
      '--avatar-spot-rotate': '-8deg',
      '--avatar-spot-2-rotate': '14deg'
    }
  },
  {
    id: 'mint',
    labelKey: 'settings.avatarOptions.mint',
    vars: {
      '--avatar-bg': 'linear-gradient(135deg, #0c1f1c 0%, #123b32 100%)',
      '--avatar-spot': 'radial-gradient(circle at 32% 24%, rgba(52, 211, 153, 0.95), rgba(52, 211, 153, 0))',
      '--avatar-spot-2': 'radial-gradient(circle at 70% 78%, rgba(16, 185, 129, 0.85), rgba(16, 185, 129, 0))',
      '--avatar-ring': 'rgba(110, 231, 183, 0.7)',
      '--avatar-glow': 'rgba(52, 211, 153, 0.35)',
      '--avatar-spot-size': '76%',
      '--avatar-spot-2-size': '46%',
      '--avatar-spot-x': '-14%',
      '--avatar-spot-y': '-16%',
      '--avatar-spot-2-x': '22%',
      '--avatar-spot-2-y': '18%',
      '--avatar-spot-rotate': '6deg',
      '--avatar-spot-2-rotate': '-12deg'
    }
  },
  {
    id: 'canyon',
    labelKey: 'settings.avatarOptions.canyon',
    vars: {
      '--avatar-bg': 'linear-gradient(135deg, #1f140b 0%, #3b2414 100%)',
      '--avatar-spot': 'radial-gradient(circle at 24% 28%, rgba(251, 191, 36, 0.9), rgba(251, 191, 36, 0))',
      '--avatar-spot-2': 'radial-gradient(circle at 68% 72%, rgba(217, 119, 6, 0.85), rgba(217, 119, 6, 0))',
      '--avatar-ring': 'rgba(253, 230, 138, 0.6)',
      '--avatar-glow': 'rgba(251, 191, 36, 0.3)',
      '--avatar-spot-size': '72%',
      '--avatar-spot-2-size': '44%',
      '--avatar-spot-x': '-18%',
      '--avatar-spot-y': '-12%',
      '--avatar-spot-2-x': '20%',
      '--avatar-spot-2-y': '22%',
      '--avatar-spot-rotate': '18deg',
      '--avatar-spot-2-rotate': '-6deg'
    }
  },
  {
    id: 'storm',
    labelKey: 'settings.avatarOptions.storm',
    vars: {
      '--avatar-bg': 'linear-gradient(135deg, #10161f 0%, #1f2937 100%)',
      '--avatar-spot': 'radial-gradient(circle at 28% 35%, rgba(148, 163, 184, 0.9), rgba(148, 163, 184, 0))',
      '--avatar-spot-2': 'radial-gradient(circle at 72% 68%, rgba(100, 116, 139, 0.85), rgba(100, 116, 139, 0))',
      '--avatar-ring': 'rgba(226, 232, 240, 0.5)',
      '--avatar-glow': 'rgba(148, 163, 184, 0.3)',
      '--avatar-spot-size': '70%',
      '--avatar-spot-2-size': '46%',
      '--avatar-spot-x': '-16%',
      '--avatar-spot-y': '-10%',
      '--avatar-spot-2-x': '18%',
      '--avatar-spot-2-y': '20%',
      '--avatar-spot-rotate': '-10deg',
      '--avatar-spot-2-rotate': '10deg'
    }
  }
];

export const DEFAULT_AVATAR_ID = AVATAR_PRESETS[0]?.id ?? 'orbit';
export const DEFAULT_AVATAR = `${CSS_AVATAR_PREFIX}${DEFAULT_AVATAR_ID}`;
