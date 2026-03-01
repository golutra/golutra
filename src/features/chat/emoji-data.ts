// 表情数据来源: emojibase-data（https://github.com/milesj/emojibase），已裁剪为渲染与检索所需字段。
export type EmojiEntry = {
  emoji: string;
  label: string;
  tags: string[];
  group: number;
  order: number;
};

export type EmojiGroup = {
  id: number;
  label: string;
  icon: string;
};

export const EMOJI_GROUPS: EmojiGroup[] = [
  {
    "label": "表情与情绪",
    "icon": "😀",
    "id": 0
  },
  {
    "label": "人物与身体",
    "icon": "🫶",
    "id": 1
  },
  {
    "label": "肤色与修饰",
    "icon": "🖐️",
    "id": 2
  },
  {
    "label": "动物与自然",
    "icon": "🐻",
    "id": 3
  },
  {
    "label": "食物与饮料",
    "icon": "🍜",
    "id": 4
  },
  {
    "label": "旅行与地点",
    "icon": "🗺️",
    "id": 5
  },
  {
    "label": "活动",
    "icon": "⚽",
    "id": 6
  },
  {
    "label": "物品",
    "icon": "💡",
    "id": 7
  },
  {
    "label": "符号",
    "icon": "✨",
    "id": 8
  }
];

export const EMOJI_DATA: EmojiEntry[] = [
  {
    "emoji": "😀",
    "label": "grinning face",
    "tags": [
      "face",
      "grin"
    ],
    "group": 0,
    "order": 1
  },
  {
    "emoji": "😃",
    "label": "grinning face with big eyes",
    "tags": [
      "face",
      "mouth",
      "open",
      "smile"
    ],
    "group": 0,
    "order": 2
  },
  {
    "emoji": "😄",
    "label": "grinning face with smiling eyes",
    "tags": [
      "eye",
      "face",
      "mouth",
      "open",
      "smile"
    ],
    "group": 0,
    "order": 3
  },
  {
    "emoji": "😁",
    "label": "beaming face with smiling eyes",
    "tags": [
      "eye",
      "face",
      "grin",
      "smile"
    ],
    "group": 0,
    "order": 4
  },
  {
    "emoji": "😆",
    "label": "grinning squinting face",
    "tags": [
      "face",
      "laugh",
      "mouth",
      "satisfied",
      "smile"
    ],
    "group": 0,
    "order": 5
  },
  {
    "emoji": "😅",
    "label": "grinning face with sweat",
    "tags": [
      "cold",
      "face",
      "open",
      "smile",
      "sweat"
    ],
    "group": 0,
    "order": 6
  },
  {
    "emoji": "🤣",
    "label": "rolling on the floor laughing",
    "tags": [
      "face",
      "floor",
      "laugh",
      "rofl",
      "rolling",
      "rotfl"
    ],
    "group": 0,
    "order": 7
  },
  {
    "emoji": "😂",
    "label": "face with tears of joy",
    "tags": [
      "face",
      "joy",
      "laugh",
      "tear"
    ],
    "group": 0,
    "order": 8
  },
  {
    "emoji": "🙂",
    "label": "slightly smiling face",
    "tags": [
      "face",
      "smile"
    ],
    "group": 0,
    "order": 9
  },
  {
    "emoji": "🙃",
    "label": "upside-down face",
    "tags": [
      "face",
      "upside-down"
    ],
    "group": 0,
    "order": 10
  },
  {
    "emoji": "🫠",
    "label": "melting face",
    "tags": [
      "disappear",
      "dissolve",
      "liquid",
      "melt"
    ],
    "group": 0,
    "order": 11
  },
  {
    "emoji": "😉",
    "label": "winking face",
    "tags": [
      "face",
      "wink"
    ],
    "group": 0,
    "order": 12
  },
  {
    "emoji": "😊",
    "label": "smiling face with smiling eyes",
    "tags": [
      "blush",
      "eye",
      "face",
      "smile"
    ],
    "group": 0,
    "order": 13
  },
  {
    "emoji": "😇",
    "label": "smiling face with halo",
    "tags": [
      "angel",
      "face",
      "fantasy",
      "halo",
      "innocent"
    ],
    "group": 0,
    "order": 14
  },
  {
    "emoji": "🥰",
    "label": "smiling face with hearts",
    "tags": [
      "adore",
      "crush",
      "hearts",
      "in love"
    ],
    "group": 0,
    "order": 15
  },
  {
    "emoji": "😍",
    "label": "smiling face with heart-eyes",
    "tags": [
      "eye",
      "face",
      "love",
      "smile"
    ],
    "group": 0,
    "order": 16
  },
  {
    "emoji": "🤩",
    "label": "star-struck",
    "tags": [
      "eyes",
      "face",
      "grinning",
      "star"
    ],
    "group": 0,
    "order": 17
  },
  {
    "emoji": "😘",
    "label": "face blowing a kiss",
    "tags": [
      "face",
      "kiss"
    ],
    "group": 0,
    "order": 18
  },
  {
    "emoji": "😗",
    "label": "kissing face",
    "tags": [
      "face",
      "kiss"
    ],
    "group": 0,
    "order": 19
  },
  {
    "emoji": "☺️",
    "label": "smiling face",
    "tags": [
      "face",
      "outlined",
      "relaxed",
      "smile"
    ],
    "group": 0,
    "order": 21
  },
  {
    "emoji": "😚",
    "label": "kissing face with closed eyes",
    "tags": [
      "closed",
      "eye",
      "face",
      "kiss"
    ],
    "group": 0,
    "order": 22
  },
  {
    "emoji": "😙",
    "label": "kissing face with smiling eyes",
    "tags": [
      "eye",
      "face",
      "kiss",
      "smile"
    ],
    "group": 0,
    "order": 23
  },
  {
    "emoji": "🥲",
    "label": "smiling face with tear",
    "tags": [
      "grateful",
      "proud",
      "relieved",
      "smiling",
      "tear",
      "touched"
    ],
    "group": 0,
    "order": 24
  },
  {
    "emoji": "😋",
    "label": "face savoring food",
    "tags": [
      "delicious",
      "face",
      "savouring",
      "smile",
      "yum"
    ],
    "group": 0,
    "order": 25
  },
  {
    "emoji": "😛",
    "label": "face with tongue",
    "tags": [
      "face",
      "tongue"
    ],
    "group": 0,
    "order": 26
  },
  {
    "emoji": "😜",
    "label": "winking face with tongue",
    "tags": [
      "eye",
      "face",
      "joke",
      "tongue",
      "wink"
    ],
    "group": 0,
    "order": 27
  },
  {
    "emoji": "🤪",
    "label": "zany face",
    "tags": [
      "eye",
      "goofy",
      "large",
      "small"
    ],
    "group": 0,
    "order": 28
  },
  {
    "emoji": "😝",
    "label": "squinting face with tongue",
    "tags": [
      "eye",
      "face",
      "horrible",
      "taste",
      "tongue"
    ],
    "group": 0,
    "order": 29
  },
  {
    "emoji": "🤑",
    "label": "money-mouth face",
    "tags": [
      "face",
      "money",
      "mouth"
    ],
    "group": 0,
    "order": 30
  },
  {
    "emoji": "🤗",
    "label": "smiling face with open hands",
    "tags": [
      "face",
      "hug",
      "hugging",
      "open hands",
      "smiling face"
    ],
    "group": 0,
    "order": 31
  },
  {
    "emoji": "🤭",
    "label": "face with hand over mouth",
    "tags": [
      "whoops"
    ],
    "group": 0,
    "order": 32
  },
  {
    "emoji": "🫢",
    "label": "face with open eyes and hand over mouth",
    "tags": [
      "amazement",
      "awe",
      "disbelief",
      "embarrass",
      "scared",
      "surprise"
    ],
    "group": 0,
    "order": 33
  },
  {
    "emoji": "🫣",
    "label": "face with peeking eye",
    "tags": [
      "captivated",
      "peep",
      "stare"
    ],
    "group": 0,
    "order": 34
  },
  {
    "emoji": "🤫",
    "label": "shushing face",
    "tags": [
      "quiet",
      "shush"
    ],
    "group": 0,
    "order": 35
  },
  {
    "emoji": "🤔",
    "label": "thinking face",
    "tags": [
      "face",
      "thinking"
    ],
    "group": 0,
    "order": 36
  },
  {
    "emoji": "🫡",
    "label": "saluting face",
    "tags": [
      "ok",
      "salute",
      "sunny",
      "troops",
      "yes"
    ],
    "group": 0,
    "order": 37
  },
  {
    "emoji": "🤐",
    "label": "zipper-mouth face",
    "tags": [
      "face",
      "mouth",
      "zip",
      "zipper"
    ],
    "group": 0,
    "order": 38
  },
  {
    "emoji": "🤨",
    "label": "face with raised eyebrow",
    "tags": [
      "distrust",
      "skeptic"
    ],
    "group": 0,
    "order": 39
  },
  {
    "emoji": "😐️",
    "label": "neutral face",
    "tags": [
      "deadpan",
      "face",
      "meh",
      "neutral"
    ],
    "group": 0,
    "order": 40
  },
  {
    "emoji": "😑",
    "label": "expressionless face",
    "tags": [
      "expressionless",
      "face",
      "inexpressive",
      "meh",
      "unexpressive"
    ],
    "group": 0,
    "order": 41
  },
  {
    "emoji": "😶",
    "label": "face without mouth",
    "tags": [
      "face",
      "mouth",
      "quiet",
      "silent"
    ],
    "group": 0,
    "order": 42
  },
  {
    "emoji": "🫥",
    "label": "dotted line face",
    "tags": [
      "depressed",
      "disappear",
      "hide",
      "introvert",
      "invisible"
    ],
    "group": 0,
    "order": 43
  },
  {
    "emoji": "😶‍🌫️",
    "label": "face in clouds",
    "tags": [
      "absentminded",
      "face in the fog",
      "head in clouds"
    ],
    "group": 0,
    "order": 44
  },
  {
    "emoji": "😏",
    "label": "smirking face",
    "tags": [
      "face",
      "smirk"
    ],
    "group": 0,
    "order": 46
  },
  {
    "emoji": "😒",
    "label": "unamused face",
    "tags": [
      "face",
      "unamused",
      "unhappy"
    ],
    "group": 0,
    "order": 47
  },
  {
    "emoji": "🙄",
    "label": "face with rolling eyes",
    "tags": [
      "eyeroll",
      "eyes",
      "face",
      "rolling"
    ],
    "group": 0,
    "order": 48
  },
  {
    "emoji": "😬",
    "label": "grimacing face",
    "tags": [
      "face",
      "grimace"
    ],
    "group": 0,
    "order": 49
  },
  {
    "emoji": "😮‍💨",
    "label": "face exhaling",
    "tags": [
      "exhale",
      "gasp",
      "groan",
      "relief",
      "whisper",
      "whistle"
    ],
    "group": 0,
    "order": 50
  },
  {
    "emoji": "🤥",
    "label": "lying face",
    "tags": [
      "face",
      "lie",
      "pinocchio"
    ],
    "group": 0,
    "order": 51
  },
  {
    "emoji": "🫨",
    "label": "shaking face",
    "tags": [
      "earthquake",
      "face",
      "shaking",
      "shock",
      "vibrate"
    ],
    "group": 0,
    "order": 52
  },
  {
    "emoji": "🙂‍↔️",
    "label": "head shaking horizontally",
    "tags": [
      "no",
      "shake"
    ],
    "group": 0,
    "order": 53
  },
  {
    "emoji": "🙂‍↕️",
    "label": "head shaking vertically",
    "tags": [
      "nod",
      "yes"
    ],
    "group": 0,
    "order": 55
  },
  {
    "emoji": "😌",
    "label": "relieved face",
    "tags": [
      "face",
      "relieved"
    ],
    "group": 0,
    "order": 57
  },
  {
    "emoji": "😔",
    "label": "pensive face",
    "tags": [
      "dejected",
      "face",
      "pensive"
    ],
    "group": 0,
    "order": 58
  },
  {
    "emoji": "😪",
    "label": "sleepy face",
    "tags": [
      "face",
      "good night",
      "sleep"
    ],
    "group": 0,
    "order": 59
  },
  {
    "emoji": "🤤",
    "label": "drooling face",
    "tags": [
      "drooling",
      "face"
    ],
    "group": 0,
    "order": 60
  },
  {
    "emoji": "😴",
    "label": "sleeping face",
    "tags": [
      "face",
      "good night",
      "sleep",
      "zzz"
    ],
    "group": 0,
    "order": 61
  },
  {
    "emoji": "😷",
    "label": "face with medical mask",
    "tags": [
      "cold",
      "doctor",
      "face",
      "mask",
      "sick"
    ],
    "group": 0,
    "order": 62
  },
  {
    "emoji": "🤒",
    "label": "face with thermometer",
    "tags": [
      "face",
      "ill",
      "sick",
      "thermometer"
    ],
    "group": 0,
    "order": 63
  },
  {
    "emoji": "🤕",
    "label": "face with head-bandage",
    "tags": [
      "bandage",
      "face",
      "hurt",
      "injury"
    ],
    "group": 0,
    "order": 64
  },
  {
    "emoji": "🤢",
    "label": "nauseated face",
    "tags": [
      "face",
      "nauseated",
      "vomit"
    ],
    "group": 0,
    "order": 65
  },
  {
    "emoji": "🤮",
    "label": "face vomiting",
    "tags": [
      "puke",
      "sick",
      "vomit"
    ],
    "group": 0,
    "order": 66
  },
  {
    "emoji": "🤧",
    "label": "sneezing face",
    "tags": [
      "face",
      "gesundheit",
      "sneeze"
    ],
    "group": 0,
    "order": 67
  },
  {
    "emoji": "🥵",
    "label": "hot face",
    "tags": [
      "feverish",
      "heat stroke",
      "hot",
      "red-faced",
      "sweating"
    ],
    "group": 0,
    "order": 68
  },
  {
    "emoji": "🥶",
    "label": "cold face",
    "tags": [
      "blue-faced",
      "cold",
      "freezing",
      "frostbite",
      "icicles"
    ],
    "group": 0,
    "order": 69
  },
  {
    "emoji": "🥴",
    "label": "woozy face",
    "tags": [
      "dizzy",
      "intoxicated",
      "tipsy",
      "uneven eyes",
      "wavy mouth"
    ],
    "group": 0,
    "order": 70
  },
  {
    "emoji": "😵",
    "label": "face with crossed-out eyes",
    "tags": [
      "crossed-out eyes",
      "dead",
      "face",
      "knocked out"
    ],
    "group": 0,
    "order": 71
  },
  {
    "emoji": "😵‍💫",
    "label": "face with spiral eyes",
    "tags": [
      "dizzy",
      "hypnotized",
      "spiral",
      "trouble",
      "whoa"
    ],
    "group": 0,
    "order": 72
  },
  {
    "emoji": "🤯",
    "label": "exploding head",
    "tags": [
      "mind blown",
      "shocked"
    ],
    "group": 0,
    "order": 73
  },
  {
    "emoji": "🤠",
    "label": "cowboy hat face",
    "tags": [
      "cowboy",
      "cowgirl",
      "face",
      "hat"
    ],
    "group": 0,
    "order": 74
  },
  {
    "emoji": "🥳",
    "label": "partying face",
    "tags": [
      "celebration",
      "hat",
      "horn",
      "party"
    ],
    "group": 0,
    "order": 75
  },
  {
    "emoji": "🥸",
    "label": "disguised face",
    "tags": [
      "disguise",
      "face",
      "glasses",
      "incognito",
      "nose"
    ],
    "group": 0,
    "order": 76
  },
  {
    "emoji": "😎",
    "label": "smiling face with sunglasses",
    "tags": [
      "bright",
      "cool",
      "face",
      "sun",
      "sunglasses"
    ],
    "group": 0,
    "order": 77
  },
  {
    "emoji": "🤓",
    "label": "nerd face",
    "tags": [
      "face",
      "geek",
      "nerd"
    ],
    "group": 0,
    "order": 78
  },
  {
    "emoji": "🧐",
    "label": "face with monocle",
    "tags": [
      "face",
      "monocle",
      "stuffy"
    ],
    "group": 0,
    "order": 79
  },
  {
    "emoji": "😕",
    "label": "confused face",
    "tags": [
      "confused",
      "face",
      "meh"
    ],
    "group": 0,
    "order": 80
  },
  {
    "emoji": "🫤",
    "label": "face with diagonal mouth",
    "tags": [
      "disappointed",
      "meh",
      "skeptical",
      "unsure"
    ],
    "group": 0,
    "order": 81
  },
  {
    "emoji": "😟",
    "label": "worried face",
    "tags": [
      "face",
      "worried"
    ],
    "group": 0,
    "order": 82
  },
  {
    "emoji": "🙁",
    "label": "slightly frowning face",
    "tags": [
      "face",
      "frown"
    ],
    "group": 0,
    "order": 83
  },
  {
    "emoji": "☹️",
    "label": "frowning face",
    "tags": [
      "face",
      "frown"
    ],
    "group": 0,
    "order": 85
  },
  {
    "emoji": "😮",
    "label": "face with open mouth",
    "tags": [
      "face",
      "mouth",
      "open",
      "sympathy"
    ],
    "group": 0,
    "order": 86
  },
  {
    "emoji": "😯",
    "label": "hushed face",
    "tags": [
      "face",
      "hushed",
      "stunned",
      "surprised"
    ],
    "group": 0,
    "order": 87
  },
  {
    "emoji": "😲",
    "label": "astonished face",
    "tags": [
      "astonished",
      "face",
      "shocked",
      "totally"
    ],
    "group": 0,
    "order": 88
  },
  {
    "emoji": "😳",
    "label": "flushed face",
    "tags": [
      "dazed",
      "face",
      "flushed"
    ],
    "group": 0,
    "order": 89
  },
  {
    "emoji": "🥺",
    "label": "pleading face",
    "tags": [
      "begging",
      "mercy",
      "puppy eyes"
    ],
    "group": 0,
    "order": 90
  },
  {
    "emoji": "🥹",
    "label": "face holding back tears",
    "tags": [
      "angry",
      "cry",
      "proud",
      "resist",
      "sad"
    ],
    "group": 0,
    "order": 91
  },
  {
    "emoji": "😦",
    "label": "frowning face with open mouth",
    "tags": [
      "face",
      "frown",
      "mouth",
      "open"
    ],
    "group": 0,
    "order": 92
  },
  {
    "emoji": "😧",
    "label": "anguished face",
    "tags": [
      "anguished",
      "face"
    ],
    "group": 0,
    "order": 93
  },
  {
    "emoji": "😨",
    "label": "fearful face",
    "tags": [
      "face",
      "fear",
      "fearful",
      "scared"
    ],
    "group": 0,
    "order": 94
  },
  {
    "emoji": "😰",
    "label": "anxious face with sweat",
    "tags": [
      "blue",
      "cold",
      "face",
      "rushed",
      "sweat"
    ],
    "group": 0,
    "order": 95
  },
  {
    "emoji": "😥",
    "label": "sad but relieved face",
    "tags": [
      "disappointed",
      "face",
      "relieved",
      "whew"
    ],
    "group": 0,
    "order": 96
  },
  {
    "emoji": "😢",
    "label": "crying face",
    "tags": [
      "cry",
      "face",
      "sad",
      "tear"
    ],
    "group": 0,
    "order": 97
  },
  {
    "emoji": "😭",
    "label": "loudly crying face",
    "tags": [
      "cry",
      "face",
      "sad",
      "sob",
      "tear"
    ],
    "group": 0,
    "order": 98
  },
  {
    "emoji": "😱",
    "label": "face screaming in fear",
    "tags": [
      "face",
      "fear",
      "munch",
      "scared",
      "scream"
    ],
    "group": 0,
    "order": 99
  },
  {
    "emoji": "😖",
    "label": "confounded face",
    "tags": [
      "confounded",
      "face"
    ],
    "group": 0,
    "order": 100
  },
  {
    "emoji": "😣",
    "label": "persevering face",
    "tags": [
      "face",
      "persevere"
    ],
    "group": 0,
    "order": 101
  },
  {
    "emoji": "😞",
    "label": "disappointed face",
    "tags": [
      "disappointed",
      "face"
    ],
    "group": 0,
    "order": 102
  },
  {
    "emoji": "😓",
    "label": "downcast face with sweat",
    "tags": [
      "cold",
      "face",
      "sweat"
    ],
    "group": 0,
    "order": 103
  },
  {
    "emoji": "😩",
    "label": "weary face",
    "tags": [
      "face",
      "tired",
      "weary"
    ],
    "group": 0,
    "order": 104
  },
  {
    "emoji": "😫",
    "label": "tired face",
    "tags": [
      "face",
      "tired"
    ],
    "group": 0,
    "order": 105
  },
  {
    "emoji": "🥱",
    "label": "yawning face",
    "tags": [
      "bored",
      "tired",
      "yawn"
    ],
    "group": 0,
    "order": 106
  },
  {
    "emoji": "😤",
    "label": "face with steam from nose",
    "tags": [
      "face",
      "triumph",
      "won"
    ],
    "group": 0,
    "order": 107
  },
  {
    "emoji": "😡",
    "label": "enraged face",
    "tags": [
      "angry",
      "enraged",
      "face",
      "mad",
      "pouting",
      "rage",
      "red"
    ],
    "group": 0,
    "order": 108
  },
  {
    "emoji": "😠",
    "label": "angry face",
    "tags": [
      "anger",
      "angry",
      "face",
      "mad"
    ],
    "group": 0,
    "order": 109
  },
  {
    "emoji": "🤬",
    "label": "face with symbols on mouth",
    "tags": [
      "swearing"
    ],
    "group": 0,
    "order": 110
  },
  {
    "emoji": "😈",
    "label": "smiling face with horns",
    "tags": [
      "face",
      "fairy tale",
      "fantasy",
      "horns",
      "smile"
    ],
    "group": 0,
    "order": 111
  },
  {
    "emoji": "👿",
    "label": "angry face with horns",
    "tags": [
      "demon",
      "devil",
      "face",
      "fantasy",
      "imp"
    ],
    "group": 0,
    "order": 112
  },
  {
    "emoji": "💀",
    "label": "skull",
    "tags": [
      "death",
      "face",
      "fairy tale",
      "monster"
    ],
    "group": 0,
    "order": 113
  },
  {
    "emoji": "☠️",
    "label": "skull and crossbones",
    "tags": [
      "crossbones",
      "death",
      "face",
      "monster",
      "skull"
    ],
    "group": 0,
    "order": 115
  },
  {
    "emoji": "💩",
    "label": "pile of poo",
    "tags": [
      "dung",
      "face",
      "monster",
      "poo",
      "poop"
    ],
    "group": 0,
    "order": 116
  },
  {
    "emoji": "🤡",
    "label": "clown face",
    "tags": [
      "clown",
      "face"
    ],
    "group": 0,
    "order": 117
  },
  {
    "emoji": "👹",
    "label": "ogre",
    "tags": [
      "creature",
      "face",
      "fairy tale",
      "fantasy",
      "monster"
    ],
    "group": 0,
    "order": 118
  },
  {
    "emoji": "👺",
    "label": "goblin",
    "tags": [
      "creature",
      "face",
      "fairy tale",
      "fantasy",
      "monster"
    ],
    "group": 0,
    "order": 119
  },
  {
    "emoji": "👻",
    "label": "ghost",
    "tags": [
      "creature",
      "face",
      "fairy tale",
      "fantasy",
      "monster"
    ],
    "group": 0,
    "order": 120
  },
  {
    "emoji": "👽️",
    "label": "alien",
    "tags": [
      "creature",
      "extraterrestrial",
      "face",
      "fantasy",
      "ufo"
    ],
    "group": 0,
    "order": 121
  },
  {
    "emoji": "👾",
    "label": "alien monster",
    "tags": [
      "alien",
      "creature",
      "extraterrestrial",
      "face",
      "monster",
      "ufo"
    ],
    "group": 0,
    "order": 122
  },
  {
    "emoji": "🤖",
    "label": "robot",
    "tags": [
      "face",
      "monster"
    ],
    "group": 0,
    "order": 123
  },
  {
    "emoji": "😺",
    "label": "grinning cat",
    "tags": [
      "cat",
      "face",
      "grinning",
      "mouth",
      "open",
      "smile"
    ],
    "group": 0,
    "order": 124
  },
  {
    "emoji": "😸",
    "label": "grinning cat with smiling eyes",
    "tags": [
      "cat",
      "eye",
      "face",
      "grin",
      "smile"
    ],
    "group": 0,
    "order": 125
  },
  {
    "emoji": "😹",
    "label": "cat with tears of joy",
    "tags": [
      "cat",
      "face",
      "joy",
      "tear"
    ],
    "group": 0,
    "order": 126
  },
  {
    "emoji": "😻",
    "label": "smiling cat with heart-eyes",
    "tags": [
      "cat",
      "eye",
      "face",
      "heart",
      "love",
      "smile"
    ],
    "group": 0,
    "order": 127
  },
  {
    "emoji": "😼",
    "label": "cat with wry smile",
    "tags": [
      "cat",
      "face",
      "ironic",
      "smile",
      "wry"
    ],
    "group": 0,
    "order": 128
  },
  {
    "emoji": "😽",
    "label": "kissing cat",
    "tags": [
      "cat",
      "eye",
      "face",
      "kiss"
    ],
    "group": 0,
    "order": 129
  },
  {
    "emoji": "🙀",
    "label": "weary cat",
    "tags": [
      "cat",
      "face",
      "oh",
      "surprised",
      "weary"
    ],
    "group": 0,
    "order": 130
  },
  {
    "emoji": "😿",
    "label": "crying cat",
    "tags": [
      "cat",
      "cry",
      "face",
      "sad",
      "tear"
    ],
    "group": 0,
    "order": 131
  },
  {
    "emoji": "😾",
    "label": "pouting cat",
    "tags": [
      "cat",
      "face",
      "pouting"
    ],
    "group": 0,
    "order": 132
  },
  {
    "emoji": "🙈",
    "label": "see-no-evil monkey",
    "tags": [
      "evil",
      "face",
      "forbidden",
      "monkey",
      "see"
    ],
    "group": 0,
    "order": 133
  },
  {
    "emoji": "🙉",
    "label": "hear-no-evil monkey",
    "tags": [
      "evil",
      "face",
      "forbidden",
      "hear",
      "monkey"
    ],
    "group": 0,
    "order": 134
  },
  {
    "emoji": "🙊",
    "label": "speak-no-evil monkey",
    "tags": [
      "evil",
      "face",
      "forbidden",
      "monkey",
      "speak"
    ],
    "group": 0,
    "order": 135
  },
  {
    "emoji": "💌",
    "label": "love letter",
    "tags": [
      "heart",
      "letter",
      "love",
      "mail"
    ],
    "group": 0,
    "order": 136
  },
  {
    "emoji": "💘",
    "label": "heart with arrow",
    "tags": [
      "arrow",
      "cupid"
    ],
    "group": 0,
    "order": 137
  },
  {
    "emoji": "💝",
    "label": "heart with ribbon",
    "tags": [
      "ribbon",
      "valentine"
    ],
    "group": 0,
    "order": 138
  },
  {
    "emoji": "💖",
    "label": "sparkling heart",
    "tags": [
      "excited",
      "sparkle"
    ],
    "group": 0,
    "order": 139
  },
  {
    "emoji": "💗",
    "label": "growing heart",
    "tags": [
      "excited",
      "growing",
      "nervous",
      "pulse"
    ],
    "group": 0,
    "order": 140
  },
  {
    "emoji": "💓",
    "label": "beating heart",
    "tags": [
      "beating",
      "heartbeat",
      "pulsating"
    ],
    "group": 0,
    "order": 141
  },
  {
    "emoji": "💞",
    "label": "revolving hearts",
    "tags": [
      "revolving"
    ],
    "group": 0,
    "order": 142
  },
  {
    "emoji": "💕",
    "label": "two hearts",
    "tags": [
      "love"
    ],
    "group": 0,
    "order": 143
  },
  {
    "emoji": "💟",
    "label": "heart decoration",
    "tags": [
      "heart"
    ],
    "group": 0,
    "order": 144
  },
  {
    "emoji": "❣️",
    "label": "heart exclamation",
    "tags": [
      "exclamation",
      "mark",
      "punctuation"
    ],
    "group": 0,
    "order": 146
  },
  {
    "emoji": "💔",
    "label": "broken heart",
    "tags": [
      "break",
      "broken"
    ],
    "group": 0,
    "order": 147
  },
  {
    "emoji": "❤️‍🔥",
    "label": "heart on fire",
    "tags": [
      "burn",
      "heart",
      "love",
      "lust",
      "sacred heart"
    ],
    "group": 0,
    "order": 148
  },
  {
    "emoji": "❤️‍🩹",
    "label": "mending heart",
    "tags": [
      "healthier",
      "improving",
      "mending",
      "recovering",
      "recuperating",
      "well"
    ],
    "group": 0,
    "order": 150
  },
  {
    "emoji": "❤️",
    "label": "red heart",
    "tags": [
      "heart"
    ],
    "group": 0,
    "order": 153
  },
  {
    "emoji": "🩷",
    "label": "pink heart",
    "tags": [
      "cute",
      "heart",
      "like",
      "love",
      "pink"
    ],
    "group": 0,
    "order": 154
  },
  {
    "emoji": "🧡",
    "label": "orange heart",
    "tags": [
      "orange"
    ],
    "group": 0,
    "order": 155
  },
  {
    "emoji": "💛",
    "label": "yellow heart",
    "tags": [
      "yellow"
    ],
    "group": 0,
    "order": 156
  },
  {
    "emoji": "💚",
    "label": "green heart",
    "tags": [
      "green"
    ],
    "group": 0,
    "order": 157
  },
  {
    "emoji": "💙",
    "label": "blue heart",
    "tags": [
      "blue"
    ],
    "group": 0,
    "order": 158
  },
  {
    "emoji": "🩵",
    "label": "light blue heart",
    "tags": [
      "cyan",
      "heart",
      "light blue",
      "teal"
    ],
    "group": 0,
    "order": 159
  },
  {
    "emoji": "💜",
    "label": "purple heart",
    "tags": [
      "purple"
    ],
    "group": 0,
    "order": 160
  },
  {
    "emoji": "🤎",
    "label": "brown heart",
    "tags": [
      "brown",
      "heart"
    ],
    "group": 0,
    "order": 161
  },
  {
    "emoji": "🖤",
    "label": "black heart",
    "tags": [
      "black",
      "evil",
      "wicked"
    ],
    "group": 0,
    "order": 162
  },
  {
    "emoji": "🩶",
    "label": "grey heart",
    "tags": [
      "gray",
      "heart",
      "silver",
      "slate"
    ],
    "group": 0,
    "order": 163
  },
  {
    "emoji": "🤍",
    "label": "white heart",
    "tags": [
      "heart",
      "white"
    ],
    "group": 0,
    "order": 164
  },
  {
    "emoji": "💋",
    "label": "kiss mark",
    "tags": [
      "kiss",
      "lips"
    ],
    "group": 0,
    "order": 165
  },
  {
    "emoji": "💯",
    "label": "hundred points",
    "tags": [
      "100",
      "full",
      "hundred",
      "score"
    ],
    "group": 0,
    "order": 166
  },
  {
    "emoji": "💢",
    "label": "anger symbol",
    "tags": [
      "angry",
      "comic",
      "mad"
    ],
    "group": 0,
    "order": 167
  },
  {
    "emoji": "💥",
    "label": "collision",
    "tags": [
      "boom",
      "comic"
    ],
    "group": 0,
    "order": 168
  },
  {
    "emoji": "💫",
    "label": "dizzy",
    "tags": [
      "comic",
      "star"
    ],
    "group": 0,
    "order": 169
  },
  {
    "emoji": "💦",
    "label": "sweat droplets",
    "tags": [
      "comic",
      "splashing",
      "sweat"
    ],
    "group": 0,
    "order": 170
  },
  {
    "emoji": "💨",
    "label": "dashing away",
    "tags": [
      "comic",
      "dash",
      "running"
    ],
    "group": 0,
    "order": 171
  },
  {
    "emoji": "🕳️",
    "label": "hole",
    "tags": [
      "hole"
    ],
    "group": 0,
    "order": 173
  },
  {
    "emoji": "💬",
    "label": "speech balloon",
    "tags": [
      "balloon",
      "bubble",
      "comic",
      "dialog",
      "speech"
    ],
    "group": 0,
    "order": 174
  },
  {
    "emoji": "👁️‍🗨️",
    "label": "eye in speech bubble",
    "tags": [
      "balloon",
      "bubble",
      "eye",
      "speech",
      "witness"
    ],
    "group": 0,
    "order": 175
  },
  {
    "emoji": "🗨️",
    "label": "left speech bubble",
    "tags": [
      "balloon",
      "bubble",
      "dialog",
      "speech"
    ],
    "group": 0,
    "order": 180
  },
  {
    "emoji": "🗯️",
    "label": "right anger bubble",
    "tags": [
      "angry",
      "balloon",
      "bubble",
      "mad"
    ],
    "group": 0,
    "order": 182
  },
  {
    "emoji": "💭",
    "label": "thought balloon",
    "tags": [
      "balloon",
      "bubble",
      "comic",
      "thought"
    ],
    "group": 0,
    "order": 183
  },
  {
    "emoji": "💤",
    "label": "ZZZ",
    "tags": [
      "comic",
      "good night",
      "sleep",
      "zzz"
    ],
    "group": 0,
    "order": 184
  },
  {
    "emoji": "👋",
    "label": "waving hand",
    "tags": [
      "hand",
      "wave",
      "waving"
    ],
    "group": 1,
    "order": 185
  },
  {
    "emoji": "🤚",
    "label": "raised back of hand",
    "tags": [
      "backhand",
      "raised"
    ],
    "group": 1,
    "order": 191
  },
  {
    "emoji": "🖐️",
    "label": "hand with fingers splayed",
    "tags": [
      "finger",
      "hand",
      "splayed"
    ],
    "group": 1,
    "order": 198
  },
  {
    "emoji": "✋️",
    "label": "raised hand",
    "tags": [
      "hand",
      "high 5",
      "high five"
    ],
    "group": 1,
    "order": 204
  },
  {
    "emoji": "🖖",
    "label": "vulcan salute",
    "tags": [
      "finger",
      "hand",
      "spock",
      "vulcan"
    ],
    "group": 1,
    "order": 210
  },
  {
    "emoji": "🫱",
    "label": "rightwards hand",
    "tags": [
      "hand",
      "right",
      "rightward"
    ],
    "group": 1,
    "order": 216
  },
  {
    "emoji": "🫲",
    "label": "leftwards hand",
    "tags": [
      "hand",
      "left",
      "leftward"
    ],
    "group": 1,
    "order": 222
  },
  {
    "emoji": "🫳",
    "label": "palm down hand",
    "tags": [
      "dismiss",
      "drop",
      "shoo"
    ],
    "group": 1,
    "order": 228
  },
  {
    "emoji": "🫴",
    "label": "palm up hand",
    "tags": [
      "beckon",
      "catch",
      "come",
      "offer"
    ],
    "group": 1,
    "order": 234
  },
  {
    "emoji": "🫷",
    "label": "leftwards pushing hand",
    "tags": [
      "high five",
      "leftward",
      "push",
      "refuse",
      "stop",
      "wait"
    ],
    "group": 1,
    "order": 240
  },
  {
    "emoji": "🫸",
    "label": "rightwards pushing hand",
    "tags": [
      "high five",
      "push",
      "refuse",
      "rightward",
      "stop",
      "wait"
    ],
    "group": 1,
    "order": 246
  },
  {
    "emoji": "👌",
    "label": "OK hand",
    "tags": [
      "hand",
      "ok"
    ],
    "group": 1,
    "order": 252
  },
  {
    "emoji": "🤌",
    "label": "pinched fingers",
    "tags": [
      "fingers",
      "hand gesture",
      "interrogation",
      "pinched",
      "sarcastic"
    ],
    "group": 1,
    "order": 258
  },
  {
    "emoji": "🤏",
    "label": "pinching hand",
    "tags": [
      "small amount"
    ],
    "group": 1,
    "order": 264
  },
  {
    "emoji": "✌️",
    "label": "victory hand",
    "tags": [
      "hand",
      "v",
      "victory"
    ],
    "group": 1,
    "order": 271
  },
  {
    "emoji": "🤞",
    "label": "crossed fingers",
    "tags": [
      "cross",
      "finger",
      "hand",
      "luck"
    ],
    "group": 1,
    "order": 277
  },
  {
    "emoji": "🫰",
    "label": "hand with index finger and thumb crossed",
    "tags": [
      "expensive",
      "heart",
      "love",
      "money",
      "snap"
    ],
    "group": 1,
    "order": 283
  },
  {
    "emoji": "🤟",
    "label": "love-you gesture",
    "tags": [
      "hand",
      "ily"
    ],
    "group": 1,
    "order": 289
  },
  {
    "emoji": "🤘",
    "label": "sign of the horns",
    "tags": [
      "finger",
      "hand",
      "horns",
      "rock-on"
    ],
    "group": 1,
    "order": 295
  },
  {
    "emoji": "🤙",
    "label": "call me hand",
    "tags": [
      "call",
      "hand",
      "hang loose",
      "shaka"
    ],
    "group": 1,
    "order": 301
  },
  {
    "emoji": "👈️",
    "label": "backhand index pointing left",
    "tags": [
      "backhand",
      "finger",
      "hand",
      "index",
      "point"
    ],
    "group": 1,
    "order": 307
  },
  {
    "emoji": "👉️",
    "label": "backhand index pointing right",
    "tags": [
      "backhand",
      "finger",
      "hand",
      "index",
      "point"
    ],
    "group": 1,
    "order": 313
  },
  {
    "emoji": "👆️",
    "label": "backhand index pointing up",
    "tags": [
      "backhand",
      "finger",
      "hand",
      "point",
      "up"
    ],
    "group": 1,
    "order": 319
  },
  {
    "emoji": "🖕",
    "label": "middle finger",
    "tags": [
      "finger",
      "hand"
    ],
    "group": 1,
    "order": 325
  },
  {
    "emoji": "👇️",
    "label": "backhand index pointing down",
    "tags": [
      "backhand",
      "down",
      "finger",
      "hand",
      "point"
    ],
    "group": 1,
    "order": 331
  },
  {
    "emoji": "☝️",
    "label": "index pointing up",
    "tags": [
      "finger",
      "hand",
      "index",
      "point",
      "up"
    ],
    "group": 1,
    "order": 338
  },
  {
    "emoji": "🫵",
    "label": "index pointing at the viewer",
    "tags": [
      "point",
      "you"
    ],
    "group": 1,
    "order": 344
  },
  {
    "emoji": "👍️",
    "label": "thumbs up",
    "tags": [
      "+1",
      "hand",
      "thumb",
      "up"
    ],
    "group": 1,
    "order": 350
  },
  {
    "emoji": "👎️",
    "label": "thumbs down",
    "tags": [
      "-1",
      "down",
      "hand",
      "thumb"
    ],
    "group": 1,
    "order": 356
  },
  {
    "emoji": "✊️",
    "label": "raised fist",
    "tags": [
      "clenched",
      "fist",
      "hand",
      "punch"
    ],
    "group": 1,
    "order": 362
  },
  {
    "emoji": "👊",
    "label": "oncoming fist",
    "tags": [
      "clenched",
      "fist",
      "hand",
      "punch"
    ],
    "group": 1,
    "order": 368
  },
  {
    "emoji": "🤛",
    "label": "left-facing fist",
    "tags": [
      "fist",
      "leftwards"
    ],
    "group": 1,
    "order": 374
  },
  {
    "emoji": "🤜",
    "label": "right-facing fist",
    "tags": [
      "fist",
      "rightwards"
    ],
    "group": 1,
    "order": 380
  },
  {
    "emoji": "👏",
    "label": "clapping hands",
    "tags": [
      "clap",
      "hand"
    ],
    "group": 1,
    "order": 386
  },
  {
    "emoji": "🙌",
    "label": "raising hands",
    "tags": [
      "celebration",
      "gesture",
      "hand",
      "hooray",
      "raised"
    ],
    "group": 1,
    "order": 392
  },
  {
    "emoji": "🫶",
    "label": "heart hands",
    "tags": [
      "love"
    ],
    "group": 1,
    "order": 398
  },
  {
    "emoji": "👐",
    "label": "open hands",
    "tags": [
      "hand",
      "open"
    ],
    "group": 1,
    "order": 404
  },
  {
    "emoji": "🤲",
    "label": "palms up together",
    "tags": [
      "prayer"
    ],
    "group": 1,
    "order": 410
  },
  {
    "emoji": "🤝",
    "label": "handshake",
    "tags": [
      "agreement",
      "hand",
      "meeting",
      "shake"
    ],
    "group": 1,
    "order": 416
  },
  {
    "emoji": "🙏",
    "label": "folded hands",
    "tags": [
      "ask",
      "hand",
      "high 5",
      "high five",
      "please",
      "pray",
      "thanks"
    ],
    "group": 1,
    "order": 442
  },
  {
    "emoji": "✍️",
    "label": "writing hand",
    "tags": [
      "hand",
      "write"
    ],
    "group": 1,
    "order": 449
  },
  {
    "emoji": "💅",
    "label": "nail polish",
    "tags": [
      "care",
      "cosmetics",
      "manicure",
      "nail",
      "polish"
    ],
    "group": 1,
    "order": 455
  },
  {
    "emoji": "🤳",
    "label": "selfie",
    "tags": [
      "camera",
      "phone"
    ],
    "group": 1,
    "order": 461
  },
  {
    "emoji": "💪",
    "label": "flexed biceps",
    "tags": [
      "biceps",
      "comic",
      "flex",
      "muscle"
    ],
    "group": 1,
    "order": 467
  },
  {
    "emoji": "🦾",
    "label": "mechanical arm",
    "tags": [
      "accessibility",
      "prosthetic"
    ],
    "group": 1,
    "order": 473
  },
  {
    "emoji": "🦿",
    "label": "mechanical leg",
    "tags": [
      "accessibility",
      "prosthetic"
    ],
    "group": 1,
    "order": 474
  },
  {
    "emoji": "🦵",
    "label": "leg",
    "tags": [
      "kick",
      "limb"
    ],
    "group": 1,
    "order": 475
  },
  {
    "emoji": "🦶",
    "label": "foot",
    "tags": [
      "kick",
      "stomp"
    ],
    "group": 1,
    "order": 481
  },
  {
    "emoji": "👂️",
    "label": "ear",
    "tags": [
      "body"
    ],
    "group": 1,
    "order": 487
  },
  {
    "emoji": "🦻",
    "label": "ear with hearing aid",
    "tags": [
      "accessibility",
      "hard of hearing"
    ],
    "group": 1,
    "order": 493
  },
  {
    "emoji": "👃",
    "label": "nose",
    "tags": [
      "body"
    ],
    "group": 1,
    "order": 499
  },
  {
    "emoji": "🧠",
    "label": "brain",
    "tags": [
      "intelligent"
    ],
    "group": 1,
    "order": 505
  },
  {
    "emoji": "🫀",
    "label": "anatomical heart",
    "tags": [
      "anatomical",
      "cardiology",
      "heart",
      "organ",
      "pulse"
    ],
    "group": 1,
    "order": 506
  },
  {
    "emoji": "🫁",
    "label": "lungs",
    "tags": [
      "breath",
      "exhalation",
      "inhalation",
      "organ",
      "respiration"
    ],
    "group": 1,
    "order": 507
  },
  {
    "emoji": "🦷",
    "label": "tooth",
    "tags": [
      "dentist"
    ],
    "group": 1,
    "order": 508
  },
  {
    "emoji": "🦴",
    "label": "bone",
    "tags": [
      "skeleton"
    ],
    "group": 1,
    "order": 509
  },
  {
    "emoji": "👀",
    "label": "eyes",
    "tags": [
      "eye",
      "face"
    ],
    "group": 1,
    "order": 510
  },
  {
    "emoji": "👁️",
    "label": "eye",
    "tags": [
      "body"
    ],
    "group": 1,
    "order": 512
  },
  {
    "emoji": "👅",
    "label": "tongue",
    "tags": [
      "body"
    ],
    "group": 1,
    "order": 513
  },
  {
    "emoji": "👄",
    "label": "mouth",
    "tags": [
      "lips"
    ],
    "group": 1,
    "order": 514
  },
  {
    "emoji": "🫦",
    "label": "biting lip",
    "tags": [
      "anxious",
      "fear",
      "flirting",
      "nervous",
      "uncomfortable",
      "worried"
    ],
    "group": 1,
    "order": 515
  },
  {
    "emoji": "👶",
    "label": "baby",
    "tags": [
      "young"
    ],
    "group": 1,
    "order": 516
  },
  {
    "emoji": "🧒",
    "label": "child",
    "tags": [
      "gender-neutral",
      "unspecified gender",
      "young"
    ],
    "group": 1,
    "order": 522
  },
  {
    "emoji": "👦",
    "label": "boy",
    "tags": [
      "young"
    ],
    "group": 1,
    "order": 528
  },
  {
    "emoji": "👧",
    "label": "girl",
    "tags": [
      "virgo",
      "young",
      "zodiac"
    ],
    "group": 1,
    "order": 534
  },
  {
    "emoji": "🧑",
    "label": "person",
    "tags": [
      "adult",
      "gender-neutral",
      "unspecified gender"
    ],
    "group": 1,
    "order": 540
  },
  {
    "emoji": "👱",
    "label": "person: blond hair",
    "tags": [
      "blond",
      "blond-haired person",
      "hair"
    ],
    "group": 1,
    "order": 546
  },
  {
    "emoji": "👨",
    "label": "man",
    "tags": [
      "adult"
    ],
    "group": 1,
    "order": 552
  },
  {
    "emoji": "🧔",
    "label": "person: beard",
    "tags": [
      "beard",
      "person"
    ],
    "group": 1,
    "order": 558
  },
  {
    "emoji": "🧔‍♂️",
    "label": "man: beard",
    "tags": [
      "beard",
      "man"
    ],
    "group": 1,
    "order": 564
  },
  {
    "emoji": "🧔‍♀️",
    "label": "woman: beard",
    "tags": [
      "beard",
      "woman"
    ],
    "group": 1,
    "order": 576
  },
  {
    "emoji": "👨‍🦰",
    "label": "man: red hair",
    "tags": [
      "adult",
      "man",
      "red hair"
    ],
    "group": 1,
    "order": 588
  },
  {
    "emoji": "👨‍🦱",
    "label": "man: curly hair",
    "tags": [
      "adult",
      "curly hair",
      "man"
    ],
    "group": 1,
    "order": 594
  },
  {
    "emoji": "👨‍🦳",
    "label": "man: white hair",
    "tags": [
      "adult",
      "man",
      "white hair"
    ],
    "group": 1,
    "order": 600
  },
  {
    "emoji": "👨‍🦲",
    "label": "man: bald",
    "tags": [
      "adult",
      "bald",
      "man"
    ],
    "group": 1,
    "order": 606
  },
  {
    "emoji": "👩",
    "label": "woman",
    "tags": [
      "adult"
    ],
    "group": 1,
    "order": 612
  },
  {
    "emoji": "👩‍🦰",
    "label": "woman: red hair",
    "tags": [
      "adult",
      "red hair",
      "woman"
    ],
    "group": 1,
    "order": 618
  },
  {
    "emoji": "🧑‍🦰",
    "label": "person: red hair",
    "tags": [
      "adult",
      "gender-neutral",
      "person",
      "red hair",
      "unspecified gender"
    ],
    "group": 1,
    "order": 624
  },
  {
    "emoji": "👩‍🦱",
    "label": "woman: curly hair",
    "tags": [
      "adult",
      "curly hair",
      "woman"
    ],
    "group": 1,
    "order": 630
  },
  {
    "emoji": "🧑‍🦱",
    "label": "person: curly hair",
    "tags": [
      "adult",
      "curly hair",
      "gender-neutral",
      "person",
      "unspecified gender"
    ],
    "group": 1,
    "order": 636
  },
  {
    "emoji": "👩‍🦳",
    "label": "woman: white hair",
    "tags": [
      "adult",
      "white hair",
      "woman"
    ],
    "group": 1,
    "order": 642
  },
  {
    "emoji": "🧑‍🦳",
    "label": "person: white hair",
    "tags": [
      "adult",
      "gender-neutral",
      "person",
      "unspecified gender",
      "white hair"
    ],
    "group": 1,
    "order": 648
  },
  {
    "emoji": "👩‍🦲",
    "label": "woman: bald",
    "tags": [
      "adult",
      "bald",
      "woman"
    ],
    "group": 1,
    "order": 654
  },
  {
    "emoji": "🧑‍🦲",
    "label": "person: bald",
    "tags": [
      "adult",
      "bald",
      "gender-neutral",
      "person",
      "unspecified gender"
    ],
    "group": 1,
    "order": 660
  },
  {
    "emoji": "👱‍♀️",
    "label": "woman: blond hair",
    "tags": [
      "blond-haired woman",
      "blonde",
      "hair",
      "woman"
    ],
    "group": 1,
    "order": 666
  },
  {
    "emoji": "👱‍♂️",
    "label": "man: blond hair",
    "tags": [
      "blond",
      "blond-haired man",
      "hair",
      "man"
    ],
    "group": 1,
    "order": 678
  },
  {
    "emoji": "🧓",
    "label": "older person",
    "tags": [
      "adult",
      "gender-neutral",
      "old",
      "unspecified gender"
    ],
    "group": 1,
    "order": 690
  },
  {
    "emoji": "👴",
    "label": "old man",
    "tags": [
      "adult",
      "man",
      "old"
    ],
    "group": 1,
    "order": 696
  },
  {
    "emoji": "👵",
    "label": "old woman",
    "tags": [
      "adult",
      "old",
      "woman"
    ],
    "group": 1,
    "order": 702
  },
  {
    "emoji": "🙍",
    "label": "person frowning",
    "tags": [
      "frown",
      "gesture"
    ],
    "group": 1,
    "order": 708
  },
  {
    "emoji": "🙍‍♂️",
    "label": "man frowning",
    "tags": [
      "frowning",
      "gesture",
      "man"
    ],
    "group": 1,
    "order": 714
  },
  {
    "emoji": "🙍‍♀️",
    "label": "woman frowning",
    "tags": [
      "frowning",
      "gesture",
      "woman"
    ],
    "group": 1,
    "order": 726
  },
  {
    "emoji": "🙎",
    "label": "person pouting",
    "tags": [
      "gesture",
      "pouting"
    ],
    "group": 1,
    "order": 738
  },
  {
    "emoji": "🙎‍♂️",
    "label": "man pouting",
    "tags": [
      "gesture",
      "man",
      "pouting"
    ],
    "group": 1,
    "order": 744
  },
  {
    "emoji": "🙎‍♀️",
    "label": "woman pouting",
    "tags": [
      "gesture",
      "pouting",
      "woman"
    ],
    "group": 1,
    "order": 756
  },
  {
    "emoji": "🙅",
    "label": "person gesturing NO",
    "tags": [
      "forbidden",
      "gesture",
      "hand",
      "person gesturing no",
      "prohibited"
    ],
    "group": 1,
    "order": 768
  },
  {
    "emoji": "🙅‍♂️",
    "label": "man gesturing NO",
    "tags": [
      "forbidden",
      "gesture",
      "hand",
      "man",
      "man gesturing no",
      "prohibited"
    ],
    "group": 1,
    "order": 774
  },
  {
    "emoji": "🙅‍♀️",
    "label": "woman gesturing NO",
    "tags": [
      "forbidden",
      "gesture",
      "hand",
      "prohibited",
      "woman",
      "woman gesturing no"
    ],
    "group": 1,
    "order": 786
  },
  {
    "emoji": "🙆",
    "label": "person gesturing OK",
    "tags": [
      "gesture",
      "hand",
      "ok",
      "person gesturing ok"
    ],
    "group": 1,
    "order": 798
  },
  {
    "emoji": "🙆‍♂️",
    "label": "man gesturing OK",
    "tags": [
      "gesture",
      "hand",
      "man",
      "man gesturing ok",
      "ok"
    ],
    "group": 1,
    "order": 804
  },
  {
    "emoji": "🙆‍♀️",
    "label": "woman gesturing OK",
    "tags": [
      "gesture",
      "hand",
      "ok",
      "woman",
      "woman gesturing ok"
    ],
    "group": 1,
    "order": 816
  },
  {
    "emoji": "💁",
    "label": "person tipping hand",
    "tags": [
      "hand",
      "help",
      "information",
      "sassy",
      "tipping"
    ],
    "group": 1,
    "order": 828
  },
  {
    "emoji": "💁‍♂️",
    "label": "man tipping hand",
    "tags": [
      "man",
      "sassy",
      "tipping hand"
    ],
    "group": 1,
    "order": 834
  },
  {
    "emoji": "💁‍♀️",
    "label": "woman tipping hand",
    "tags": [
      "sassy",
      "tipping hand",
      "woman"
    ],
    "group": 1,
    "order": 846
  },
  {
    "emoji": "🙋",
    "label": "person raising hand",
    "tags": [
      "gesture",
      "hand",
      "happy",
      "raised"
    ],
    "group": 1,
    "order": 858
  },
  {
    "emoji": "🙋‍♂️",
    "label": "man raising hand",
    "tags": [
      "gesture",
      "man",
      "raising hand"
    ],
    "group": 1,
    "order": 864
  },
  {
    "emoji": "🙋‍♀️",
    "label": "woman raising hand",
    "tags": [
      "gesture",
      "raising hand",
      "woman"
    ],
    "group": 1,
    "order": 876
  },
  {
    "emoji": "🧏",
    "label": "deaf person",
    "tags": [
      "accessibility",
      "deaf",
      "ear",
      "hear"
    ],
    "group": 1,
    "order": 888
  },
  {
    "emoji": "🧏‍♂️",
    "label": "deaf man",
    "tags": [
      "deaf",
      "man"
    ],
    "group": 1,
    "order": 894
  },
  {
    "emoji": "🧏‍♀️",
    "label": "deaf woman",
    "tags": [
      "deaf",
      "woman"
    ],
    "group": 1,
    "order": 906
  },
  {
    "emoji": "🙇",
    "label": "person bowing",
    "tags": [
      "apology",
      "bow",
      "gesture",
      "sorry"
    ],
    "group": 1,
    "order": 918
  },
  {
    "emoji": "🙇‍♂️",
    "label": "man bowing",
    "tags": [
      "apology",
      "bowing",
      "favor",
      "gesture",
      "man",
      "sorry"
    ],
    "group": 1,
    "order": 924
  },
  {
    "emoji": "🙇‍♀️",
    "label": "woman bowing",
    "tags": [
      "apology",
      "bowing",
      "favor",
      "gesture",
      "sorry",
      "woman"
    ],
    "group": 1,
    "order": 936
  },
  {
    "emoji": "🤦",
    "label": "person facepalming",
    "tags": [
      "disbelief",
      "exasperation",
      "face",
      "palm"
    ],
    "group": 1,
    "order": 948
  },
  {
    "emoji": "🤦‍♂️",
    "label": "man facepalming",
    "tags": [
      "disbelief",
      "exasperation",
      "facepalm",
      "man"
    ],
    "group": 1,
    "order": 954
  },
  {
    "emoji": "🤦‍♀️",
    "label": "woman facepalming",
    "tags": [
      "disbelief",
      "exasperation",
      "facepalm",
      "woman"
    ],
    "group": 1,
    "order": 966
  },
  {
    "emoji": "🤷",
    "label": "person shrugging",
    "tags": [
      "doubt",
      "ignorance",
      "indifference",
      "shrug"
    ],
    "group": 1,
    "order": 978
  },
  {
    "emoji": "🤷‍♂️",
    "label": "man shrugging",
    "tags": [
      "doubt",
      "ignorance",
      "indifference",
      "man",
      "shrug"
    ],
    "group": 1,
    "order": 984
  },
  {
    "emoji": "🤷‍♀️",
    "label": "woman shrugging",
    "tags": [
      "doubt",
      "ignorance",
      "indifference",
      "shrug",
      "woman"
    ],
    "group": 1,
    "order": 996
  },
  {
    "emoji": "🧑‍⚕️",
    "label": "health worker",
    "tags": [
      "doctor",
      "healthcare",
      "nurse",
      "therapist"
    ],
    "group": 1,
    "order": 1008
  },
  {
    "emoji": "👨‍⚕️",
    "label": "man health worker",
    "tags": [
      "doctor",
      "healthcare",
      "man",
      "nurse",
      "therapist"
    ],
    "group": 1,
    "order": 1020
  },
  {
    "emoji": "👩‍⚕️",
    "label": "woman health worker",
    "tags": [
      "doctor",
      "healthcare",
      "nurse",
      "therapist",
      "woman"
    ],
    "group": 1,
    "order": 1032
  },
  {
    "emoji": "🧑‍🎓",
    "label": "student",
    "tags": [
      "graduate"
    ],
    "group": 1,
    "order": 1044
  },
  {
    "emoji": "👨‍🎓",
    "label": "man student",
    "tags": [
      "graduate",
      "man",
      "student"
    ],
    "group": 1,
    "order": 1050
  },
  {
    "emoji": "👩‍🎓",
    "label": "woman student",
    "tags": [
      "graduate",
      "student",
      "woman"
    ],
    "group": 1,
    "order": 1056
  },
  {
    "emoji": "🧑‍🏫",
    "label": "teacher",
    "tags": [
      "instructor",
      "lecturer",
      "professor"
    ],
    "group": 1,
    "order": 1062
  },
  {
    "emoji": "👨‍🏫",
    "label": "man teacher",
    "tags": [
      "instructor",
      "lecturer",
      "man",
      "professor",
      "teacher"
    ],
    "group": 1,
    "order": 1068
  },
  {
    "emoji": "👩‍🏫",
    "label": "woman teacher",
    "tags": [
      "instructor",
      "lecturer",
      "professor",
      "teacher",
      "woman"
    ],
    "group": 1,
    "order": 1074
  },
  {
    "emoji": "🧑‍⚖️",
    "label": "judge",
    "tags": [
      "justice",
      "law",
      "scales"
    ],
    "group": 1,
    "order": 1080
  },
  {
    "emoji": "👨‍⚖️",
    "label": "man judge",
    "tags": [
      "judge",
      "justice",
      "law",
      "man",
      "scales"
    ],
    "group": 1,
    "order": 1092
  },
  {
    "emoji": "👩‍⚖️",
    "label": "woman judge",
    "tags": [
      "judge",
      "justice",
      "law",
      "scales",
      "woman"
    ],
    "group": 1,
    "order": 1104
  },
  {
    "emoji": "🧑‍🌾",
    "label": "farmer",
    "tags": [
      "gardener",
      "rancher"
    ],
    "group": 1,
    "order": 1116
  },
  {
    "emoji": "👨‍🌾",
    "label": "man farmer",
    "tags": [
      "farmer",
      "gardener",
      "man",
      "rancher"
    ],
    "group": 1,
    "order": 1122
  },
  {
    "emoji": "👩‍🌾",
    "label": "woman farmer",
    "tags": [
      "farmer",
      "gardener",
      "rancher",
      "woman"
    ],
    "group": 1,
    "order": 1128
  },
  {
    "emoji": "🧑‍🍳",
    "label": "cook",
    "tags": [
      "chef"
    ],
    "group": 1,
    "order": 1134
  },
  {
    "emoji": "👨‍🍳",
    "label": "man cook",
    "tags": [
      "chef",
      "cook",
      "man"
    ],
    "group": 1,
    "order": 1140
  },
  {
    "emoji": "👩‍🍳",
    "label": "woman cook",
    "tags": [
      "chef",
      "cook",
      "woman"
    ],
    "group": 1,
    "order": 1146
  },
  {
    "emoji": "🧑‍🔧",
    "label": "mechanic",
    "tags": [
      "electrician",
      "plumber",
      "tradesperson"
    ],
    "group": 1,
    "order": 1152
  },
  {
    "emoji": "👨‍🔧",
    "label": "man mechanic",
    "tags": [
      "electrician",
      "man",
      "mechanic",
      "plumber",
      "tradesperson"
    ],
    "group": 1,
    "order": 1158
  },
  {
    "emoji": "👩‍🔧",
    "label": "woman mechanic",
    "tags": [
      "electrician",
      "mechanic",
      "plumber",
      "tradesperson",
      "woman"
    ],
    "group": 1,
    "order": 1164
  },
  {
    "emoji": "🧑‍🏭",
    "label": "factory worker",
    "tags": [
      "assembly",
      "factory",
      "industrial",
      "worker"
    ],
    "group": 1,
    "order": 1170
  },
  {
    "emoji": "👨‍🏭",
    "label": "man factory worker",
    "tags": [
      "assembly",
      "factory",
      "industrial",
      "man",
      "worker"
    ],
    "group": 1,
    "order": 1176
  },
  {
    "emoji": "👩‍🏭",
    "label": "woman factory worker",
    "tags": [
      "assembly",
      "factory",
      "industrial",
      "woman",
      "worker"
    ],
    "group": 1,
    "order": 1182
  },
  {
    "emoji": "🧑‍💼",
    "label": "office worker",
    "tags": [
      "architect",
      "business",
      "manager",
      "white-collar"
    ],
    "group": 1,
    "order": 1188
  },
  {
    "emoji": "👨‍💼",
    "label": "man office worker",
    "tags": [
      "architect",
      "business",
      "man",
      "manager",
      "white-collar"
    ],
    "group": 1,
    "order": 1194
  },
  {
    "emoji": "👩‍💼",
    "label": "woman office worker",
    "tags": [
      "architect",
      "business",
      "manager",
      "white-collar",
      "woman"
    ],
    "group": 1,
    "order": 1200
  },
  {
    "emoji": "🧑‍🔬",
    "label": "scientist",
    "tags": [
      "biologist",
      "chemist",
      "engineer",
      "physicist"
    ],
    "group": 1,
    "order": 1206
  },
  {
    "emoji": "👨‍🔬",
    "label": "man scientist",
    "tags": [
      "biologist",
      "chemist",
      "engineer",
      "man",
      "physicist",
      "scientist"
    ],
    "group": 1,
    "order": 1212
  },
  {
    "emoji": "👩‍🔬",
    "label": "woman scientist",
    "tags": [
      "biologist",
      "chemist",
      "engineer",
      "physicist",
      "scientist",
      "woman"
    ],
    "group": 1,
    "order": 1218
  },
  {
    "emoji": "🧑‍💻",
    "label": "technologist",
    "tags": [
      "coder",
      "developer",
      "inventor",
      "software"
    ],
    "group": 1,
    "order": 1224
  },
  {
    "emoji": "👨‍💻",
    "label": "man technologist",
    "tags": [
      "coder",
      "developer",
      "inventor",
      "man",
      "software",
      "technologist"
    ],
    "group": 1,
    "order": 1230
  },
  {
    "emoji": "👩‍💻",
    "label": "woman technologist",
    "tags": [
      "coder",
      "developer",
      "inventor",
      "software",
      "technologist",
      "woman"
    ],
    "group": 1,
    "order": 1236
  },
  {
    "emoji": "🧑‍🎤",
    "label": "singer",
    "tags": [
      "actor",
      "entertainer",
      "rock",
      "star"
    ],
    "group": 1,
    "order": 1242
  },
  {
    "emoji": "👨‍🎤",
    "label": "man singer",
    "tags": [
      "actor",
      "entertainer",
      "man",
      "rock",
      "singer",
      "star"
    ],
    "group": 1,
    "order": 1248
  },
  {
    "emoji": "👩‍🎤",
    "label": "woman singer",
    "tags": [
      "actor",
      "entertainer",
      "rock",
      "singer",
      "star",
      "woman"
    ],
    "group": 1,
    "order": 1254
  },
  {
    "emoji": "🧑‍🎨",
    "label": "artist",
    "tags": [
      "palette"
    ],
    "group": 1,
    "order": 1260
  },
  {
    "emoji": "👨‍🎨",
    "label": "man artist",
    "tags": [
      "artist",
      "man",
      "palette"
    ],
    "group": 1,
    "order": 1266
  },
  {
    "emoji": "👩‍🎨",
    "label": "woman artist",
    "tags": [
      "artist",
      "palette",
      "woman"
    ],
    "group": 1,
    "order": 1272
  },
  {
    "emoji": "🧑‍✈️",
    "label": "pilot",
    "tags": [
      "plane"
    ],
    "group": 1,
    "order": 1278
  },
  {
    "emoji": "👨‍✈️",
    "label": "man pilot",
    "tags": [
      "man",
      "pilot",
      "plane"
    ],
    "group": 1,
    "order": 1290
  },
  {
    "emoji": "👩‍✈️",
    "label": "woman pilot",
    "tags": [
      "pilot",
      "plane",
      "woman"
    ],
    "group": 1,
    "order": 1302
  },
  {
    "emoji": "🧑‍🚀",
    "label": "astronaut",
    "tags": [
      "rocket"
    ],
    "group": 1,
    "order": 1314
  },
  {
    "emoji": "👨‍🚀",
    "label": "man astronaut",
    "tags": [
      "astronaut",
      "man",
      "rocket"
    ],
    "group": 1,
    "order": 1320
  },
  {
    "emoji": "👩‍🚀",
    "label": "woman astronaut",
    "tags": [
      "astronaut",
      "rocket",
      "woman"
    ],
    "group": 1,
    "order": 1326
  },
  {
    "emoji": "🧑‍🚒",
    "label": "firefighter",
    "tags": [
      "fire",
      "firetruck"
    ],
    "group": 1,
    "order": 1332
  },
  {
    "emoji": "👨‍🚒",
    "label": "man firefighter",
    "tags": [
      "firefighter",
      "firetruck",
      "man"
    ],
    "group": 1,
    "order": 1338
  },
  {
    "emoji": "👩‍🚒",
    "label": "woman firefighter",
    "tags": [
      "firefighter",
      "firetruck",
      "woman"
    ],
    "group": 1,
    "order": 1344
  },
  {
    "emoji": "👮",
    "label": "police officer",
    "tags": [
      "cop",
      "officer",
      "police"
    ],
    "group": 1,
    "order": 1350
  },
  {
    "emoji": "👮‍♂️",
    "label": "man police officer",
    "tags": [
      "cop",
      "man",
      "officer",
      "police"
    ],
    "group": 1,
    "order": 1356
  },
  {
    "emoji": "👮‍♀️",
    "label": "woman police officer",
    "tags": [
      "cop",
      "officer",
      "police",
      "woman"
    ],
    "group": 1,
    "order": 1368
  },
  {
    "emoji": "🕵️",
    "label": "detective",
    "tags": [
      "sleuth",
      "spy"
    ],
    "group": 1,
    "order": 1381
  },
  {
    "emoji": "🕵️‍♂️",
    "label": "man detective",
    "tags": [
      "detective",
      "man",
      "sleuth",
      "spy"
    ],
    "group": 1,
    "order": 1387
  },
  {
    "emoji": "🕵️‍♀️",
    "label": "woman detective",
    "tags": [
      "detective",
      "sleuth",
      "spy",
      "woman"
    ],
    "group": 1,
    "order": 1401
  },
  {
    "emoji": "💂",
    "label": "guard",
    "tags": [
      "guard"
    ],
    "group": 1,
    "order": 1415
  },
  {
    "emoji": "💂‍♂️",
    "label": "man guard",
    "tags": [
      "guard",
      "man"
    ],
    "group": 1,
    "order": 1421
  },
  {
    "emoji": "💂‍♀️",
    "label": "woman guard",
    "tags": [
      "guard",
      "woman"
    ],
    "group": 1,
    "order": 1433
  },
  {
    "emoji": "🥷",
    "label": "ninja",
    "tags": [
      "fighter",
      "hidden",
      "stealth"
    ],
    "group": 1,
    "order": 1445
  },
  {
    "emoji": "👷",
    "label": "construction worker",
    "tags": [
      "construction",
      "hat",
      "worker"
    ],
    "group": 1,
    "order": 1451
  },
  {
    "emoji": "👷‍♂️",
    "label": "man construction worker",
    "tags": [
      "construction",
      "man",
      "worker"
    ],
    "group": 1,
    "order": 1457
  },
  {
    "emoji": "👷‍♀️",
    "label": "woman construction worker",
    "tags": [
      "construction",
      "woman",
      "worker"
    ],
    "group": 1,
    "order": 1469
  },
  {
    "emoji": "🫅",
    "label": "person with crown",
    "tags": [
      "monarch",
      "noble",
      "regal",
      "royalty"
    ],
    "group": 1,
    "order": 1481
  },
  {
    "emoji": "🤴",
    "label": "prince",
    "tags": [
      "prince"
    ],
    "group": 1,
    "order": 1487
  },
  {
    "emoji": "👸",
    "label": "princess",
    "tags": [
      "fairy tale",
      "fantasy"
    ],
    "group": 1,
    "order": 1493
  },
  {
    "emoji": "👳",
    "label": "person wearing turban",
    "tags": [
      "turban"
    ],
    "group": 1,
    "order": 1499
  },
  {
    "emoji": "👳‍♂️",
    "label": "man wearing turban",
    "tags": [
      "man",
      "turban"
    ],
    "group": 1,
    "order": 1505
  },
  {
    "emoji": "👳‍♀️",
    "label": "woman wearing turban",
    "tags": [
      "turban",
      "woman"
    ],
    "group": 1,
    "order": 1517
  },
  {
    "emoji": "👲",
    "label": "person with skullcap",
    "tags": [
      "cap",
      "gua pi mao",
      "hat",
      "person",
      "skullcap"
    ],
    "group": 1,
    "order": 1529
  },
  {
    "emoji": "🧕",
    "label": "woman with headscarf",
    "tags": [
      "headscarf",
      "hijab",
      "mantilla",
      "tichel"
    ],
    "group": 1,
    "order": 1535
  },
  {
    "emoji": "🤵",
    "label": "person in tuxedo",
    "tags": [
      "groom",
      "person",
      "tuxedo"
    ],
    "group": 1,
    "order": 1541
  },
  {
    "emoji": "🤵‍♂️",
    "label": "man in tuxedo",
    "tags": [
      "man",
      "tuxedo"
    ],
    "group": 1,
    "order": 1547
  },
  {
    "emoji": "🤵‍♀️",
    "label": "woman in tuxedo",
    "tags": [
      "tuxedo",
      "woman"
    ],
    "group": 1,
    "order": 1559
  },
  {
    "emoji": "👰",
    "label": "person with veil",
    "tags": [
      "bride",
      "person",
      "veil",
      "wedding"
    ],
    "group": 1,
    "order": 1571
  },
  {
    "emoji": "👰‍♂️",
    "label": "man with veil",
    "tags": [
      "man",
      "veil"
    ],
    "group": 1,
    "order": 1577
  },
  {
    "emoji": "👰‍♀️",
    "label": "woman with veil",
    "tags": [
      "veil",
      "woman"
    ],
    "group": 1,
    "order": 1589
  },
  {
    "emoji": "🤰",
    "label": "pregnant woman",
    "tags": [
      "pregnant",
      "woman"
    ],
    "group": 1,
    "order": 1601
  },
  {
    "emoji": "🫃",
    "label": "pregnant man",
    "tags": [
      "belly",
      "bloated",
      "full",
      "pregnant"
    ],
    "group": 1,
    "order": 1607
  },
  {
    "emoji": "🫄",
    "label": "pregnant person",
    "tags": [
      "belly",
      "bloated",
      "full",
      "pregnant"
    ],
    "group": 1,
    "order": 1613
  },
  {
    "emoji": "🤱",
    "label": "breast-feeding",
    "tags": [
      "baby",
      "breast",
      "nursing"
    ],
    "group": 1,
    "order": 1619
  },
  {
    "emoji": "👩‍🍼",
    "label": "woman feeding baby",
    "tags": [
      "baby",
      "feeding",
      "nursing",
      "woman"
    ],
    "group": 1,
    "order": 1625
  },
  {
    "emoji": "👨‍🍼",
    "label": "man feeding baby",
    "tags": [
      "baby",
      "feeding",
      "man",
      "nursing"
    ],
    "group": 1,
    "order": 1631
  },
  {
    "emoji": "🧑‍🍼",
    "label": "person feeding baby",
    "tags": [
      "baby",
      "feeding",
      "nursing",
      "person"
    ],
    "group": 1,
    "order": 1637
  },
  {
    "emoji": "👼",
    "label": "baby angel",
    "tags": [
      "angel",
      "baby",
      "face",
      "fairy tale",
      "fantasy"
    ],
    "group": 1,
    "order": 1643
  },
  {
    "emoji": "🎅",
    "label": "Santa Claus",
    "tags": [
      "celebration",
      "christmas",
      "claus",
      "father",
      "santa"
    ],
    "group": 1,
    "order": 1649
  },
  {
    "emoji": "🤶",
    "label": "Mrs. Claus",
    "tags": [
      "celebration",
      "christmas",
      "claus",
      "mother",
      "mrs."
    ],
    "group": 1,
    "order": 1655
  },
  {
    "emoji": "🧑‍🎄",
    "label": "mx claus",
    "tags": [
      "christmas",
      "claus"
    ],
    "group": 1,
    "order": 1661
  },
  {
    "emoji": "🦸",
    "label": "superhero",
    "tags": [
      "good",
      "hero",
      "heroine",
      "superpower"
    ],
    "group": 1,
    "order": 1667
  },
  {
    "emoji": "🦸‍♂️",
    "label": "man superhero",
    "tags": [
      "good",
      "hero",
      "man",
      "superpower"
    ],
    "group": 1,
    "order": 1673
  },
  {
    "emoji": "🦸‍♀️",
    "label": "woman superhero",
    "tags": [
      "good",
      "hero",
      "heroine",
      "superpower",
      "woman"
    ],
    "group": 1,
    "order": 1685
  },
  {
    "emoji": "🦹",
    "label": "supervillain",
    "tags": [
      "criminal",
      "evil",
      "superpower",
      "villain"
    ],
    "group": 1,
    "order": 1697
  },
  {
    "emoji": "🦹‍♂️",
    "label": "man supervillain",
    "tags": [
      "criminal",
      "evil",
      "man",
      "superpower",
      "villain"
    ],
    "group": 1,
    "order": 1703
  },
  {
    "emoji": "🦹‍♀️",
    "label": "woman supervillain",
    "tags": [
      "criminal",
      "evil",
      "superpower",
      "villain",
      "woman"
    ],
    "group": 1,
    "order": 1715
  },
  {
    "emoji": "🧙",
    "label": "mage",
    "tags": [
      "sorcerer",
      "sorceress",
      "witch",
      "wizard"
    ],
    "group": 1,
    "order": 1727
  },
  {
    "emoji": "🧙‍♂️",
    "label": "man mage",
    "tags": [
      "sorcerer",
      "wizard"
    ],
    "group": 1,
    "order": 1733
  },
  {
    "emoji": "🧙‍♀️",
    "label": "woman mage",
    "tags": [
      "sorceress",
      "witch"
    ],
    "group": 1,
    "order": 1745
  },
  {
    "emoji": "🧚",
    "label": "fairy",
    "tags": [
      "oberon",
      "puck",
      "titania"
    ],
    "group": 1,
    "order": 1757
  },
  {
    "emoji": "🧚‍♂️",
    "label": "man fairy",
    "tags": [
      "oberon",
      "puck"
    ],
    "group": 1,
    "order": 1763
  },
  {
    "emoji": "🧚‍♀️",
    "label": "woman fairy",
    "tags": [
      "titania"
    ],
    "group": 1,
    "order": 1775
  },
  {
    "emoji": "🧛",
    "label": "vampire",
    "tags": [
      "dracula",
      "undead"
    ],
    "group": 1,
    "order": 1787
  },
  {
    "emoji": "🧛‍♂️",
    "label": "man vampire",
    "tags": [
      "dracula",
      "undead"
    ],
    "group": 1,
    "order": 1793
  },
  {
    "emoji": "🧛‍♀️",
    "label": "woman vampire",
    "tags": [
      "undead"
    ],
    "group": 1,
    "order": 1805
  },
  {
    "emoji": "🧜",
    "label": "merperson",
    "tags": [
      "mermaid",
      "merman",
      "merwoman"
    ],
    "group": 1,
    "order": 1817
  },
  {
    "emoji": "🧜‍♂️",
    "label": "merman",
    "tags": [
      "triton"
    ],
    "group": 1,
    "order": 1823
  },
  {
    "emoji": "🧜‍♀️",
    "label": "mermaid",
    "tags": [
      "merwoman"
    ],
    "group": 1,
    "order": 1835
  },
  {
    "emoji": "🧝",
    "label": "elf",
    "tags": [
      "magical"
    ],
    "group": 1,
    "order": 1847
  },
  {
    "emoji": "🧝‍♂️",
    "label": "man elf",
    "tags": [
      "magical"
    ],
    "group": 1,
    "order": 1853
  },
  {
    "emoji": "🧝‍♀️",
    "label": "woman elf",
    "tags": [
      "magical"
    ],
    "group": 1,
    "order": 1865
  },
  {
    "emoji": "🧞",
    "label": "genie",
    "tags": [
      "djinn"
    ],
    "group": 1,
    "order": 1877
  },
  {
    "emoji": "🧞‍♂️",
    "label": "man genie",
    "tags": [
      "djinn"
    ],
    "group": 1,
    "order": 1878
  },
  {
    "emoji": "🧞‍♀️",
    "label": "woman genie",
    "tags": [
      "djinn"
    ],
    "group": 1,
    "order": 1880
  },
  {
    "emoji": "🧟",
    "label": "zombie",
    "tags": [
      "undead",
      "walking dead"
    ],
    "group": 1,
    "order": 1882
  },
  {
    "emoji": "🧟‍♂️",
    "label": "man zombie",
    "tags": [
      "undead",
      "walking dead"
    ],
    "group": 1,
    "order": 1883
  },
  {
    "emoji": "🧟‍♀️",
    "label": "woman zombie",
    "tags": [
      "undead",
      "walking dead"
    ],
    "group": 1,
    "order": 1885
  },
  {
    "emoji": "🧌",
    "label": "troll",
    "tags": [
      "fairy tale",
      "fantasy",
      "monster"
    ],
    "group": 1,
    "order": 1887
  },
  {
    "emoji": "💆",
    "label": "person getting massage",
    "tags": [
      "face",
      "massage",
      "salon"
    ],
    "group": 1,
    "order": 1888
  },
  {
    "emoji": "💆‍♂️",
    "label": "man getting massage",
    "tags": [
      "face",
      "man",
      "massage"
    ],
    "group": 1,
    "order": 1894
  },
  {
    "emoji": "💆‍♀️",
    "label": "woman getting massage",
    "tags": [
      "face",
      "massage",
      "woman"
    ],
    "group": 1,
    "order": 1906
  },
  {
    "emoji": "💇",
    "label": "person getting haircut",
    "tags": [
      "barber",
      "beauty",
      "haircut",
      "parlor"
    ],
    "group": 1,
    "order": 1918
  },
  {
    "emoji": "💇‍♂️",
    "label": "man getting haircut",
    "tags": [
      "haircut",
      "man"
    ],
    "group": 1,
    "order": 1924
  },
  {
    "emoji": "💇‍♀️",
    "label": "woman getting haircut",
    "tags": [
      "haircut",
      "woman"
    ],
    "group": 1,
    "order": 1936
  },
  {
    "emoji": "🚶",
    "label": "person walking",
    "tags": [
      "hike",
      "walk",
      "walking"
    ],
    "group": 1,
    "order": 1948
  },
  {
    "emoji": "🚶‍♂️",
    "label": "man walking",
    "tags": [
      "hike",
      "man",
      "walk"
    ],
    "group": 1,
    "order": 1954
  },
  {
    "emoji": "🚶‍♀️",
    "label": "woman walking",
    "tags": [
      "hike",
      "walk",
      "woman"
    ],
    "group": 1,
    "order": 1966
  },
  {
    "emoji": "🚶‍➡️",
    "label": "person walking facing right",
    "tags": [
      "hike",
      "person walking",
      "walk",
      "walking"
    ],
    "group": 1,
    "order": 1978
  },
  {
    "emoji": "🚶‍♀️‍➡️",
    "label": "woman walking facing right",
    "tags": [
      "hike",
      "walk",
      "woman",
      "woman walking"
    ],
    "group": 1,
    "order": 1990
  },
  {
    "emoji": "🚶‍♂️‍➡️",
    "label": "man walking facing right",
    "tags": [
      "hike",
      "man",
      "man walking",
      "walk"
    ],
    "group": 1,
    "order": 2014
  },
  {
    "emoji": "🧍",
    "label": "person standing",
    "tags": [
      "stand",
      "standing"
    ],
    "group": 1,
    "order": 2038
  },
  {
    "emoji": "🧍‍♂️",
    "label": "man standing",
    "tags": [
      "man",
      "standing"
    ],
    "group": 1,
    "order": 2044
  },
  {
    "emoji": "🧍‍♀️",
    "label": "woman standing",
    "tags": [
      "standing",
      "woman"
    ],
    "group": 1,
    "order": 2056
  },
  {
    "emoji": "🧎",
    "label": "person kneeling",
    "tags": [
      "kneel",
      "kneeling"
    ],
    "group": 1,
    "order": 2068
  },
  {
    "emoji": "🧎‍♂️",
    "label": "man kneeling",
    "tags": [
      "kneeling",
      "man"
    ],
    "group": 1,
    "order": 2074
  },
  {
    "emoji": "🧎‍♀️",
    "label": "woman kneeling",
    "tags": [
      "kneeling",
      "woman"
    ],
    "group": 1,
    "order": 2086
  },
  {
    "emoji": "🧎‍➡️",
    "label": "person kneeling facing right",
    "tags": [
      "kneel",
      "kneeling",
      "person kneeling"
    ],
    "group": 1,
    "order": 2098
  },
  {
    "emoji": "🧎‍♀️‍➡️",
    "label": "woman kneeling facing right",
    "tags": [
      "kneeling",
      "woman"
    ],
    "group": 1,
    "order": 2110
  },
  {
    "emoji": "🧎‍♂️‍➡️",
    "label": "man kneeling facing right",
    "tags": [
      "kneeling",
      "man"
    ],
    "group": 1,
    "order": 2134
  },
  {
    "emoji": "🧑‍🦯",
    "label": "person with white cane",
    "tags": [
      "accessibility",
      "blind"
    ],
    "group": 1,
    "order": 2158
  },
  {
    "emoji": "🧑‍🦯‍➡️",
    "label": "person with white cane facing right",
    "tags": [
      "accessibility",
      "blind",
      "person with white cane"
    ],
    "group": 1,
    "order": 2164
  },
  {
    "emoji": "👨‍🦯",
    "label": "man with white cane",
    "tags": [
      "accessibility",
      "blind",
      "man"
    ],
    "group": 1,
    "order": 2176
  },
  {
    "emoji": "👨‍🦯‍➡️",
    "label": "man with white cane facing right",
    "tags": [
      "accessibility",
      "blind",
      "man",
      "man with white cane"
    ],
    "group": 1,
    "order": 2182
  },
  {
    "emoji": "👩‍🦯",
    "label": "woman with white cane",
    "tags": [
      "accessibility",
      "blind",
      "woman"
    ],
    "group": 1,
    "order": 2194
  },
  {
    "emoji": "👩‍🦯‍➡️",
    "label": "woman with white cane facing right",
    "tags": [
      "accessibility",
      "blind",
      "woman",
      "woman with white cane"
    ],
    "group": 1,
    "order": 2200
  },
  {
    "emoji": "🧑‍🦼",
    "label": "person in motorized wheelchair",
    "tags": [
      "accessibility",
      "wheelchair"
    ],
    "group": 1,
    "order": 2212
  },
  {
    "emoji": "🧑‍🦼‍➡️",
    "label": "person in motorized wheelchair facing right",
    "tags": [
      "accessibility",
      "person in motorized wheelchair",
      "wheelchair"
    ],
    "group": 1,
    "order": 2218
  },
  {
    "emoji": "👨‍🦼",
    "label": "man in motorized wheelchair",
    "tags": [
      "accessibility",
      "man",
      "wheelchair"
    ],
    "group": 1,
    "order": 2230
  },
  {
    "emoji": "👨‍🦼‍➡️",
    "label": "man in motorized wheelchair facing right",
    "tags": [
      "accessibility",
      "man",
      "man in motorized wheelchair",
      "wheelchair"
    ],
    "group": 1,
    "order": 2236
  },
  {
    "emoji": "👩‍🦼",
    "label": "woman in motorized wheelchair",
    "tags": [
      "accessibility",
      "wheelchair",
      "woman"
    ],
    "group": 1,
    "order": 2248
  },
  {
    "emoji": "👩‍🦼‍➡️",
    "label": "woman in motorized wheelchair facing right",
    "tags": [
      "accessibility",
      "wheelchair",
      "woman",
      "woman in motorized wheelchair"
    ],
    "group": 1,
    "order": 2254
  },
  {
    "emoji": "🧑‍🦽",
    "label": "person in manual wheelchair",
    "tags": [
      "accessibility",
      "wheelchair"
    ],
    "group": 1,
    "order": 2266
  },
  {
    "emoji": "🧑‍🦽‍➡️",
    "label": "person in manual wheelchair facing right",
    "tags": [
      "accessibility",
      "person in manual wheelchair",
      "wheelchair"
    ],
    "group": 1,
    "order": 2272
  },
  {
    "emoji": "👨‍🦽",
    "label": "man in manual wheelchair",
    "tags": [
      "accessibility",
      "man",
      "wheelchair"
    ],
    "group": 1,
    "order": 2284
  },
  {
    "emoji": "👨‍🦽‍➡️",
    "label": "man in manual wheelchair facing right",
    "tags": [
      "accessibility",
      "man",
      "man in manual wheelchair",
      "wheelchair"
    ],
    "group": 1,
    "order": 2290
  },
  {
    "emoji": "👩‍🦽",
    "label": "woman in manual wheelchair",
    "tags": [
      "accessibility",
      "wheelchair",
      "woman"
    ],
    "group": 1,
    "order": 2302
  },
  {
    "emoji": "👩‍🦽‍➡️",
    "label": "woman in manual wheelchair facing right",
    "tags": [
      "accessibility",
      "wheelchair",
      "woman",
      "woman in manual wheelchair"
    ],
    "group": 1,
    "order": 2308
  },
  {
    "emoji": "🏃",
    "label": "person running",
    "tags": [
      "marathon",
      "running"
    ],
    "group": 1,
    "order": 2320
  },
  {
    "emoji": "🏃‍♂️",
    "label": "man running",
    "tags": [
      "man",
      "marathon",
      "racing",
      "running"
    ],
    "group": 1,
    "order": 2326
  },
  {
    "emoji": "🏃‍♀️",
    "label": "woman running",
    "tags": [
      "marathon",
      "racing",
      "running",
      "woman"
    ],
    "group": 1,
    "order": 2338
  },
  {
    "emoji": "🏃‍➡️",
    "label": "person running facing right",
    "tags": [
      "marathon",
      "person running",
      "running"
    ],
    "group": 1,
    "order": 2350
  },
  {
    "emoji": "🏃‍♀️‍➡️",
    "label": "woman running facing right",
    "tags": [
      "marathon",
      "racing",
      "running",
      "woman"
    ],
    "group": 1,
    "order": 2362
  },
  {
    "emoji": "🏃‍♂️‍➡️",
    "label": "man running facing right",
    "tags": [
      "man",
      "marathon",
      "racing",
      "running"
    ],
    "group": 1,
    "order": 2386
  },
  {
    "emoji": "💃",
    "label": "woman dancing",
    "tags": [
      "dance",
      "dancing",
      "woman"
    ],
    "group": 1,
    "order": 2410
  },
  {
    "emoji": "🕺",
    "label": "man dancing",
    "tags": [
      "dance",
      "dancing",
      "man"
    ],
    "group": 1,
    "order": 2416
  },
  {
    "emoji": "🕴️",
    "label": "person in suit levitating",
    "tags": [
      "business",
      "person",
      "suit"
    ],
    "group": 1,
    "order": 2423
  },
  {
    "emoji": "👯",
    "label": "people with bunny ears",
    "tags": [
      "bunny ear",
      "dancer",
      "partying"
    ],
    "group": 1,
    "order": 2429
  },
  {
    "emoji": "👯‍♂️",
    "label": "men with bunny ears",
    "tags": [
      "bunny ear",
      "dancer",
      "men",
      "partying"
    ],
    "group": 1,
    "order": 2430
  },
  {
    "emoji": "👯‍♀️",
    "label": "women with bunny ears",
    "tags": [
      "bunny ear",
      "dancer",
      "partying",
      "women"
    ],
    "group": 1,
    "order": 2432
  },
  {
    "emoji": "🧖",
    "label": "person in steamy room",
    "tags": [
      "sauna",
      "steam room"
    ],
    "group": 1,
    "order": 2434
  },
  {
    "emoji": "🧖‍♂️",
    "label": "man in steamy room",
    "tags": [
      "sauna",
      "steam room"
    ],
    "group": 1,
    "order": 2440
  },
  {
    "emoji": "🧖‍♀️",
    "label": "woman in steamy room",
    "tags": [
      "sauna",
      "steam room"
    ],
    "group": 1,
    "order": 2452
  },
  {
    "emoji": "🧗",
    "label": "person climbing",
    "tags": [
      "climber"
    ],
    "group": 1,
    "order": 2464
  },
  {
    "emoji": "🧗‍♂️",
    "label": "man climbing",
    "tags": [
      "climber"
    ],
    "group": 1,
    "order": 2470
  },
  {
    "emoji": "🧗‍♀️",
    "label": "woman climbing",
    "tags": [
      "climber"
    ],
    "group": 1,
    "order": 2482
  },
  {
    "emoji": "🤺",
    "label": "person fencing",
    "tags": [
      "fencer",
      "fencing",
      "sword"
    ],
    "group": 1,
    "order": 2494
  },
  {
    "emoji": "🏇",
    "label": "horse racing",
    "tags": [
      "horse",
      "jockey",
      "racehorse",
      "racing"
    ],
    "group": 1,
    "order": 2495
  },
  {
    "emoji": "⛷️",
    "label": "skier",
    "tags": [
      "ski",
      "snow"
    ],
    "group": 1,
    "order": 2502
  },
  {
    "emoji": "🏂️",
    "label": "snowboarder",
    "tags": [
      "ski",
      "snow",
      "snowboard"
    ],
    "group": 1,
    "order": 2503
  },
  {
    "emoji": "🏌️",
    "label": "person golfing",
    "tags": [
      "ball",
      "golf"
    ],
    "group": 1,
    "order": 2510
  },
  {
    "emoji": "🏌️‍♂️",
    "label": "man golfing",
    "tags": [
      "golf",
      "man"
    ],
    "group": 1,
    "order": 2516
  },
  {
    "emoji": "🏌️‍♀️",
    "label": "woman golfing",
    "tags": [
      "golf",
      "woman"
    ],
    "group": 1,
    "order": 2530
  },
  {
    "emoji": "🏄️",
    "label": "person surfing",
    "tags": [
      "surfing"
    ],
    "group": 1,
    "order": 2544
  },
  {
    "emoji": "🏄‍♂️",
    "label": "man surfing",
    "tags": [
      "man",
      "surfing"
    ],
    "group": 1,
    "order": 2550
  },
  {
    "emoji": "🏄‍♀️",
    "label": "woman surfing",
    "tags": [
      "surfing",
      "woman"
    ],
    "group": 1,
    "order": 2562
  },
  {
    "emoji": "🚣",
    "label": "person rowing boat",
    "tags": [
      "boat",
      "rowboat"
    ],
    "group": 1,
    "order": 2574
  },
  {
    "emoji": "🚣‍♂️",
    "label": "man rowing boat",
    "tags": [
      "boat",
      "man",
      "rowboat"
    ],
    "group": 1,
    "order": 2580
  },
  {
    "emoji": "🚣‍♀️",
    "label": "woman rowing boat",
    "tags": [
      "boat",
      "rowboat",
      "woman"
    ],
    "group": 1,
    "order": 2592
  },
  {
    "emoji": "🏊️",
    "label": "person swimming",
    "tags": [
      "swim"
    ],
    "group": 1,
    "order": 2604
  },
  {
    "emoji": "🏊‍♂️",
    "label": "man swimming",
    "tags": [
      "man",
      "swim"
    ],
    "group": 1,
    "order": 2610
  },
  {
    "emoji": "🏊‍♀️",
    "label": "woman swimming",
    "tags": [
      "swim",
      "woman"
    ],
    "group": 1,
    "order": 2622
  },
  {
    "emoji": "⛹️",
    "label": "person bouncing ball",
    "tags": [
      "ball"
    ],
    "group": 1,
    "order": 2635
  },
  {
    "emoji": "⛹️‍♂️",
    "label": "man bouncing ball",
    "tags": [
      "ball",
      "man"
    ],
    "group": 1,
    "order": 2641
  },
  {
    "emoji": "⛹️‍♀️",
    "label": "woman bouncing ball",
    "tags": [
      "ball",
      "woman"
    ],
    "group": 1,
    "order": 2655
  },
  {
    "emoji": "🏋️",
    "label": "person lifting weights",
    "tags": [
      "lifter",
      "weight"
    ],
    "group": 1,
    "order": 2670
  },
  {
    "emoji": "🏋️‍♂️",
    "label": "man lifting weights",
    "tags": [
      "man",
      "weight lifter"
    ],
    "group": 1,
    "order": 2676
  },
  {
    "emoji": "🏋️‍♀️",
    "label": "woman lifting weights",
    "tags": [
      "weight lifter",
      "woman"
    ],
    "group": 1,
    "order": 2690
  },
  {
    "emoji": "🚴",
    "label": "person biking",
    "tags": [
      "bicycle",
      "biking",
      "cyclist"
    ],
    "group": 1,
    "order": 2704
  },
  {
    "emoji": "🚴‍♂️",
    "label": "man biking",
    "tags": [
      "bicycle",
      "biking",
      "cyclist",
      "man"
    ],
    "group": 1,
    "order": 2710
  },
  {
    "emoji": "🚴‍♀️",
    "label": "woman biking",
    "tags": [
      "bicycle",
      "biking",
      "cyclist",
      "woman"
    ],
    "group": 1,
    "order": 2722
  },
  {
    "emoji": "🚵",
    "label": "person mountain biking",
    "tags": [
      "bicycle",
      "bicyclist",
      "bike",
      "cyclist",
      "mountain"
    ],
    "group": 1,
    "order": 2734
  },
  {
    "emoji": "🚵‍♂️",
    "label": "man mountain biking",
    "tags": [
      "bicycle",
      "bike",
      "cyclist",
      "man",
      "mountain"
    ],
    "group": 1,
    "order": 2740
  },
  {
    "emoji": "🚵‍♀️",
    "label": "woman mountain biking",
    "tags": [
      "bicycle",
      "bike",
      "biking",
      "cyclist",
      "mountain",
      "woman"
    ],
    "group": 1,
    "order": 2752
  },
  {
    "emoji": "🤸",
    "label": "person cartwheeling",
    "tags": [
      "cartwheel",
      "gymnastics"
    ],
    "group": 1,
    "order": 2764
  },
  {
    "emoji": "🤸‍♂️",
    "label": "man cartwheeling",
    "tags": [
      "cartwheel",
      "gymnastics",
      "man"
    ],
    "group": 1,
    "order": 2770
  },
  {
    "emoji": "🤸‍♀️",
    "label": "woman cartwheeling",
    "tags": [
      "cartwheel",
      "gymnastics",
      "woman"
    ],
    "group": 1,
    "order": 2782
  },
  {
    "emoji": "🤼",
    "label": "people wrestling",
    "tags": [
      "wrestle",
      "wrestler"
    ],
    "group": 1,
    "order": 2794
  },
  {
    "emoji": "🤼‍♂️",
    "label": "men wrestling",
    "tags": [
      "men",
      "wrestle"
    ],
    "group": 1,
    "order": 2795
  },
  {
    "emoji": "🤼‍♀️",
    "label": "women wrestling",
    "tags": [
      "women",
      "wrestle"
    ],
    "group": 1,
    "order": 2797
  },
  {
    "emoji": "🤽",
    "label": "person playing water polo",
    "tags": [
      "polo",
      "water"
    ],
    "group": 1,
    "order": 2799
  },
  {
    "emoji": "🤽‍♂️",
    "label": "man playing water polo",
    "tags": [
      "man",
      "water polo"
    ],
    "group": 1,
    "order": 2805
  },
  {
    "emoji": "🤽‍♀️",
    "label": "woman playing water polo",
    "tags": [
      "water polo",
      "woman"
    ],
    "group": 1,
    "order": 2817
  },
  {
    "emoji": "🤾",
    "label": "person playing handball",
    "tags": [
      "ball",
      "handball"
    ],
    "group": 1,
    "order": 2829
  },
  {
    "emoji": "🤾‍♂️",
    "label": "man playing handball",
    "tags": [
      "handball",
      "man"
    ],
    "group": 1,
    "order": 2835
  },
  {
    "emoji": "🤾‍♀️",
    "label": "woman playing handball",
    "tags": [
      "handball",
      "woman"
    ],
    "group": 1,
    "order": 2847
  },
  {
    "emoji": "🤹",
    "label": "person juggling",
    "tags": [
      "balance",
      "juggle",
      "multitask",
      "skill"
    ],
    "group": 1,
    "order": 2859
  },
  {
    "emoji": "🤹‍♂️",
    "label": "man juggling",
    "tags": [
      "juggling",
      "man",
      "multitask"
    ],
    "group": 1,
    "order": 2865
  },
  {
    "emoji": "🤹‍♀️",
    "label": "woman juggling",
    "tags": [
      "juggling",
      "multitask",
      "woman"
    ],
    "group": 1,
    "order": 2877
  },
  {
    "emoji": "🧘",
    "label": "person in lotus position",
    "tags": [
      "meditation",
      "yoga"
    ],
    "group": 1,
    "order": 2889
  },
  {
    "emoji": "🧘‍♂️",
    "label": "man in lotus position",
    "tags": [
      "meditation",
      "yoga"
    ],
    "group": 1,
    "order": 2895
  },
  {
    "emoji": "🧘‍♀️",
    "label": "woman in lotus position",
    "tags": [
      "meditation",
      "yoga"
    ],
    "group": 1,
    "order": 2907
  },
  {
    "emoji": "🛀",
    "label": "person taking bath",
    "tags": [
      "bath",
      "bathtub"
    ],
    "group": 1,
    "order": 2919
  },
  {
    "emoji": "🛌",
    "label": "person in bed",
    "tags": [
      "good night",
      "hotel",
      "sleep"
    ],
    "group": 1,
    "order": 2925
  },
  {
    "emoji": "🧑‍🤝‍🧑",
    "label": "people holding hands",
    "tags": [
      "couple",
      "hand",
      "hold",
      "holding hands",
      "person"
    ],
    "group": 1,
    "order": 2931
  },
  {
    "emoji": "👭",
    "label": "women holding hands",
    "tags": [
      "couple",
      "hand",
      "holding hands",
      "women"
    ],
    "group": 1,
    "order": 2957
  },
  {
    "emoji": "👫",
    "label": "woman and man holding hands",
    "tags": [
      "couple",
      "hand",
      "hold",
      "holding hands",
      "man",
      "woman"
    ],
    "group": 1,
    "order": 2983
  },
  {
    "emoji": "👬",
    "label": "men holding hands",
    "tags": [
      "couple",
      "gemini",
      "holding hands",
      "man",
      "men",
      "twins",
      "zodiac"
    ],
    "group": 1,
    "order": 3009
  },
  {
    "emoji": "💏",
    "label": "kiss",
    "tags": [
      "couple"
    ],
    "group": 1,
    "order": 3035
  },
  {
    "emoji": "👩‍❤️‍💋‍👨",
    "label": "kiss: woman, man",
    "tags": [
      "couple",
      "kiss",
      "man",
      "woman"
    ],
    "group": 1,
    "order": 3081
  },
  {
    "emoji": "👨‍❤️‍💋‍👨",
    "label": "kiss: man, man",
    "tags": [
      "couple",
      "kiss",
      "man"
    ],
    "group": 1,
    "order": 3133
  },
  {
    "emoji": "👩‍❤️‍💋‍👩",
    "label": "kiss: woman, woman",
    "tags": [
      "couple",
      "kiss",
      "woman"
    ],
    "group": 1,
    "order": 3185
  },
  {
    "emoji": "💑",
    "label": "couple with heart",
    "tags": [
      "couple",
      "love"
    ],
    "group": 1,
    "order": 3237
  },
  {
    "emoji": "👩‍❤️‍👨",
    "label": "couple with heart: woman, man",
    "tags": [
      "couple",
      "couple with heart",
      "love",
      "man",
      "woman"
    ],
    "group": 1,
    "order": 3283
  },
  {
    "emoji": "👨‍❤️‍👨",
    "label": "couple with heart: man, man",
    "tags": [
      "couple",
      "couple with heart",
      "love",
      "man"
    ],
    "group": 1,
    "order": 3335
  },
  {
    "emoji": "👩‍❤️‍👩",
    "label": "couple with heart: woman, woman",
    "tags": [
      "couple",
      "couple with heart",
      "love",
      "woman"
    ],
    "group": 1,
    "order": 3387
  },
  {
    "emoji": "👨‍👩‍👦",
    "label": "family: man, woman, boy",
    "tags": [
      "boy",
      "family",
      "man",
      "woman"
    ],
    "group": 1,
    "order": 3439
  },
  {
    "emoji": "👨‍👩‍👧",
    "label": "family: man, woman, girl",
    "tags": [
      "family",
      "girl",
      "man",
      "woman"
    ],
    "group": 1,
    "order": 3440
  },
  {
    "emoji": "👨‍👩‍👧‍👦",
    "label": "family: man, woman, girl, boy",
    "tags": [
      "boy",
      "family",
      "girl",
      "man",
      "woman"
    ],
    "group": 1,
    "order": 3441
  },
  {
    "emoji": "👨‍👩‍👦‍👦",
    "label": "family: man, woman, boy, boy",
    "tags": [
      "boy",
      "family",
      "man",
      "woman"
    ],
    "group": 1,
    "order": 3442
  },
  {
    "emoji": "👨‍👩‍👧‍👧",
    "label": "family: man, woman, girl, girl",
    "tags": [
      "family",
      "girl",
      "man",
      "woman"
    ],
    "group": 1,
    "order": 3443
  },
  {
    "emoji": "👨‍👨‍👦",
    "label": "family: man, man, boy",
    "tags": [
      "boy",
      "family",
      "man"
    ],
    "group": 1,
    "order": 3444
  },
  {
    "emoji": "👨‍👨‍👧",
    "label": "family: man, man, girl",
    "tags": [
      "family",
      "girl",
      "man"
    ],
    "group": 1,
    "order": 3445
  },
  {
    "emoji": "👨‍👨‍👧‍👦",
    "label": "family: man, man, girl, boy",
    "tags": [
      "boy",
      "family",
      "girl",
      "man"
    ],
    "group": 1,
    "order": 3446
  },
  {
    "emoji": "👨‍👨‍👦‍👦",
    "label": "family: man, man, boy, boy",
    "tags": [
      "boy",
      "family",
      "man"
    ],
    "group": 1,
    "order": 3447
  },
  {
    "emoji": "👨‍👨‍👧‍👧",
    "label": "family: man, man, girl, girl",
    "tags": [
      "family",
      "girl",
      "man"
    ],
    "group": 1,
    "order": 3448
  },
  {
    "emoji": "👩‍👩‍👦",
    "label": "family: woman, woman, boy",
    "tags": [
      "boy",
      "family",
      "woman"
    ],
    "group": 1,
    "order": 3449
  },
  {
    "emoji": "👩‍👩‍👧",
    "label": "family: woman, woman, girl",
    "tags": [
      "family",
      "girl",
      "woman"
    ],
    "group": 1,
    "order": 3450
  },
  {
    "emoji": "👩‍👩‍👧‍👦",
    "label": "family: woman, woman, girl, boy",
    "tags": [
      "boy",
      "family",
      "girl",
      "woman"
    ],
    "group": 1,
    "order": 3451
  },
  {
    "emoji": "👩‍👩‍👦‍👦",
    "label": "family: woman, woman, boy, boy",
    "tags": [
      "boy",
      "family",
      "woman"
    ],
    "group": 1,
    "order": 3452
  },
  {
    "emoji": "👩‍👩‍👧‍👧",
    "label": "family: woman, woman, girl, girl",
    "tags": [
      "family",
      "girl",
      "woman"
    ],
    "group": 1,
    "order": 3453
  },
  {
    "emoji": "👨‍👦",
    "label": "family: man, boy",
    "tags": [
      "boy",
      "family",
      "man"
    ],
    "group": 1,
    "order": 3454
  },
  {
    "emoji": "👨‍👦‍👦",
    "label": "family: man, boy, boy",
    "tags": [
      "boy",
      "family",
      "man"
    ],
    "group": 1,
    "order": 3455
  },
  {
    "emoji": "👨‍👧",
    "label": "family: man, girl",
    "tags": [
      "family",
      "girl",
      "man"
    ],
    "group": 1,
    "order": 3456
  },
  {
    "emoji": "👨‍👧‍👦",
    "label": "family: man, girl, boy",
    "tags": [
      "boy",
      "family",
      "girl",
      "man"
    ],
    "group": 1,
    "order": 3457
  },
  {
    "emoji": "👨‍👧‍👧",
    "label": "family: man, girl, girl",
    "tags": [
      "family",
      "girl",
      "man"
    ],
    "group": 1,
    "order": 3458
  },
  {
    "emoji": "👩‍👦",
    "label": "family: woman, boy",
    "tags": [
      "boy",
      "family",
      "woman"
    ],
    "group": 1,
    "order": 3459
  },
  {
    "emoji": "👩‍👦‍👦",
    "label": "family: woman, boy, boy",
    "tags": [
      "boy",
      "family",
      "woman"
    ],
    "group": 1,
    "order": 3460
  },
  {
    "emoji": "👩‍👧",
    "label": "family: woman, girl",
    "tags": [
      "family",
      "girl",
      "woman"
    ],
    "group": 1,
    "order": 3461
  },
  {
    "emoji": "👩‍👧‍👦",
    "label": "family: woman, girl, boy",
    "tags": [
      "boy",
      "family",
      "girl",
      "woman"
    ],
    "group": 1,
    "order": 3462
  },
  {
    "emoji": "👩‍👧‍👧",
    "label": "family: woman, girl, girl",
    "tags": [
      "family",
      "girl",
      "woman"
    ],
    "group": 1,
    "order": 3463
  },
  {
    "emoji": "🗣️",
    "label": "speaking head",
    "tags": [
      "face",
      "head",
      "silhouette",
      "speak",
      "speaking"
    ],
    "group": 1,
    "order": 3465
  },
  {
    "emoji": "👤",
    "label": "bust in silhouette",
    "tags": [
      "bust",
      "silhouette"
    ],
    "group": 1,
    "order": 3466
  },
  {
    "emoji": "👥",
    "label": "busts in silhouette",
    "tags": [
      "bust",
      "silhouette"
    ],
    "group": 1,
    "order": 3467
  },
  {
    "emoji": "🫂",
    "label": "people hugging",
    "tags": [
      "goodbye",
      "hello",
      "hug",
      "thanks"
    ],
    "group": 1,
    "order": 3468
  },
  {
    "emoji": "👪️",
    "label": "family",
    "tags": [
      "family"
    ],
    "group": 1,
    "order": 3469
  },
  {
    "emoji": "🧑‍🧑‍🧒",
    "label": "family: adult, adult, child",
    "tags": [
      "family: adult, adult, child"
    ],
    "group": 1,
    "order": 3470
  },
  {
    "emoji": "🧑‍🧑‍🧒‍🧒",
    "label": "family: adult, adult, child, child",
    "tags": [
      "family: adult, adult, child, child"
    ],
    "group": 1,
    "order": 3471
  },
  {
    "emoji": "🧑‍🧒",
    "label": "family: adult, child",
    "tags": [
      "family: adult, child"
    ],
    "group": 1,
    "order": 3472
  },
  {
    "emoji": "🧑‍🧒‍🧒",
    "label": "family: adult, child, child",
    "tags": [
      "family: adult, child, child"
    ],
    "group": 1,
    "order": 3473
  },
  {
    "emoji": "👣",
    "label": "footprints",
    "tags": [
      "clothing",
      "footprint",
      "print"
    ],
    "group": 1,
    "order": 3474
  },
  {
    "emoji": "🏻",
    "label": "light skin tone",
    "tags": [
      "skin tone",
      "type 1–2"
    ],
    "group": 2,
    "order": 3475
  },
  {
    "emoji": "🏼",
    "label": "medium-light skin tone",
    "tags": [
      "skin tone",
      "type 3"
    ],
    "group": 2,
    "order": 3476
  },
  {
    "emoji": "🏽",
    "label": "medium skin tone",
    "tags": [
      "skin tone",
      "type 4"
    ],
    "group": 2,
    "order": 3477
  },
  {
    "emoji": "🏾",
    "label": "medium-dark skin tone",
    "tags": [
      "skin tone",
      "type 5"
    ],
    "group": 2,
    "order": 3478
  },
  {
    "emoji": "🏿",
    "label": "dark skin tone",
    "tags": [
      "skin tone",
      "type 6"
    ],
    "group": 2,
    "order": 3479
  },
  {
    "emoji": "🦰",
    "label": "red hair",
    "tags": [
      "ginger",
      "redhead"
    ],
    "group": 2,
    "order": 3480
  },
  {
    "emoji": "🦱",
    "label": "curly hair",
    "tags": [
      "afro",
      "curly",
      "ringlets"
    ],
    "group": 2,
    "order": 3481
  },
  {
    "emoji": "🦳",
    "label": "white hair",
    "tags": [
      "gray",
      "hair",
      "old",
      "white"
    ],
    "group": 2,
    "order": 3482
  },
  {
    "emoji": "🦲",
    "label": "bald",
    "tags": [
      "chemotherapy",
      "hairless",
      "no hair",
      "shaven"
    ],
    "group": 2,
    "order": 3483
  },
  {
    "emoji": "🐵",
    "label": "monkey face",
    "tags": [
      "face",
      "monkey"
    ],
    "group": 3,
    "order": 3484
  },
  {
    "emoji": "🐒",
    "label": "monkey",
    "tags": [
      "monkey"
    ],
    "group": 3,
    "order": 3485
  },
  {
    "emoji": "🦍",
    "label": "gorilla",
    "tags": [
      "gorilla"
    ],
    "group": 3,
    "order": 3486
  },
  {
    "emoji": "🦧",
    "label": "orangutan",
    "tags": [
      "ape"
    ],
    "group": 3,
    "order": 3487
  },
  {
    "emoji": "🐶",
    "label": "dog face",
    "tags": [
      "dog",
      "face",
      "pet"
    ],
    "group": 3,
    "order": 3488
  },
  {
    "emoji": "🐕️",
    "label": "dog",
    "tags": [
      "pet"
    ],
    "group": 3,
    "order": 3489
  },
  {
    "emoji": "🦮",
    "label": "guide dog",
    "tags": [
      "accessibility",
      "blind",
      "guide"
    ],
    "group": 3,
    "order": 3490
  },
  {
    "emoji": "🐕‍🦺",
    "label": "service dog",
    "tags": [
      "accessibility",
      "assistance",
      "dog",
      "service"
    ],
    "group": 3,
    "order": 3491
  },
  {
    "emoji": "🐩",
    "label": "poodle",
    "tags": [
      "dog"
    ],
    "group": 3,
    "order": 3492
  },
  {
    "emoji": "🐺",
    "label": "wolf",
    "tags": [
      "face"
    ],
    "group": 3,
    "order": 3493
  },
  {
    "emoji": "🦊",
    "label": "fox",
    "tags": [
      "face"
    ],
    "group": 3,
    "order": 3494
  },
  {
    "emoji": "🦝",
    "label": "raccoon",
    "tags": [
      "curious",
      "sly"
    ],
    "group": 3,
    "order": 3495
  },
  {
    "emoji": "🐱",
    "label": "cat face",
    "tags": [
      "cat",
      "face",
      "pet"
    ],
    "group": 3,
    "order": 3496
  },
  {
    "emoji": "🐈️",
    "label": "cat",
    "tags": [
      "pet"
    ],
    "group": 3,
    "order": 3497
  },
  {
    "emoji": "🐈‍⬛",
    "label": "black cat",
    "tags": [
      "black",
      "cat",
      "unlucky"
    ],
    "group": 3,
    "order": 3498
  },
  {
    "emoji": "🦁",
    "label": "lion",
    "tags": [
      "face",
      "leo",
      "zodiac"
    ],
    "group": 3,
    "order": 3499
  },
  {
    "emoji": "🐯",
    "label": "tiger face",
    "tags": [
      "face",
      "tiger"
    ],
    "group": 3,
    "order": 3500
  },
  {
    "emoji": "🐅",
    "label": "tiger",
    "tags": [
      "tiger"
    ],
    "group": 3,
    "order": 3501
  },
  {
    "emoji": "🐆",
    "label": "leopard",
    "tags": [
      "leopard"
    ],
    "group": 3,
    "order": 3502
  },
  {
    "emoji": "🐴",
    "label": "horse face",
    "tags": [
      "face",
      "horse"
    ],
    "group": 3,
    "order": 3503
  },
  {
    "emoji": "🫎",
    "label": "moose",
    "tags": [
      "animal",
      "antlers",
      "elk",
      "mammal"
    ],
    "group": 3,
    "order": 3504
  },
  {
    "emoji": "🫏",
    "label": "donkey",
    "tags": [
      "animal",
      "ass",
      "burro",
      "mammal",
      "mule",
      "stubborn"
    ],
    "group": 3,
    "order": 3505
  },
  {
    "emoji": "🐎",
    "label": "horse",
    "tags": [
      "equestrian",
      "racehorse",
      "racing"
    ],
    "group": 3,
    "order": 3506
  },
  {
    "emoji": "🦄",
    "label": "unicorn",
    "tags": [
      "face"
    ],
    "group": 3,
    "order": 3507
  },
  {
    "emoji": "🦓",
    "label": "zebra",
    "tags": [
      "stripe"
    ],
    "group": 3,
    "order": 3508
  },
  {
    "emoji": "🦌",
    "label": "deer",
    "tags": [
      "deer"
    ],
    "group": 3,
    "order": 3509
  },
  {
    "emoji": "🦬",
    "label": "bison",
    "tags": [
      "buffalo",
      "herd",
      "wisent"
    ],
    "group": 3,
    "order": 3510
  },
  {
    "emoji": "🐮",
    "label": "cow face",
    "tags": [
      "cow",
      "face"
    ],
    "group": 3,
    "order": 3511
  },
  {
    "emoji": "🐂",
    "label": "ox",
    "tags": [
      "bull",
      "taurus",
      "zodiac"
    ],
    "group": 3,
    "order": 3512
  },
  {
    "emoji": "🐃",
    "label": "water buffalo",
    "tags": [
      "buffalo",
      "water"
    ],
    "group": 3,
    "order": 3513
  },
  {
    "emoji": "🐄",
    "label": "cow",
    "tags": [
      "cow"
    ],
    "group": 3,
    "order": 3514
  },
  {
    "emoji": "🐷",
    "label": "pig face",
    "tags": [
      "face",
      "pig"
    ],
    "group": 3,
    "order": 3515
  },
  {
    "emoji": "🐖",
    "label": "pig",
    "tags": [
      "sow"
    ],
    "group": 3,
    "order": 3516
  },
  {
    "emoji": "🐗",
    "label": "boar",
    "tags": [
      "pig"
    ],
    "group": 3,
    "order": 3517
  },
  {
    "emoji": "🐽",
    "label": "pig nose",
    "tags": [
      "face",
      "nose",
      "pig"
    ],
    "group": 3,
    "order": 3518
  },
  {
    "emoji": "🐏",
    "label": "ram",
    "tags": [
      "aries",
      "male",
      "sheep",
      "zodiac"
    ],
    "group": 3,
    "order": 3519
  },
  {
    "emoji": "🐑",
    "label": "ewe",
    "tags": [
      "female",
      "sheep"
    ],
    "group": 3,
    "order": 3520
  },
  {
    "emoji": "🐐",
    "label": "goat",
    "tags": [
      "capricorn",
      "zodiac"
    ],
    "group": 3,
    "order": 3521
  },
  {
    "emoji": "🐪",
    "label": "camel",
    "tags": [
      "dromedary",
      "hump"
    ],
    "group": 3,
    "order": 3522
  },
  {
    "emoji": "🐫",
    "label": "two-hump camel",
    "tags": [
      "bactrian",
      "camel",
      "hump"
    ],
    "group": 3,
    "order": 3523
  },
  {
    "emoji": "🦙",
    "label": "llama",
    "tags": [
      "alpaca",
      "guanaco",
      "vicuña",
      "wool"
    ],
    "group": 3,
    "order": 3524
  },
  {
    "emoji": "🦒",
    "label": "giraffe",
    "tags": [
      "spots"
    ],
    "group": 3,
    "order": 3525
  },
  {
    "emoji": "🐘",
    "label": "elephant",
    "tags": [
      "elephant"
    ],
    "group": 3,
    "order": 3526
  },
  {
    "emoji": "🦣",
    "label": "mammoth",
    "tags": [
      "extinction",
      "large",
      "tusk",
      "woolly"
    ],
    "group": 3,
    "order": 3527
  },
  {
    "emoji": "🦏",
    "label": "rhinoceros",
    "tags": [
      "rhinoceros"
    ],
    "group": 3,
    "order": 3528
  },
  {
    "emoji": "🦛",
    "label": "hippopotamus",
    "tags": [
      "hippo"
    ],
    "group": 3,
    "order": 3529
  },
  {
    "emoji": "🐭",
    "label": "mouse face",
    "tags": [
      "face",
      "mouse"
    ],
    "group": 3,
    "order": 3530
  },
  {
    "emoji": "🐁",
    "label": "mouse",
    "tags": [
      "mouse"
    ],
    "group": 3,
    "order": 3531
  },
  {
    "emoji": "🐀",
    "label": "rat",
    "tags": [
      "rat"
    ],
    "group": 3,
    "order": 3532
  },
  {
    "emoji": "🐹",
    "label": "hamster",
    "tags": [
      "face",
      "pet"
    ],
    "group": 3,
    "order": 3533
  },
  {
    "emoji": "🐰",
    "label": "rabbit face",
    "tags": [
      "bunny",
      "face",
      "pet",
      "rabbit"
    ],
    "group": 3,
    "order": 3534
  },
  {
    "emoji": "🐇",
    "label": "rabbit",
    "tags": [
      "bunny",
      "pet"
    ],
    "group": 3,
    "order": 3535
  },
  {
    "emoji": "🐿️",
    "label": "chipmunk",
    "tags": [
      "squirrel"
    ],
    "group": 3,
    "order": 3537
  },
  {
    "emoji": "🦫",
    "label": "beaver",
    "tags": [
      "dam"
    ],
    "group": 3,
    "order": 3538
  },
  {
    "emoji": "🦔",
    "label": "hedgehog",
    "tags": [
      "spiny"
    ],
    "group": 3,
    "order": 3539
  },
  {
    "emoji": "🦇",
    "label": "bat",
    "tags": [
      "vampire"
    ],
    "group": 3,
    "order": 3540
  },
  {
    "emoji": "🐻",
    "label": "bear",
    "tags": [
      "face"
    ],
    "group": 3,
    "order": 3541
  },
  {
    "emoji": "🐻‍❄️",
    "label": "polar bear",
    "tags": [
      "arctic",
      "bear",
      "white"
    ],
    "group": 3,
    "order": 3542
  },
  {
    "emoji": "🐨",
    "label": "koala",
    "tags": [
      "face",
      "marsupial"
    ],
    "group": 3,
    "order": 3544
  },
  {
    "emoji": "🐼",
    "label": "panda",
    "tags": [
      "face"
    ],
    "group": 3,
    "order": 3545
  },
  {
    "emoji": "🦥",
    "label": "sloth",
    "tags": [
      "lazy",
      "slow"
    ],
    "group": 3,
    "order": 3546
  },
  {
    "emoji": "🦦",
    "label": "otter",
    "tags": [
      "fishing",
      "playful"
    ],
    "group": 3,
    "order": 3547
  },
  {
    "emoji": "🦨",
    "label": "skunk",
    "tags": [
      "stink"
    ],
    "group": 3,
    "order": 3548
  },
  {
    "emoji": "🦘",
    "label": "kangaroo",
    "tags": [
      "joey",
      "jump",
      "marsupial"
    ],
    "group": 3,
    "order": 3549
  },
  {
    "emoji": "🦡",
    "label": "badger",
    "tags": [
      "honey badger",
      "pester"
    ],
    "group": 3,
    "order": 3550
  },
  {
    "emoji": "🐾",
    "label": "paw prints",
    "tags": [
      "feet",
      "paw",
      "print"
    ],
    "group": 3,
    "order": 3551
  },
  {
    "emoji": "🦃",
    "label": "turkey",
    "tags": [
      "bird"
    ],
    "group": 3,
    "order": 3552
  },
  {
    "emoji": "🐔",
    "label": "chicken",
    "tags": [
      "bird"
    ],
    "group": 3,
    "order": 3553
  },
  {
    "emoji": "🐓",
    "label": "rooster",
    "tags": [
      "bird"
    ],
    "group": 3,
    "order": 3554
  },
  {
    "emoji": "🐣",
    "label": "hatching chick",
    "tags": [
      "baby",
      "bird",
      "chick",
      "hatching"
    ],
    "group": 3,
    "order": 3555
  },
  {
    "emoji": "🐤",
    "label": "baby chick",
    "tags": [
      "baby",
      "bird",
      "chick"
    ],
    "group": 3,
    "order": 3556
  },
  {
    "emoji": "🐥",
    "label": "front-facing baby chick",
    "tags": [
      "baby",
      "bird",
      "chick"
    ],
    "group": 3,
    "order": 3557
  },
  {
    "emoji": "🐦️",
    "label": "bird",
    "tags": [
      "bird"
    ],
    "group": 3,
    "order": 3558
  },
  {
    "emoji": "🐧",
    "label": "penguin",
    "tags": [
      "bird"
    ],
    "group": 3,
    "order": 3559
  },
  {
    "emoji": "🕊️",
    "label": "dove",
    "tags": [
      "bird",
      "fly",
      "peace"
    ],
    "group": 3,
    "order": 3561
  },
  {
    "emoji": "🦅",
    "label": "eagle",
    "tags": [
      "bird"
    ],
    "group": 3,
    "order": 3562
  },
  {
    "emoji": "🦆",
    "label": "duck",
    "tags": [
      "bird"
    ],
    "group": 3,
    "order": 3563
  },
  {
    "emoji": "🦢",
    "label": "swan",
    "tags": [
      "bird",
      "cygnet",
      "ugly duckling"
    ],
    "group": 3,
    "order": 3564
  },
  {
    "emoji": "🦉",
    "label": "owl",
    "tags": [
      "bird",
      "wise"
    ],
    "group": 3,
    "order": 3565
  },
  {
    "emoji": "🦤",
    "label": "dodo",
    "tags": [
      "extinction",
      "large",
      "mauritius"
    ],
    "group": 3,
    "order": 3566
  },
  {
    "emoji": "🪶",
    "label": "feather",
    "tags": [
      "bird",
      "flight",
      "light",
      "plumage"
    ],
    "group": 3,
    "order": 3567
  },
  {
    "emoji": "🦩",
    "label": "flamingo",
    "tags": [
      "flamboyant",
      "tropical"
    ],
    "group": 3,
    "order": 3568
  },
  {
    "emoji": "🦚",
    "label": "peacock",
    "tags": [
      "bird",
      "ostentatious",
      "peahen",
      "proud"
    ],
    "group": 3,
    "order": 3569
  },
  {
    "emoji": "🦜",
    "label": "parrot",
    "tags": [
      "bird",
      "pirate",
      "talk"
    ],
    "group": 3,
    "order": 3570
  },
  {
    "emoji": "🪽",
    "label": "wing",
    "tags": [
      "angelic",
      "aviation",
      "bird",
      "flying",
      "mythology"
    ],
    "group": 3,
    "order": 3571
  },
  {
    "emoji": "🐦‍⬛",
    "label": "black bird",
    "tags": [
      "bird",
      "black",
      "crow",
      "raven",
      "rook"
    ],
    "group": 3,
    "order": 3572
  },
  {
    "emoji": "🪿",
    "label": "goose",
    "tags": [
      "bird",
      "fowl",
      "honk",
      "silly"
    ],
    "group": 3,
    "order": 3573
  },
  {
    "emoji": "🐦‍🔥",
    "label": "phoenix",
    "tags": [
      "fantasy",
      "firebird",
      "rebirth",
      "reincarnation"
    ],
    "group": 3,
    "order": 3574
  },
  {
    "emoji": "🐸",
    "label": "frog",
    "tags": [
      "face"
    ],
    "group": 3,
    "order": 3575
  },
  {
    "emoji": "🐊",
    "label": "crocodile",
    "tags": [
      "crocodile"
    ],
    "group": 3,
    "order": 3576
  },
  {
    "emoji": "🐢",
    "label": "turtle",
    "tags": [
      "terrapin",
      "tortoise"
    ],
    "group": 3,
    "order": 3577
  },
  {
    "emoji": "🦎",
    "label": "lizard",
    "tags": [
      "reptile"
    ],
    "group": 3,
    "order": 3578
  },
  {
    "emoji": "🐍",
    "label": "snake",
    "tags": [
      "bearer",
      "ophiuchus",
      "serpent",
      "zodiac"
    ],
    "group": 3,
    "order": 3579
  },
  {
    "emoji": "🐲",
    "label": "dragon face",
    "tags": [
      "dragon",
      "face",
      "fairy tale"
    ],
    "group": 3,
    "order": 3580
  },
  {
    "emoji": "🐉",
    "label": "dragon",
    "tags": [
      "fairy tale"
    ],
    "group": 3,
    "order": 3581
  },
  {
    "emoji": "🦕",
    "label": "sauropod",
    "tags": [
      "brachiosaurus",
      "brontosaurus",
      "diplodocus"
    ],
    "group": 3,
    "order": 3582
  },
  {
    "emoji": "🦖",
    "label": "T-Rex",
    "tags": [
      "t-rex",
      "tyrannosaurus rex"
    ],
    "group": 3,
    "order": 3583
  },
  {
    "emoji": "🐳",
    "label": "spouting whale",
    "tags": [
      "face",
      "spouting",
      "whale"
    ],
    "group": 3,
    "order": 3584
  },
  {
    "emoji": "🐋",
    "label": "whale",
    "tags": [
      "whale"
    ],
    "group": 3,
    "order": 3585
  },
  {
    "emoji": "🐬",
    "label": "dolphin",
    "tags": [
      "flipper"
    ],
    "group": 3,
    "order": 3586
  },
  {
    "emoji": "🦭",
    "label": "seal",
    "tags": [
      "sea lion"
    ],
    "group": 3,
    "order": 3587
  },
  {
    "emoji": "🐟️",
    "label": "fish",
    "tags": [
      "pisces",
      "zodiac"
    ],
    "group": 3,
    "order": 3588
  },
  {
    "emoji": "🐠",
    "label": "tropical fish",
    "tags": [
      "fish",
      "tropical"
    ],
    "group": 3,
    "order": 3589
  },
  {
    "emoji": "🐡",
    "label": "blowfish",
    "tags": [
      "fish"
    ],
    "group": 3,
    "order": 3590
  },
  {
    "emoji": "🦈",
    "label": "shark",
    "tags": [
      "fish"
    ],
    "group": 3,
    "order": 3591
  },
  {
    "emoji": "🐙",
    "label": "octopus",
    "tags": [
      "octopus"
    ],
    "group": 3,
    "order": 3592
  },
  {
    "emoji": "🐚",
    "label": "spiral shell",
    "tags": [
      "shell",
      "spiral"
    ],
    "group": 3,
    "order": 3593
  },
  {
    "emoji": "🪸",
    "label": "coral",
    "tags": [
      "ocean",
      "reef"
    ],
    "group": 3,
    "order": 3594
  },
  {
    "emoji": "🪼",
    "label": "jellyfish",
    "tags": [
      "burn",
      "invertebrate",
      "jelly",
      "marine",
      "ouch",
      "stinger"
    ],
    "group": 3,
    "order": 3595
  },
  {
    "emoji": "🐌",
    "label": "snail",
    "tags": [
      "snail"
    ],
    "group": 3,
    "order": 3596
  },
  {
    "emoji": "🦋",
    "label": "butterfly",
    "tags": [
      "insect",
      "pretty"
    ],
    "group": 3,
    "order": 3597
  },
  {
    "emoji": "🐛",
    "label": "bug",
    "tags": [
      "insect"
    ],
    "group": 3,
    "order": 3598
  },
  {
    "emoji": "🐜",
    "label": "ant",
    "tags": [
      "insect"
    ],
    "group": 3,
    "order": 3599
  },
  {
    "emoji": "🐝",
    "label": "honeybee",
    "tags": [
      "bee",
      "insect"
    ],
    "group": 3,
    "order": 3600
  },
  {
    "emoji": "🪲",
    "label": "beetle",
    "tags": [
      "bug",
      "insect"
    ],
    "group": 3,
    "order": 3601
  },
  {
    "emoji": "🐞",
    "label": "lady beetle",
    "tags": [
      "beetle",
      "insect",
      "ladybird",
      "ladybug"
    ],
    "group": 3,
    "order": 3602
  },
  {
    "emoji": "🦗",
    "label": "cricket",
    "tags": [
      "grasshopper"
    ],
    "group": 3,
    "order": 3603
  },
  {
    "emoji": "🪳",
    "label": "cockroach",
    "tags": [
      "insect",
      "pest",
      "roach"
    ],
    "group": 3,
    "order": 3604
  },
  {
    "emoji": "🕷️",
    "label": "spider",
    "tags": [
      "insect"
    ],
    "group": 3,
    "order": 3606
  },
  {
    "emoji": "🕸️",
    "label": "spider web",
    "tags": [
      "spider",
      "web"
    ],
    "group": 3,
    "order": 3608
  },
  {
    "emoji": "🦂",
    "label": "scorpion",
    "tags": [
      "scorpio",
      "zodiac"
    ],
    "group": 3,
    "order": 3609
  },
  {
    "emoji": "🦟",
    "label": "mosquito",
    "tags": [
      "disease",
      "fever",
      "malaria",
      "pest",
      "virus"
    ],
    "group": 3,
    "order": 3610
  },
  {
    "emoji": "🪰",
    "label": "fly",
    "tags": [
      "disease",
      "maggot",
      "pest",
      "rotting"
    ],
    "group": 3,
    "order": 3611
  },
  {
    "emoji": "🪱",
    "label": "worm",
    "tags": [
      "annelid",
      "earthworm",
      "parasite"
    ],
    "group": 3,
    "order": 3612
  },
  {
    "emoji": "🦠",
    "label": "microbe",
    "tags": [
      "amoeba",
      "bacteria",
      "virus"
    ],
    "group": 3,
    "order": 3613
  },
  {
    "emoji": "💐",
    "label": "bouquet",
    "tags": [
      "flower"
    ],
    "group": 3,
    "order": 3614
  },
  {
    "emoji": "🌸",
    "label": "cherry blossom",
    "tags": [
      "blossom",
      "cherry",
      "flower"
    ],
    "group": 3,
    "order": 3615
  },
  {
    "emoji": "💮",
    "label": "white flower",
    "tags": [
      "flower"
    ],
    "group": 3,
    "order": 3616
  },
  {
    "emoji": "🪷",
    "label": "lotus",
    "tags": [
      "buddhism",
      "flower",
      "hinduism",
      "purity"
    ],
    "group": 3,
    "order": 3617
  },
  {
    "emoji": "🏵️",
    "label": "rosette",
    "tags": [
      "plant"
    ],
    "group": 3,
    "order": 3619
  },
  {
    "emoji": "🌹",
    "label": "rose",
    "tags": [
      "flower"
    ],
    "group": 3,
    "order": 3620
  },
  {
    "emoji": "🥀",
    "label": "wilted flower",
    "tags": [
      "flower",
      "wilted"
    ],
    "group": 3,
    "order": 3621
  },
  {
    "emoji": "🌺",
    "label": "hibiscus",
    "tags": [
      "flower"
    ],
    "group": 3,
    "order": 3622
  },
  {
    "emoji": "🌻",
    "label": "sunflower",
    "tags": [
      "flower",
      "sun"
    ],
    "group": 3,
    "order": 3623
  },
  {
    "emoji": "🌼",
    "label": "blossom",
    "tags": [
      "flower"
    ],
    "group": 3,
    "order": 3624
  },
  {
    "emoji": "🌷",
    "label": "tulip",
    "tags": [
      "flower"
    ],
    "group": 3,
    "order": 3625
  },
  {
    "emoji": "🪻",
    "label": "hyacinth",
    "tags": [
      "bluebonnet",
      "flower",
      "lavender",
      "lupine",
      "snapdragon"
    ],
    "group": 3,
    "order": 3626
  },
  {
    "emoji": "🌱",
    "label": "seedling",
    "tags": [
      "young"
    ],
    "group": 3,
    "order": 3627
  },
  {
    "emoji": "🪴",
    "label": "potted plant",
    "tags": [
      "boring",
      "grow",
      "house",
      "nurturing",
      "plant",
      "useless"
    ],
    "group": 3,
    "order": 3628
  },
  {
    "emoji": "🌲",
    "label": "evergreen tree",
    "tags": [
      "tree"
    ],
    "group": 3,
    "order": 3629
  },
  {
    "emoji": "🌳",
    "label": "deciduous tree",
    "tags": [
      "deciduous",
      "shedding",
      "tree"
    ],
    "group": 3,
    "order": 3630
  },
  {
    "emoji": "🌴",
    "label": "palm tree",
    "tags": [
      "palm",
      "tree"
    ],
    "group": 3,
    "order": 3631
  },
  {
    "emoji": "🌵",
    "label": "cactus",
    "tags": [
      "plant"
    ],
    "group": 3,
    "order": 3632
  },
  {
    "emoji": "🌾",
    "label": "sheaf of rice",
    "tags": [
      "ear",
      "grain",
      "rice"
    ],
    "group": 3,
    "order": 3633
  },
  {
    "emoji": "🌿",
    "label": "herb",
    "tags": [
      "leaf"
    ],
    "group": 3,
    "order": 3634
  },
  {
    "emoji": "☘️",
    "label": "shamrock",
    "tags": [
      "plant"
    ],
    "group": 3,
    "order": 3636
  },
  {
    "emoji": "🍀",
    "label": "four leaf clover",
    "tags": [
      "4",
      "clover",
      "four",
      "four-leaf clover",
      "leaf"
    ],
    "group": 3,
    "order": 3637
  },
  {
    "emoji": "🍁",
    "label": "maple leaf",
    "tags": [
      "falling",
      "leaf",
      "maple"
    ],
    "group": 3,
    "order": 3638
  },
  {
    "emoji": "🍂",
    "label": "fallen leaf",
    "tags": [
      "falling",
      "leaf"
    ],
    "group": 3,
    "order": 3639
  },
  {
    "emoji": "🍃",
    "label": "leaf fluttering in wind",
    "tags": [
      "blow",
      "flutter",
      "leaf",
      "wind"
    ],
    "group": 3,
    "order": 3640
  },
  {
    "emoji": "🪹",
    "label": "empty nest",
    "tags": [
      "nesting"
    ],
    "group": 3,
    "order": 3641
  },
  {
    "emoji": "🪺",
    "label": "nest with eggs",
    "tags": [
      "nesting"
    ],
    "group": 3,
    "order": 3642
  },
  {
    "emoji": "🍄",
    "label": "mushroom",
    "tags": [
      "toadstool"
    ],
    "group": 3,
    "order": 3643
  },
  {
    "emoji": "🍇",
    "label": "grapes",
    "tags": [
      "fruit",
      "grape"
    ],
    "group": 4,
    "order": 3644
  },
  {
    "emoji": "🍈",
    "label": "melon",
    "tags": [
      "fruit"
    ],
    "group": 4,
    "order": 3645
  },
  {
    "emoji": "🍉",
    "label": "watermelon",
    "tags": [
      "fruit"
    ],
    "group": 4,
    "order": 3646
  },
  {
    "emoji": "🍊",
    "label": "tangerine",
    "tags": [
      "fruit",
      "orange"
    ],
    "group": 4,
    "order": 3647
  },
  {
    "emoji": "🍋",
    "label": "lemon",
    "tags": [
      "citrus",
      "fruit"
    ],
    "group": 4,
    "order": 3648
  },
  {
    "emoji": "🍋‍🟩",
    "label": "lime",
    "tags": [
      "citrus",
      "fruit",
      "tropical"
    ],
    "group": 4,
    "order": 3649
  },
  {
    "emoji": "🍌",
    "label": "banana",
    "tags": [
      "fruit"
    ],
    "group": 4,
    "order": 3650
  },
  {
    "emoji": "🍍",
    "label": "pineapple",
    "tags": [
      "fruit"
    ],
    "group": 4,
    "order": 3651
  },
  {
    "emoji": "🥭",
    "label": "mango",
    "tags": [
      "fruit",
      "tropical"
    ],
    "group": 4,
    "order": 3652
  },
  {
    "emoji": "🍎",
    "label": "red apple",
    "tags": [
      "apple",
      "fruit",
      "red"
    ],
    "group": 4,
    "order": 3653
  },
  {
    "emoji": "🍏",
    "label": "green apple",
    "tags": [
      "apple",
      "fruit",
      "green"
    ],
    "group": 4,
    "order": 3654
  },
  {
    "emoji": "🍐",
    "label": "pear",
    "tags": [
      "fruit"
    ],
    "group": 4,
    "order": 3655
  },
  {
    "emoji": "🍑",
    "label": "peach",
    "tags": [
      "fruit"
    ],
    "group": 4,
    "order": 3656
  },
  {
    "emoji": "🍒",
    "label": "cherries",
    "tags": [
      "berries",
      "cherry",
      "fruit",
      "red"
    ],
    "group": 4,
    "order": 3657
  },
  {
    "emoji": "🍓",
    "label": "strawberry",
    "tags": [
      "berry",
      "fruit"
    ],
    "group": 4,
    "order": 3658
  },
  {
    "emoji": "🫐",
    "label": "blueberries",
    "tags": [
      "berry",
      "bilberry",
      "blue",
      "blueberry"
    ],
    "group": 4,
    "order": 3659
  },
  {
    "emoji": "🥝",
    "label": "kiwi fruit",
    "tags": [
      "food",
      "fruit",
      "kiwi"
    ],
    "group": 4,
    "order": 3660
  },
  {
    "emoji": "🍅",
    "label": "tomato",
    "tags": [
      "fruit",
      "vegetable"
    ],
    "group": 4,
    "order": 3661
  },
  {
    "emoji": "🫒",
    "label": "olive",
    "tags": [
      "food"
    ],
    "group": 4,
    "order": 3662
  },
  {
    "emoji": "🥥",
    "label": "coconut",
    "tags": [
      "palm",
      "piña colada"
    ],
    "group": 4,
    "order": 3663
  },
  {
    "emoji": "🥑",
    "label": "avocado",
    "tags": [
      "food",
      "fruit"
    ],
    "group": 4,
    "order": 3664
  },
  {
    "emoji": "🍆",
    "label": "eggplant",
    "tags": [
      "aubergine",
      "vegetable"
    ],
    "group": 4,
    "order": 3665
  },
  {
    "emoji": "🥔",
    "label": "potato",
    "tags": [
      "food",
      "vegetable"
    ],
    "group": 4,
    "order": 3666
  },
  {
    "emoji": "🥕",
    "label": "carrot",
    "tags": [
      "food",
      "vegetable"
    ],
    "group": 4,
    "order": 3667
  },
  {
    "emoji": "🌽",
    "label": "ear of corn",
    "tags": [
      "corn",
      "ear",
      "maize",
      "maze"
    ],
    "group": 4,
    "order": 3668
  },
  {
    "emoji": "🌶️",
    "label": "hot pepper",
    "tags": [
      "hot",
      "pepper"
    ],
    "group": 4,
    "order": 3670
  },
  {
    "emoji": "🫑",
    "label": "bell pepper",
    "tags": [
      "capsicum",
      "pepper",
      "vegetable"
    ],
    "group": 4,
    "order": 3671
  },
  {
    "emoji": "🥒",
    "label": "cucumber",
    "tags": [
      "food",
      "pickle",
      "vegetable"
    ],
    "group": 4,
    "order": 3672
  },
  {
    "emoji": "🥬",
    "label": "leafy green",
    "tags": [
      "bok choy",
      "cabbage",
      "kale",
      "lettuce"
    ],
    "group": 4,
    "order": 3673
  },
  {
    "emoji": "🥦",
    "label": "broccoli",
    "tags": [
      "wild cabbage"
    ],
    "group": 4,
    "order": 3674
  },
  {
    "emoji": "🧄",
    "label": "garlic",
    "tags": [
      "flavoring"
    ],
    "group": 4,
    "order": 3675
  },
  {
    "emoji": "🧅",
    "label": "onion",
    "tags": [
      "flavoring"
    ],
    "group": 4,
    "order": 3676
  },
  {
    "emoji": "🥜",
    "label": "peanuts",
    "tags": [
      "food",
      "nut",
      "peanut",
      "vegetable"
    ],
    "group": 4,
    "order": 3677
  },
  {
    "emoji": "🫘",
    "label": "beans",
    "tags": [
      "food",
      "kidney",
      "legume"
    ],
    "group": 4,
    "order": 3678
  },
  {
    "emoji": "🌰",
    "label": "chestnut",
    "tags": [
      "plant"
    ],
    "group": 4,
    "order": 3679
  },
  {
    "emoji": "🫚",
    "label": "ginger root",
    "tags": [
      "beer",
      "root",
      "spice"
    ],
    "group": 4,
    "order": 3680
  },
  {
    "emoji": "🫛",
    "label": "pea pod",
    "tags": [
      "beans",
      "edamame",
      "legume",
      "pea",
      "pod",
      "vegetable"
    ],
    "group": 4,
    "order": 3681
  },
  {
    "emoji": "🍄‍🟫",
    "label": "brown mushroom",
    "tags": [
      "food",
      "fungus",
      "nature",
      "vegetable"
    ],
    "group": 4,
    "order": 3682
  },
  {
    "emoji": "🍞",
    "label": "bread",
    "tags": [
      "loaf"
    ],
    "group": 4,
    "order": 3683
  },
  {
    "emoji": "🥐",
    "label": "croissant",
    "tags": [
      "bread",
      "breakfast",
      "food",
      "french",
      "roll"
    ],
    "group": 4,
    "order": 3684
  },
  {
    "emoji": "🥖",
    "label": "baguette bread",
    "tags": [
      "baguette",
      "bread",
      "food",
      "french"
    ],
    "group": 4,
    "order": 3685
  },
  {
    "emoji": "🫓",
    "label": "flatbread",
    "tags": [
      "arepa",
      "lavash",
      "naan",
      "pita"
    ],
    "group": 4,
    "order": 3686
  },
  {
    "emoji": "🥨",
    "label": "pretzel",
    "tags": [
      "twisted"
    ],
    "group": 4,
    "order": 3687
  },
  {
    "emoji": "🥯",
    "label": "bagel",
    "tags": [
      "bakery",
      "breakfast",
      "schmear"
    ],
    "group": 4,
    "order": 3688
  },
  {
    "emoji": "🥞",
    "label": "pancakes",
    "tags": [
      "breakfast",
      "crêpe",
      "food",
      "hotcake",
      "pancake"
    ],
    "group": 4,
    "order": 3689
  },
  {
    "emoji": "🧇",
    "label": "waffle",
    "tags": [
      "breakfast",
      "indecisive",
      "iron"
    ],
    "group": 4,
    "order": 3690
  },
  {
    "emoji": "🧀",
    "label": "cheese wedge",
    "tags": [
      "cheese"
    ],
    "group": 4,
    "order": 3691
  },
  {
    "emoji": "🍖",
    "label": "meat on bone",
    "tags": [
      "bone",
      "meat"
    ],
    "group": 4,
    "order": 3692
  },
  {
    "emoji": "🍗",
    "label": "poultry leg",
    "tags": [
      "bone",
      "chicken",
      "drumstick",
      "leg",
      "poultry"
    ],
    "group": 4,
    "order": 3693
  },
  {
    "emoji": "🥩",
    "label": "cut of meat",
    "tags": [
      "chop",
      "lambchop",
      "porkchop",
      "steak"
    ],
    "group": 4,
    "order": 3694
  },
  {
    "emoji": "🥓",
    "label": "bacon",
    "tags": [
      "breakfast",
      "food",
      "meat"
    ],
    "group": 4,
    "order": 3695
  },
  {
    "emoji": "🍔",
    "label": "hamburger",
    "tags": [
      "burger"
    ],
    "group": 4,
    "order": 3696
  },
  {
    "emoji": "🍟",
    "label": "french fries",
    "tags": [
      "french",
      "fries"
    ],
    "group": 4,
    "order": 3697
  },
  {
    "emoji": "🍕",
    "label": "pizza",
    "tags": [
      "cheese",
      "slice"
    ],
    "group": 4,
    "order": 3698
  },
  {
    "emoji": "🌭",
    "label": "hot dog",
    "tags": [
      "frankfurter",
      "hotdog",
      "sausage"
    ],
    "group": 4,
    "order": 3699
  },
  {
    "emoji": "🥪",
    "label": "sandwich",
    "tags": [
      "bread"
    ],
    "group": 4,
    "order": 3700
  },
  {
    "emoji": "🌮",
    "label": "taco",
    "tags": [
      "mexican"
    ],
    "group": 4,
    "order": 3701
  },
  {
    "emoji": "🌯",
    "label": "burrito",
    "tags": [
      "mexican",
      "wrap"
    ],
    "group": 4,
    "order": 3702
  },
  {
    "emoji": "🫔",
    "label": "tamale",
    "tags": [
      "mexican",
      "wrapped"
    ],
    "group": 4,
    "order": 3703
  },
  {
    "emoji": "🥙",
    "label": "stuffed flatbread",
    "tags": [
      "falafel",
      "flatbread",
      "food",
      "gyro",
      "kebab",
      "stuffed"
    ],
    "group": 4,
    "order": 3704
  },
  {
    "emoji": "🧆",
    "label": "falafel",
    "tags": [
      "chickpea",
      "meatball"
    ],
    "group": 4,
    "order": 3705
  },
  {
    "emoji": "🥚",
    "label": "egg",
    "tags": [
      "breakfast",
      "food"
    ],
    "group": 4,
    "order": 3706
  },
  {
    "emoji": "🍳",
    "label": "cooking",
    "tags": [
      "breakfast",
      "egg",
      "frying",
      "pan"
    ],
    "group": 4,
    "order": 3707
  },
  {
    "emoji": "🥘",
    "label": "shallow pan of food",
    "tags": [
      "casserole",
      "food",
      "paella",
      "pan",
      "shallow"
    ],
    "group": 4,
    "order": 3708
  },
  {
    "emoji": "🍲",
    "label": "pot of food",
    "tags": [
      "pot",
      "stew"
    ],
    "group": 4,
    "order": 3709
  },
  {
    "emoji": "🫕",
    "label": "fondue",
    "tags": [
      "cheese",
      "chocolate",
      "melted",
      "pot",
      "swiss"
    ],
    "group": 4,
    "order": 3710
  },
  {
    "emoji": "🥣",
    "label": "bowl with spoon",
    "tags": [
      "breakfast",
      "cereal",
      "congee"
    ],
    "group": 4,
    "order": 3711
  },
  {
    "emoji": "🥗",
    "label": "green salad",
    "tags": [
      "food",
      "green",
      "salad"
    ],
    "group": 4,
    "order": 3712
  },
  {
    "emoji": "🍿",
    "label": "popcorn",
    "tags": [
      "popcorn"
    ],
    "group": 4,
    "order": 3713
  },
  {
    "emoji": "🧈",
    "label": "butter",
    "tags": [
      "dairy"
    ],
    "group": 4,
    "order": 3714
  },
  {
    "emoji": "🧂",
    "label": "salt",
    "tags": [
      "condiment",
      "shaker"
    ],
    "group": 4,
    "order": 3715
  },
  {
    "emoji": "🥫",
    "label": "canned food",
    "tags": [
      "can"
    ],
    "group": 4,
    "order": 3716
  },
  {
    "emoji": "🍱",
    "label": "bento box",
    "tags": [
      "bento",
      "box"
    ],
    "group": 4,
    "order": 3717
  },
  {
    "emoji": "🍘",
    "label": "rice cracker",
    "tags": [
      "cracker",
      "rice"
    ],
    "group": 4,
    "order": 3718
  },
  {
    "emoji": "🍙",
    "label": "rice ball",
    "tags": [
      "ball",
      "japanese",
      "rice"
    ],
    "group": 4,
    "order": 3719
  },
  {
    "emoji": "🍚",
    "label": "cooked rice",
    "tags": [
      "cooked",
      "rice"
    ],
    "group": 4,
    "order": 3720
  },
  {
    "emoji": "🍛",
    "label": "curry rice",
    "tags": [
      "curry",
      "rice"
    ],
    "group": 4,
    "order": 3721
  },
  {
    "emoji": "🍜",
    "label": "steaming bowl",
    "tags": [
      "bowl",
      "noodle",
      "ramen",
      "steaming"
    ],
    "group": 4,
    "order": 3722
  },
  {
    "emoji": "🍝",
    "label": "spaghetti",
    "tags": [
      "pasta"
    ],
    "group": 4,
    "order": 3723
  },
  {
    "emoji": "🍠",
    "label": "roasted sweet potato",
    "tags": [
      "potato",
      "roasted",
      "sweet"
    ],
    "group": 4,
    "order": 3724
  },
  {
    "emoji": "🍢",
    "label": "oden",
    "tags": [
      "kebab",
      "seafood",
      "skewer",
      "stick"
    ],
    "group": 4,
    "order": 3725
  },
  {
    "emoji": "🍣",
    "label": "sushi",
    "tags": [
      "sushi"
    ],
    "group": 4,
    "order": 3726
  },
  {
    "emoji": "🍤",
    "label": "fried shrimp",
    "tags": [
      "fried",
      "prawn",
      "shrimp",
      "tempura"
    ],
    "group": 4,
    "order": 3727
  },
  {
    "emoji": "🍥",
    "label": "fish cake with swirl",
    "tags": [
      "cake",
      "fish",
      "pastry",
      "swirl"
    ],
    "group": 4,
    "order": 3728
  },
  {
    "emoji": "🥮",
    "label": "moon cake",
    "tags": [
      "autumn",
      "festival",
      "yuèbǐng"
    ],
    "group": 4,
    "order": 3729
  },
  {
    "emoji": "🍡",
    "label": "dango",
    "tags": [
      "dessert",
      "japanese",
      "skewer",
      "stick",
      "sweet"
    ],
    "group": 4,
    "order": 3730
  },
  {
    "emoji": "🥟",
    "label": "dumpling",
    "tags": [
      "empanada",
      "gyōza",
      "jiaozi",
      "pierogi",
      "potsticker"
    ],
    "group": 4,
    "order": 3731
  },
  {
    "emoji": "🥠",
    "label": "fortune cookie",
    "tags": [
      "prophecy"
    ],
    "group": 4,
    "order": 3732
  },
  {
    "emoji": "🥡",
    "label": "takeout box",
    "tags": [
      "oyster pail"
    ],
    "group": 4,
    "order": 3733
  },
  {
    "emoji": "🦀",
    "label": "crab",
    "tags": [
      "cancer",
      "zodiac"
    ],
    "group": 4,
    "order": 3734
  },
  {
    "emoji": "🦞",
    "label": "lobster",
    "tags": [
      "bisque",
      "claws",
      "seafood"
    ],
    "group": 4,
    "order": 3735
  },
  {
    "emoji": "🦐",
    "label": "shrimp",
    "tags": [
      "food",
      "shellfish",
      "small"
    ],
    "group": 4,
    "order": 3736
  },
  {
    "emoji": "🦑",
    "label": "squid",
    "tags": [
      "food",
      "molusc"
    ],
    "group": 4,
    "order": 3737
  },
  {
    "emoji": "🦪",
    "label": "oyster",
    "tags": [
      "diving",
      "pearl"
    ],
    "group": 4,
    "order": 3738
  },
  {
    "emoji": "🍦",
    "label": "soft ice cream",
    "tags": [
      "cream",
      "dessert",
      "ice",
      "icecream",
      "soft",
      "sweet"
    ],
    "group": 4,
    "order": 3739
  },
  {
    "emoji": "🍧",
    "label": "shaved ice",
    "tags": [
      "dessert",
      "ice",
      "shaved",
      "sweet"
    ],
    "group": 4,
    "order": 3740
  },
  {
    "emoji": "🍨",
    "label": "ice cream",
    "tags": [
      "cream",
      "dessert",
      "ice",
      "sweet"
    ],
    "group": 4,
    "order": 3741
  },
  {
    "emoji": "🍩",
    "label": "doughnut",
    "tags": [
      "breakfast",
      "dessert",
      "donut",
      "sweet"
    ],
    "group": 4,
    "order": 3742
  },
  {
    "emoji": "🍪",
    "label": "cookie",
    "tags": [
      "dessert",
      "sweet"
    ],
    "group": 4,
    "order": 3743
  },
  {
    "emoji": "🎂",
    "label": "birthday cake",
    "tags": [
      "birthday",
      "cake",
      "celebration",
      "dessert",
      "pastry",
      "sweet"
    ],
    "group": 4,
    "order": 3744
  },
  {
    "emoji": "🍰",
    "label": "shortcake",
    "tags": [
      "cake",
      "dessert",
      "pastry",
      "slice",
      "sweet"
    ],
    "group": 4,
    "order": 3745
  },
  {
    "emoji": "🧁",
    "label": "cupcake",
    "tags": [
      "bakery",
      "sweet"
    ],
    "group": 4,
    "order": 3746
  },
  {
    "emoji": "🥧",
    "label": "pie",
    "tags": [
      "filling",
      "pastry"
    ],
    "group": 4,
    "order": 3747
  },
  {
    "emoji": "🍫",
    "label": "chocolate bar",
    "tags": [
      "bar",
      "chocolate",
      "dessert",
      "sweet"
    ],
    "group": 4,
    "order": 3748
  },
  {
    "emoji": "🍬",
    "label": "candy",
    "tags": [
      "dessert",
      "sweet"
    ],
    "group": 4,
    "order": 3749
  },
  {
    "emoji": "🍭",
    "label": "lollipop",
    "tags": [
      "candy",
      "dessert",
      "sweet"
    ],
    "group": 4,
    "order": 3750
  },
  {
    "emoji": "🍮",
    "label": "custard",
    "tags": [
      "dessert",
      "pudding",
      "sweet"
    ],
    "group": 4,
    "order": 3751
  },
  {
    "emoji": "🍯",
    "label": "honey pot",
    "tags": [
      "honey",
      "honeypot",
      "pot",
      "sweet"
    ],
    "group": 4,
    "order": 3752
  },
  {
    "emoji": "🍼",
    "label": "baby bottle",
    "tags": [
      "baby",
      "bottle",
      "drink",
      "milk"
    ],
    "group": 4,
    "order": 3753
  },
  {
    "emoji": "🥛",
    "label": "glass of milk",
    "tags": [
      "drink",
      "glass",
      "milk"
    ],
    "group": 4,
    "order": 3754
  },
  {
    "emoji": "☕️",
    "label": "hot beverage",
    "tags": [
      "beverage",
      "coffee",
      "drink",
      "hot",
      "steaming",
      "tea"
    ],
    "group": 4,
    "order": 3755
  },
  {
    "emoji": "🫖",
    "label": "teapot",
    "tags": [
      "drink",
      "pot",
      "tea"
    ],
    "group": 4,
    "order": 3756
  },
  {
    "emoji": "🍵",
    "label": "teacup without handle",
    "tags": [
      "beverage",
      "cup",
      "drink",
      "tea",
      "teacup"
    ],
    "group": 4,
    "order": 3757
  },
  {
    "emoji": "🍶",
    "label": "sake",
    "tags": [
      "bar",
      "beverage",
      "bottle",
      "cup",
      "drink"
    ],
    "group": 4,
    "order": 3758
  },
  {
    "emoji": "🍾",
    "label": "bottle with popping cork",
    "tags": [
      "bar",
      "bottle",
      "cork",
      "drink",
      "popping"
    ],
    "group": 4,
    "order": 3759
  },
  {
    "emoji": "🍷",
    "label": "wine glass",
    "tags": [
      "bar",
      "beverage",
      "drink",
      "glass",
      "wine"
    ],
    "group": 4,
    "order": 3760
  },
  {
    "emoji": "🍸️",
    "label": "cocktail glass",
    "tags": [
      "bar",
      "cocktail",
      "drink",
      "glass"
    ],
    "group": 4,
    "order": 3761
  },
  {
    "emoji": "🍹",
    "label": "tropical drink",
    "tags": [
      "bar",
      "drink",
      "tropical"
    ],
    "group": 4,
    "order": 3762
  },
  {
    "emoji": "🍺",
    "label": "beer mug",
    "tags": [
      "bar",
      "beer",
      "drink",
      "mug"
    ],
    "group": 4,
    "order": 3763
  },
  {
    "emoji": "🍻",
    "label": "clinking beer mugs",
    "tags": [
      "bar",
      "beer",
      "clink",
      "drink",
      "mug"
    ],
    "group": 4,
    "order": 3764
  },
  {
    "emoji": "🥂",
    "label": "clinking glasses",
    "tags": [
      "celebrate",
      "clink",
      "drink",
      "glass"
    ],
    "group": 4,
    "order": 3765
  },
  {
    "emoji": "🥃",
    "label": "tumbler glass",
    "tags": [
      "glass",
      "liquor",
      "shot",
      "tumbler",
      "whisky"
    ],
    "group": 4,
    "order": 3766
  },
  {
    "emoji": "🫗",
    "label": "pouring liquid",
    "tags": [
      "drink",
      "empty",
      "glass",
      "spill"
    ],
    "group": 4,
    "order": 3767
  },
  {
    "emoji": "🥤",
    "label": "cup with straw",
    "tags": [
      "juice",
      "soda"
    ],
    "group": 4,
    "order": 3768
  },
  {
    "emoji": "🧋",
    "label": "bubble tea",
    "tags": [
      "bubble",
      "milk",
      "pearl",
      "tea"
    ],
    "group": 4,
    "order": 3769
  },
  {
    "emoji": "🧃",
    "label": "beverage box",
    "tags": [
      "beverage",
      "box",
      "juice",
      "straw",
      "sweet"
    ],
    "group": 4,
    "order": 3770
  },
  {
    "emoji": "🧉",
    "label": "mate",
    "tags": [
      "drink"
    ],
    "group": 4,
    "order": 3771
  },
  {
    "emoji": "🧊",
    "label": "ice",
    "tags": [
      "cold",
      "ice cube",
      "iceberg"
    ],
    "group": 4,
    "order": 3772
  },
  {
    "emoji": "🥢",
    "label": "chopsticks",
    "tags": [
      "hashi"
    ],
    "group": 4,
    "order": 3773
  },
  {
    "emoji": "🍽️",
    "label": "fork and knife with plate",
    "tags": [
      "cooking",
      "fork",
      "knife",
      "plate"
    ],
    "group": 4,
    "order": 3775
  },
  {
    "emoji": "🍴",
    "label": "fork and knife",
    "tags": [
      "cooking",
      "cutlery",
      "fork",
      "knife"
    ],
    "group": 4,
    "order": 3776
  },
  {
    "emoji": "🥄",
    "label": "spoon",
    "tags": [
      "tableware"
    ],
    "group": 4,
    "order": 3777
  },
  {
    "emoji": "🔪",
    "label": "kitchen knife",
    "tags": [
      "cooking",
      "hocho",
      "knife",
      "tool",
      "weapon"
    ],
    "group": 4,
    "order": 3778
  },
  {
    "emoji": "🫙",
    "label": "jar",
    "tags": [
      "condiment",
      "container",
      "empty",
      "sauce",
      "store"
    ],
    "group": 4,
    "order": 3779
  },
  {
    "emoji": "🏺",
    "label": "amphora",
    "tags": [
      "aquarius",
      "cooking",
      "drink",
      "jug",
      "zodiac"
    ],
    "group": 4,
    "order": 3780
  },
  {
    "emoji": "🌍️",
    "label": "globe showing Europe-Africa",
    "tags": [
      "africa",
      "earth",
      "europe",
      "globe",
      "globe showing europe-africa",
      "world"
    ],
    "group": 5,
    "order": 3781
  },
  {
    "emoji": "🌎️",
    "label": "globe showing Americas",
    "tags": [
      "americas",
      "earth",
      "globe",
      "globe showing americas",
      "world"
    ],
    "group": 5,
    "order": 3782
  },
  {
    "emoji": "🌏️",
    "label": "globe showing Asia-Australia",
    "tags": [
      "asia",
      "australia",
      "earth",
      "globe",
      "globe showing asia-australia",
      "world"
    ],
    "group": 5,
    "order": 3783
  },
  {
    "emoji": "🌐",
    "label": "globe with meridians",
    "tags": [
      "earth",
      "globe",
      "meridians",
      "world"
    ],
    "group": 5,
    "order": 3784
  },
  {
    "emoji": "🗺️",
    "label": "world map",
    "tags": [
      "map",
      "world"
    ],
    "group": 5,
    "order": 3786
  },
  {
    "emoji": "🗾",
    "label": "map of Japan",
    "tags": [
      "japan",
      "map",
      "map of japan"
    ],
    "group": 5,
    "order": 3787
  },
  {
    "emoji": "🧭",
    "label": "compass",
    "tags": [
      "magnetic",
      "navigation",
      "orienteering"
    ],
    "group": 5,
    "order": 3788
  },
  {
    "emoji": "🏔️",
    "label": "snow-capped mountain",
    "tags": [
      "cold",
      "mountain",
      "snow"
    ],
    "group": 5,
    "order": 3790
  },
  {
    "emoji": "⛰️",
    "label": "mountain",
    "tags": [
      "mountain"
    ],
    "group": 5,
    "order": 3792
  },
  {
    "emoji": "🌋",
    "label": "volcano",
    "tags": [
      "eruption",
      "mountain"
    ],
    "group": 5,
    "order": 3793
  },
  {
    "emoji": "🗻",
    "label": "mount fuji",
    "tags": [
      "fuji",
      "mountain"
    ],
    "group": 5,
    "order": 3794
  },
  {
    "emoji": "🏕️",
    "label": "camping",
    "tags": [
      "camping"
    ],
    "group": 5,
    "order": 3796
  },
  {
    "emoji": "🏖️",
    "label": "beach with umbrella",
    "tags": [
      "beach",
      "umbrella"
    ],
    "group": 5,
    "order": 3798
  },
  {
    "emoji": "🏜️",
    "label": "desert",
    "tags": [
      "desert"
    ],
    "group": 5,
    "order": 3800
  },
  {
    "emoji": "🏝️",
    "label": "desert island",
    "tags": [
      "desert",
      "island"
    ],
    "group": 5,
    "order": 3802
  },
  {
    "emoji": "🏞️",
    "label": "national park",
    "tags": [
      "park"
    ],
    "group": 5,
    "order": 3804
  },
  {
    "emoji": "🏟️",
    "label": "stadium",
    "tags": [
      "stadium"
    ],
    "group": 5,
    "order": 3806
  },
  {
    "emoji": "🏛️",
    "label": "classical building",
    "tags": [
      "classical"
    ],
    "group": 5,
    "order": 3808
  },
  {
    "emoji": "🏗️",
    "label": "building construction",
    "tags": [
      "construction"
    ],
    "group": 5,
    "order": 3810
  },
  {
    "emoji": "🧱",
    "label": "brick",
    "tags": [
      "bricks",
      "clay",
      "mortar",
      "wall"
    ],
    "group": 5,
    "order": 3811
  },
  {
    "emoji": "🪨",
    "label": "rock",
    "tags": [
      "boulder",
      "heavy",
      "solid",
      "stone"
    ],
    "group": 5,
    "order": 3812
  },
  {
    "emoji": "🪵",
    "label": "wood",
    "tags": [
      "log",
      "lumber",
      "timber"
    ],
    "group": 5,
    "order": 3813
  },
  {
    "emoji": "🛖",
    "label": "hut",
    "tags": [
      "house",
      "roundhouse",
      "yurt"
    ],
    "group": 5,
    "order": 3814
  },
  {
    "emoji": "🏘️",
    "label": "houses",
    "tags": [
      "houses"
    ],
    "group": 5,
    "order": 3816
  },
  {
    "emoji": "🏚️",
    "label": "derelict house",
    "tags": [
      "derelict",
      "house"
    ],
    "group": 5,
    "order": 3818
  },
  {
    "emoji": "🏠️",
    "label": "house",
    "tags": [
      "home"
    ],
    "group": 5,
    "order": 3819
  },
  {
    "emoji": "🏡",
    "label": "house with garden",
    "tags": [
      "garden",
      "home",
      "house"
    ],
    "group": 5,
    "order": 3820
  },
  {
    "emoji": "🏢",
    "label": "office building",
    "tags": [
      "building"
    ],
    "group": 5,
    "order": 3821
  },
  {
    "emoji": "🏣",
    "label": "Japanese post office",
    "tags": [
      "japanese",
      "japanese post office",
      "post"
    ],
    "group": 5,
    "order": 3822
  },
  {
    "emoji": "🏤",
    "label": "post office",
    "tags": [
      "european",
      "post"
    ],
    "group": 5,
    "order": 3823
  },
  {
    "emoji": "🏥",
    "label": "hospital",
    "tags": [
      "doctor",
      "medicine"
    ],
    "group": 5,
    "order": 3824
  },
  {
    "emoji": "🏦",
    "label": "bank",
    "tags": [
      "building"
    ],
    "group": 5,
    "order": 3825
  },
  {
    "emoji": "🏨",
    "label": "hotel",
    "tags": [
      "building"
    ],
    "group": 5,
    "order": 3826
  },
  {
    "emoji": "🏩",
    "label": "love hotel",
    "tags": [
      "hotel",
      "love"
    ],
    "group": 5,
    "order": 3827
  },
  {
    "emoji": "🏪",
    "label": "convenience store",
    "tags": [
      "convenience",
      "store"
    ],
    "group": 5,
    "order": 3828
  },
  {
    "emoji": "🏫",
    "label": "school",
    "tags": [
      "building"
    ],
    "group": 5,
    "order": 3829
  },
  {
    "emoji": "🏬",
    "label": "department store",
    "tags": [
      "department",
      "store"
    ],
    "group": 5,
    "order": 3830
  },
  {
    "emoji": "🏭️",
    "label": "factory",
    "tags": [
      "building"
    ],
    "group": 5,
    "order": 3831
  },
  {
    "emoji": "🏯",
    "label": "Japanese castle",
    "tags": [
      "castle",
      "japanese"
    ],
    "group": 5,
    "order": 3832
  },
  {
    "emoji": "🏰",
    "label": "castle",
    "tags": [
      "european"
    ],
    "group": 5,
    "order": 3833
  },
  {
    "emoji": "💒",
    "label": "wedding",
    "tags": [
      "chapel",
      "romance"
    ],
    "group": 5,
    "order": 3834
  },
  {
    "emoji": "🗼",
    "label": "Tokyo tower",
    "tags": [
      "tokyo",
      "tower"
    ],
    "group": 5,
    "order": 3835
  },
  {
    "emoji": "🗽",
    "label": "Statue of Liberty",
    "tags": [
      "liberty",
      "statue",
      "statue of liberty"
    ],
    "group": 5,
    "order": 3836
  },
  {
    "emoji": "⛪️",
    "label": "church",
    "tags": [
      "christian",
      "cross",
      "religion"
    ],
    "group": 5,
    "order": 3837
  },
  {
    "emoji": "🕌",
    "label": "mosque",
    "tags": [
      "islam",
      "muslim",
      "religion"
    ],
    "group": 5,
    "order": 3838
  },
  {
    "emoji": "🛕",
    "label": "hindu temple",
    "tags": [
      "hindu",
      "temple"
    ],
    "group": 5,
    "order": 3839
  },
  {
    "emoji": "🕍",
    "label": "synagogue",
    "tags": [
      "jew",
      "jewish",
      "religion",
      "temple"
    ],
    "group": 5,
    "order": 3840
  },
  {
    "emoji": "⛩️",
    "label": "shinto shrine",
    "tags": [
      "religion",
      "shinto",
      "shrine"
    ],
    "group": 5,
    "order": 3842
  },
  {
    "emoji": "🕋",
    "label": "kaaba",
    "tags": [
      "islam",
      "muslim",
      "religion"
    ],
    "group": 5,
    "order": 3843
  },
  {
    "emoji": "⛲️",
    "label": "fountain",
    "tags": [
      "fountain"
    ],
    "group": 5,
    "order": 3844
  },
  {
    "emoji": "⛺️",
    "label": "tent",
    "tags": [
      "camping"
    ],
    "group": 5,
    "order": 3845
  },
  {
    "emoji": "🌁",
    "label": "foggy",
    "tags": [
      "fog"
    ],
    "group": 5,
    "order": 3846
  },
  {
    "emoji": "🌃",
    "label": "night with stars",
    "tags": [
      "night",
      "star"
    ],
    "group": 5,
    "order": 3847
  },
  {
    "emoji": "🏙️",
    "label": "cityscape",
    "tags": [
      "city"
    ],
    "group": 5,
    "order": 3849
  },
  {
    "emoji": "🌄",
    "label": "sunrise over mountains",
    "tags": [
      "morning",
      "mountain",
      "sun",
      "sunrise"
    ],
    "group": 5,
    "order": 3850
  },
  {
    "emoji": "🌅",
    "label": "sunrise",
    "tags": [
      "morning",
      "sun"
    ],
    "group": 5,
    "order": 3851
  },
  {
    "emoji": "🌆",
    "label": "cityscape at dusk",
    "tags": [
      "city",
      "dusk",
      "evening",
      "landscape",
      "sunset"
    ],
    "group": 5,
    "order": 3852
  },
  {
    "emoji": "🌇",
    "label": "sunset",
    "tags": [
      "dusk",
      "sun"
    ],
    "group": 5,
    "order": 3853
  },
  {
    "emoji": "🌉",
    "label": "bridge at night",
    "tags": [
      "bridge",
      "night"
    ],
    "group": 5,
    "order": 3854
  },
  {
    "emoji": "♨️",
    "label": "hot springs",
    "tags": [
      "hot",
      "hotsprings",
      "springs",
      "steaming"
    ],
    "group": 5,
    "order": 3856
  },
  {
    "emoji": "🎠",
    "label": "carousel horse",
    "tags": [
      "carousel",
      "horse"
    ],
    "group": 5,
    "order": 3857
  },
  {
    "emoji": "🛝",
    "label": "playground slide",
    "tags": [
      "amusement park",
      "play",
      "theme park"
    ],
    "group": 5,
    "order": 3858
  },
  {
    "emoji": "🎡",
    "label": "ferris wheel",
    "tags": [
      "amusement park",
      "ferris",
      "theme park",
      "wheel"
    ],
    "group": 5,
    "order": 3859
  },
  {
    "emoji": "🎢",
    "label": "roller coaster",
    "tags": [
      "amusement park",
      "coaster",
      "roller",
      "theme park"
    ],
    "group": 5,
    "order": 3860
  },
  {
    "emoji": "💈",
    "label": "barber pole",
    "tags": [
      "barber",
      "haircut",
      "pole"
    ],
    "group": 5,
    "order": 3861
  },
  {
    "emoji": "🎪",
    "label": "circus tent",
    "tags": [
      "circus",
      "tent"
    ],
    "group": 5,
    "order": 3862
  },
  {
    "emoji": "🚂",
    "label": "locomotive",
    "tags": [
      "engine",
      "railway",
      "steam",
      "train"
    ],
    "group": 5,
    "order": 3863
  },
  {
    "emoji": "🚃",
    "label": "railway car",
    "tags": [
      "car",
      "electric",
      "railway",
      "train",
      "tram",
      "trolleybus"
    ],
    "group": 5,
    "order": 3864
  },
  {
    "emoji": "🚄",
    "label": "high-speed train",
    "tags": [
      "railway",
      "shinkansen",
      "speed",
      "train"
    ],
    "group": 5,
    "order": 3865
  },
  {
    "emoji": "🚅",
    "label": "bullet train",
    "tags": [
      "bullet",
      "railway",
      "shinkansen",
      "speed",
      "train"
    ],
    "group": 5,
    "order": 3866
  },
  {
    "emoji": "🚆",
    "label": "train",
    "tags": [
      "railway"
    ],
    "group": 5,
    "order": 3867
  },
  {
    "emoji": "🚇️",
    "label": "metro",
    "tags": [
      "subway"
    ],
    "group": 5,
    "order": 3868
  },
  {
    "emoji": "🚈",
    "label": "light rail",
    "tags": [
      "railway"
    ],
    "group": 5,
    "order": 3869
  },
  {
    "emoji": "🚉",
    "label": "station",
    "tags": [
      "railway",
      "train"
    ],
    "group": 5,
    "order": 3870
  },
  {
    "emoji": "🚊",
    "label": "tram",
    "tags": [
      "trolleybus"
    ],
    "group": 5,
    "order": 3871
  },
  {
    "emoji": "🚝",
    "label": "monorail",
    "tags": [
      "vehicle"
    ],
    "group": 5,
    "order": 3872
  },
  {
    "emoji": "🚞",
    "label": "mountain railway",
    "tags": [
      "car",
      "mountain",
      "railway"
    ],
    "group": 5,
    "order": 3873
  },
  {
    "emoji": "🚋",
    "label": "tram car",
    "tags": [
      "car",
      "tram",
      "trolleybus"
    ],
    "group": 5,
    "order": 3874
  },
  {
    "emoji": "🚌",
    "label": "bus",
    "tags": [
      "vehicle"
    ],
    "group": 5,
    "order": 3875
  },
  {
    "emoji": "🚍️",
    "label": "oncoming bus",
    "tags": [
      "bus",
      "oncoming"
    ],
    "group": 5,
    "order": 3876
  },
  {
    "emoji": "🚎",
    "label": "trolleybus",
    "tags": [
      "bus",
      "tram",
      "trolley"
    ],
    "group": 5,
    "order": 3877
  },
  {
    "emoji": "🚐",
    "label": "minibus",
    "tags": [
      "bus"
    ],
    "group": 5,
    "order": 3878
  },
  {
    "emoji": "🚑️",
    "label": "ambulance",
    "tags": [
      "vehicle"
    ],
    "group": 5,
    "order": 3879
  },
  {
    "emoji": "🚒",
    "label": "fire engine",
    "tags": [
      "engine",
      "fire",
      "truck"
    ],
    "group": 5,
    "order": 3880
  },
  {
    "emoji": "🚓",
    "label": "police car",
    "tags": [
      "car",
      "patrol",
      "police"
    ],
    "group": 5,
    "order": 3881
  },
  {
    "emoji": "🚔️",
    "label": "oncoming police car",
    "tags": [
      "car",
      "oncoming",
      "police"
    ],
    "group": 5,
    "order": 3882
  },
  {
    "emoji": "🚕",
    "label": "taxi",
    "tags": [
      "vehicle"
    ],
    "group": 5,
    "order": 3883
  },
  {
    "emoji": "🚖",
    "label": "oncoming taxi",
    "tags": [
      "oncoming",
      "taxi"
    ],
    "group": 5,
    "order": 3884
  },
  {
    "emoji": "🚗",
    "label": "automobile",
    "tags": [
      "car"
    ],
    "group": 5,
    "order": 3885
  },
  {
    "emoji": "🚘️",
    "label": "oncoming automobile",
    "tags": [
      "automobile",
      "car",
      "oncoming"
    ],
    "group": 5,
    "order": 3886
  },
  {
    "emoji": "🚙",
    "label": "sport utility vehicle",
    "tags": [
      "recreational",
      "sport utility"
    ],
    "group": 5,
    "order": 3887
  },
  {
    "emoji": "🛻",
    "label": "pickup truck",
    "tags": [
      "pick-up",
      "pickup",
      "truck"
    ],
    "group": 5,
    "order": 3888
  },
  {
    "emoji": "🚚",
    "label": "delivery truck",
    "tags": [
      "delivery",
      "truck"
    ],
    "group": 5,
    "order": 3889
  },
  {
    "emoji": "🚛",
    "label": "articulated lorry",
    "tags": [
      "lorry",
      "semi",
      "truck"
    ],
    "group": 5,
    "order": 3890
  },
  {
    "emoji": "🚜",
    "label": "tractor",
    "tags": [
      "vehicle"
    ],
    "group": 5,
    "order": 3891
  },
  {
    "emoji": "🏎️",
    "label": "racing car",
    "tags": [
      "car",
      "racing"
    ],
    "group": 5,
    "order": 3893
  },
  {
    "emoji": "🏍️",
    "label": "motorcycle",
    "tags": [
      "racing"
    ],
    "group": 5,
    "order": 3895
  },
  {
    "emoji": "🛵",
    "label": "motor scooter",
    "tags": [
      "motor",
      "scooter"
    ],
    "group": 5,
    "order": 3896
  },
  {
    "emoji": "🦽",
    "label": "manual wheelchair",
    "tags": [
      "accessibility"
    ],
    "group": 5,
    "order": 3897
  },
  {
    "emoji": "🦼",
    "label": "motorized wheelchair",
    "tags": [
      "accessibility"
    ],
    "group": 5,
    "order": 3898
  },
  {
    "emoji": "🛺",
    "label": "auto rickshaw",
    "tags": [
      "tuk tuk"
    ],
    "group": 5,
    "order": 3899
  },
  {
    "emoji": "🚲️",
    "label": "bicycle",
    "tags": [
      "bike"
    ],
    "group": 5,
    "order": 3900
  },
  {
    "emoji": "🛴",
    "label": "kick scooter",
    "tags": [
      "kick",
      "scooter"
    ],
    "group": 5,
    "order": 3901
  },
  {
    "emoji": "🛹",
    "label": "skateboard",
    "tags": [
      "board"
    ],
    "group": 5,
    "order": 3902
  },
  {
    "emoji": "🛼",
    "label": "roller skate",
    "tags": [
      "roller",
      "skate"
    ],
    "group": 5,
    "order": 3903
  },
  {
    "emoji": "🚏",
    "label": "bus stop",
    "tags": [
      "bus",
      "stop"
    ],
    "group": 5,
    "order": 3904
  },
  {
    "emoji": "🛣️",
    "label": "motorway",
    "tags": [
      "highway",
      "road"
    ],
    "group": 5,
    "order": 3906
  },
  {
    "emoji": "🛤️",
    "label": "railway track",
    "tags": [
      "railway",
      "train"
    ],
    "group": 5,
    "order": 3908
  },
  {
    "emoji": "🛢️",
    "label": "oil drum",
    "tags": [
      "drum",
      "oil"
    ],
    "group": 5,
    "order": 3910
  },
  {
    "emoji": "⛽️",
    "label": "fuel pump",
    "tags": [
      "diesel",
      "fuel",
      "fuelpump",
      "gas",
      "pump",
      "station"
    ],
    "group": 5,
    "order": 3911
  },
  {
    "emoji": "🛞",
    "label": "wheel",
    "tags": [
      "circle",
      "tire",
      "turn"
    ],
    "group": 5,
    "order": 3912
  },
  {
    "emoji": "🚨",
    "label": "police car light",
    "tags": [
      "beacon",
      "car",
      "light",
      "police",
      "revolving"
    ],
    "group": 5,
    "order": 3913
  },
  {
    "emoji": "🚥",
    "label": "horizontal traffic light",
    "tags": [
      "light",
      "signal",
      "traffic"
    ],
    "group": 5,
    "order": 3914
  },
  {
    "emoji": "🚦",
    "label": "vertical traffic light",
    "tags": [
      "light",
      "signal",
      "traffic"
    ],
    "group": 5,
    "order": 3915
  },
  {
    "emoji": "🛑",
    "label": "stop sign",
    "tags": [
      "octagonal",
      "sign",
      "stop"
    ],
    "group": 5,
    "order": 3916
  },
  {
    "emoji": "🚧",
    "label": "construction",
    "tags": [
      "barrier"
    ],
    "group": 5,
    "order": 3917
  },
  {
    "emoji": "⚓️",
    "label": "anchor",
    "tags": [
      "ship",
      "tool"
    ],
    "group": 5,
    "order": 3918
  },
  {
    "emoji": "🛟",
    "label": "ring buoy",
    "tags": [
      "float",
      "life preserver",
      "life saver",
      "rescue",
      "safety"
    ],
    "group": 5,
    "order": 3919
  },
  {
    "emoji": "⛵️",
    "label": "sailboat",
    "tags": [
      "boat",
      "resort",
      "sea",
      "yacht"
    ],
    "group": 5,
    "order": 3920
  },
  {
    "emoji": "🛶",
    "label": "canoe",
    "tags": [
      "boat"
    ],
    "group": 5,
    "order": 3921
  },
  {
    "emoji": "🚤",
    "label": "speedboat",
    "tags": [
      "boat"
    ],
    "group": 5,
    "order": 3922
  },
  {
    "emoji": "🛳️",
    "label": "passenger ship",
    "tags": [
      "passenger",
      "ship"
    ],
    "group": 5,
    "order": 3924
  },
  {
    "emoji": "⛴️",
    "label": "ferry",
    "tags": [
      "boat",
      "passenger"
    ],
    "group": 5,
    "order": 3926
  },
  {
    "emoji": "🛥️",
    "label": "motor boat",
    "tags": [
      "boat",
      "motorboat"
    ],
    "group": 5,
    "order": 3928
  },
  {
    "emoji": "🚢",
    "label": "ship",
    "tags": [
      "boat",
      "passenger"
    ],
    "group": 5,
    "order": 3929
  },
  {
    "emoji": "✈️",
    "label": "airplane",
    "tags": [
      "aeroplane"
    ],
    "group": 5,
    "order": 3931
  },
  {
    "emoji": "🛩️",
    "label": "small airplane",
    "tags": [
      "aeroplane",
      "airplane"
    ],
    "group": 5,
    "order": 3933
  },
  {
    "emoji": "🛫",
    "label": "airplane departure",
    "tags": [
      "aeroplane",
      "airplane",
      "check-in",
      "departure",
      "departures"
    ],
    "group": 5,
    "order": 3934
  },
  {
    "emoji": "🛬",
    "label": "airplane arrival",
    "tags": [
      "aeroplane",
      "airplane",
      "arrivals",
      "arriving",
      "landing"
    ],
    "group": 5,
    "order": 3935
  },
  {
    "emoji": "🪂",
    "label": "parachute",
    "tags": [
      "hang-glide",
      "parasail",
      "skydive"
    ],
    "group": 5,
    "order": 3936
  },
  {
    "emoji": "💺",
    "label": "seat",
    "tags": [
      "chair"
    ],
    "group": 5,
    "order": 3937
  },
  {
    "emoji": "🚁",
    "label": "helicopter",
    "tags": [
      "vehicle"
    ],
    "group": 5,
    "order": 3938
  },
  {
    "emoji": "🚟",
    "label": "suspension railway",
    "tags": [
      "railway",
      "suspension"
    ],
    "group": 5,
    "order": 3939
  },
  {
    "emoji": "🚠",
    "label": "mountain cableway",
    "tags": [
      "cable",
      "gondola",
      "mountain"
    ],
    "group": 5,
    "order": 3940
  },
  {
    "emoji": "🚡",
    "label": "aerial tramway",
    "tags": [
      "aerial",
      "cable",
      "car",
      "gondola",
      "tramway"
    ],
    "group": 5,
    "order": 3941
  },
  {
    "emoji": "🛰️",
    "label": "satellite",
    "tags": [
      "space"
    ],
    "group": 5,
    "order": 3943
  },
  {
    "emoji": "🚀",
    "label": "rocket",
    "tags": [
      "space"
    ],
    "group": 5,
    "order": 3944
  },
  {
    "emoji": "🛸",
    "label": "flying saucer",
    "tags": [
      "ufo"
    ],
    "group": 5,
    "order": 3945
  },
  {
    "emoji": "🛎️",
    "label": "bellhop bell",
    "tags": [
      "bell",
      "bellhop",
      "hotel"
    ],
    "group": 5,
    "order": 3947
  },
  {
    "emoji": "🧳",
    "label": "luggage",
    "tags": [
      "packing",
      "travel"
    ],
    "group": 5,
    "order": 3948
  },
  {
    "emoji": "⌛️",
    "label": "hourglass done",
    "tags": [
      "sand",
      "timer"
    ],
    "group": 5,
    "order": 3949
  },
  {
    "emoji": "⏳️",
    "label": "hourglass not done",
    "tags": [
      "hourglass",
      "sand",
      "timer"
    ],
    "group": 5,
    "order": 3950
  },
  {
    "emoji": "⌚️",
    "label": "watch",
    "tags": [
      "clock"
    ],
    "group": 5,
    "order": 3951
  },
  {
    "emoji": "⏰️",
    "label": "alarm clock",
    "tags": [
      "alarm",
      "clock"
    ],
    "group": 5,
    "order": 3952
  },
  {
    "emoji": "⏱️",
    "label": "stopwatch",
    "tags": [
      "clock"
    ],
    "group": 5,
    "order": 3954
  },
  {
    "emoji": "⏲️",
    "label": "timer clock",
    "tags": [
      "clock",
      "timer"
    ],
    "group": 5,
    "order": 3956
  },
  {
    "emoji": "🕰️",
    "label": "mantelpiece clock",
    "tags": [
      "clock"
    ],
    "group": 5,
    "order": 3958
  },
  {
    "emoji": "🕛️",
    "label": "twelve o’clock",
    "tags": [
      "00",
      "12",
      "12:00",
      "clock",
      "o’clock",
      "twelve"
    ],
    "group": 5,
    "order": 3959
  },
  {
    "emoji": "🕧️",
    "label": "twelve-thirty",
    "tags": [
      "12",
      "12:30",
      "clock",
      "thirty",
      "twelve"
    ],
    "group": 5,
    "order": 3960
  },
  {
    "emoji": "🕐️",
    "label": "one o’clock",
    "tags": [
      "00",
      "1",
      "1:00",
      "clock",
      "one",
      "o’clock"
    ],
    "group": 5,
    "order": 3961
  },
  {
    "emoji": "🕜️",
    "label": "one-thirty",
    "tags": [
      "1",
      "1:30",
      "clock",
      "one",
      "thirty"
    ],
    "group": 5,
    "order": 3962
  },
  {
    "emoji": "🕑️",
    "label": "two o’clock",
    "tags": [
      "00",
      "2",
      "2:00",
      "clock",
      "o’clock",
      "two"
    ],
    "group": 5,
    "order": 3963
  },
  {
    "emoji": "🕝️",
    "label": "two-thirty",
    "tags": [
      "2",
      "2:30",
      "clock",
      "thirty",
      "two"
    ],
    "group": 5,
    "order": 3964
  },
  {
    "emoji": "🕒️",
    "label": "three o’clock",
    "tags": [
      "00",
      "3",
      "3:00",
      "clock",
      "o’clock",
      "three"
    ],
    "group": 5,
    "order": 3965
  },
  {
    "emoji": "🕞️",
    "label": "three-thirty",
    "tags": [
      "3",
      "3:30",
      "clock",
      "thirty",
      "three"
    ],
    "group": 5,
    "order": 3966
  },
  {
    "emoji": "🕓️",
    "label": "four o’clock",
    "tags": [
      "00",
      "4",
      "4:00",
      "clock",
      "four",
      "o’clock"
    ],
    "group": 5,
    "order": 3967
  },
  {
    "emoji": "🕟️",
    "label": "four-thirty",
    "tags": [
      "4",
      "4:30",
      "clock",
      "four",
      "thirty"
    ],
    "group": 5,
    "order": 3968
  },
  {
    "emoji": "🕔️",
    "label": "five o’clock",
    "tags": [
      "00",
      "5",
      "5:00",
      "clock",
      "five",
      "o’clock"
    ],
    "group": 5,
    "order": 3969
  },
  {
    "emoji": "🕠️",
    "label": "five-thirty",
    "tags": [
      "5",
      "5:30",
      "clock",
      "five",
      "thirty"
    ],
    "group": 5,
    "order": 3970
  },
  {
    "emoji": "🕕️",
    "label": "six o’clock",
    "tags": [
      "00",
      "6",
      "6:00",
      "clock",
      "o’clock",
      "six"
    ],
    "group": 5,
    "order": 3971
  },
  {
    "emoji": "🕡️",
    "label": "six-thirty",
    "tags": [
      "6",
      "6:30",
      "clock",
      "six",
      "thirty"
    ],
    "group": 5,
    "order": 3972
  },
  {
    "emoji": "🕖️",
    "label": "seven o’clock",
    "tags": [
      "00",
      "7",
      "7:00",
      "clock",
      "o’clock",
      "seven"
    ],
    "group": 5,
    "order": 3973
  },
  {
    "emoji": "🕢️",
    "label": "seven-thirty",
    "tags": [
      "7",
      "7:30",
      "clock",
      "seven",
      "thirty"
    ],
    "group": 5,
    "order": 3974
  },
  {
    "emoji": "🕗️",
    "label": "eight o’clock",
    "tags": [
      "00",
      "8",
      "8:00",
      "clock",
      "eight",
      "o’clock"
    ],
    "group": 5,
    "order": 3975
  },
  {
    "emoji": "🕣️",
    "label": "eight-thirty",
    "tags": [
      "8",
      "8:30",
      "clock",
      "eight",
      "thirty"
    ],
    "group": 5,
    "order": 3976
  },
  {
    "emoji": "🕘️",
    "label": "nine o’clock",
    "tags": [
      "00",
      "9",
      "9:00",
      "clock",
      "nine",
      "o’clock"
    ],
    "group": 5,
    "order": 3977
  },
  {
    "emoji": "🕤️",
    "label": "nine-thirty",
    "tags": [
      "9",
      "9:30",
      "clock",
      "nine",
      "thirty"
    ],
    "group": 5,
    "order": 3978
  },
  {
    "emoji": "🕙️",
    "label": "ten o’clock",
    "tags": [
      "00",
      "10",
      "10:00",
      "clock",
      "o’clock",
      "ten"
    ],
    "group": 5,
    "order": 3979
  },
  {
    "emoji": "🕥️",
    "label": "ten-thirty",
    "tags": [
      "10",
      "10:30",
      "clock",
      "ten",
      "thirty"
    ],
    "group": 5,
    "order": 3980
  },
  {
    "emoji": "🕚️",
    "label": "eleven o’clock",
    "tags": [
      "00",
      "11",
      "11:00",
      "clock",
      "eleven",
      "o’clock"
    ],
    "group": 5,
    "order": 3981
  },
  {
    "emoji": "🕦️",
    "label": "eleven-thirty",
    "tags": [
      "11",
      "11:30",
      "clock",
      "eleven",
      "thirty"
    ],
    "group": 5,
    "order": 3982
  },
  {
    "emoji": "🌑",
    "label": "new moon",
    "tags": [
      "dark",
      "moon"
    ],
    "group": 5,
    "order": 3983
  },
  {
    "emoji": "🌒",
    "label": "waxing crescent moon",
    "tags": [
      "crescent",
      "moon",
      "waxing"
    ],
    "group": 5,
    "order": 3984
  },
  {
    "emoji": "🌓",
    "label": "first quarter moon",
    "tags": [
      "moon",
      "quarter"
    ],
    "group": 5,
    "order": 3985
  },
  {
    "emoji": "🌔",
    "label": "waxing gibbous moon",
    "tags": [
      "gibbous",
      "moon",
      "waxing"
    ],
    "group": 5,
    "order": 3986
  },
  {
    "emoji": "🌕️",
    "label": "full moon",
    "tags": [
      "full",
      "moon"
    ],
    "group": 5,
    "order": 3987
  },
  {
    "emoji": "🌖",
    "label": "waning gibbous moon",
    "tags": [
      "gibbous",
      "moon",
      "waning"
    ],
    "group": 5,
    "order": 3988
  },
  {
    "emoji": "🌗",
    "label": "last quarter moon",
    "tags": [
      "moon",
      "quarter"
    ],
    "group": 5,
    "order": 3989
  },
  {
    "emoji": "🌘",
    "label": "waning crescent moon",
    "tags": [
      "crescent",
      "moon",
      "waning"
    ],
    "group": 5,
    "order": 3990
  },
  {
    "emoji": "🌙",
    "label": "crescent moon",
    "tags": [
      "crescent",
      "moon"
    ],
    "group": 5,
    "order": 3991
  },
  {
    "emoji": "🌚",
    "label": "new moon face",
    "tags": [
      "face",
      "moon"
    ],
    "group": 5,
    "order": 3992
  },
  {
    "emoji": "🌛",
    "label": "first quarter moon face",
    "tags": [
      "face",
      "moon",
      "quarter"
    ],
    "group": 5,
    "order": 3993
  },
  {
    "emoji": "🌜️",
    "label": "last quarter moon face",
    "tags": [
      "face",
      "moon",
      "quarter"
    ],
    "group": 5,
    "order": 3994
  },
  {
    "emoji": "🌡️",
    "label": "thermometer",
    "tags": [
      "weather"
    ],
    "group": 5,
    "order": 3996
  },
  {
    "emoji": "☀️",
    "label": "sun",
    "tags": [
      "bright",
      "rays",
      "sunny"
    ],
    "group": 5,
    "order": 3998
  },
  {
    "emoji": "🌝",
    "label": "full moon face",
    "tags": [
      "bright",
      "face",
      "full",
      "moon"
    ],
    "group": 5,
    "order": 3999
  },
  {
    "emoji": "🌞",
    "label": "sun with face",
    "tags": [
      "bright",
      "face",
      "sun"
    ],
    "group": 5,
    "order": 4000
  },
  {
    "emoji": "🪐",
    "label": "ringed planet",
    "tags": [
      "saturn",
      "saturnine"
    ],
    "group": 5,
    "order": 4001
  },
  {
    "emoji": "⭐️",
    "label": "star",
    "tags": [
      "star"
    ],
    "group": 5,
    "order": 4002
  },
  {
    "emoji": "🌟",
    "label": "glowing star",
    "tags": [
      "glittery",
      "glow",
      "shining",
      "sparkle",
      "star"
    ],
    "group": 5,
    "order": 4003
  },
  {
    "emoji": "🌠",
    "label": "shooting star",
    "tags": [
      "falling",
      "shooting",
      "star"
    ],
    "group": 5,
    "order": 4004
  },
  {
    "emoji": "🌌",
    "label": "milky way",
    "tags": [
      "space"
    ],
    "group": 5,
    "order": 4005
  },
  {
    "emoji": "☁️",
    "label": "cloud",
    "tags": [
      "weather"
    ],
    "group": 5,
    "order": 4007
  },
  {
    "emoji": "⛅️",
    "label": "sun behind cloud",
    "tags": [
      "cloud",
      "sun"
    ],
    "group": 5,
    "order": 4008
  },
  {
    "emoji": "⛈️",
    "label": "cloud with lightning and rain",
    "tags": [
      "cloud",
      "rain",
      "thunder"
    ],
    "group": 5,
    "order": 4010
  },
  {
    "emoji": "🌤️",
    "label": "sun behind small cloud",
    "tags": [
      "cloud",
      "sun"
    ],
    "group": 5,
    "order": 4012
  },
  {
    "emoji": "🌥️",
    "label": "sun behind large cloud",
    "tags": [
      "cloud",
      "sun"
    ],
    "group": 5,
    "order": 4014
  },
  {
    "emoji": "🌦️",
    "label": "sun behind rain cloud",
    "tags": [
      "cloud",
      "rain",
      "sun"
    ],
    "group": 5,
    "order": 4016
  },
  {
    "emoji": "🌧️",
    "label": "cloud with rain",
    "tags": [
      "cloud",
      "rain"
    ],
    "group": 5,
    "order": 4018
  },
  {
    "emoji": "🌨️",
    "label": "cloud with snow",
    "tags": [
      "cloud",
      "cold",
      "snow"
    ],
    "group": 5,
    "order": 4020
  },
  {
    "emoji": "🌩️",
    "label": "cloud with lightning",
    "tags": [
      "cloud",
      "lightning"
    ],
    "group": 5,
    "order": 4022
  },
  {
    "emoji": "🌪️",
    "label": "tornado",
    "tags": [
      "cloud",
      "whirlwind"
    ],
    "group": 5,
    "order": 4024
  },
  {
    "emoji": "🌫️",
    "label": "fog",
    "tags": [
      "cloud"
    ],
    "group": 5,
    "order": 4026
  },
  {
    "emoji": "🌬️",
    "label": "wind face",
    "tags": [
      "blow",
      "cloud",
      "face",
      "wind"
    ],
    "group": 5,
    "order": 4028
  },
  {
    "emoji": "🌀",
    "label": "cyclone",
    "tags": [
      "dizzy",
      "hurricane",
      "twister",
      "typhoon"
    ],
    "group": 5,
    "order": 4029
  },
  {
    "emoji": "🌈",
    "label": "rainbow",
    "tags": [
      "rain"
    ],
    "group": 5,
    "order": 4030
  },
  {
    "emoji": "🌂",
    "label": "closed umbrella",
    "tags": [
      "clothing",
      "rain",
      "umbrella"
    ],
    "group": 5,
    "order": 4031
  },
  {
    "emoji": "☂️",
    "label": "umbrella",
    "tags": [
      "clothing",
      "rain"
    ],
    "group": 5,
    "order": 4033
  },
  {
    "emoji": "☔️",
    "label": "umbrella with rain drops",
    "tags": [
      "clothing",
      "drop",
      "rain",
      "umbrella"
    ],
    "group": 5,
    "order": 4034
  },
  {
    "emoji": "⛱️",
    "label": "umbrella on ground",
    "tags": [
      "rain",
      "sun",
      "umbrella"
    ],
    "group": 5,
    "order": 4036
  },
  {
    "emoji": "⚡️",
    "label": "high voltage",
    "tags": [
      "danger",
      "electric",
      "lightning",
      "voltage",
      "zap"
    ],
    "group": 5,
    "order": 4037
  },
  {
    "emoji": "❄️",
    "label": "snowflake",
    "tags": [
      "cold",
      "snow"
    ],
    "group": 5,
    "order": 4039
  },
  {
    "emoji": "☃️",
    "label": "snowman",
    "tags": [
      "cold",
      "snow"
    ],
    "group": 5,
    "order": 4041
  },
  {
    "emoji": "⛄️",
    "label": "snowman without snow",
    "tags": [
      "cold",
      "snow",
      "snowman"
    ],
    "group": 5,
    "order": 4042
  },
  {
    "emoji": "☄️",
    "label": "comet",
    "tags": [
      "space"
    ],
    "group": 5,
    "order": 4044
  },
  {
    "emoji": "🔥",
    "label": "fire",
    "tags": [
      "flame",
      "tool"
    ],
    "group": 5,
    "order": 4045
  },
  {
    "emoji": "💧",
    "label": "droplet",
    "tags": [
      "cold",
      "comic",
      "drop",
      "sweat"
    ],
    "group": 5,
    "order": 4046
  },
  {
    "emoji": "🌊",
    "label": "water wave",
    "tags": [
      "ocean",
      "water",
      "wave"
    ],
    "group": 5,
    "order": 4047
  },
  {
    "emoji": "🎃",
    "label": "jack-o-lantern",
    "tags": [
      "celebration",
      "halloween",
      "jack",
      "lantern"
    ],
    "group": 6,
    "order": 4048
  },
  {
    "emoji": "🎄",
    "label": "Christmas tree",
    "tags": [
      "celebration",
      "christmas",
      "tree"
    ],
    "group": 6,
    "order": 4049
  },
  {
    "emoji": "🎆",
    "label": "fireworks",
    "tags": [
      "celebration"
    ],
    "group": 6,
    "order": 4050
  },
  {
    "emoji": "🎇",
    "label": "sparkler",
    "tags": [
      "celebration",
      "fireworks",
      "sparkle"
    ],
    "group": 6,
    "order": 4051
  },
  {
    "emoji": "🧨",
    "label": "firecracker",
    "tags": [
      "dynamite",
      "explosive",
      "fireworks"
    ],
    "group": 6,
    "order": 4052
  },
  {
    "emoji": "✨️",
    "label": "sparkles",
    "tags": [
      "*",
      "sparkle",
      "star"
    ],
    "group": 6,
    "order": 4053
  },
  {
    "emoji": "🎈",
    "label": "balloon",
    "tags": [
      "celebration"
    ],
    "group": 6,
    "order": 4054
  },
  {
    "emoji": "🎉",
    "label": "party popper",
    "tags": [
      "celebration",
      "party",
      "popper",
      "tada"
    ],
    "group": 6,
    "order": 4055
  },
  {
    "emoji": "🎊",
    "label": "confetti ball",
    "tags": [
      "ball",
      "celebration",
      "confetti"
    ],
    "group": 6,
    "order": 4056
  },
  {
    "emoji": "🎋",
    "label": "tanabata tree",
    "tags": [
      "banner",
      "celebration",
      "japanese",
      "tree"
    ],
    "group": 6,
    "order": 4057
  },
  {
    "emoji": "🎍",
    "label": "pine decoration",
    "tags": [
      "bamboo",
      "celebration",
      "japanese",
      "pine"
    ],
    "group": 6,
    "order": 4058
  },
  {
    "emoji": "🎎",
    "label": "Japanese dolls",
    "tags": [
      "celebration",
      "doll",
      "festival",
      "japanese",
      "japanese dolls"
    ],
    "group": 6,
    "order": 4059
  },
  {
    "emoji": "🎏",
    "label": "carp streamer",
    "tags": [
      "carp",
      "celebration",
      "streamer"
    ],
    "group": 6,
    "order": 4060
  },
  {
    "emoji": "🎐",
    "label": "wind chime",
    "tags": [
      "bell",
      "celebration",
      "chime",
      "wind"
    ],
    "group": 6,
    "order": 4061
  },
  {
    "emoji": "🎑",
    "label": "moon viewing ceremony",
    "tags": [
      "celebration",
      "ceremony",
      "moon"
    ],
    "group": 6,
    "order": 4062
  },
  {
    "emoji": "🧧",
    "label": "red envelope",
    "tags": [
      "gift",
      "good luck",
      "hóngbāo",
      "lai see",
      "money"
    ],
    "group": 6,
    "order": 4063
  },
  {
    "emoji": "🎀",
    "label": "ribbon",
    "tags": [
      "celebration"
    ],
    "group": 6,
    "order": 4064
  },
  {
    "emoji": "🎁",
    "label": "wrapped gift",
    "tags": [
      "box",
      "celebration",
      "gift",
      "present",
      "wrapped"
    ],
    "group": 6,
    "order": 4065
  },
  {
    "emoji": "🎗️",
    "label": "reminder ribbon",
    "tags": [
      "celebration",
      "reminder",
      "ribbon"
    ],
    "group": 6,
    "order": 4067
  },
  {
    "emoji": "🎟️",
    "label": "admission tickets",
    "tags": [
      "admission",
      "ticket"
    ],
    "group": 6,
    "order": 4069
  },
  {
    "emoji": "🎫",
    "label": "ticket",
    "tags": [
      "admission"
    ],
    "group": 6,
    "order": 4070
  },
  {
    "emoji": "🎖️",
    "label": "military medal",
    "tags": [
      "celebration",
      "medal",
      "military"
    ],
    "group": 6,
    "order": 4072
  },
  {
    "emoji": "🏆️",
    "label": "trophy",
    "tags": [
      "prize"
    ],
    "group": 6,
    "order": 4073
  },
  {
    "emoji": "🏅",
    "label": "sports medal",
    "tags": [
      "medal"
    ],
    "group": 6,
    "order": 4074
  },
  {
    "emoji": "🥇",
    "label": "1st place medal",
    "tags": [
      "first",
      "gold",
      "medal"
    ],
    "group": 6,
    "order": 4075
  },
  {
    "emoji": "🥈",
    "label": "2nd place medal",
    "tags": [
      "medal",
      "second",
      "silver"
    ],
    "group": 6,
    "order": 4076
  },
  {
    "emoji": "🥉",
    "label": "3rd place medal",
    "tags": [
      "bronze",
      "medal",
      "third"
    ],
    "group": 6,
    "order": 4077
  },
  {
    "emoji": "⚽️",
    "label": "soccer ball",
    "tags": [
      "ball",
      "football",
      "soccer"
    ],
    "group": 6,
    "order": 4078
  },
  {
    "emoji": "⚾️",
    "label": "baseball",
    "tags": [
      "ball"
    ],
    "group": 6,
    "order": 4079
  },
  {
    "emoji": "🥎",
    "label": "softball",
    "tags": [
      "ball",
      "glove",
      "underarm"
    ],
    "group": 6,
    "order": 4080
  },
  {
    "emoji": "🏀",
    "label": "basketball",
    "tags": [
      "ball",
      "hoop"
    ],
    "group": 6,
    "order": 4081
  },
  {
    "emoji": "🏐",
    "label": "volleyball",
    "tags": [
      "ball",
      "game"
    ],
    "group": 6,
    "order": 4082
  },
  {
    "emoji": "🏈",
    "label": "american football",
    "tags": [
      "american",
      "ball",
      "football"
    ],
    "group": 6,
    "order": 4083
  },
  {
    "emoji": "🏉",
    "label": "rugby football",
    "tags": [
      "ball",
      "football",
      "rugby"
    ],
    "group": 6,
    "order": 4084
  },
  {
    "emoji": "🎾",
    "label": "tennis",
    "tags": [
      "ball",
      "racquet"
    ],
    "group": 6,
    "order": 4085
  },
  {
    "emoji": "🥏",
    "label": "flying disc",
    "tags": [
      "ultimate"
    ],
    "group": 6,
    "order": 4086
  },
  {
    "emoji": "🎳",
    "label": "bowling",
    "tags": [
      "ball",
      "game"
    ],
    "group": 6,
    "order": 4087
  },
  {
    "emoji": "🏏",
    "label": "cricket game",
    "tags": [
      "ball",
      "bat",
      "game"
    ],
    "group": 6,
    "order": 4088
  },
  {
    "emoji": "🏑",
    "label": "field hockey",
    "tags": [
      "ball",
      "field",
      "game",
      "hockey",
      "stick"
    ],
    "group": 6,
    "order": 4089
  },
  {
    "emoji": "🏒",
    "label": "ice hockey",
    "tags": [
      "game",
      "hockey",
      "ice",
      "puck",
      "stick"
    ],
    "group": 6,
    "order": 4090
  },
  {
    "emoji": "🥍",
    "label": "lacrosse",
    "tags": [
      "ball",
      "goal",
      "stick"
    ],
    "group": 6,
    "order": 4091
  },
  {
    "emoji": "🏓",
    "label": "ping pong",
    "tags": [
      "ball",
      "bat",
      "game",
      "paddle",
      "table tennis"
    ],
    "group": 6,
    "order": 4092
  },
  {
    "emoji": "🏸",
    "label": "badminton",
    "tags": [
      "birdie",
      "game",
      "racquet",
      "shuttlecock"
    ],
    "group": 6,
    "order": 4093
  },
  {
    "emoji": "🥊",
    "label": "boxing glove",
    "tags": [
      "boxing",
      "glove"
    ],
    "group": 6,
    "order": 4094
  },
  {
    "emoji": "🥋",
    "label": "martial arts uniform",
    "tags": [
      "judo",
      "karate",
      "martial arts",
      "taekwondo",
      "uniform"
    ],
    "group": 6,
    "order": 4095
  },
  {
    "emoji": "🥅",
    "label": "goal net",
    "tags": [
      "goal",
      "net"
    ],
    "group": 6,
    "order": 4096
  },
  {
    "emoji": "⛳️",
    "label": "flag in hole",
    "tags": [
      "golf",
      "hole"
    ],
    "group": 6,
    "order": 4097
  },
  {
    "emoji": "⛸️",
    "label": "ice skate",
    "tags": [
      "ice",
      "skate"
    ],
    "group": 6,
    "order": 4099
  },
  {
    "emoji": "🎣",
    "label": "fishing pole",
    "tags": [
      "fish",
      "pole"
    ],
    "group": 6,
    "order": 4100
  },
  {
    "emoji": "🤿",
    "label": "diving mask",
    "tags": [
      "diving",
      "scuba",
      "snorkeling"
    ],
    "group": 6,
    "order": 4101
  },
  {
    "emoji": "🎽",
    "label": "running shirt",
    "tags": [
      "athletics",
      "running",
      "sash",
      "shirt"
    ],
    "group": 6,
    "order": 4102
  },
  {
    "emoji": "🎿",
    "label": "skis",
    "tags": [
      "ski",
      "snow"
    ],
    "group": 6,
    "order": 4103
  },
  {
    "emoji": "🛷",
    "label": "sled",
    "tags": [
      "sledge",
      "sleigh"
    ],
    "group": 6,
    "order": 4104
  },
  {
    "emoji": "🥌",
    "label": "curling stone",
    "tags": [
      "game",
      "rock"
    ],
    "group": 6,
    "order": 4105
  },
  {
    "emoji": "🎯",
    "label": "bullseye",
    "tags": [
      "dart",
      "direct hit",
      "game",
      "hit",
      "target"
    ],
    "group": 6,
    "order": 4106
  },
  {
    "emoji": "🪀",
    "label": "yo-yo",
    "tags": [
      "fluctuate",
      "toy"
    ],
    "group": 6,
    "order": 4107
  },
  {
    "emoji": "🪁",
    "label": "kite",
    "tags": [
      "fly",
      "soar"
    ],
    "group": 6,
    "order": 4108
  },
  {
    "emoji": "🔫",
    "label": "water pistol",
    "tags": [
      "gun",
      "handgun",
      "pistol",
      "revolver",
      "tool",
      "water",
      "weapon"
    ],
    "group": 6,
    "order": 4109
  },
  {
    "emoji": "🎱",
    "label": "pool 8 ball",
    "tags": [
      "8",
      "ball",
      "billiard",
      "eight",
      "game"
    ],
    "group": 6,
    "order": 4110
  },
  {
    "emoji": "🔮",
    "label": "crystal ball",
    "tags": [
      "ball",
      "crystal",
      "fairy tale",
      "fantasy",
      "fortune",
      "tool"
    ],
    "group": 6,
    "order": 4111
  },
  {
    "emoji": "🪄",
    "label": "magic wand",
    "tags": [
      "magic",
      "witch",
      "wizard"
    ],
    "group": 6,
    "order": 4112
  },
  {
    "emoji": "🎮️",
    "label": "video game",
    "tags": [
      "controller",
      "game"
    ],
    "group": 6,
    "order": 4113
  },
  {
    "emoji": "🕹️",
    "label": "joystick",
    "tags": [
      "game",
      "video game"
    ],
    "group": 6,
    "order": 4115
  },
  {
    "emoji": "🎰",
    "label": "slot machine",
    "tags": [
      "game",
      "slot"
    ],
    "group": 6,
    "order": 4116
  },
  {
    "emoji": "🎲",
    "label": "game die",
    "tags": [
      "dice",
      "die",
      "game"
    ],
    "group": 6,
    "order": 4117
  },
  {
    "emoji": "🧩",
    "label": "puzzle piece",
    "tags": [
      "clue",
      "interlocking",
      "jigsaw",
      "piece",
      "puzzle"
    ],
    "group": 6,
    "order": 4118
  },
  {
    "emoji": "🧸",
    "label": "teddy bear",
    "tags": [
      "plaything",
      "plush",
      "stuffed",
      "toy"
    ],
    "group": 6,
    "order": 4119
  },
  {
    "emoji": "🪅",
    "label": "piñata",
    "tags": [
      "celebration",
      "party"
    ],
    "group": 6,
    "order": 4120
  },
  {
    "emoji": "🪩",
    "label": "mirror ball",
    "tags": [
      "dance",
      "disco",
      "glitter",
      "party"
    ],
    "group": 6,
    "order": 4121
  },
  {
    "emoji": "🪆",
    "label": "nesting dolls",
    "tags": [
      "doll",
      "nesting",
      "russia"
    ],
    "group": 6,
    "order": 4122
  },
  {
    "emoji": "♠️",
    "label": "spade suit",
    "tags": [
      "card",
      "game"
    ],
    "group": 6,
    "order": 4124
  },
  {
    "emoji": "♥️",
    "label": "heart suit",
    "tags": [
      "card",
      "game"
    ],
    "group": 6,
    "order": 4126
  },
  {
    "emoji": "♦️",
    "label": "diamond suit",
    "tags": [
      "card",
      "game"
    ],
    "group": 6,
    "order": 4128
  },
  {
    "emoji": "♣️",
    "label": "club suit",
    "tags": [
      "card",
      "game"
    ],
    "group": 6,
    "order": 4130
  },
  {
    "emoji": "♟️",
    "label": "chess pawn",
    "tags": [
      "chess",
      "dupe",
      "expendable"
    ],
    "group": 6,
    "order": 4132
  },
  {
    "emoji": "🃏",
    "label": "joker",
    "tags": [
      "card",
      "game",
      "wildcard"
    ],
    "group": 6,
    "order": 4133
  },
  {
    "emoji": "🀄️",
    "label": "mahjong red dragon",
    "tags": [
      "game",
      "mahjong",
      "red"
    ],
    "group": 6,
    "order": 4134
  },
  {
    "emoji": "🎴",
    "label": "flower playing cards",
    "tags": [
      "card",
      "flower",
      "game",
      "japanese",
      "playing"
    ],
    "group": 6,
    "order": 4135
  },
  {
    "emoji": "🎭️",
    "label": "performing arts",
    "tags": [
      "art",
      "mask",
      "performing",
      "theater",
      "theatre"
    ],
    "group": 6,
    "order": 4136
  },
  {
    "emoji": "🖼️",
    "label": "framed picture",
    "tags": [
      "art",
      "frame",
      "museum",
      "painting",
      "picture"
    ],
    "group": 6,
    "order": 4138
  },
  {
    "emoji": "🎨",
    "label": "artist palette",
    "tags": [
      "art",
      "museum",
      "painting",
      "palette"
    ],
    "group": 6,
    "order": 4139
  },
  {
    "emoji": "🧵",
    "label": "thread",
    "tags": [
      "needle",
      "sewing",
      "spool",
      "string"
    ],
    "group": 6,
    "order": 4140
  },
  {
    "emoji": "🪡",
    "label": "sewing needle",
    "tags": [
      "embroidery",
      "needle",
      "sewing",
      "stitches",
      "sutures",
      "tailoring"
    ],
    "group": 6,
    "order": 4141
  },
  {
    "emoji": "🧶",
    "label": "yarn",
    "tags": [
      "ball",
      "crochet",
      "knit"
    ],
    "group": 6,
    "order": 4142
  },
  {
    "emoji": "🪢",
    "label": "knot",
    "tags": [
      "rope",
      "tangled",
      "tie",
      "twine",
      "twist"
    ],
    "group": 6,
    "order": 4143
  },
  {
    "emoji": "👓️",
    "label": "glasses",
    "tags": [
      "clothing",
      "eye",
      "eyeglasses",
      "eyewear"
    ],
    "group": 7,
    "order": 4144
  },
  {
    "emoji": "🕶️",
    "label": "sunglasses",
    "tags": [
      "dark",
      "eye",
      "eyewear",
      "glasses"
    ],
    "group": 7,
    "order": 4146
  },
  {
    "emoji": "🥽",
    "label": "goggles",
    "tags": [
      "eye protection",
      "swimming",
      "welding"
    ],
    "group": 7,
    "order": 4147
  },
  {
    "emoji": "🥼",
    "label": "lab coat",
    "tags": [
      "doctor",
      "experiment",
      "scientist"
    ],
    "group": 7,
    "order": 4148
  },
  {
    "emoji": "🦺",
    "label": "safety vest",
    "tags": [
      "emergency",
      "safety",
      "vest"
    ],
    "group": 7,
    "order": 4149
  },
  {
    "emoji": "👔",
    "label": "necktie",
    "tags": [
      "clothing",
      "tie"
    ],
    "group": 7,
    "order": 4150
  },
  {
    "emoji": "👕",
    "label": "t-shirt",
    "tags": [
      "clothing",
      "shirt",
      "tshirt"
    ],
    "group": 7,
    "order": 4151
  },
  {
    "emoji": "👖",
    "label": "jeans",
    "tags": [
      "clothing",
      "pants",
      "trousers"
    ],
    "group": 7,
    "order": 4152
  },
  {
    "emoji": "🧣",
    "label": "scarf",
    "tags": [
      "neck"
    ],
    "group": 7,
    "order": 4153
  },
  {
    "emoji": "🧤",
    "label": "gloves",
    "tags": [
      "hand"
    ],
    "group": 7,
    "order": 4154
  },
  {
    "emoji": "🧥",
    "label": "coat",
    "tags": [
      "jacket"
    ],
    "group": 7,
    "order": 4155
  },
  {
    "emoji": "🧦",
    "label": "socks",
    "tags": [
      "stocking"
    ],
    "group": 7,
    "order": 4156
  },
  {
    "emoji": "👗",
    "label": "dress",
    "tags": [
      "clothing"
    ],
    "group": 7,
    "order": 4157
  },
  {
    "emoji": "👘",
    "label": "kimono",
    "tags": [
      "clothing"
    ],
    "group": 7,
    "order": 4158
  },
  {
    "emoji": "🥻",
    "label": "sari",
    "tags": [
      "clothing",
      "dress"
    ],
    "group": 7,
    "order": 4159
  },
  {
    "emoji": "🩱",
    "label": "one-piece swimsuit",
    "tags": [
      "bathing suit"
    ],
    "group": 7,
    "order": 4160
  },
  {
    "emoji": "🩲",
    "label": "briefs",
    "tags": [
      "bathing suit",
      "one-piece",
      "swimsuit",
      "underwear"
    ],
    "group": 7,
    "order": 4161
  },
  {
    "emoji": "🩳",
    "label": "shorts",
    "tags": [
      "bathing suit",
      "pants",
      "underwear"
    ],
    "group": 7,
    "order": 4162
  },
  {
    "emoji": "👙",
    "label": "bikini",
    "tags": [
      "clothing",
      "swim"
    ],
    "group": 7,
    "order": 4163
  },
  {
    "emoji": "👚",
    "label": "woman’s clothes",
    "tags": [
      "clothing",
      "woman"
    ],
    "group": 7,
    "order": 4164
  },
  {
    "emoji": "🪭",
    "label": "folding hand fan",
    "tags": [
      "cooling",
      "dance",
      "fan",
      "flutter",
      "hot",
      "shy"
    ],
    "group": 7,
    "order": 4165
  },
  {
    "emoji": "👛",
    "label": "purse",
    "tags": [
      "clothing",
      "coin"
    ],
    "group": 7,
    "order": 4166
  },
  {
    "emoji": "👜",
    "label": "handbag",
    "tags": [
      "bag",
      "clothing",
      "purse"
    ],
    "group": 7,
    "order": 4167
  },
  {
    "emoji": "👝",
    "label": "clutch bag",
    "tags": [
      "bag",
      "clothing",
      "pouch"
    ],
    "group": 7,
    "order": 4168
  },
  {
    "emoji": "🛍️",
    "label": "shopping bags",
    "tags": [
      "bag",
      "hotel",
      "shopping"
    ],
    "group": 7,
    "order": 4170
  },
  {
    "emoji": "🎒",
    "label": "backpack",
    "tags": [
      "bag",
      "rucksack",
      "satchel",
      "school"
    ],
    "group": 7,
    "order": 4171
  },
  {
    "emoji": "🩴",
    "label": "thong sandal",
    "tags": [
      "beach sandals",
      "sandals",
      "thong sandals",
      "thongs",
      "zōri"
    ],
    "group": 7,
    "order": 4172
  },
  {
    "emoji": "👞",
    "label": "man’s shoe",
    "tags": [
      "clothing",
      "man",
      "shoe"
    ],
    "group": 7,
    "order": 4173
  },
  {
    "emoji": "👟",
    "label": "running shoe",
    "tags": [
      "athletic",
      "clothing",
      "shoe",
      "sneaker"
    ],
    "group": 7,
    "order": 4174
  },
  {
    "emoji": "🥾",
    "label": "hiking boot",
    "tags": [
      "backpacking",
      "boot",
      "camping",
      "hiking"
    ],
    "group": 7,
    "order": 4175
  },
  {
    "emoji": "🥿",
    "label": "flat shoe",
    "tags": [
      "ballet flat",
      "slip-on",
      "slipper"
    ],
    "group": 7,
    "order": 4176
  },
  {
    "emoji": "👠",
    "label": "high-heeled shoe",
    "tags": [
      "clothing",
      "heel",
      "shoe",
      "woman"
    ],
    "group": 7,
    "order": 4177
  },
  {
    "emoji": "👡",
    "label": "woman’s sandal",
    "tags": [
      "clothing",
      "sandal",
      "shoe",
      "woman"
    ],
    "group": 7,
    "order": 4178
  },
  {
    "emoji": "🩰",
    "label": "ballet shoes",
    "tags": [
      "ballet",
      "dance"
    ],
    "group": 7,
    "order": 4179
  },
  {
    "emoji": "👢",
    "label": "woman’s boot",
    "tags": [
      "boot",
      "clothing",
      "shoe",
      "woman"
    ],
    "group": 7,
    "order": 4180
  },
  {
    "emoji": "🪮",
    "label": "hair pick",
    "tags": [
      "afro",
      "comb",
      "hair",
      "pick"
    ],
    "group": 7,
    "order": 4181
  },
  {
    "emoji": "👑",
    "label": "crown",
    "tags": [
      "clothing",
      "king",
      "queen"
    ],
    "group": 7,
    "order": 4182
  },
  {
    "emoji": "👒",
    "label": "woman’s hat",
    "tags": [
      "clothing",
      "hat",
      "woman"
    ],
    "group": 7,
    "order": 4183
  },
  {
    "emoji": "🎩",
    "label": "top hat",
    "tags": [
      "clothing",
      "hat",
      "top",
      "tophat"
    ],
    "group": 7,
    "order": 4184
  },
  {
    "emoji": "🎓️",
    "label": "graduation cap",
    "tags": [
      "cap",
      "celebration",
      "clothing",
      "graduation",
      "hat"
    ],
    "group": 7,
    "order": 4185
  },
  {
    "emoji": "🧢",
    "label": "billed cap",
    "tags": [
      "baseball cap"
    ],
    "group": 7,
    "order": 4186
  },
  {
    "emoji": "🪖",
    "label": "military helmet",
    "tags": [
      "army",
      "helmet",
      "military",
      "soldier",
      "warrior"
    ],
    "group": 7,
    "order": 4187
  },
  {
    "emoji": "⛑️",
    "label": "rescue worker’s helmet",
    "tags": [
      "aid",
      "cross",
      "face",
      "hat",
      "helmet"
    ],
    "group": 7,
    "order": 4189
  },
  {
    "emoji": "📿",
    "label": "prayer beads",
    "tags": [
      "beads",
      "clothing",
      "necklace",
      "prayer",
      "religion"
    ],
    "group": 7,
    "order": 4190
  },
  {
    "emoji": "💄",
    "label": "lipstick",
    "tags": [
      "cosmetics",
      "makeup"
    ],
    "group": 7,
    "order": 4191
  },
  {
    "emoji": "💍",
    "label": "ring",
    "tags": [
      "diamond"
    ],
    "group": 7,
    "order": 4192
  },
  {
    "emoji": "💎",
    "label": "gem stone",
    "tags": [
      "diamond",
      "gem",
      "jewel"
    ],
    "group": 7,
    "order": 4193
  },
  {
    "emoji": "🔇",
    "label": "muted speaker",
    "tags": [
      "mute",
      "quiet",
      "silent",
      "speaker"
    ],
    "group": 7,
    "order": 4194
  },
  {
    "emoji": "🔈️",
    "label": "speaker low volume",
    "tags": [
      "soft"
    ],
    "group": 7,
    "order": 4195
  },
  {
    "emoji": "🔉",
    "label": "speaker medium volume",
    "tags": [
      "medium"
    ],
    "group": 7,
    "order": 4196
  },
  {
    "emoji": "🔊",
    "label": "speaker high volume",
    "tags": [
      "loud"
    ],
    "group": 7,
    "order": 4197
  },
  {
    "emoji": "📢",
    "label": "loudspeaker",
    "tags": [
      "loud",
      "public address"
    ],
    "group": 7,
    "order": 4198
  },
  {
    "emoji": "📣",
    "label": "megaphone",
    "tags": [
      "cheering"
    ],
    "group": 7,
    "order": 4199
  },
  {
    "emoji": "📯",
    "label": "postal horn",
    "tags": [
      "horn",
      "post",
      "postal"
    ],
    "group": 7,
    "order": 4200
  },
  {
    "emoji": "🔔",
    "label": "bell",
    "tags": [
      "bell"
    ],
    "group": 7,
    "order": 4201
  },
  {
    "emoji": "🔕",
    "label": "bell with slash",
    "tags": [
      "bell",
      "forbidden",
      "mute",
      "quiet",
      "silent"
    ],
    "group": 7,
    "order": 4202
  },
  {
    "emoji": "🎼",
    "label": "musical score",
    "tags": [
      "music",
      "score"
    ],
    "group": 7,
    "order": 4203
  },
  {
    "emoji": "🎵",
    "label": "musical note",
    "tags": [
      "music",
      "note"
    ],
    "group": 7,
    "order": 4204
  },
  {
    "emoji": "🎶",
    "label": "musical notes",
    "tags": [
      "music",
      "note",
      "notes"
    ],
    "group": 7,
    "order": 4205
  },
  {
    "emoji": "🎙️",
    "label": "studio microphone",
    "tags": [
      "mic",
      "microphone",
      "music",
      "studio"
    ],
    "group": 7,
    "order": 4207
  },
  {
    "emoji": "🎚️",
    "label": "level slider",
    "tags": [
      "level",
      "music",
      "slider"
    ],
    "group": 7,
    "order": 4209
  },
  {
    "emoji": "🎛️",
    "label": "control knobs",
    "tags": [
      "control",
      "knobs",
      "music"
    ],
    "group": 7,
    "order": 4211
  },
  {
    "emoji": "🎤",
    "label": "microphone",
    "tags": [
      "karaoke",
      "mic"
    ],
    "group": 7,
    "order": 4212
  },
  {
    "emoji": "🎧️",
    "label": "headphone",
    "tags": [
      "earbud"
    ],
    "group": 7,
    "order": 4213
  },
  {
    "emoji": "📻️",
    "label": "radio",
    "tags": [
      "video"
    ],
    "group": 7,
    "order": 4214
  },
  {
    "emoji": "🎷",
    "label": "saxophone",
    "tags": [
      "instrument",
      "music",
      "sax"
    ],
    "group": 7,
    "order": 4215
  },
  {
    "emoji": "🪗",
    "label": "accordion",
    "tags": [
      "concertina",
      "squeeze box"
    ],
    "group": 7,
    "order": 4216
  },
  {
    "emoji": "🎸",
    "label": "guitar",
    "tags": [
      "instrument",
      "music"
    ],
    "group": 7,
    "order": 4217
  },
  {
    "emoji": "🎹",
    "label": "musical keyboard",
    "tags": [
      "instrument",
      "keyboard",
      "music",
      "piano"
    ],
    "group": 7,
    "order": 4218
  },
  {
    "emoji": "🎺",
    "label": "trumpet",
    "tags": [
      "instrument",
      "music"
    ],
    "group": 7,
    "order": 4219
  },
  {
    "emoji": "🎻",
    "label": "violin",
    "tags": [
      "instrument",
      "music"
    ],
    "group": 7,
    "order": 4220
  },
  {
    "emoji": "🪕",
    "label": "banjo",
    "tags": [
      "music",
      "stringed"
    ],
    "group": 7,
    "order": 4221
  },
  {
    "emoji": "🥁",
    "label": "drum",
    "tags": [
      "drumsticks",
      "music"
    ],
    "group": 7,
    "order": 4222
  },
  {
    "emoji": "🪘",
    "label": "long drum",
    "tags": [
      "beat",
      "conga",
      "drum",
      "rhythm"
    ],
    "group": 7,
    "order": 4223
  },
  {
    "emoji": "🪇",
    "label": "maracas",
    "tags": [
      "instrument",
      "music",
      "percussion",
      "rattle",
      "shake"
    ],
    "group": 7,
    "order": 4224
  },
  {
    "emoji": "🪈",
    "label": "flute",
    "tags": [
      "fife",
      "music",
      "pipe",
      "recorder",
      "woodwind"
    ],
    "group": 7,
    "order": 4225
  },
  {
    "emoji": "📱",
    "label": "mobile phone",
    "tags": [
      "cell",
      "mobile",
      "phone",
      "telephone"
    ],
    "group": 7,
    "order": 4226
  },
  {
    "emoji": "📲",
    "label": "mobile phone with arrow",
    "tags": [
      "arrow",
      "cell",
      "mobile",
      "phone",
      "receive"
    ],
    "group": 7,
    "order": 4227
  },
  {
    "emoji": "☎️",
    "label": "telephone",
    "tags": [
      "phone"
    ],
    "group": 7,
    "order": 4229
  },
  {
    "emoji": "📞",
    "label": "telephone receiver",
    "tags": [
      "phone",
      "receiver",
      "telephone"
    ],
    "group": 7,
    "order": 4230
  },
  {
    "emoji": "📟️",
    "label": "pager",
    "tags": [
      "pager"
    ],
    "group": 7,
    "order": 4231
  },
  {
    "emoji": "📠",
    "label": "fax machine",
    "tags": [
      "fax"
    ],
    "group": 7,
    "order": 4232
  },
  {
    "emoji": "🔋",
    "label": "battery",
    "tags": [
      "battery"
    ],
    "group": 7,
    "order": 4233
  },
  {
    "emoji": "🪫",
    "label": "low battery",
    "tags": [
      "electronic",
      "low energy"
    ],
    "group": 7,
    "order": 4234
  },
  {
    "emoji": "🔌",
    "label": "electric plug",
    "tags": [
      "electric",
      "electricity",
      "plug"
    ],
    "group": 7,
    "order": 4235
  },
  {
    "emoji": "💻️",
    "label": "laptop",
    "tags": [
      "computer",
      "pc",
      "personal"
    ],
    "group": 7,
    "order": 4236
  },
  {
    "emoji": "🖥️",
    "label": "desktop computer",
    "tags": [
      "computer",
      "desktop"
    ],
    "group": 7,
    "order": 4238
  },
  {
    "emoji": "🖨️",
    "label": "printer",
    "tags": [
      "computer"
    ],
    "group": 7,
    "order": 4240
  },
  {
    "emoji": "⌨️",
    "label": "keyboard",
    "tags": [
      "computer"
    ],
    "group": 7,
    "order": 4242
  },
  {
    "emoji": "🖱️",
    "label": "computer mouse",
    "tags": [
      "computer"
    ],
    "group": 7,
    "order": 4244
  },
  {
    "emoji": "🖲️",
    "label": "trackball",
    "tags": [
      "computer"
    ],
    "group": 7,
    "order": 4246
  },
  {
    "emoji": "💽",
    "label": "computer disk",
    "tags": [
      "computer",
      "disk",
      "minidisk",
      "optical"
    ],
    "group": 7,
    "order": 4247
  },
  {
    "emoji": "💾",
    "label": "floppy disk",
    "tags": [
      "computer",
      "disk",
      "floppy"
    ],
    "group": 7,
    "order": 4248
  },
  {
    "emoji": "💿️",
    "label": "optical disk",
    "tags": [
      "cd",
      "computer",
      "disk",
      "optical"
    ],
    "group": 7,
    "order": 4249
  },
  {
    "emoji": "📀",
    "label": "dvd",
    "tags": [
      "blu-ray",
      "computer",
      "disk",
      "optical"
    ],
    "group": 7,
    "order": 4250
  },
  {
    "emoji": "🧮",
    "label": "abacus",
    "tags": [
      "calculation"
    ],
    "group": 7,
    "order": 4251
  },
  {
    "emoji": "🎥",
    "label": "movie camera",
    "tags": [
      "camera",
      "cinema",
      "movie"
    ],
    "group": 7,
    "order": 4252
  },
  {
    "emoji": "🎞️",
    "label": "film frames",
    "tags": [
      "cinema",
      "film",
      "frames",
      "movie"
    ],
    "group": 7,
    "order": 4254
  },
  {
    "emoji": "📽️",
    "label": "film projector",
    "tags": [
      "cinema",
      "film",
      "movie",
      "projector",
      "video"
    ],
    "group": 7,
    "order": 4256
  },
  {
    "emoji": "🎬️",
    "label": "clapper board",
    "tags": [
      "clapper",
      "movie"
    ],
    "group": 7,
    "order": 4257
  },
  {
    "emoji": "📺️",
    "label": "television",
    "tags": [
      "tv",
      "video"
    ],
    "group": 7,
    "order": 4258
  },
  {
    "emoji": "📷️",
    "label": "camera",
    "tags": [
      "video"
    ],
    "group": 7,
    "order": 4259
  },
  {
    "emoji": "📸",
    "label": "camera with flash",
    "tags": [
      "camera",
      "flash",
      "video"
    ],
    "group": 7,
    "order": 4260
  },
  {
    "emoji": "📹️",
    "label": "video camera",
    "tags": [
      "camera",
      "video"
    ],
    "group": 7,
    "order": 4261
  },
  {
    "emoji": "📼",
    "label": "videocassette",
    "tags": [
      "tape",
      "vhs",
      "video"
    ],
    "group": 7,
    "order": 4262
  },
  {
    "emoji": "🔍️",
    "label": "magnifying glass tilted left",
    "tags": [
      "glass",
      "magnifying",
      "search",
      "tool"
    ],
    "group": 7,
    "order": 4263
  },
  {
    "emoji": "🔎",
    "label": "magnifying glass tilted right",
    "tags": [
      "glass",
      "magnifying",
      "search",
      "tool"
    ],
    "group": 7,
    "order": 4264
  },
  {
    "emoji": "🕯️",
    "label": "candle",
    "tags": [
      "light"
    ],
    "group": 7,
    "order": 4266
  },
  {
    "emoji": "💡",
    "label": "light bulb",
    "tags": [
      "bulb",
      "comic",
      "electric",
      "idea",
      "light"
    ],
    "group": 7,
    "order": 4267
  },
  {
    "emoji": "🔦",
    "label": "flashlight",
    "tags": [
      "electric",
      "light",
      "tool",
      "torch"
    ],
    "group": 7,
    "order": 4268
  },
  {
    "emoji": "🏮",
    "label": "red paper lantern",
    "tags": [
      "bar",
      "lantern",
      "light",
      "red"
    ],
    "group": 7,
    "order": 4269
  },
  {
    "emoji": "🪔",
    "label": "diya lamp",
    "tags": [
      "diya",
      "lamp",
      "oil"
    ],
    "group": 7,
    "order": 4270
  },
  {
    "emoji": "📔",
    "label": "notebook with decorative cover",
    "tags": [
      "book",
      "cover",
      "decorated",
      "notebook"
    ],
    "group": 7,
    "order": 4271
  },
  {
    "emoji": "📕",
    "label": "closed book",
    "tags": [
      "book",
      "closed"
    ],
    "group": 7,
    "order": 4272
  },
  {
    "emoji": "📖",
    "label": "open book",
    "tags": [
      "book",
      "open"
    ],
    "group": 7,
    "order": 4273
  },
  {
    "emoji": "📗",
    "label": "green book",
    "tags": [
      "book",
      "green"
    ],
    "group": 7,
    "order": 4274
  },
  {
    "emoji": "📘",
    "label": "blue book",
    "tags": [
      "blue",
      "book"
    ],
    "group": 7,
    "order": 4275
  },
  {
    "emoji": "📙",
    "label": "orange book",
    "tags": [
      "book",
      "orange"
    ],
    "group": 7,
    "order": 4276
  },
  {
    "emoji": "📚️",
    "label": "books",
    "tags": [
      "book"
    ],
    "group": 7,
    "order": 4277
  },
  {
    "emoji": "📓",
    "label": "notebook",
    "tags": [
      "notebook"
    ],
    "group": 7,
    "order": 4278
  },
  {
    "emoji": "📒",
    "label": "ledger",
    "tags": [
      "notebook"
    ],
    "group": 7,
    "order": 4279
  },
  {
    "emoji": "📃",
    "label": "page with curl",
    "tags": [
      "curl",
      "document",
      "page"
    ],
    "group": 7,
    "order": 4280
  },
  {
    "emoji": "📜",
    "label": "scroll",
    "tags": [
      "paper"
    ],
    "group": 7,
    "order": 4281
  },
  {
    "emoji": "📄",
    "label": "page facing up",
    "tags": [
      "document",
      "page"
    ],
    "group": 7,
    "order": 4282
  },
  {
    "emoji": "📰",
    "label": "newspaper",
    "tags": [
      "news",
      "paper"
    ],
    "group": 7,
    "order": 4283
  },
  {
    "emoji": "🗞️",
    "label": "rolled-up newspaper",
    "tags": [
      "news",
      "newspaper",
      "paper",
      "rolled"
    ],
    "group": 7,
    "order": 4285
  },
  {
    "emoji": "📑",
    "label": "bookmark tabs",
    "tags": [
      "bookmark",
      "mark",
      "marker",
      "tabs"
    ],
    "group": 7,
    "order": 4286
  },
  {
    "emoji": "🔖",
    "label": "bookmark",
    "tags": [
      "mark"
    ],
    "group": 7,
    "order": 4287
  },
  {
    "emoji": "🏷️",
    "label": "label",
    "tags": [
      "label"
    ],
    "group": 7,
    "order": 4289
  },
  {
    "emoji": "💰️",
    "label": "money bag",
    "tags": [
      "bag",
      "dollar",
      "money",
      "moneybag"
    ],
    "group": 7,
    "order": 4290
  },
  {
    "emoji": "🪙",
    "label": "coin",
    "tags": [
      "gold",
      "metal",
      "money",
      "silver",
      "treasure"
    ],
    "group": 7,
    "order": 4291
  },
  {
    "emoji": "💴",
    "label": "yen banknote",
    "tags": [
      "banknote",
      "bill",
      "currency",
      "money",
      "note",
      "yen"
    ],
    "group": 7,
    "order": 4292
  },
  {
    "emoji": "💵",
    "label": "dollar banknote",
    "tags": [
      "banknote",
      "bill",
      "currency",
      "dollar",
      "money",
      "note"
    ],
    "group": 7,
    "order": 4293
  },
  {
    "emoji": "💶",
    "label": "euro banknote",
    "tags": [
      "banknote",
      "bill",
      "currency",
      "euro",
      "money",
      "note"
    ],
    "group": 7,
    "order": 4294
  },
  {
    "emoji": "💷",
    "label": "pound banknote",
    "tags": [
      "banknote",
      "bill",
      "currency",
      "money",
      "note",
      "pound"
    ],
    "group": 7,
    "order": 4295
  },
  {
    "emoji": "💸",
    "label": "money with wings",
    "tags": [
      "banknote",
      "bill",
      "fly",
      "money",
      "wings"
    ],
    "group": 7,
    "order": 4296
  },
  {
    "emoji": "💳️",
    "label": "credit card",
    "tags": [
      "card",
      "credit",
      "money"
    ],
    "group": 7,
    "order": 4297
  },
  {
    "emoji": "🧾",
    "label": "receipt",
    "tags": [
      "accounting",
      "bookkeeping",
      "evidence",
      "proof"
    ],
    "group": 7,
    "order": 4298
  },
  {
    "emoji": "💹",
    "label": "chart increasing with yen",
    "tags": [
      "chart",
      "graph",
      "growth",
      "money",
      "yen"
    ],
    "group": 7,
    "order": 4299
  },
  {
    "emoji": "✉️",
    "label": "envelope",
    "tags": [
      "email",
      "letter"
    ],
    "group": 7,
    "order": 4301
  },
  {
    "emoji": "📧",
    "label": "e-mail",
    "tags": [
      "email",
      "letter",
      "mail"
    ],
    "group": 7,
    "order": 4302
  },
  {
    "emoji": "📨",
    "label": "incoming envelope",
    "tags": [
      "e-mail",
      "email",
      "envelope",
      "incoming",
      "letter",
      "receive"
    ],
    "group": 7,
    "order": 4303
  },
  {
    "emoji": "📩",
    "label": "envelope with arrow",
    "tags": [
      "arrow",
      "e-mail",
      "email",
      "envelope",
      "outgoing"
    ],
    "group": 7,
    "order": 4304
  },
  {
    "emoji": "📤️",
    "label": "outbox tray",
    "tags": [
      "box",
      "letter",
      "mail",
      "outbox",
      "sent",
      "tray"
    ],
    "group": 7,
    "order": 4305
  },
  {
    "emoji": "📥️",
    "label": "inbox tray",
    "tags": [
      "box",
      "inbox",
      "letter",
      "mail",
      "receive",
      "tray"
    ],
    "group": 7,
    "order": 4306
  },
  {
    "emoji": "📦️",
    "label": "package",
    "tags": [
      "box",
      "parcel"
    ],
    "group": 7,
    "order": 4307
  },
  {
    "emoji": "📫️",
    "label": "closed mailbox with raised flag",
    "tags": [
      "closed",
      "mail",
      "mailbox",
      "postbox"
    ],
    "group": 7,
    "order": 4308
  },
  {
    "emoji": "📪️",
    "label": "closed mailbox with lowered flag",
    "tags": [
      "closed",
      "lowered",
      "mail",
      "mailbox",
      "postbox"
    ],
    "group": 7,
    "order": 4309
  },
  {
    "emoji": "📬️",
    "label": "open mailbox with raised flag",
    "tags": [
      "mail",
      "mailbox",
      "open",
      "postbox"
    ],
    "group": 7,
    "order": 4310
  },
  {
    "emoji": "📭️",
    "label": "open mailbox with lowered flag",
    "tags": [
      "lowered",
      "mail",
      "mailbox",
      "open",
      "postbox"
    ],
    "group": 7,
    "order": 4311
  },
  {
    "emoji": "📮",
    "label": "postbox",
    "tags": [
      "mail",
      "mailbox"
    ],
    "group": 7,
    "order": 4312
  },
  {
    "emoji": "🗳️",
    "label": "ballot box with ballot",
    "tags": [
      "ballot",
      "box"
    ],
    "group": 7,
    "order": 4314
  },
  {
    "emoji": "✏️",
    "label": "pencil",
    "tags": [
      "pencil"
    ],
    "group": 7,
    "order": 4316
  },
  {
    "emoji": "✒️",
    "label": "black nib",
    "tags": [
      "nib",
      "pen"
    ],
    "group": 7,
    "order": 4318
  },
  {
    "emoji": "🖋️",
    "label": "fountain pen",
    "tags": [
      "fountain",
      "pen"
    ],
    "group": 7,
    "order": 4320
  },
  {
    "emoji": "🖊️",
    "label": "pen",
    "tags": [
      "ballpoint"
    ],
    "group": 7,
    "order": 4322
  },
  {
    "emoji": "🖌️",
    "label": "paintbrush",
    "tags": [
      "painting"
    ],
    "group": 7,
    "order": 4324
  },
  {
    "emoji": "🖍️",
    "label": "crayon",
    "tags": [
      "crayon"
    ],
    "group": 7,
    "order": 4326
  },
  {
    "emoji": "📝",
    "label": "memo",
    "tags": [
      "pencil"
    ],
    "group": 7,
    "order": 4327
  },
  {
    "emoji": "💼",
    "label": "briefcase",
    "tags": [
      "briefcase"
    ],
    "group": 7,
    "order": 4328
  },
  {
    "emoji": "📁",
    "label": "file folder",
    "tags": [
      "file",
      "folder"
    ],
    "group": 7,
    "order": 4329
  },
  {
    "emoji": "📂",
    "label": "open file folder",
    "tags": [
      "file",
      "folder",
      "open"
    ],
    "group": 7,
    "order": 4330
  },
  {
    "emoji": "🗂️",
    "label": "card index dividers",
    "tags": [
      "card",
      "dividers",
      "index"
    ],
    "group": 7,
    "order": 4332
  },
  {
    "emoji": "📅",
    "label": "calendar",
    "tags": [
      "date"
    ],
    "group": 7,
    "order": 4333
  },
  {
    "emoji": "📆",
    "label": "tear-off calendar",
    "tags": [
      "calendar"
    ],
    "group": 7,
    "order": 4334
  },
  {
    "emoji": "🗒️",
    "label": "spiral notepad",
    "tags": [
      "note",
      "pad",
      "spiral"
    ],
    "group": 7,
    "order": 4336
  },
  {
    "emoji": "🗓️",
    "label": "spiral calendar",
    "tags": [
      "calendar",
      "pad",
      "spiral"
    ],
    "group": 7,
    "order": 4338
  },
  {
    "emoji": "📇",
    "label": "card index",
    "tags": [
      "card",
      "index",
      "rolodex"
    ],
    "group": 7,
    "order": 4339
  },
  {
    "emoji": "📈",
    "label": "chart increasing",
    "tags": [
      "chart",
      "graph",
      "growth",
      "trend",
      "upward"
    ],
    "group": 7,
    "order": 4340
  },
  {
    "emoji": "📉",
    "label": "chart decreasing",
    "tags": [
      "chart",
      "down",
      "graph",
      "trend"
    ],
    "group": 7,
    "order": 4341
  },
  {
    "emoji": "📊",
    "label": "bar chart",
    "tags": [
      "bar",
      "chart",
      "graph"
    ],
    "group": 7,
    "order": 4342
  },
  {
    "emoji": "📋️",
    "label": "clipboard",
    "tags": [
      "clipboard"
    ],
    "group": 7,
    "order": 4343
  },
  {
    "emoji": "📌",
    "label": "pushpin",
    "tags": [
      "pin"
    ],
    "group": 7,
    "order": 4344
  },
  {
    "emoji": "📍",
    "label": "round pushpin",
    "tags": [
      "pin",
      "pushpin"
    ],
    "group": 7,
    "order": 4345
  },
  {
    "emoji": "📎",
    "label": "paperclip",
    "tags": [
      "paperclip"
    ],
    "group": 7,
    "order": 4346
  },
  {
    "emoji": "🖇️",
    "label": "linked paperclips",
    "tags": [
      "link",
      "paperclip"
    ],
    "group": 7,
    "order": 4348
  },
  {
    "emoji": "📏",
    "label": "straight ruler",
    "tags": [
      "ruler",
      "straight edge"
    ],
    "group": 7,
    "order": 4349
  },
  {
    "emoji": "📐",
    "label": "triangular ruler",
    "tags": [
      "ruler",
      "set",
      "triangle"
    ],
    "group": 7,
    "order": 4350
  },
  {
    "emoji": "✂️",
    "label": "scissors",
    "tags": [
      "cutting",
      "tool"
    ],
    "group": 7,
    "order": 4352
  },
  {
    "emoji": "🗃️",
    "label": "card file box",
    "tags": [
      "box",
      "card",
      "file"
    ],
    "group": 7,
    "order": 4354
  },
  {
    "emoji": "🗄️",
    "label": "file cabinet",
    "tags": [
      "cabinet",
      "file",
      "filing"
    ],
    "group": 7,
    "order": 4356
  },
  {
    "emoji": "🗑️",
    "label": "wastebasket",
    "tags": [
      "wastebasket"
    ],
    "group": 7,
    "order": 4358
  },
  {
    "emoji": "🔒️",
    "label": "locked",
    "tags": [
      "closed"
    ],
    "group": 7,
    "order": 4359
  },
  {
    "emoji": "🔓️",
    "label": "unlocked",
    "tags": [
      "lock",
      "open",
      "unlock"
    ],
    "group": 7,
    "order": 4360
  },
  {
    "emoji": "🔏",
    "label": "locked with pen",
    "tags": [
      "ink",
      "lock",
      "nib",
      "pen",
      "privacy"
    ],
    "group": 7,
    "order": 4361
  },
  {
    "emoji": "🔐",
    "label": "locked with key",
    "tags": [
      "closed",
      "key",
      "lock",
      "secure"
    ],
    "group": 7,
    "order": 4362
  },
  {
    "emoji": "🔑",
    "label": "key",
    "tags": [
      "lock",
      "password"
    ],
    "group": 7,
    "order": 4363
  },
  {
    "emoji": "🗝️",
    "label": "old key",
    "tags": [
      "clue",
      "key",
      "lock",
      "old"
    ],
    "group": 7,
    "order": 4365
  },
  {
    "emoji": "🔨",
    "label": "hammer",
    "tags": [
      "tool"
    ],
    "group": 7,
    "order": 4366
  },
  {
    "emoji": "🪓",
    "label": "axe",
    "tags": [
      "chop",
      "hatchet",
      "split",
      "wood"
    ],
    "group": 7,
    "order": 4367
  },
  {
    "emoji": "⛏️",
    "label": "pick",
    "tags": [
      "mining",
      "tool"
    ],
    "group": 7,
    "order": 4369
  },
  {
    "emoji": "⚒️",
    "label": "hammer and pick",
    "tags": [
      "hammer",
      "pick",
      "tool"
    ],
    "group": 7,
    "order": 4371
  },
  {
    "emoji": "🛠️",
    "label": "hammer and wrench",
    "tags": [
      "hammer",
      "spanner",
      "tool",
      "wrench"
    ],
    "group": 7,
    "order": 4373
  },
  {
    "emoji": "🗡️",
    "label": "dagger",
    "tags": [
      "knife",
      "weapon"
    ],
    "group": 7,
    "order": 4375
  },
  {
    "emoji": "⚔️",
    "label": "crossed swords",
    "tags": [
      "crossed",
      "swords",
      "weapon"
    ],
    "group": 7,
    "order": 4377
  },
  {
    "emoji": "💣️",
    "label": "bomb",
    "tags": [
      "comic"
    ],
    "group": 7,
    "order": 4378
  },
  {
    "emoji": "🪃",
    "label": "boomerang",
    "tags": [
      "rebound",
      "repercussion"
    ],
    "group": 7,
    "order": 4379
  },
  {
    "emoji": "🏹",
    "label": "bow and arrow",
    "tags": [
      "archer",
      "arrow",
      "bow",
      "sagittarius",
      "zodiac"
    ],
    "group": 7,
    "order": 4380
  },
  {
    "emoji": "🛡️",
    "label": "shield",
    "tags": [
      "weapon"
    ],
    "group": 7,
    "order": 4382
  },
  {
    "emoji": "🪚",
    "label": "carpentry saw",
    "tags": [
      "carpenter",
      "lumber",
      "saw",
      "tool"
    ],
    "group": 7,
    "order": 4383
  },
  {
    "emoji": "🔧",
    "label": "wrench",
    "tags": [
      "spanner",
      "tool"
    ],
    "group": 7,
    "order": 4384
  },
  {
    "emoji": "🪛",
    "label": "screwdriver",
    "tags": [
      "screw",
      "tool"
    ],
    "group": 7,
    "order": 4385
  },
  {
    "emoji": "🔩",
    "label": "nut and bolt",
    "tags": [
      "bolt",
      "nut",
      "tool"
    ],
    "group": 7,
    "order": 4386
  },
  {
    "emoji": "⚙️",
    "label": "gear",
    "tags": [
      "cog",
      "cogwheel",
      "tool"
    ],
    "group": 7,
    "order": 4388
  },
  {
    "emoji": "🗜️",
    "label": "clamp",
    "tags": [
      "compress",
      "tool",
      "vice"
    ],
    "group": 7,
    "order": 4390
  },
  {
    "emoji": "⚖️",
    "label": "balance scale",
    "tags": [
      "balance",
      "justice",
      "libra",
      "scale",
      "zodiac"
    ],
    "group": 7,
    "order": 4392
  },
  {
    "emoji": "🦯",
    "label": "white cane",
    "tags": [
      "accessibility",
      "blind"
    ],
    "group": 7,
    "order": 4393
  },
  {
    "emoji": "🔗",
    "label": "link",
    "tags": [
      "link"
    ],
    "group": 7,
    "order": 4394
  },
  {
    "emoji": "⛓️‍💥",
    "label": "broken chain",
    "tags": [
      "break",
      "breaking",
      "chain",
      "cuffs",
      "freedom"
    ],
    "group": 7,
    "order": 4395
  },
  {
    "emoji": "⛓️",
    "label": "chains",
    "tags": [
      "chain"
    ],
    "group": 7,
    "order": 4398
  },
  {
    "emoji": "🪝",
    "label": "hook",
    "tags": [
      "catch",
      "crook",
      "curve",
      "ensnare",
      "selling point"
    ],
    "group": 7,
    "order": 4399
  },
  {
    "emoji": "🧰",
    "label": "toolbox",
    "tags": [
      "chest",
      "mechanic",
      "tool"
    ],
    "group": 7,
    "order": 4400
  },
  {
    "emoji": "🧲",
    "label": "magnet",
    "tags": [
      "attraction",
      "horseshoe",
      "magnetic"
    ],
    "group": 7,
    "order": 4401
  },
  {
    "emoji": "🪜",
    "label": "ladder",
    "tags": [
      "climb",
      "rung",
      "step"
    ],
    "group": 7,
    "order": 4402
  },
  {
    "emoji": "⚗️",
    "label": "alembic",
    "tags": [
      "chemistry",
      "tool"
    ],
    "group": 7,
    "order": 4404
  },
  {
    "emoji": "🧪",
    "label": "test tube",
    "tags": [
      "chemist",
      "chemistry",
      "experiment",
      "lab",
      "science"
    ],
    "group": 7,
    "order": 4405
  },
  {
    "emoji": "🧫",
    "label": "petri dish",
    "tags": [
      "bacteria",
      "biologist",
      "biology",
      "culture",
      "lab"
    ],
    "group": 7,
    "order": 4406
  },
  {
    "emoji": "🧬",
    "label": "dna",
    "tags": [
      "biologist",
      "evolution",
      "gene",
      "genetics",
      "life"
    ],
    "group": 7,
    "order": 4407
  },
  {
    "emoji": "🔬",
    "label": "microscope",
    "tags": [
      "science",
      "tool"
    ],
    "group": 7,
    "order": 4408
  },
  {
    "emoji": "🔭",
    "label": "telescope",
    "tags": [
      "science",
      "tool"
    ],
    "group": 7,
    "order": 4409
  },
  {
    "emoji": "📡",
    "label": "satellite antenna",
    "tags": [
      "antenna",
      "dish",
      "satellite"
    ],
    "group": 7,
    "order": 4410
  },
  {
    "emoji": "💉",
    "label": "syringe",
    "tags": [
      "medicine",
      "needle",
      "shot",
      "sick"
    ],
    "group": 7,
    "order": 4411
  },
  {
    "emoji": "🩸",
    "label": "drop of blood",
    "tags": [
      "bleed",
      "blood donation",
      "injury",
      "medicine",
      "menstruation"
    ],
    "group": 7,
    "order": 4412
  },
  {
    "emoji": "💊",
    "label": "pill",
    "tags": [
      "doctor",
      "medicine",
      "sick"
    ],
    "group": 7,
    "order": 4413
  },
  {
    "emoji": "🩹",
    "label": "adhesive bandage",
    "tags": [
      "bandage"
    ],
    "group": 7,
    "order": 4414
  },
  {
    "emoji": "🩼",
    "label": "crutch",
    "tags": [
      "cane",
      "disability",
      "hurt",
      "mobility aid",
      "stick"
    ],
    "group": 7,
    "order": 4415
  },
  {
    "emoji": "🩺",
    "label": "stethoscope",
    "tags": [
      "doctor",
      "heart",
      "medicine"
    ],
    "group": 7,
    "order": 4416
  },
  {
    "emoji": "🩻",
    "label": "x-ray",
    "tags": [
      "bones",
      "doctor",
      "medical",
      "skeleton"
    ],
    "group": 7,
    "order": 4417
  },
  {
    "emoji": "🚪",
    "label": "door",
    "tags": [
      "door"
    ],
    "group": 7,
    "order": 4418
  },
  {
    "emoji": "🛗",
    "label": "elevator",
    "tags": [
      "accessibility",
      "hoist",
      "lift"
    ],
    "group": 7,
    "order": 4419
  },
  {
    "emoji": "🪞",
    "label": "mirror",
    "tags": [
      "reflection",
      "reflector",
      "speculum"
    ],
    "group": 7,
    "order": 4420
  },
  {
    "emoji": "🪟",
    "label": "window",
    "tags": [
      "frame",
      "fresh air",
      "opening",
      "transparent",
      "view"
    ],
    "group": 7,
    "order": 4421
  },
  {
    "emoji": "🛏️",
    "label": "bed",
    "tags": [
      "hotel",
      "sleep"
    ],
    "group": 7,
    "order": 4423
  },
  {
    "emoji": "🛋️",
    "label": "couch and lamp",
    "tags": [
      "couch",
      "hotel",
      "lamp"
    ],
    "group": 7,
    "order": 4425
  },
  {
    "emoji": "🪑",
    "label": "chair",
    "tags": [
      "seat",
      "sit"
    ],
    "group": 7,
    "order": 4426
  },
  {
    "emoji": "🚽",
    "label": "toilet",
    "tags": [
      "toilet"
    ],
    "group": 7,
    "order": 4427
  },
  {
    "emoji": "🪠",
    "label": "plunger",
    "tags": [
      "force cup",
      "plumber",
      "suction",
      "toilet"
    ],
    "group": 7,
    "order": 4428
  },
  {
    "emoji": "🚿",
    "label": "shower",
    "tags": [
      "water"
    ],
    "group": 7,
    "order": 4429
  },
  {
    "emoji": "🛁",
    "label": "bathtub",
    "tags": [
      "bath"
    ],
    "group": 7,
    "order": 4430
  },
  {
    "emoji": "🪤",
    "label": "mouse trap",
    "tags": [
      "bait",
      "mousetrap",
      "snare",
      "trap"
    ],
    "group": 7,
    "order": 4431
  },
  {
    "emoji": "🪒",
    "label": "razor",
    "tags": [
      "sharp",
      "shave"
    ],
    "group": 7,
    "order": 4432
  },
  {
    "emoji": "🧴",
    "label": "lotion bottle",
    "tags": [
      "lotion",
      "moisturizer",
      "shampoo",
      "sunscreen"
    ],
    "group": 7,
    "order": 4433
  },
  {
    "emoji": "🧷",
    "label": "safety pin",
    "tags": [
      "diaper",
      "punk rock"
    ],
    "group": 7,
    "order": 4434
  },
  {
    "emoji": "🧹",
    "label": "broom",
    "tags": [
      "cleaning",
      "sweeping",
      "witch"
    ],
    "group": 7,
    "order": 4435
  },
  {
    "emoji": "🧺",
    "label": "basket",
    "tags": [
      "farming",
      "laundry",
      "picnic"
    ],
    "group": 7,
    "order": 4436
  },
  {
    "emoji": "🧻",
    "label": "roll of paper",
    "tags": [
      "paper towels",
      "toilet paper"
    ],
    "group": 7,
    "order": 4437
  },
  {
    "emoji": "🪣",
    "label": "bucket",
    "tags": [
      "cask",
      "pail",
      "vat"
    ],
    "group": 7,
    "order": 4438
  },
  {
    "emoji": "🧼",
    "label": "soap",
    "tags": [
      "bar",
      "bathing",
      "cleaning",
      "lather",
      "soapdish"
    ],
    "group": 7,
    "order": 4439
  },
  {
    "emoji": "🫧",
    "label": "bubbles",
    "tags": [
      "burp",
      "clean",
      "soap",
      "underwater"
    ],
    "group": 7,
    "order": 4440
  },
  {
    "emoji": "🪥",
    "label": "toothbrush",
    "tags": [
      "bathroom",
      "brush",
      "clean",
      "dental",
      "hygiene",
      "teeth"
    ],
    "group": 7,
    "order": 4441
  },
  {
    "emoji": "🧽",
    "label": "sponge",
    "tags": [
      "absorbing",
      "cleaning",
      "porous"
    ],
    "group": 7,
    "order": 4442
  },
  {
    "emoji": "🧯",
    "label": "fire extinguisher",
    "tags": [
      "extinguish",
      "fire",
      "quench"
    ],
    "group": 7,
    "order": 4443
  },
  {
    "emoji": "🛒",
    "label": "shopping cart",
    "tags": [
      "cart",
      "shopping",
      "trolley"
    ],
    "group": 7,
    "order": 4444
  },
  {
    "emoji": "🚬",
    "label": "cigarette",
    "tags": [
      "smoking"
    ],
    "group": 7,
    "order": 4445
  },
  {
    "emoji": "⚰️",
    "label": "coffin",
    "tags": [
      "death"
    ],
    "group": 7,
    "order": 4447
  },
  {
    "emoji": "🪦",
    "label": "headstone",
    "tags": [
      "cemetery",
      "grave",
      "graveyard",
      "tombstone"
    ],
    "group": 7,
    "order": 4448
  },
  {
    "emoji": "⚱️",
    "label": "funeral urn",
    "tags": [
      "ashes",
      "death",
      "funeral",
      "urn"
    ],
    "group": 7,
    "order": 4450
  },
  {
    "emoji": "🧿",
    "label": "nazar amulet",
    "tags": [
      "bead",
      "charm",
      "evil-eye",
      "nazar",
      "talisman"
    ],
    "group": 7,
    "order": 4451
  },
  {
    "emoji": "🪬",
    "label": "hamsa",
    "tags": [
      "amulet",
      "fatima",
      "hand",
      "mary",
      "miriam",
      "protection"
    ],
    "group": 7,
    "order": 4452
  },
  {
    "emoji": "🗿",
    "label": "moai",
    "tags": [
      "face",
      "moyai",
      "statue"
    ],
    "group": 7,
    "order": 4453
  },
  {
    "emoji": "🪧",
    "label": "placard",
    "tags": [
      "demonstration",
      "picket",
      "protest",
      "sign"
    ],
    "group": 7,
    "order": 4454
  },
  {
    "emoji": "🪪",
    "label": "identification card",
    "tags": [
      "credentials",
      "id",
      "license",
      "security"
    ],
    "group": 7,
    "order": 4455
  },
  {
    "emoji": "🏧",
    "label": "ATM sign",
    "tags": [
      "atm",
      "atm sign",
      "automated",
      "bank",
      "teller"
    ],
    "group": 8,
    "order": 4456
  },
  {
    "emoji": "🚮",
    "label": "litter in bin sign",
    "tags": [
      "litter",
      "litter bin"
    ],
    "group": 8,
    "order": 4457
  },
  {
    "emoji": "🚰",
    "label": "potable water",
    "tags": [
      "drinking",
      "potable",
      "water"
    ],
    "group": 8,
    "order": 4458
  },
  {
    "emoji": "♿️",
    "label": "wheelchair symbol",
    "tags": [
      "access"
    ],
    "group": 8,
    "order": 4459
  },
  {
    "emoji": "🚹️",
    "label": "men’s room",
    "tags": [
      "bathroom",
      "lavatory",
      "man",
      "restroom",
      "toilet",
      "wc"
    ],
    "group": 8,
    "order": 4460
  },
  {
    "emoji": "🚺️",
    "label": "women’s room",
    "tags": [
      "bathroom",
      "lavatory",
      "restroom",
      "toilet",
      "wc",
      "woman"
    ],
    "group": 8,
    "order": 4461
  },
  {
    "emoji": "🚻",
    "label": "restroom",
    "tags": [
      "bathroom",
      "lavatory",
      "toilet",
      "wc"
    ],
    "group": 8,
    "order": 4462
  },
  {
    "emoji": "🚼️",
    "label": "baby symbol",
    "tags": [
      "baby",
      "changing"
    ],
    "group": 8,
    "order": 4463
  },
  {
    "emoji": "🚾",
    "label": "water closet",
    "tags": [
      "bathroom",
      "closet",
      "lavatory",
      "restroom",
      "toilet",
      "water",
      "wc"
    ],
    "group": 8,
    "order": 4464
  },
  {
    "emoji": "🛂",
    "label": "passport control",
    "tags": [
      "control",
      "passport"
    ],
    "group": 8,
    "order": 4465
  },
  {
    "emoji": "🛃",
    "label": "customs",
    "tags": [
      "customs"
    ],
    "group": 8,
    "order": 4466
  },
  {
    "emoji": "🛄",
    "label": "baggage claim",
    "tags": [
      "baggage",
      "claim"
    ],
    "group": 8,
    "order": 4467
  },
  {
    "emoji": "🛅",
    "label": "left luggage",
    "tags": [
      "baggage",
      "locker",
      "luggage"
    ],
    "group": 8,
    "order": 4468
  },
  {
    "emoji": "⚠️",
    "label": "warning",
    "tags": [
      "warning"
    ],
    "group": 8,
    "order": 4470
  },
  {
    "emoji": "🚸",
    "label": "children crossing",
    "tags": [
      "child",
      "crossing",
      "pedestrian",
      "traffic"
    ],
    "group": 8,
    "order": 4471
  },
  {
    "emoji": "⛔️",
    "label": "no entry",
    "tags": [
      "entry",
      "forbidden",
      "no",
      "not",
      "prohibited",
      "traffic"
    ],
    "group": 8,
    "order": 4472
  },
  {
    "emoji": "🚫",
    "label": "prohibited",
    "tags": [
      "entry",
      "forbidden",
      "no",
      "not"
    ],
    "group": 8,
    "order": 4473
  },
  {
    "emoji": "🚳",
    "label": "no bicycles",
    "tags": [
      "bicycle",
      "bike",
      "forbidden",
      "no",
      "prohibited"
    ],
    "group": 8,
    "order": 4474
  },
  {
    "emoji": "🚭️",
    "label": "no smoking",
    "tags": [
      "forbidden",
      "no",
      "not",
      "prohibited",
      "smoking"
    ],
    "group": 8,
    "order": 4475
  },
  {
    "emoji": "🚯",
    "label": "no littering",
    "tags": [
      "forbidden",
      "litter",
      "no",
      "not",
      "prohibited"
    ],
    "group": 8,
    "order": 4476
  },
  {
    "emoji": "🚱",
    "label": "non-potable water",
    "tags": [
      "non-drinking",
      "non-potable",
      "water"
    ],
    "group": 8,
    "order": 4477
  },
  {
    "emoji": "🚷",
    "label": "no pedestrians",
    "tags": [
      "forbidden",
      "no",
      "not",
      "pedestrian",
      "prohibited"
    ],
    "group": 8,
    "order": 4478
  },
  {
    "emoji": "📵",
    "label": "no mobile phones",
    "tags": [
      "cell",
      "forbidden",
      "mobile",
      "no",
      "phone"
    ],
    "group": 8,
    "order": 4479
  },
  {
    "emoji": "🔞",
    "label": "no one under eighteen",
    "tags": [
      "18",
      "age restriction",
      "eighteen",
      "prohibited",
      "underage"
    ],
    "group": 8,
    "order": 4480
  },
  {
    "emoji": "☢️",
    "label": "radioactive",
    "tags": [
      "sign"
    ],
    "group": 8,
    "order": 4482
  },
  {
    "emoji": "☣️",
    "label": "biohazard",
    "tags": [
      "sign"
    ],
    "group": 8,
    "order": 4484
  },
  {
    "emoji": "⬆️",
    "label": "up arrow",
    "tags": [
      "arrow",
      "cardinal",
      "direction",
      "north"
    ],
    "group": 8,
    "order": 4486
  },
  {
    "emoji": "↗️",
    "label": "up-right arrow",
    "tags": [
      "arrow",
      "direction",
      "intercardinal",
      "northeast"
    ],
    "group": 8,
    "order": 4488
  },
  {
    "emoji": "➡️",
    "label": "right arrow",
    "tags": [
      "arrow",
      "cardinal",
      "direction",
      "east"
    ],
    "group": 8,
    "order": 4490
  },
  {
    "emoji": "↘️",
    "label": "down-right arrow",
    "tags": [
      "arrow",
      "direction",
      "intercardinal",
      "southeast"
    ],
    "group": 8,
    "order": 4492
  },
  {
    "emoji": "⬇️",
    "label": "down arrow",
    "tags": [
      "arrow",
      "cardinal",
      "direction",
      "down",
      "south"
    ],
    "group": 8,
    "order": 4494
  },
  {
    "emoji": "↙️",
    "label": "down-left arrow",
    "tags": [
      "arrow",
      "direction",
      "intercardinal",
      "southwest"
    ],
    "group": 8,
    "order": 4496
  },
  {
    "emoji": "⬅️",
    "label": "left arrow",
    "tags": [
      "arrow",
      "cardinal",
      "direction",
      "west"
    ],
    "group": 8,
    "order": 4498
  },
  {
    "emoji": "↖️",
    "label": "up-left arrow",
    "tags": [
      "arrow",
      "direction",
      "intercardinal",
      "northwest"
    ],
    "group": 8,
    "order": 4500
  },
  {
    "emoji": "↕️",
    "label": "up-down arrow",
    "tags": [
      "arrow"
    ],
    "group": 8,
    "order": 4502
  },
  {
    "emoji": "↔️",
    "label": "left-right arrow",
    "tags": [
      "arrow"
    ],
    "group": 8,
    "order": 4504
  },
  {
    "emoji": "↩️",
    "label": "right arrow curving left",
    "tags": [
      "arrow"
    ],
    "group": 8,
    "order": 4506
  },
  {
    "emoji": "↪️",
    "label": "left arrow curving right",
    "tags": [
      "arrow"
    ],
    "group": 8,
    "order": 4508
  },
  {
    "emoji": "⤴️",
    "label": "right arrow curving up",
    "tags": [
      "arrow"
    ],
    "group": 8,
    "order": 4510
  },
  {
    "emoji": "⤵️",
    "label": "right arrow curving down",
    "tags": [
      "arrow",
      "down"
    ],
    "group": 8,
    "order": 4512
  },
  {
    "emoji": "🔃",
    "label": "clockwise vertical arrows",
    "tags": [
      "arrow",
      "clockwise",
      "reload"
    ],
    "group": 8,
    "order": 4513
  },
  {
    "emoji": "🔄",
    "label": "counterclockwise arrows button",
    "tags": [
      "anticlockwise",
      "arrow",
      "counterclockwise",
      "withershins"
    ],
    "group": 8,
    "order": 4514
  },
  {
    "emoji": "🔙",
    "label": "BACK arrow",
    "tags": [
      "arrow",
      "back"
    ],
    "group": 8,
    "order": 4515
  },
  {
    "emoji": "🔚",
    "label": "END arrow",
    "tags": [
      "arrow",
      "end"
    ],
    "group": 8,
    "order": 4516
  },
  {
    "emoji": "🔛",
    "label": "ON! arrow",
    "tags": [
      "arrow",
      "mark",
      "on",
      "on!"
    ],
    "group": 8,
    "order": 4517
  },
  {
    "emoji": "🔜",
    "label": "SOON arrow",
    "tags": [
      "arrow",
      "soon"
    ],
    "group": 8,
    "order": 4518
  },
  {
    "emoji": "🔝",
    "label": "TOP arrow",
    "tags": [
      "arrow",
      "top",
      "up"
    ],
    "group": 8,
    "order": 4519
  },
  {
    "emoji": "🛐",
    "label": "place of worship",
    "tags": [
      "religion",
      "worship"
    ],
    "group": 8,
    "order": 4520
  },
  {
    "emoji": "⚛️",
    "label": "atom symbol",
    "tags": [
      "atheist",
      "atom"
    ],
    "group": 8,
    "order": 4522
  },
  {
    "emoji": "🕉️",
    "label": "om",
    "tags": [
      "hindu",
      "religion"
    ],
    "group": 8,
    "order": 4524
  },
  {
    "emoji": "✡️",
    "label": "star of David",
    "tags": [
      "david",
      "jew",
      "jewish",
      "religion",
      "star",
      "star of david"
    ],
    "group": 8,
    "order": 4526
  },
  {
    "emoji": "☸️",
    "label": "wheel of dharma",
    "tags": [
      "buddhist",
      "dharma",
      "religion",
      "wheel"
    ],
    "group": 8,
    "order": 4528
  },
  {
    "emoji": "☯️",
    "label": "yin yang",
    "tags": [
      "religion",
      "tao",
      "taoist",
      "yang",
      "yin"
    ],
    "group": 8,
    "order": 4530
  },
  {
    "emoji": "✝️",
    "label": "latin cross",
    "tags": [
      "christian",
      "cross",
      "religion"
    ],
    "group": 8,
    "order": 4532
  },
  {
    "emoji": "☦️",
    "label": "orthodox cross",
    "tags": [
      "christian",
      "cross",
      "religion"
    ],
    "group": 8,
    "order": 4534
  },
  {
    "emoji": "☪️",
    "label": "star and crescent",
    "tags": [
      "islam",
      "muslim",
      "religion"
    ],
    "group": 8,
    "order": 4536
  },
  {
    "emoji": "☮️",
    "label": "peace symbol",
    "tags": [
      "peace"
    ],
    "group": 8,
    "order": 4538
  },
  {
    "emoji": "🕎",
    "label": "menorah",
    "tags": [
      "candelabrum",
      "candlestick",
      "religion"
    ],
    "group": 8,
    "order": 4539
  },
  {
    "emoji": "🔯",
    "label": "dotted six-pointed star",
    "tags": [
      "fortune",
      "star"
    ],
    "group": 8,
    "order": 4540
  },
  {
    "emoji": "🪯",
    "label": "khanda",
    "tags": [
      "religion",
      "sikh"
    ],
    "group": 8,
    "order": 4541
  },
  {
    "emoji": "♈️",
    "label": "Aries",
    "tags": [
      "aries",
      "ram",
      "zodiac"
    ],
    "group": 8,
    "order": 4542
  },
  {
    "emoji": "♉️",
    "label": "Taurus",
    "tags": [
      "bull",
      "ox",
      "taurus",
      "zodiac"
    ],
    "group": 8,
    "order": 4543
  },
  {
    "emoji": "♊️",
    "label": "Gemini",
    "tags": [
      "gemini",
      "twins",
      "zodiac"
    ],
    "group": 8,
    "order": 4544
  },
  {
    "emoji": "♋️",
    "label": "Cancer",
    "tags": [
      "cancer",
      "crab",
      "zodiac"
    ],
    "group": 8,
    "order": 4545
  },
  {
    "emoji": "♌️",
    "label": "Leo",
    "tags": [
      "leo",
      "lion",
      "zodiac"
    ],
    "group": 8,
    "order": 4546
  },
  {
    "emoji": "♍️",
    "label": "Virgo",
    "tags": [
      "virgo",
      "zodiac"
    ],
    "group": 8,
    "order": 4547
  },
  {
    "emoji": "♎️",
    "label": "Libra",
    "tags": [
      "balance",
      "justice",
      "libra",
      "scales",
      "zodiac"
    ],
    "group": 8,
    "order": 4548
  },
  {
    "emoji": "♏️",
    "label": "Scorpio",
    "tags": [
      "scorpio",
      "scorpion",
      "scorpius",
      "zodiac"
    ],
    "group": 8,
    "order": 4549
  },
  {
    "emoji": "♐️",
    "label": "Sagittarius",
    "tags": [
      "archer",
      "sagittarius",
      "zodiac"
    ],
    "group": 8,
    "order": 4550
  },
  {
    "emoji": "♑️",
    "label": "Capricorn",
    "tags": [
      "capricorn",
      "goat",
      "zodiac"
    ],
    "group": 8,
    "order": 4551
  },
  {
    "emoji": "♒️",
    "label": "Aquarius",
    "tags": [
      "aquarius",
      "bearer",
      "water",
      "zodiac"
    ],
    "group": 8,
    "order": 4552
  },
  {
    "emoji": "♓️",
    "label": "Pisces",
    "tags": [
      "fish",
      "pisces",
      "zodiac"
    ],
    "group": 8,
    "order": 4553
  },
  {
    "emoji": "⛎️",
    "label": "Ophiuchus",
    "tags": [
      "bearer",
      "ophiuchus",
      "serpent",
      "snake",
      "zodiac"
    ],
    "group": 8,
    "order": 4554
  },
  {
    "emoji": "🔀",
    "label": "shuffle tracks button",
    "tags": [
      "arrow",
      "crossed"
    ],
    "group": 8,
    "order": 4555
  },
  {
    "emoji": "🔁",
    "label": "repeat button",
    "tags": [
      "arrow",
      "clockwise",
      "repeat"
    ],
    "group": 8,
    "order": 4556
  },
  {
    "emoji": "🔂",
    "label": "repeat single button",
    "tags": [
      "arrow",
      "clockwise",
      "once"
    ],
    "group": 8,
    "order": 4557
  },
  {
    "emoji": "▶️",
    "label": "play button",
    "tags": [
      "arrow",
      "play",
      "right",
      "triangle"
    ],
    "group": 8,
    "order": 4559
  },
  {
    "emoji": "⏩️",
    "label": "fast-forward button",
    "tags": [
      "arrow",
      "double",
      "fast",
      "forward"
    ],
    "group": 8,
    "order": 4560
  },
  {
    "emoji": "⏭️",
    "label": "next track button",
    "tags": [
      "arrow",
      "next scene",
      "next track",
      "triangle"
    ],
    "group": 8,
    "order": 4562
  },
  {
    "emoji": "⏯️",
    "label": "play or pause button",
    "tags": [
      "arrow",
      "pause",
      "play",
      "right",
      "triangle"
    ],
    "group": 8,
    "order": 4564
  },
  {
    "emoji": "◀️",
    "label": "reverse button",
    "tags": [
      "arrow",
      "left",
      "reverse",
      "triangle"
    ],
    "group": 8,
    "order": 4566
  },
  {
    "emoji": "⏪️",
    "label": "fast reverse button",
    "tags": [
      "arrow",
      "double",
      "rewind"
    ],
    "group": 8,
    "order": 4567
  },
  {
    "emoji": "⏮️",
    "label": "last track button",
    "tags": [
      "arrow",
      "previous scene",
      "previous track",
      "triangle"
    ],
    "group": 8,
    "order": 4569
  },
  {
    "emoji": "🔼",
    "label": "upwards button",
    "tags": [
      "arrow",
      "button"
    ],
    "group": 8,
    "order": 4570
  },
  {
    "emoji": "⏫️",
    "label": "fast up button",
    "tags": [
      "arrow",
      "double"
    ],
    "group": 8,
    "order": 4571
  },
  {
    "emoji": "🔽",
    "label": "downwards button",
    "tags": [
      "arrow",
      "button",
      "down"
    ],
    "group": 8,
    "order": 4572
  },
  {
    "emoji": "⏬️",
    "label": "fast down button",
    "tags": [
      "arrow",
      "double",
      "down"
    ],
    "group": 8,
    "order": 4573
  },
  {
    "emoji": "⏸️",
    "label": "pause button",
    "tags": [
      "bar",
      "double",
      "pause",
      "vertical"
    ],
    "group": 8,
    "order": 4575
  },
  {
    "emoji": "⏹️",
    "label": "stop button",
    "tags": [
      "square",
      "stop"
    ],
    "group": 8,
    "order": 4577
  },
  {
    "emoji": "⏺️",
    "label": "record button",
    "tags": [
      "circle",
      "record"
    ],
    "group": 8,
    "order": 4579
  },
  {
    "emoji": "⏏️",
    "label": "eject button",
    "tags": [
      "eject"
    ],
    "group": 8,
    "order": 4581
  },
  {
    "emoji": "🎦",
    "label": "cinema",
    "tags": [
      "camera",
      "film",
      "movie"
    ],
    "group": 8,
    "order": 4582
  },
  {
    "emoji": "🔅",
    "label": "dim button",
    "tags": [
      "brightness",
      "dim",
      "low"
    ],
    "group": 8,
    "order": 4583
  },
  {
    "emoji": "🔆",
    "label": "bright button",
    "tags": [
      "bright",
      "brightness"
    ],
    "group": 8,
    "order": 4584
  },
  {
    "emoji": "📶",
    "label": "antenna bars",
    "tags": [
      "antenna",
      "bar",
      "cell",
      "mobile",
      "phone"
    ],
    "group": 8,
    "order": 4585
  },
  {
    "emoji": "🛜",
    "label": "wireless",
    "tags": [
      "computer",
      "internet",
      "network",
      "wi-fi",
      "wifi"
    ],
    "group": 8,
    "order": 4586
  },
  {
    "emoji": "📳",
    "label": "vibration mode",
    "tags": [
      "cell",
      "mobile",
      "mode",
      "phone",
      "telephone",
      "vibration"
    ],
    "group": 8,
    "order": 4587
  },
  {
    "emoji": "📴",
    "label": "mobile phone off",
    "tags": [
      "cell",
      "mobile",
      "off",
      "phone",
      "telephone"
    ],
    "group": 8,
    "order": 4588
  },
  {
    "emoji": "♀️",
    "label": "female sign",
    "tags": [
      "woman"
    ],
    "group": 8,
    "order": 4590
  },
  {
    "emoji": "♂️",
    "label": "male sign",
    "tags": [
      "man"
    ],
    "group": 8,
    "order": 4592
  },
  {
    "emoji": "⚧️",
    "label": "transgender symbol",
    "tags": [
      "transgender"
    ],
    "group": 8,
    "order": 4594
  },
  {
    "emoji": "✖️",
    "label": "multiply",
    "tags": [
      "cancel",
      "multiplication",
      "sign",
      "x",
      "×"
    ],
    "group": 8,
    "order": 4596
  },
  {
    "emoji": "➕️",
    "label": "plus",
    "tags": [
      "+",
      "math",
      "sign"
    ],
    "group": 8,
    "order": 4597
  },
  {
    "emoji": "➖️",
    "label": "minus",
    "tags": [
      "-",
      "math",
      "sign",
      "−"
    ],
    "group": 8,
    "order": 4598
  },
  {
    "emoji": "➗️",
    "label": "divide",
    "tags": [
      "division",
      "math",
      "sign",
      "÷"
    ],
    "group": 8,
    "order": 4599
  },
  {
    "emoji": "🟰",
    "label": "heavy equals sign",
    "tags": [
      "equality",
      "math"
    ],
    "group": 8,
    "order": 4600
  },
  {
    "emoji": "♾️",
    "label": "infinity",
    "tags": [
      "forever",
      "unbounded",
      "universal"
    ],
    "group": 8,
    "order": 4602
  },
  {
    "emoji": "‼️",
    "label": "double exclamation mark",
    "tags": [
      "!",
      "!!",
      "bangbang",
      "exclamation",
      "mark"
    ],
    "group": 8,
    "order": 4604
  },
  {
    "emoji": "⁉️",
    "label": "exclamation question mark",
    "tags": [
      "!",
      "!?",
      "?",
      "exclamation",
      "interrobang",
      "mark",
      "punctuation",
      "question"
    ],
    "group": 8,
    "order": 4606
  },
  {
    "emoji": "❓️",
    "label": "red question mark",
    "tags": [
      "?",
      "mark",
      "punctuation",
      "question"
    ],
    "group": 8,
    "order": 4607
  },
  {
    "emoji": "❔️",
    "label": "white question mark",
    "tags": [
      "?",
      "mark",
      "outlined",
      "punctuation",
      "question"
    ],
    "group": 8,
    "order": 4608
  },
  {
    "emoji": "❕️",
    "label": "white exclamation mark",
    "tags": [
      "!",
      "exclamation",
      "mark",
      "outlined",
      "punctuation"
    ],
    "group": 8,
    "order": 4609
  },
  {
    "emoji": "❗️",
    "label": "red exclamation mark",
    "tags": [
      "!",
      "exclamation",
      "mark",
      "punctuation"
    ],
    "group": 8,
    "order": 4610
  },
  {
    "emoji": "〰️",
    "label": "wavy dash",
    "tags": [
      "dash",
      "punctuation",
      "wavy"
    ],
    "group": 8,
    "order": 4612
  },
  {
    "emoji": "💱",
    "label": "currency exchange",
    "tags": [
      "bank",
      "currency",
      "exchange",
      "money"
    ],
    "group": 8,
    "order": 4613
  },
  {
    "emoji": "💲",
    "label": "heavy dollar sign",
    "tags": [
      "currency",
      "dollar",
      "money"
    ],
    "group": 8,
    "order": 4614
  },
  {
    "emoji": "⚕️",
    "label": "medical symbol",
    "tags": [
      "aesculapius",
      "medicine",
      "staff"
    ],
    "group": 8,
    "order": 4616
  },
  {
    "emoji": "♻️",
    "label": "recycling symbol",
    "tags": [
      "recycle"
    ],
    "group": 8,
    "order": 4618
  },
  {
    "emoji": "⚜️",
    "label": "fleur-de-lis",
    "tags": [
      "fleur-de-lis"
    ],
    "group": 8,
    "order": 4620
  },
  {
    "emoji": "🔱",
    "label": "trident emblem",
    "tags": [
      "anchor",
      "emblem",
      "ship",
      "tool",
      "trident"
    ],
    "group": 8,
    "order": 4621
  },
  {
    "emoji": "📛",
    "label": "name badge",
    "tags": [
      "badge",
      "name"
    ],
    "group": 8,
    "order": 4622
  },
  {
    "emoji": "🔰",
    "label": "Japanese symbol for beginner",
    "tags": [
      "beginner",
      "chevron",
      "japanese",
      "japanese symbol for beginner",
      "leaf"
    ],
    "group": 8,
    "order": 4623
  },
  {
    "emoji": "⭕️",
    "label": "hollow red circle",
    "tags": [
      "circle",
      "large",
      "o",
      "red"
    ],
    "group": 8,
    "order": 4624
  },
  {
    "emoji": "✅️",
    "label": "check mark button",
    "tags": [
      "button",
      "check",
      "mark",
      "✓"
    ],
    "group": 8,
    "order": 4625
  },
  {
    "emoji": "☑️",
    "label": "check box with check",
    "tags": [
      "box",
      "check",
      "✓"
    ],
    "group": 8,
    "order": 4627
  },
  {
    "emoji": "✔️",
    "label": "check mark",
    "tags": [
      "check",
      "mark",
      "✓"
    ],
    "group": 8,
    "order": 4629
  },
  {
    "emoji": "❌️",
    "label": "cross mark",
    "tags": [
      "cancel",
      "cross",
      "mark",
      "multiplication",
      "multiply",
      "x",
      "×"
    ],
    "group": 8,
    "order": 4630
  },
  {
    "emoji": "❎️",
    "label": "cross mark button",
    "tags": [
      "mark",
      "square",
      "x",
      "×"
    ],
    "group": 8,
    "order": 4631
  },
  {
    "emoji": "➰️",
    "label": "curly loop",
    "tags": [
      "curl",
      "loop"
    ],
    "group": 8,
    "order": 4632
  },
  {
    "emoji": "➿️",
    "label": "double curly loop",
    "tags": [
      "curl",
      "double",
      "loop"
    ],
    "group": 8,
    "order": 4633
  },
  {
    "emoji": "〽️",
    "label": "part alternation mark",
    "tags": [
      "mark",
      "part"
    ],
    "group": 8,
    "order": 4635
  },
  {
    "emoji": "✳️",
    "label": "eight-spoked asterisk",
    "tags": [
      "*",
      "asterisk"
    ],
    "group": 8,
    "order": 4637
  },
  {
    "emoji": "✴️",
    "label": "eight-pointed star",
    "tags": [
      "*",
      "star"
    ],
    "group": 8,
    "order": 4639
  },
  {
    "emoji": "❇️",
    "label": "sparkle",
    "tags": [
      "*"
    ],
    "group": 8,
    "order": 4641
  },
  {
    "emoji": "©️",
    "label": "copyright",
    "tags": [
      "c"
    ],
    "group": 8,
    "order": 4643
  },
  {
    "emoji": "®️",
    "label": "registered",
    "tags": [
      "r"
    ],
    "group": 8,
    "order": 4645
  },
  {
    "emoji": "™️",
    "label": "trade mark",
    "tags": [
      "mark",
      "tm",
      "trademark"
    ],
    "group": 8,
    "order": 4647
  },
  {
    "emoji": "#️⃣",
    "label": "keycap: #",
    "tags": [
      "keycap"
    ],
    "group": 8,
    "order": 4648
  },
  {
    "emoji": "*️⃣",
    "label": "keycap: *",
    "tags": [
      "keycap"
    ],
    "group": 8,
    "order": 4650
  },
  {
    "emoji": "0️⃣",
    "label": "keycap: 0",
    "tags": [
      "keycap"
    ],
    "group": 8,
    "order": 4652
  },
  {
    "emoji": "1️⃣",
    "label": "keycap: 1",
    "tags": [
      "keycap"
    ],
    "group": 8,
    "order": 4654
  },
  {
    "emoji": "2️⃣",
    "label": "keycap: 2",
    "tags": [
      "keycap"
    ],
    "group": 8,
    "order": 4656
  },
  {
    "emoji": "3️⃣",
    "label": "keycap: 3",
    "tags": [
      "keycap"
    ],
    "group": 8,
    "order": 4658
  },
  {
    "emoji": "4️⃣",
    "label": "keycap: 4",
    "tags": [
      "keycap"
    ],
    "group": 8,
    "order": 4660
  },
  {
    "emoji": "5️⃣",
    "label": "keycap: 5",
    "tags": [
      "keycap"
    ],
    "group": 8,
    "order": 4662
  },
  {
    "emoji": "6️⃣",
    "label": "keycap: 6",
    "tags": [
      "keycap"
    ],
    "group": 8,
    "order": 4664
  },
  {
    "emoji": "7️⃣",
    "label": "keycap: 7",
    "tags": [
      "keycap"
    ],
    "group": 8,
    "order": 4666
  },
  {
    "emoji": "8️⃣",
    "label": "keycap: 8",
    "tags": [
      "keycap"
    ],
    "group": 8,
    "order": 4668
  },
  {
    "emoji": "9️⃣",
    "label": "keycap: 9",
    "tags": [
      "keycap"
    ],
    "group": 8,
    "order": 4670
  },
  {
    "emoji": "🔟",
    "label": "keycap: 10",
    "tags": [
      "keycap"
    ],
    "group": 8,
    "order": 4672
  },
  {
    "emoji": "🔠",
    "label": "input latin uppercase",
    "tags": [
      "abcd",
      "input",
      "latin",
      "letters",
      "uppercase"
    ],
    "group": 8,
    "order": 4673
  },
  {
    "emoji": "🔡",
    "label": "input latin lowercase",
    "tags": [
      "abcd",
      "input",
      "latin",
      "letters",
      "lowercase"
    ],
    "group": 8,
    "order": 4674
  },
  {
    "emoji": "🔢",
    "label": "input numbers",
    "tags": [
      "1234",
      "input",
      "numbers"
    ],
    "group": 8,
    "order": 4675
  },
  {
    "emoji": "🔣",
    "label": "input symbols",
    "tags": [
      "input",
      "〒♪&%"
    ],
    "group": 8,
    "order": 4676
  },
  {
    "emoji": "🔤",
    "label": "input latin letters",
    "tags": [
      "abc",
      "alphabet",
      "input",
      "latin",
      "letters"
    ],
    "group": 8,
    "order": 4677
  },
  {
    "emoji": "🅰️",
    "label": "A button (blood type)",
    "tags": [
      "a",
      "a button (blood type)",
      "blood type"
    ],
    "group": 8,
    "order": 4679
  },
  {
    "emoji": "🆎",
    "label": "AB button (blood type)",
    "tags": [
      "ab",
      "ab button (blood type)",
      "blood type"
    ],
    "group": 8,
    "order": 4680
  },
  {
    "emoji": "🅱️",
    "label": "B button (blood type)",
    "tags": [
      "b",
      "b button (blood type)",
      "blood type"
    ],
    "group": 8,
    "order": 4682
  },
  {
    "emoji": "🆑",
    "label": "CL button",
    "tags": [
      "cl",
      "cl button"
    ],
    "group": 8,
    "order": 4683
  },
  {
    "emoji": "🆒",
    "label": "COOL button",
    "tags": [
      "cool",
      "cool button"
    ],
    "group": 8,
    "order": 4684
  },
  {
    "emoji": "🆓",
    "label": "FREE button",
    "tags": [
      "free",
      "free button"
    ],
    "group": 8,
    "order": 4685
  },
  {
    "emoji": "ℹ️",
    "label": "information",
    "tags": [
      "i"
    ],
    "group": 8,
    "order": 4687
  },
  {
    "emoji": "🆔",
    "label": "ID button",
    "tags": [
      "id",
      "id button",
      "identity"
    ],
    "group": 8,
    "order": 4688
  },
  {
    "emoji": "Ⓜ️",
    "label": "circled M",
    "tags": [
      "circle",
      "circled m",
      "m"
    ],
    "group": 8,
    "order": 4690
  },
  {
    "emoji": "🆕",
    "label": "NEW button",
    "tags": [
      "new",
      "new button"
    ],
    "group": 8,
    "order": 4691
  },
  {
    "emoji": "🆖",
    "label": "NG button",
    "tags": [
      "ng",
      "ng button"
    ],
    "group": 8,
    "order": 4692
  },
  {
    "emoji": "🅾️",
    "label": "O button (blood type)",
    "tags": [
      "blood type",
      "o",
      "o button (blood type)"
    ],
    "group": 8,
    "order": 4694
  },
  {
    "emoji": "🆗",
    "label": "OK button",
    "tags": [
      "ok",
      "ok button"
    ],
    "group": 8,
    "order": 4695
  },
  {
    "emoji": "🅿️",
    "label": "P button",
    "tags": [
      "p",
      "p button",
      "parking"
    ],
    "group": 8,
    "order": 4697
  },
  {
    "emoji": "🆘",
    "label": "SOS button",
    "tags": [
      "help",
      "sos",
      "sos button"
    ],
    "group": 8,
    "order": 4698
  },
  {
    "emoji": "🆙",
    "label": "UP! button",
    "tags": [
      "mark",
      "up",
      "up!",
      "up! button"
    ],
    "group": 8,
    "order": 4699
  },
  {
    "emoji": "🆚",
    "label": "VS button",
    "tags": [
      "versus",
      "vs",
      "vs button"
    ],
    "group": 8,
    "order": 4700
  },
  {
    "emoji": "🈁",
    "label": "Japanese “here” button",
    "tags": [
      "japanese",
      "japanese “here” button",
      "katakana",
      "“here”",
      "ココ"
    ],
    "group": 8,
    "order": 4701
  },
  {
    "emoji": "🈂️",
    "label": "Japanese “service charge” button",
    "tags": [
      "japanese",
      "japanese “service charge” button",
      "katakana",
      "“service charge”",
      "サ"
    ],
    "group": 8,
    "order": 4703
  },
  {
    "emoji": "🈷️",
    "label": "Japanese “monthly amount” button",
    "tags": [
      "ideograph",
      "japanese",
      "japanese “monthly amount” button",
      "“monthly amount”",
      "月"
    ],
    "group": 8,
    "order": 4705
  },
  {
    "emoji": "🈶",
    "label": "Japanese “not free of charge” button",
    "tags": [
      "ideograph",
      "japanese",
      "japanese “not free of charge” button",
      "“not free of charge”",
      "有"
    ],
    "group": 8,
    "order": 4706
  },
  {
    "emoji": "🈯️",
    "label": "Japanese “reserved” button",
    "tags": [
      "ideograph",
      "japanese",
      "japanese “reserved” button",
      "“reserved”",
      "指"
    ],
    "group": 8,
    "order": 4707
  },
  {
    "emoji": "🉐",
    "label": "Japanese “bargain” button",
    "tags": [
      "ideograph",
      "japanese",
      "japanese “bargain” button",
      "“bargain”",
      "得"
    ],
    "group": 8,
    "order": 4708
  },
  {
    "emoji": "🈹",
    "label": "Japanese “discount” button",
    "tags": [
      "ideograph",
      "japanese",
      "japanese “discount” button",
      "“discount”",
      "割"
    ],
    "group": 8,
    "order": 4709
  },
  {
    "emoji": "🈚️",
    "label": "Japanese “free of charge” button",
    "tags": [
      "ideograph",
      "japanese",
      "japanese “free of charge” button",
      "“free of charge”",
      "無"
    ],
    "group": 8,
    "order": 4710
  },
  {
    "emoji": "🈲",
    "label": "Japanese “prohibited” button",
    "tags": [
      "ideograph",
      "japanese",
      "japanese “prohibited” button",
      "“prohibited”",
      "禁"
    ],
    "group": 8,
    "order": 4711
  },
  {
    "emoji": "🉑",
    "label": "Japanese “acceptable” button",
    "tags": [
      "ideograph",
      "japanese",
      "japanese “acceptable” button",
      "“acceptable”",
      "可"
    ],
    "group": 8,
    "order": 4712
  },
  {
    "emoji": "🈸",
    "label": "Japanese “application” button",
    "tags": [
      "ideograph",
      "japanese",
      "japanese “application” button",
      "“application”",
      "申"
    ],
    "group": 8,
    "order": 4713
  },
  {
    "emoji": "🈴",
    "label": "Japanese “passing grade” button",
    "tags": [
      "ideograph",
      "japanese",
      "japanese “passing grade” button",
      "“passing grade”",
      "合"
    ],
    "group": 8,
    "order": 4714
  },
  {
    "emoji": "🈳",
    "label": "Japanese “vacancy” button",
    "tags": [
      "ideograph",
      "japanese",
      "japanese “vacancy” button",
      "“vacancy”",
      "空"
    ],
    "group": 8,
    "order": 4715
  },
  {
    "emoji": "㊗️",
    "label": "Japanese “congratulations” button",
    "tags": [
      "ideograph",
      "japanese",
      "japanese “congratulations” button",
      "“congratulations”",
      "祝"
    ],
    "group": 8,
    "order": 4717
  },
  {
    "emoji": "㊙️",
    "label": "Japanese “secret” button",
    "tags": [
      "ideograph",
      "japanese",
      "japanese “secret” button",
      "“secret”",
      "秘"
    ],
    "group": 8,
    "order": 4719
  },
  {
    "emoji": "🈺",
    "label": "Japanese “open for business” button",
    "tags": [
      "ideograph",
      "japanese",
      "japanese “open for business” button",
      "“open for business”",
      "営"
    ],
    "group": 8,
    "order": 4720
  },
  {
    "emoji": "🈵",
    "label": "Japanese “no vacancy” button",
    "tags": [
      "ideograph",
      "japanese",
      "japanese “no vacancy” button",
      "“no vacancy”",
      "満"
    ],
    "group": 8,
    "order": 4721
  },
  {
    "emoji": "🔴",
    "label": "red circle",
    "tags": [
      "circle",
      "geometric",
      "red"
    ],
    "group": 8,
    "order": 4722
  },
  {
    "emoji": "🟠",
    "label": "orange circle",
    "tags": [
      "circle",
      "orange"
    ],
    "group": 8,
    "order": 4723
  },
  {
    "emoji": "🟡",
    "label": "yellow circle",
    "tags": [
      "circle",
      "yellow"
    ],
    "group": 8,
    "order": 4724
  },
  {
    "emoji": "🟢",
    "label": "green circle",
    "tags": [
      "circle",
      "green"
    ],
    "group": 8,
    "order": 4725
  },
  {
    "emoji": "🔵",
    "label": "blue circle",
    "tags": [
      "blue",
      "circle",
      "geometric"
    ],
    "group": 8,
    "order": 4726
  },
  {
    "emoji": "🟣",
    "label": "purple circle",
    "tags": [
      "circle",
      "purple"
    ],
    "group": 8,
    "order": 4727
  },
  {
    "emoji": "🟤",
    "label": "brown circle",
    "tags": [
      "brown",
      "circle"
    ],
    "group": 8,
    "order": 4728
  },
  {
    "emoji": "⚫️",
    "label": "black circle",
    "tags": [
      "circle",
      "geometric"
    ],
    "group": 8,
    "order": 4729
  },
  {
    "emoji": "⚪️",
    "label": "white circle",
    "tags": [
      "circle",
      "geometric"
    ],
    "group": 8,
    "order": 4730
  },
  {
    "emoji": "🟥",
    "label": "red square",
    "tags": [
      "red",
      "square"
    ],
    "group": 8,
    "order": 4731
  },
  {
    "emoji": "🟧",
    "label": "orange square",
    "tags": [
      "orange",
      "square"
    ],
    "group": 8,
    "order": 4732
  },
  {
    "emoji": "🟨",
    "label": "yellow square",
    "tags": [
      "square",
      "yellow"
    ],
    "group": 8,
    "order": 4733
  },
  {
    "emoji": "🟩",
    "label": "green square",
    "tags": [
      "green",
      "square"
    ],
    "group": 8,
    "order": 4734
  },
  {
    "emoji": "🟦",
    "label": "blue square",
    "tags": [
      "blue",
      "square"
    ],
    "group": 8,
    "order": 4735
  },
  {
    "emoji": "🟪",
    "label": "purple square",
    "tags": [
      "purple",
      "square"
    ],
    "group": 8,
    "order": 4736
  },
  {
    "emoji": "🟫",
    "label": "brown square",
    "tags": [
      "brown",
      "square"
    ],
    "group": 8,
    "order": 4737
  },
  {
    "emoji": "⬛️",
    "label": "black large square",
    "tags": [
      "geometric",
      "square"
    ],
    "group": 8,
    "order": 4738
  },
  {
    "emoji": "⬜️",
    "label": "white large square",
    "tags": [
      "geometric",
      "square"
    ],
    "group": 8,
    "order": 4739
  },
  {
    "emoji": "◼️",
    "label": "black medium square",
    "tags": [
      "geometric",
      "square"
    ],
    "group": 8,
    "order": 4741
  },
  {
    "emoji": "◻️",
    "label": "white medium square",
    "tags": [
      "geometric",
      "square"
    ],
    "group": 8,
    "order": 4743
  },
  {
    "emoji": "◾️",
    "label": "black medium-small square",
    "tags": [
      "geometric",
      "square"
    ],
    "group": 8,
    "order": 4744
  },
  {
    "emoji": "◽️",
    "label": "white medium-small square",
    "tags": [
      "geometric",
      "square"
    ],
    "group": 8,
    "order": 4745
  },
  {
    "emoji": "▪️",
    "label": "black small square",
    "tags": [
      "geometric",
      "square"
    ],
    "group": 8,
    "order": 4747
  },
  {
    "emoji": "▫️",
    "label": "white small square",
    "tags": [
      "geometric",
      "square"
    ],
    "group": 8,
    "order": 4749
  },
  {
    "emoji": "🔶",
    "label": "large orange diamond",
    "tags": [
      "diamond",
      "geometric",
      "orange"
    ],
    "group": 8,
    "order": 4750
  },
  {
    "emoji": "🔷",
    "label": "large blue diamond",
    "tags": [
      "blue",
      "diamond",
      "geometric"
    ],
    "group": 8,
    "order": 4751
  },
  {
    "emoji": "🔸",
    "label": "small orange diamond",
    "tags": [
      "diamond",
      "geometric",
      "orange"
    ],
    "group": 8,
    "order": 4752
  },
  {
    "emoji": "🔹",
    "label": "small blue diamond",
    "tags": [
      "blue",
      "diamond",
      "geometric"
    ],
    "group": 8,
    "order": 4753
  },
  {
    "emoji": "🔺",
    "label": "red triangle pointed up",
    "tags": [
      "geometric",
      "red"
    ],
    "group": 8,
    "order": 4754
  },
  {
    "emoji": "🔻",
    "label": "red triangle pointed down",
    "tags": [
      "down",
      "geometric",
      "red"
    ],
    "group": 8,
    "order": 4755
  },
  {
    "emoji": "💠",
    "label": "diamond with a dot",
    "tags": [
      "comic",
      "diamond",
      "geometric",
      "inside"
    ],
    "group": 8,
    "order": 4756
  },
  {
    "emoji": "🔘",
    "label": "radio button",
    "tags": [
      "button",
      "geometric",
      "radio"
    ],
    "group": 8,
    "order": 4757
  },
  {
    "emoji": "🔳",
    "label": "white square button",
    "tags": [
      "button",
      "geometric",
      "outlined",
      "square"
    ],
    "group": 8,
    "order": 4758
  },
  {
    "emoji": "🔲",
    "label": "black square button",
    "tags": [
      "button",
      "geometric",
      "square"
    ],
    "group": 8,
    "order": 4759
  }
];
