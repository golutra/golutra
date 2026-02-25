import type { Member, Message } from './types';
import { PRIMARY_USER_AVATAR_URL } from '../../shared/constants/avatars';
import { formatMessageTime } from './utils';

export const CURRENT_USER_ID = 'me';
export const AI_ASSISTANT_ID = 'ai-assistant';
export const DEFAULT_CHANNEL_ID = 'design-critique';
export const DEFAULT_CHANNEL_NAME_KEY = 'chat.sidebar.channelList.designCritique';

const seedBaseTime = Date.now() - 1000 * 60 * 45;

const getMemberAvatar = (id: string) => initialMembers.find((member) => member.id === id)?.avatar ?? PRIMARY_USER_AVATAR_URL;
const SARAH_AVATAR_URL = 'https://ui-avatars.com/api/?name=Sarah+Jenkins&background=f97316&color=fff';

export const initialMembers: Member[] = [
  {
    id: 'me',
    name: 'You (Owner)',
    role: '',
    roleKey: 'members.activity.reviewingPRs',
    roleType: 'owner',
    avatar: PRIMARY_USER_AVATAR_URL,
    status: 'online'
  },
  {
    id: AI_ASSISTANT_ID,
    name: 'AI Assistant',
    role: '',
    roleKey: 'members.roles.aiAssistant',
    roleType: 'assistant',
    avatar: 'https://ui-avatars.com/api/?name=AI+Assistant&background=0b57d0&color=fff',
    status: 'online'
  },
  {
    id: 'alex',
    name: 'Alex Designer',
    role: '',
    roleKey: 'members.activity.listeningSpotify',
    roleType: 'admin',
    avatar: 'https://lh3.googleusercontent.com/aida-public/AB6AXuBaSWKB6mJCxTBrtvopWrUkwsow24JW16yM25iBaJ-i9zei2LLH2S2RuQAjClLKuBNbaKzFR9KdlrlnzjOONBvHumzYm6gwPCtacfn8Q1j2EM62pNme3MdwszqcLe-WnRiGFfDWnjvZna6DoiK0mpMNiTQyR5BfbSkirNIuJkIsXfO-uNqs0bwysJtFjeHP22BouFsb7HsObMaenKoOCxLgfgmYhMwTA_0ycwngEFMPuRnSXwfrAJThiBYH1y7nP7_pdwvn5SLJEVQ',
    status: 'online'
  },
  {
    id: 'marcus',
    name: 'Marcus Chen',
    role: '',
    roleKey: 'members.activity.fixingBugs',
    roleType: 'admin',
    avatar: 'https://lh3.googleusercontent.com/aida-public/AB6AXuA6Ru1OP242ZM7Pqmcc0E4azuCTf6vQIbkDx_iqStSVeKqyw3_us_PT6uR2ieVDWo2PvDla6v8RvelP6L8mpxUNyJEzg5ch_a6A74BqRQ5ET3yvc0Iffy76hUZm2OG1xulIanIzIgTq6Roi7YG89KWLFjdKvww42xqUfnZE-BZMBVOm3n5EPuj-uUKP38BsWLfg7sPsv6-RquG7EMzizpe131L1FOy338mP5E_lX8bj_DGkZB_LtAa4onKR6wNSUm0x523rU124FFU',
    status: 'offline'
  },
  {
    id: 'elena',
    name: 'Elena Rodriguez',
    role: '',
    roleKey: 'members.activity.doNotDisturb',
    roleType: 'assistant',
    avatar: 'https://lh3.googleusercontent.com/aida-public/AB6AXuANDYn5G_lzwb69JjXV0UV20RCfWcczalPe0pR-bPyraoHXJHGDyu6TWus4ohJqw3XZHMdjNjeg6nPslzIBQsDvxzpZhZ1N-4Qrz9kJGhzxY29NPp5Om9mssWsQnYbUZX6yxX44uj8TVVPb25Wft6Xsf39EMCb0Phw7liJjW4G1jbw0_nkffIq8h6UGHjoYAszOmHSKsfeIPKB78f5MICNpst_Cg6gYCc_PrX4DfXeAFupP9Ur1SlE1gH-hw2OPj9oFc46WNApWrEA',
    status: 'dnd'
  },
  {
    id: 'david',
    name: 'David Kim',
    role: '',
    roleType: 'member',
    avatar: 'https://lh3.googleusercontent.com/aida-public/AB6AXuBFQzuk64FLzX_ajt_3zGChUGLWu1uKdEIOD2Dhf-vzZ_qsa4W9PaFKPoBZbHJWwOzqMuqSzlcI-8qJsp1S7zzFOiNOrVtRhwe3pyPbWdd6KXY_EiNKRTk7T7odtGYfgww3ZB98j9bs0kY0wXgMLPGvKUHECZtbXbPX3OUW67dhx5Yj_IxM-cQnmYCMSAs7S_eYfBi7aVonF2Ab8Yb8lhxA5mC8S9Vdp2i3hmSihMheDqZY97OTqwh4vX-yzFai4f_K1WK-nRWyB7Q',
    status: 'offline'
  },
  {
    id: 'priya',
    name: 'Priya Patel',
    role: '',
    roleType: 'member',
    avatar: 'https://lh3.googleusercontent.com/aida-public/AB6AXuBS4LtfNbFLYhN3jlSqNIjwYajAlCLQehKY1GrzGpz7HuLfg_5Hp2_Nant5AasdB-kQg-3K9XgPfV2yHnT3k0OYMi4hAb7ZgtgG21tEINu95frS1lG58-HH_NYb4eH-2BLSvnxpxkUKjWxueAkDNBcJ4VM3FZMYZ94039lGD17Z7ha6oGb9XgD73hrhR6PaiAam5uaJxxUOTEpvlBgRbeYoSVIgQfh3B_fVZmSgppRfZliShDFtm4uWn16K4WIopmqqiNe68biESLE',
    status: 'offline'
  }
];

export const initialMessages: Message[] = [
  {
    id: 1,
    user: 'Sarah Jenkins',
    avatar: SARAH_AVATAR_URL,
    text: 'Hey everyone! 👋 I just pushed the new mockups for the dashboard. Would love some eyes on the spacing for the sidebar.',
    time: formatMessageTime(seedBaseTime),
    createdAt: seedBaseTime,
    isAi: false,
    status: 'sent',
    attachment: {
      type: 'image',
      name: 'dashboard_v2_final.png',
      size: '2.4 MB',
      url: 'https://lh3.googleusercontent.com/aida-public/AB6AXuAXwrGif3OTugu7R6Ry57VrWDqm-ccgrPxj2i2TLmIiwGt1GBqG5ekNNmrF-8FN0auYgBF_kd72SzQayNrryi57HXUT9eQXLeEqdnFwo4T4uwg8vzfF_l9qg_fRwZcqL6DmPlnYBxfErcbpj4ehD1UqZP5QdlZTFcpIfdcYP01LJ77YSTXSwjiEL3oK2eFAyD6lJlYoKzuOQgyvC6o4hmfwW7QG_mhp_6IlG207xJQ0YJJuvpE7CP3MSJAP0iiFESKx7OiNguxi-IE'
    }
  },
  {
    id: 2,
    senderId: 'alex',
    user: 'Alex Designer',
    avatar: getMemberAvatar('alex'),
    text: 'Looks great! I love the coral accent color you chose. It really pops against the dark background. 🔥',
    time: formatMessageTime(seedBaseTime + 1000 * 60 * 3),
    createdAt: seedBaseTime + 1000 * 60 * 3,
    isAi: false,
    status: 'sent'
  },
  {
    id: 3,
    senderId: 'marcus',
    user: 'Marcus Chen',
    avatar: getMemberAvatar('marcus'),
    text: 'Agreed. @Sarah Jenkins when do we plan to ship this? I need to prep the backend.',
    time: formatMessageTime(seedBaseTime + 1000 * 60 * 10),
    createdAt: seedBaseTime + 1000 * 60 * 10,
    isAi: false,
    status: 'sent'
  }
];
