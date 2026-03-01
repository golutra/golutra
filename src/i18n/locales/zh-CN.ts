// [2026-01-23 00:59] 目的: 维护中文界面文案与键值映射，集中管理以便一致性与可翻译性; 边界: 仅包含文案资源，不引入业务逻辑或运行时行为; 设计: 键结构与功能域对齐以降低跨模块引用成本与回归风险。
export default {
  app: {
    name: 'golutra',
    windowControls: {
      minimize: '最小化',
      maximize: '最大化',
      restore: '还原',
      close: '关闭'
    }
  },
  nav: {
    chat: '聊天与弹窗',
    friends: '好友',
    terminal: '终端',
    workspaces: '工作区',
    store: '技能商店',
    plugins: '插件',
    settings: '设置'
  },
  terminal: {
    title: '终端',
    subtitle: '在一个工作区内运行多个终端会话。',
    newTab: '新建标签',
    tabSearchPlaceholder: '搜索标签...',
    tabSearchCreate: '新建标签',
    tabSearchEmpty: '未找到匹配的标签。',
    recentClosedTabs: '最近关闭的标签页（{count}）',
    emptyTabs: '暂无终端。',
    emptyTitle: '没有活动终端',
    emptySubtitle: '新建一个终端标签开始。',
    splitEmpty: '拖动标签到这里，或新建一个终端。',
    unavailableTitle: '终端不可用',
    unavailableSubtitle: '请在 Tauri 桌面应用中打开该页面。',
    errorTitle: '终端启动失败',
    errorSubtitle: '请检查桌面运行时后重试。',
    resourceLimit: '系统资源不足，无法启动新终端，请先关闭部分后台任务。',
    statusLabel: '终端状态',
    statusOptions: {
      pending: '待确认',
      connecting: '连接中',
      connected: '已连接',
      working: '工作中',
      disconnected: '已断开'
    },
    tabMenu: {
      close: '关闭当前',
      closeOthers: '关闭其它',
      closeRight: '关闭右侧',
      pin: '置顶',
      unpin: '取消置顶',
      layoutSingle: '单窗口',
      layoutSplitVertical: '左右分屏',
      layoutSplitHorizontal: '上下分屏',
      layoutGrid: '四分屏'
    },
    contextMenu: {
      clear: '清除',
      find: '查找'
    },
    findPlaceholder: '查找...',
    findNoResults: '无结果',
    findCaseSensitive: '区分大小写',
    findWholeWord: '全字匹配',
    findRegex: '使用正则'
  },
  contextMenu: {
    copy: '复制',
    cut: '剪切',
    paste: '粘贴',
    selectAll: '全选'
  },
    common: {
      userAvatarAlt: '用户',
      remove: '删除',
      openFolder: '打开文件夹'
    },
  chat: {
    channelName: 'general',
    channelDisplay: '#general',
    channelDescription: '',
    directMessageDescription: '与 {name} 的私信',
    header: {
      todo: '待办',
      inventory: '技能库'
    },
    input: {
      placeholder: '发送到 {channel}',
      directPlaceholder: '发送给 {name}',
      send: '发送',
      stop: '停止生成',
      mentions: '提及',
      removeMention: '移除提及 {name}',
      hint: 'Enter 发送 • Shift+Enter 换行',
      quickPrompts: {
        summarize: '总结最新讨论',
        draftReply: '生成礼貌回复',
        extractTasks: '提取行动项'
      },
      emoji: {
        recent: '最近使用',
        search: '搜索表情...',
        empty: '没有匹配的表情',
        emptyRecent: '还没有常用表情',
        groups: {
          smileys: '表情与情绪',
          people: '人物与身体',
          component: '肤色与修饰',
          animals: '动物与自然',
          food: '食物与饮料',
          travel: '旅行与地点',
          activities: '活动',
          objects: '物品',
          symbols: '符号'
        }
      }
    },
    sidebar: {
      workspaceName: '工作区',
      channels: '频道',
      directMessages: '私信',
      channelList: {
        announcements: 'announcements',
        generalChat: 'general-chat',
        designCritique: 'design-critique',
        resources: 'resources'
      }
    },
    messages: {
      dateSeparator: '2023年10月24日',
      roadmapHint: '点击查看路线图',
      userJoined: '{name} 加入了服务器',
      joinedUser: 'James',
      sampleMessage: {
        user: 'Sarah Jenkins',
        time: '11:05 AM',
        text: '希望周五前完成！只差产品的最终确认。'
      },
      autoReply: '我已处理你的请求。还有其他需要吗？',
      autoReplyQuestion: '我正在查看这个问题，稍后给你更新。',
      loadHistory: '加载更早消息',
      loadingHistory: '正在加载历史...',
      jumpToLatest: '跳到最新消息',
      typing: '{name} 正在输入...',
      status: {
        sending: '发送中...',
        failed: '发送失败'
      }
    },
    conversation: {
      actions: {
        pin: '置顶',
        unpin: '取消置顶',
        rename: '修改群聊名称',
        mute: '消息免打扰',
        unmute: '取消免打扰',
        clear: '清空聊天记录',
        deleteChannel: '删除群聊',
        deleteDirect: '删除对话'
      },
      renameTitle: '修改群聊名称',
      renameLabel: '群聊名称',
      renamePlaceholder: '输入群聊名称'
    }
  },
  members: {
    title: '成员',
    sections: {
      owner: '群主 — {count}',
      admin: '管理员 — {count}',
      assistant: '助手 — {count}',
      member: '普通成员 — {count}'
    },
    roles: {
      owner: '群主',
      admin: '管理员',
      assistant: '助手',
      member: '成员',
      aiAssistant: 'AI 助手'
    },
    actions: {
      sendMessage: '发送消息',
      mention: '提及',
      rename: '更改名称'
    },
    manage: {
      title: '管理成员',
      displayName: '显示名称',
      remove: '移出群组'
    },
    activity: {
      reviewingPRs: '审核 PR',
      listeningSpotify: '在听 Spotify',
      fixingBugs: '修复 Bug',
      doNotDisturb: '请勿打扰'
    }
  },
  friends: {
    title: '好友',
    add: '添加',
    invite: '邀请',
    empty: '暂无好友',
    sections: {
      project: '项目好友',
      global: '全局好友'
    },
    inviteModal: {
      titleChannel: '邀请好友',
      titleDm: '创建群聊',
      search: '搜索好友',
      actionInvite: '邀请',
      actionCreate: '创建群聊'
    }
  },
  invite: {
    menu: {
      title: '邀请加入服务器',
      subtitle: '生成唯一邀请链接',
      admin: '以管理员身份邀请',
      adminDesc: '完全服务器权限',
      assistant: '以助手身份邀请',
      assistantDesc: '管理权限',
      member: '普通成员',
      memberDesc: '标准访问权限'
    },
    admin: {
      title: '以管理员身份邀请',
      subtitle: '配置访问级别与时长',
      uniqueLink: '唯一邀请链接',
      regenerate: '重新生成',
      userIdentifier: '用户标识',
      userPlaceholder: '用户名或邮箱地址',
      permissions: '权限等级',
      send: '发送邀请',
      permissionsList: {
        fullAccess: {
          title: '完全服务器权限',
          desc: '可修改设置、频道与角色'
        },
        billing: {
          title: '账单权限',
          desc: '管理订阅与付款'
        },
        memberManagement: {
          title: '成员管理',
          desc: '踢出、封禁并分配低级角色'
        }
      }
    },
    assistant: {
      title: '以助手身份邀请',
      subtitle: '选择要加入工作区的 AI 模型',
      instances: '实例数量',
      instanceLimit: '最多 {count} 个实例',
      unlimitedAccess: '无限制模式',
      unlimitedAccessDesc: '绕过使用限制',
      sandboxed: '沙盒环境',
      send: '发送邀请',
      models: {
        gemini: 'Gemini CLI',
        codex: 'Codex',
        claude: 'Claude Code',
        custom: '自定义 CLI'
      }
    },
    member: {
      title: '以成员身份邀请'
    }
  },
  roadmap: {
    title: '项目路线图',
    objectiveLabel: '目标：',
    taskPlaceholder: '输入任务标题...',
    newTask: '新任务',
    status: {
      done: '已完成',
      inProgress: '进行中',
      pending: '待处理'
    },
    actions: {
      edit: '编辑任务',
      changeOrder: '调整顺序',
      markPriority: '标记为优先',
      delete: '删除'
    },
    footer: '{count} 个任务 • 完成 {percent}%',
    addTask: '添加任务'
  },
  skills: {
    management: {
      title: '技能管理',
      subtitle: '为 {channel} 配置已启用技能',
      tabs: {
        current: '当前技能',
        library: '我的技能'
      }
    },
    current: {
      activeFolders: '已启用的文件夹',
      syncAll: '全部同步',
      updated: '2 小时前更新',
      active: '启用中'
    },
    project: {
      title: '项目技能',
      import: '导入我的技能',
      pickerSubtitle: '从技能库中选择并关联到当前工作区。',
      searchPlaceholder: '搜索技能库中的技能...',
      linkAction: '关联',
      loading: '正在加载项目技能...',
      empty: '当前还没有关联项目技能。',
      emptyLibrary: '我的技能库里暂无可用技能。',
      emptySearch: '没有匹配的技能。',
      readOnlyHint: '工作区为只读状态，无法关联技能。',
      removeConfirmTitle: '移除项目技能？',
      removeConfirmMessage: '此操作将从当前工作区取消关联“{name}”。',
      removeConfirmOk: '取消关联',
      removeConfirmCancel: '取消'
    },
    library: {
      searchPlaceholder: '搜索你的技能库...',
      refresh: '刷新',
      importTitle: '导入技能',
      importSubtitle: '从 URL 或本地文件',
      removeConfirmTitle: '删除技能文件夹？',
      removeConfirmMessage: '这将从本地技能库中删除“{name}”。',
      removeConfirmOk: '删除',
      removeConfirmCancel: '取消',
      browseShop: '浏览技能商店'
    },
    footer: {
      documentation: '文档',
      privacy: '隐私',
      newFolder: '新建文件夹',
      lastSynced: '上次同步：2 分钟前'
    },
    tags: {
      typography: '排版',
      colorPalette: '色板',
      components: '组件'
    },
    items: {
      designSystemCore: {
        name: '设计系统核心'
      },
      uxResearchPatterns: {
        name: 'UX 研究模式'
      },
      a11yGuidelines: {
        name: '无障碍指南'
      },
      frontendToolkit: {
        name: '前端工具包',
        desc: '适用于 React、Vue 和 Tailwind CSS 的常用片段。'
      },
      iconAssetPack: {
        name: '图标资源包',
        desc: '现代界面设计的高品质线框与实心图标。'
      },
      motionPresets: {
        name: '动效预设',
        desc: '标准化的动画曲线与过渡时序。'
      },
      shellCommands: {
        name: 'Shell 命令集',
        desc: '快速访问常用 CLI 脚本和部署钩子。'
      },
      brandColors: {
        name: '品牌色板',
        desc: '公司色板与可访问性对比度规范。'
      }
    },
    assets: {
      frontendToolkit: '45 个资源',
      iconAssetPack: '1.2k 图标',
      motionPresets: '12 个预设',
      shellCommands: '24 条命令',
      brandColors: '8 个色板'
    },
    detail: {
      sourceConfig: '来源配置',
      source: {
        github: 'GitHub 仓库',
        command: '命令来源',
        local: '本地路径'
      },
      repoLabel: '仓库地址',
      repoPlaceholder: 'https://github.com/username/repository.git',
      repoHint: '支持 GitHub、GitLab 与 Bitbucket 的 HTTPS 和 SSH 地址。',
      syncPreferences: '同步偏好',
      autoSync: '自动同步更新',
      autoSyncDesc: '自动拉取来源的最新变更',
      updateFrequency: '更新频率',
      updateFrequencyDesc: '多久检查一次新版本',
      frequency: {
        every15: '每 15 分钟',
        hour: '每小时',
        daily: '每日',
        manual: '仅手动'
      },
      targetBranch: '目标分支',
      targetBranchDesc: '用于更新跟踪的分支',
      deleteSkill: '删除技能',
      cancel: '取消',
      saveChanges: '保存修改'
    }
  },
  marketplace: {
    title: '插件市场',
    searchPlaceholder: '搜索插件、集成和主题...',
    browseStore: '浏览商店',
    myPlugins: '我的插件',
    importTitle: '导入插件',
    importSubtitle: '从 URL 或本地文件',
    categories: {
      all: '全部插件',
      productivity: '效率',
      development: '开发',
      design: '设计',
      communication: '沟通',
      music: '音乐'
    },
    install: '安装',
    installed: '已安装',
    plugins: {
      github: {
        title: 'GitHub 集成',
        desc: '连接你的仓库，跟踪问题并直接管理 PR。'
      },
      spotify: {
        title: 'Spotify 播放器',
        desc: '一起听歌。控制播放并分享喜欢的曲目。'
      },
      taskManager: {
        title: '任务管理',
        desc: '适合团队的简洁看板。创建、分配并完成任务。'
      },
      calendar: {
        title: '日历同步',
        desc: '不错过会议。同步 Google 日历与 Outlook。'
      },
      aiAssistant: {
        title: 'AI 助手',
        desc: '你的 AI 伙伴。提问、生成文本并做摘要。'
      },
      terminal: {
        title: '终端',
        desc: '从聊天直接运行命令和脚本。适合高阶用户。'
      },
      figma: {
        title: 'Figma 预览',
        desc: '嵌入实时 Figma 文件与原型，即时反馈。'
      },
      quickNotes: {
        title: '快速笔记',
        desc: '记录想法并与团队分享。支持 Markdown。'
      }
    }
  },
  skillStore: {
    title: '技能商店',
    searchPlaceholder: '搜索技能文件夹、模板和工具包...',
    tabs: {
      store: '商店',
      installed: '我的技能'
    },
    filters: {
      all: '全部技能',
      engineering: '工程',
      design: '设计',
      management: '管理',
      marketing: '营销',
      finance: '财务'
    },
    syncPlaceholder: '粘贴同步 URL...',
    syncNow: '立即同步',
    installFolder: '安装文件夹',
    installed: '我的技能',
    skills: {
      automation: {
        title: '自动化技能',
        desc: '通过预置脚本简化流程，可与仓库自动同步。'
      },
      uiToolkit: {
        title: 'UI 设计工具包',
        desc: '集中式设计资产与品牌规范，同步 Figma 文件。'
      },
      projectTracking: {
        title: '项目跟踪',
        desc: '用于冲刺的任务清单与看板，可同步 Jira 或 Trello。'
      },
      marketingAssets: {
        title: '营销素材',
        desc: '活动物料与社媒模板，同步 Drive/Dropbox。'
      },
      devOpsConfig: {
        title: 'Dev Ops 配置',
        desc: '共享环境变量与 Docker 配置，同步安全密钥库。'
      },
      researchLibrary: {
        title: '研究资料库',
        desc: '竞品分析与市场趋势，Notion/Evernote 同步。'
      }
    }
  },
  workspace: {
    openTitle: '打开文件夹',
    openSubtitle: '选择一个文件夹开始或恢复工作区',
    recentTitle: '最近的工作区',
    more: '更多',
    searchPlaceholder: '搜索文件夹...',
    emptyTitle: '暂无最近工作区',
    emptySubtitle: '打开文件夹以创建你的第一个工作区。',
    openAction: '打开',
    noResults: '未找到匹配的工作区',
    openErrorTitle: '无法打开工作区',
    readOnlyTitle: '工作区只读',
    readOnlySubtitle: '当前文件夹不可写，项目数据不会保存。',
    registryMismatchTitle: '工作区位置变化',
    registryMismatchMessage: '检测到项目位置发生变化。\n旧路径: {oldPath}\n当前路径: {newPath}\n你是移动了原项目，还是复制了一个新副本？',
    registryMismatchMoved: '移动了',
    registryMismatchCopied: '复制副本'
  },
  settings: {
    title: '设置',
    preferences: '偏好设置',
    preferencesSubtitle: '自定义账号、通知与工作区偏好。',
    accountSubtitle: '管理你的资料、状态与联系方式。',
    avatarTitle: '头像',
    avatarSubtitle: '选择样式或上传图片。',
    avatarUpload: '上传图片',
    avatarReset: '恢复样式',
    avatarHint: '支持 PNG/JPG/WEBP，最大 2MB。',
    avatarUploads: '已上传',
    avatarErrors: {
      invalidType: '请上传图片文件。',
      tooLarge: '图片超过 2MB。',
      storageFailed: '头像存储失败。'
    },
    avatarOptions: {
      orbit: '轨道',
      ember: '余烬',
      mint: '薄荷',
      canyon: '峡谷',
      storm: '风暴'
    },
    displayName: '显示名称',
    displayNamePlaceholder: '输入显示名称',
    emailAddress: '邮箱地址',
    emailPlaceholder: 'name{at}example.com',
    changeEmail: '更改邮箱',
    jobTitle: '职位',
    jobTitlePlaceholder: '例如 产品设计师',
    timeZone: '时区',
    timeZones: {
      utc: 'UTC协调世界时',
      pacificMidway: '太平洋/中途岛',
      pacificHonolulu: '太平洋/檀香山',
      americaAnchorage: '美洲/安克雷奇',
      americaLosAngeles: '美洲/洛杉矶',
      americaDenver: '美洲/丹佛',
      americaChicago: '美洲/芝加哥',
      americaNewYork: '美洲/纽约',
      americaHalifax: '美洲/哈利法克斯',
      americaSaoPaulo: '美洲/圣保罗',
      atlanticAzores: '大西洋/亚速尔',
      europeLondon: '欧洲/伦敦',
      europeParis: '欧洲/巴黎',
      europeHelsinki: '欧洲/赫尔辛基',
      europeMoscow: '欧洲/莫斯科',
      asiaDubai: '亚洲/迪拜',
      asiaKarachi: '亚洲/卡拉奇',
      asiaDhaka: '亚洲/达卡',
      asiaBangkok: '亚洲/曼谷',
      asiaShanghai: '亚洲/上海',
      asiaTokyo: '亚洲/东京',
      australiaSydney: '澳大利亚/悉尼',
      pacificNoumea: '太平洋/努美阿',
      pacificAuckland: '太平洋/奥克兰'
    },
    status: '状态',
    statusMessage: '状态信息',
    statusMessagePlaceholder: '分享你正在做的事情',
    statusOptions: {
      online: '在线',
      working: '工作中',
      dnd: '请勿打扰',
      offline: '离线'
    },
    language: '语言',
    languageDefault: '默认',
    changesApply: '重启后生效。',
    defaultMember: '默认成员',
    selectMember: '选择成员',
    selectTerminal: '选择终端',
    terminalAuto: '系统默认',
    terminalAutoHint: '使用系统默认 shell。',
    terminalCustom: '自定义终端',
    terminalName: '终端名称',
    terminalNamePlaceholder: '例如 PowerShell',
    terminalPath: '终端可执行文件',
    terminalPathPlaceholder: '选择终端可执行文件',
    terminalBrowse: '选择文件',
    terminalEmpty: '未检测到可用终端。',
    terminalNotAvailable: '仅桌面端可用。',
    refreshList: '刷新列表',
    cancel: '取消',
    saveChanges: '保存修改',
    userSettings: '用户设置',
    myAccount: '我的账号',
    appSettings: '应用设置',
    appearance: '外观',
    appearanceSubtitle: '切换主题并匹配你的工作环境。',
    members: '成员',
    notifications: '通知',
    keybinds: '快捷键',
    createTeam: '新建团队',
    leaveTeam: '退出团队',
    logOut: '退出账号',
    memberName: '成员名称',
    memberNamePlaceholder: '输入成员标签',
    commandInput: '命令行输入',
    commandPlaceholder: '终端启动命令 例如:gemini',
    confirm: '确认',
    notificationsSubtitle: '选择你接收通知的方式与时机。',
    notificationOptions: {
      desktop: '桌面通知',
      desktopDesc: '新消息时显示系统通知。',
      sound: '声音提醒',
      soundDesc: '新消息到达时播放提示音。',
      mentionsOnly: '仅提醒提及',
      mentionsOnlyDesc: '只有被提及时才提醒。',
      previews: '消息预览',
      previewsDesc: '在通知中显示消息内容。',
      quietHours: '静默时段',
      quietHoursDesc: '在设定时间段内关闭提醒。'
    },
    quietHoursFrom: '开始时间',
    quietHoursTo: '结束时间',
    keybindsSubtitle: '配置快捷键与键位方案。',
    keybindsProfile: '键位方案',
    keybindsEnable: '启用快捷键',
    keybindsHints: '显示快捷键提示',
    keybindsReset: '恢复默认',
    keybindsListTitle: '快捷键参考',
    data: '数据',
    dataSubtitle: '修复或重置当前工作区的聊天存储。',
    dataRepairTitle: '修复消息库',
    dataRepairHint: '扫描并移除无法读取或损坏的消息。',
    dataRepairAction: '修复消息',
    dataRepairConfirm: '扫描并修复当前工作区的聊天库，移除不可读消息？',
    dataRepairResult: '修复完成，已移除 {count} 条消息。',
    dataClearTitle: '清空聊天记录',
    dataClearHint: '删除当前工作区的全部消息和附件。',
    dataClearAction: '清空所有消息',
    chatStreamTitle: '聊天流式输出',
    chatStreamHint: '执行中展示终端流式输出，关闭后仅回写最终结果。',
    dataClearConfirm: '此操作会永久删除当前工作区的所有聊天记录，是否继续？',
    dataClearResult: '已清空 {messages} 条消息，{attachments} 个附件。',
    dataActionFailed: '操作失败，请重试。',
    terminalCallChainsTitle: '调用链详情',
    terminalSnapshotAuditTitle: '终端快照审计',
    terminalSnapshotAuditHint: '打开并关闭终端窗口，对比前端与后端快照。',
    terminalSnapshotAuditAction: '运行快照审计',
    terminalSnapshotAuditRunning: '正在运行快照审计...',
    terminalSnapshotAuditNotAvailable: '仅桌面端可用。',
    terminalSnapshotAuditStatus: {
      idle: '快照审计待运行。',
      running: '快照审计进行中...',
      passed: '快照审计通过。',
      failed: '快照审计失败。'
    },
    terminalSnapshotAuditLegend: 'FB：前端对后端 · FR：前端对重开 · BR：后端对重开',
    terminalSnapshotAuditRound: '第 {round} 轮',
    terminalSnapshotAuditCreated: '审计临时创建',
    terminalSnapshotAuditNoMembers: '暂无终端成员可测试。',
    terminalSnapshotAuditCheck: {
      ok: 'OK',
      ng: 'NG'
    },
    keybindProfiles: {
      default: '默认',
      vscode: 'VS Code',
      slack: 'Slack'
    },
    keybindActions: {
      focusSearch: '聚焦搜索',
      newMessage: '新建消息',
      toggleSidebar: '切换侧边栏',
      toggleMute: '切换静音',
      jumpToLatest: '跳到最新消息',
      openSettings: '打开设置'
    },
    memberOptions: {
      gemini: 'Gemini CLI',
      codex: 'Codex',
      claude: 'Claude Code',
      opencode: 'opencode',
      qwen: 'Qwen Code',
      terminal: '终端',
      custom: '自定义 CLI'
    },
    memberKind: {
      default: '默认终端',
      custom: '自定义终端'
    },
    memberActions: {
      menuLabel: '终端操作',
      test: '测试终端',
      edit: '更改',
      remove: '删除'
    },
    terminalFriends: {
      title: '终端好友',
      desc: '删除终端成员并重置命名序号。',
      action: '删除终端好友',
      confirmCurrent: '仅删除当前项目的终端好友并重置序号？',
      resultCurrent: '已删除当前项目的 {count} 个终端好友。',
      failed: '删除终端好友失败。',
      noWorkspace: '请先打开工作区。'
    },
    terminalTestErrors: {
      shellBinaryNotFound: '找不到终端可执行文件：{path}',
      shellLaunchFailed: '终端启动失败：{error}'
    },
    terminalTestFailed: '测试终端失败：{error}',
    themeOptions: {
      dark: {
        label: '深色',
        desc: '默认深色主题，更适合低光环境。'
      },
      light: {
        label: '浅色',
        desc: '明亮布局，适合白天或高亮环境。'
      },
      system: {
        label: '系统',
        desc: '跟随操作系统外观设置。'
      }
    }
  },
  language: {
    enUS: '英文（美国）',
    zhCN: '中文（简体）'
  }
};
