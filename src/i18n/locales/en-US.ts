// [2026-01-23 00:59] 目的: 维护英文界面文案与键值映射，集中管理以便一致性与可翻译性; 边界: 仅包含文案资源，不引入业务逻辑或运行时行为; 设计: 键结构与功能域对齐以降低跨模块引用成本与回归风险。
export default {
  app: {
    name: 'golutra',
    windowControls: {
      minimize: 'Minimize',
      maximize: 'Maximize',
      restore: 'Restore',
      close: 'Close'
    }
  },
  nav: {
    chat: 'Chat & Modals',
    friends: 'Friends',
    terminal: 'Terminal',
    workspaces: 'Workspaces',
    store: 'Skill Store',
    plugins: 'Plugins',
    settings: 'Settings'
  },
  terminal: {
    title: 'Terminal',
    subtitle: 'Run multiple shells in one workspace.',
    newTab: 'New Tab',
    tabSearchPlaceholder: 'Search tabs...',
    tabSearchCreate: 'New tab',
    tabSearchEmpty: 'No matching tabs.',
    recentClosedTabs: 'Recently closed ({count})',
    emptyTabs: 'No terminals yet.',
    emptyTitle: 'No active terminals',
    emptySubtitle: 'Create a new terminal tab to get started.',
    splitEmpty: 'Drag a tab here or create a new terminal.',
    unavailableTitle: 'Terminal unavailable',
    unavailableSubtitle: 'Open this view inside the Tauri desktop app to use terminals.',
    errorTitle: 'Terminal failed to start',
    errorSubtitle: 'Check the desktop runtime and try again.',
    resourceLimit: 'System resources are low. Close some background terminals and try again.',
    statusLabel: 'Terminal status',
    statusOptions: {
      pending: 'Pending',
      connecting: 'Connecting',
      connected: 'Connected',
      working: 'Working',
      disconnected: 'Disconnected'
    },
    tabMenu: {
      close: 'Close',
      closeOthers: 'Close Others',
      closeRight: 'Close to the Right',
      pin: 'Pin to Front',
      unpin: 'Unpin',
      layoutSingle: 'Single pane',
      layoutSplitVertical: 'Split left/right',
      layoutSplitHorizontal: 'Split top/bottom',
      layoutGrid: '2x2 grid'
    },
    contextMenu: {
      clear: 'Clear',
      find: 'Find'
    },
    findPlaceholder: 'Find...',
    findNoResults: 'No results',
    findCaseSensitive: 'Match case',
    findWholeWord: 'Whole word',
    findRegex: 'Use regex'
  },
  contextMenu: {
    copy: 'Copy',
    cut: 'Cut',
    paste: 'Paste',
    selectAll: 'Select All'
  },
    common: {
      userAvatarAlt: 'User',
      remove: 'Remove',
      openFolder: 'Open folder'
    },
  chat: {
    channelName: 'general',
    channelDisplay: '#general',
    channelDescription: '',
    directMessageDescription: 'Direct message with {name}',
    header: {
      todo: 'Todo',
      inventory: 'Inventory'
    },
    input: {
      placeholder: 'Message {channel}',
      directPlaceholder: 'Message {name}',
      send: 'Send',
      stop: 'Stop',
      mentions: 'Mentions',
      removeMention: 'Remove mention of {name}',
      hint: 'Enter to send • Shift+Enter for newline',
      quickPrompts: {
        summarize: 'Summarize the latest discussion',
        draftReply: 'Draft a polite reply',
        extractTasks: 'Extract action items'
      },
      emoji: {
        recent: 'Recent',
        search: 'Search emoji...',
        empty: 'No matching emoji',
        emptyRecent: 'No recent emoji yet',
        groups: {
          smileys: 'Smileys & Emotion',
          people: 'People & Body',
          component: 'Component',
          animals: 'Animals & Nature',
          food: 'Food & Drink',
          travel: 'Travel & Places',
          activities: 'Activities',
          objects: 'Objects',
          symbols: 'Symbols'
        }
      }
    },
    sidebar: {
      workspaceName: 'Workspace',
      channels: 'Channels',
      directMessages: 'Direct Messages',
      channelList: {
        announcements: 'announcements',
        generalChat: 'general-chat',
        designCritique: 'design-critique',
        resources: 'resources'
      }
    },
    messages: {
      dateSeparator: 'October 24, 2023',
      roadmapHint: 'Click to view roadmap',
      userJoined: '{name} joined the server',
      joinedUser: 'James',
      sampleMessage: {
        user: 'Sarah Jenkins',
        time: '11:05 AM',
        text: 'Hopefully by Friday! Just need final sign-off from product.'
      },
      autoReply: 'I have processed your request. Is there anything else you need?',
      autoReplyQuestion: 'Let me take a closer look and get back to you shortly.',
      loadHistory: 'Load earlier messages',
      loadingHistory: 'Loading history...',
      jumpToLatest: 'Jump to latest',
      typing: '{name} is typing...',
      status: {
        sending: 'Sending...',
        failed: 'Failed to send'
      }
    },
    conversation: {
      actions: {
        pin: 'Pin',
        unpin: 'Unpin',
        rename: 'Rename Group',
        mute: 'Mute Notifications',
        unmute: 'Unmute Notifications',
        clear: 'Clear Chat History',
        deleteChannel: 'Delete Group Chat',
        deleteDirect: 'Delete Conversation'
      },
      renameTitle: 'Rename Group Chat',
      renameLabel: 'Group Name',
      renamePlaceholder: 'Enter group name'
    }
  },
  members: {
    title: 'Members',
    sections: {
      owner: 'Group Owner — {count}',
      admin: 'Admins — {count}',
      assistant: 'Assistants — {count}',
      member: 'General Members — {count}'
    },
    roles: {
      owner: 'Owner',
      admin: 'Admin',
      assistant: 'Assistant',
      member: 'Member',
      aiAssistant: 'AI Assistant'
    },
    actions: {
      sendMessage: 'Send Message',
      mention: 'Mention',
      rename: 'Rename'
    },
    manage: {
      title: 'Manage Member',
      displayName: 'Display Name',
      remove: 'Remove from Group'
    },
    activity: {
      reviewingPRs: 'Reviewing PRs',
      listeningSpotify: 'Listening to Spotify',
      fixingBugs: 'Fixing bugs',
      doNotDisturb: 'Do Not Disturb'
    }
  },
  friends: {
    title: 'Friends',
    add: 'Add',
    invite: 'Invite',
    empty: 'No friends yet',
    sections: {
      project: 'Project Friends',
      global: 'Global Friends'
    },
    inviteModal: {
      titleChannel: 'Invite Friends',
      titleDm: 'Create Group Chat',
      search: 'Search friends',
      actionInvite: 'Invite',
      actionCreate: 'Create Group'
    }
  },
  invite: {
    menu: {
      title: 'Invite to Server',
      subtitle: 'Generate a unique invite link',
      admin: 'Invite as Admin',
      adminDesc: 'Full server access',
      assistant: 'Invite as Assistant',
      assistantDesc: 'Moderation permissions',
      member: 'General Member',
      memberDesc: 'Standard access'
    },
    admin: {
      title: 'Invite as Admin',
      subtitle: 'Configure access level and duration',
      uniqueLink: 'Unique Invite Link',
      regenerate: 'Regenerate',
      userIdentifier: 'User Identifier',
      userPlaceholder: 'Username or email address',
      permissions: 'Permissions Level',
      send: 'Send Invitation',
      permissionsList: {
        fullAccess: {
          title: 'Full Server Access',
          desc: 'Can modify settings, channels & roles'
        },
        billing: {
          title: 'Billing Access',
          desc: 'Manage subscription and payments'
        },
        memberManagement: {
          title: 'Member Management',
          desc: 'Kick, ban, and assign lower roles'
        }
      }
    },
    assistant: {
      title: 'Invite as Assistant',
      subtitle: 'Select an AI model to join the workspace',
      instances: 'Number of Instances',
      instanceLimit: 'Max {count} instances',
      unlimitedAccess: 'Unlimited Mode',
      unlimitedAccessDesc: 'Bypass usage limits',
      sandboxed: 'Sandboxed environment',
      send: 'Send Invitation',
      models: {
        gemini: 'Gemini CLI',
        codex: 'Codex',
        claude: 'Claude Code',
        custom: 'Custom CLI'
      }
    },
    member: {
      title: 'Invite as Member'
    }
  },
  roadmap: {
    title: 'Project Roadmap',
    objectiveLabel: 'Objective:',
    taskPlaceholder: 'Enter task title...',
    newTask: 'New Task',
    status: {
      done: 'DONE',
      inProgress: 'IN PROGRESS',
      pending: 'PENDING'
    },
    actions: {
      edit: 'Edit Task',
      changeOrder: 'Change Order',
      markPriority: 'Mark as Priority',
      delete: 'Delete'
    },
    footer: '{count} Tasks • {percent}% Complete',
    addTask: 'Add Task'
  },
  skills: {
    management: {
      title: 'Skill Management',
      subtitle: 'Configure active skills for {channel}',
      tabs: {
        current: 'Current Skills',
        library: 'My Skills'
      }
    },
    current: {
      activeFolders: 'Active Folders',
      syncAll: 'Sync All',
      updated: 'Updated 2h ago',
      active: 'Active'
    },
    project: {
      title: 'Project Skills',
      import: 'Import My Skills',
      pickerSubtitle: 'Select skills from your library to link into this workspace.',
      searchPlaceholder: 'Search skills in your library...',
      linkAction: 'Link',
      loading: 'Loading project skills...',
      empty: 'No project skills linked yet.',
      emptyLibrary: 'No skills available in your library.',
      emptySearch: 'No skills match your search.',
      readOnlyHint: 'Workspace is read-only. Linking skills is disabled.',
      removeConfirmTitle: 'Unlink project skill?',
      removeConfirmMessage: 'This will unlink "{name}" from this workspace.',
      removeConfirmOk: 'Unlink',
      removeConfirmCancel: 'Cancel'
    },
    library: {
      searchPlaceholder: 'Search your library...',
      refresh: 'Refresh',
      importTitle: 'Import Skill',
      importSubtitle: 'From URL or Local File',
      removeConfirmTitle: 'Remove skill folder?',
      removeConfirmMessage: 'This will delete "{name}" from your local skills library.',
      removeConfirmOk: 'Delete',
      removeConfirmCancel: 'Cancel',
      browseShop: 'Browse Skill Shop'
    },
    footer: {
      documentation: 'Documentation',
      privacy: 'Privacy',
      newFolder: 'New Folder',
      lastSynced: 'Last synced: 2 mins ago'
    },
    tags: {
      typography: 'Typography',
      colorPalette: 'Color Palette',
      components: 'Components'
    },
    items: {
      designSystemCore: {
        name: 'Design System Core'
      },
      uxResearchPatterns: {
        name: 'UX Research Patterns'
      },
      a11yGuidelines: {
        name: 'A11y Guidelines'
      },
      frontendToolkit: {
        name: 'Frontend Toolkit',
        desc: 'Essential snippets for React, Vue, and Tailwind CSS development.'
      },
      iconAssetPack: {
        name: 'Icon Asset Pack',
        desc: 'Premium outline and solid icons for modern interface design.'
      },
      motionPresets: {
        name: 'Motion Presets',
        desc: 'Standardized animation curves and transition timings.'
      },
      shellCommands: {
        name: 'Shell Commands',
        desc: 'Quick access to common CLI scripts and deployment hooks.'
      },
      brandColors: {
        name: 'Brand Colors',
        desc: 'Company color palettes and accessible contrast ratios.'
      }
    },
    assets: {
      frontendToolkit: '45 assets',
      iconAssetPack: '1.2k icons',
      motionPresets: '12 presets',
      shellCommands: '24 cmds',
      brandColors: '8 swatches'
    },
    detail: {
      sourceConfig: 'Source Configuration',
      source: {
        github: 'GitHub Repo',
        command: 'Command Source',
        local: 'Local Path'
      },
      repoLabel: 'Repository URL',
      repoPlaceholder: 'https://github.com/username/repository.git',
      repoHint: 'Supports HTTPS and SSH URLs from GitHub, GitLab, and Bitbucket.',
      syncPreferences: 'Sync Preferences',
      autoSync: 'Auto-sync Updates',
      autoSyncDesc: 'Automatically pull latest changes from source',
      updateFrequency: 'Update Frequency',
      updateFrequencyDesc: 'How often to check for new versions',
      frequency: {
        every15: 'Every 15 minutes',
        hour: 'Every hour',
        daily: 'Daily',
        manual: 'Manual only'
      },
      targetBranch: 'Target Branch',
      targetBranchDesc: 'Branch to track for updates',
      deleteSkill: 'Delete Skill',
      cancel: 'Cancel',
      saveChanges: 'Save Changes'
    }
  },
  marketplace: {
    title: 'Plugin Marketplace',
    searchPlaceholder: 'Search plugins, integrations, and themes...',
    browseStore: 'Browse Store',
    myPlugins: 'My Plugins',
    importTitle: 'Import Plugin',
    importSubtitle: 'From URL or Local File',
    categories: {
      all: 'All Plugins',
      productivity: 'Productivity',
      development: 'Development',
      design: 'Design',
      communication: 'Communication',
      music: 'Music'
    },
    install: 'Install',
    installed: 'Installed',
    plugins: {
      github: {
        title: 'GitHub Integration',
        desc: 'Connect your repositories, track issues, and manage pull requests directly.'
      },
      spotify: {
        title: 'Spotify Player',
        desc: 'Listen together. Control playback and share your favorite tracks.'
      },
      taskManager: {
        title: 'Task Manager',
        desc: 'A simple Kanban board for your team. Create, assign, and complete tasks.'
      },
      calendar: {
        title: 'Calendar Sync',
        desc: 'Never miss a meeting. Sync with Google Calendar and Outlook.'
      },
      aiAssistant: {
        title: 'AI Assistant',
        desc: 'Your personal AI companion. Ask questions, generate text, and summarize.'
      },
      terminal: {
        title: 'Terminal',
        desc: 'Run commands and scripts directly from the chat. For power users only.'
      },
      figma: {
        title: 'Figma Preview',
        desc: 'Embed live Figma files and prototypes. Get feedback instantly.'
      },
      quickNotes: {
        title: 'Quick Notes',
        desc: 'Jot down ideas and share them with your team. Supports markdown.'
      }
    }
  },
  skillStore: {
    title: 'Skill Store',
    searchPlaceholder: 'Search skill folders, templates, and toolkits...',
    tabs: {
      store: 'Store',
      installed: 'My Skills'
    },
    filters: {
      all: 'All Skills',
      engineering: 'Engineering',
      design: 'Design',
      management: 'Management',
      marketing: 'Marketing',
      finance: 'Finance'
    },
    syncPlaceholder: 'Paste sync URL...',
    syncNow: 'Sync now',
    installFolder: 'Install Folder',
    installed: 'My Skills',
    skills: {
      automation: {
        title: 'Automation Skill',
        desc: 'Streamline workflows with pre-built scripts. Auto-syncs with your repo.'
      },
      uiToolkit: {
        title: 'UI Design Toolkit',
        desc: 'Centralized design assets and brand guidelines. Syncs with Figma files.'
      },
      projectTracking: {
        title: 'Project Tracking',
        desc: 'Task lists and kanban boards for active sprints. Syncs with Jira or Trello.'
      },
      marketingAssets: {
        title: 'Marketing Assets',
        desc: 'Campaign materials and social media templates. Syncs with Drive/Dropbox.'
      },
      devOpsConfig: {
        title: 'Dev Ops Config',
        desc: 'Shared environment variables and docker configs. Syncs secure vaults.'
      },
      researchLibrary: {
        title: 'Research Library',
        desc: 'Competitor analysis and market trends. Syncs with Notion or Evernote pages.'
      }
    }
  },
  workspace: {
    openTitle: 'Open Folder',
    openSubtitle: 'Pick a folder to start or resume a workspace',
    recentTitle: 'Recent Workspaces',
    more: 'More',
    searchPlaceholder: 'Search folders...',
    emptyTitle: 'No recent workspaces',
    emptySubtitle: 'Open a folder to start your first workspace.',
    openAction: 'Open',
    noResults: 'No matching workspaces',
    openErrorTitle: 'Unable to open workspace',
    readOnlyTitle: 'Workspace is read-only',
    readOnlySubtitle: 'This folder is not writable. Project metadata will not be saved.',
    registryMismatchTitle: 'Workspace location changed',
    registryMismatchMessage: 'The workspace location has changed.\nPrevious path: {oldPath}\nCurrent path: {newPath}\nDid you move the original project or copy a new duplicate?',
    registryMismatchMoved: 'Moved',
    registryMismatchCopied: 'Copied'
  },
  settings: {
    title: 'Settings',
    preferences: 'Preferences',
    preferencesSubtitle: 'Customize your account, notifications, and workspace preferences.',
    accountSubtitle: 'Manage your profile, presence, and contact details.',
    avatarTitle: 'Avatar',
    avatarSubtitle: 'Pick a style or upload your own.',
    avatarUpload: 'Upload Image',
    avatarReset: 'Use Style',
    avatarHint: 'PNG/JPG/WEBP up to 2MB.',
    avatarUploads: 'Uploads',
    avatarErrors: {
      invalidType: 'Please upload an image file.',
      tooLarge: 'Image is larger than 2MB.',
      storageFailed: 'Unable to save avatar.'
    },
    avatarOptions: {
      orbit: 'Orbit',
      ember: 'Ember',
      mint: 'Mint',
      canyon: 'Canyon',
      storm: 'Storm'
    },
    displayName: 'Display Name',
    displayNamePlaceholder: 'Enter your display name',
    emailAddress: 'Email Address',
    emailPlaceholder: 'name{at}example.com',
    changeEmail: 'Change Email',
    jobTitle: 'Job Title',
    jobTitlePlaceholder: 'e.g. Product Designer',
    timeZone: 'Time Zone',
    timeZones: {
      utc: 'UTC',
      pacificMidway: 'Pacific/Midway',
      pacificHonolulu: 'Pacific/Honolulu',
      americaAnchorage: 'America/Anchorage',
      americaLosAngeles: 'America/Los_Angeles',
      americaDenver: 'America/Denver',
      americaChicago: 'America/Chicago',
      americaNewYork: 'America/New_York',
      americaHalifax: 'America/Halifax',
      americaSaoPaulo: 'America/Sao_Paulo',
      atlanticAzores: 'Atlantic/Azores',
      europeLondon: 'Europe/London',
      europeParis: 'Europe/Paris',
      europeHelsinki: 'Europe/Helsinki',
      europeMoscow: 'Europe/Moscow',
      asiaDubai: 'Asia/Dubai',
      asiaKarachi: 'Asia/Karachi',
      asiaDhaka: 'Asia/Dhaka',
      asiaBangkok: 'Asia/Bangkok',
      asiaShanghai: 'Asia/Shanghai',
      asiaTokyo: 'Asia/Tokyo',
      australiaSydney: 'Australia/Sydney',
      pacificNoumea: 'Pacific/Noumea',
      pacificAuckland: 'Pacific/Auckland'
    },
    status: 'Status',
    statusMessage: 'Status Message',
    statusMessagePlaceholder: 'Share what you are working on',
    statusOptions: {
      online: 'Online',
      working: 'Working',
      dnd: 'Do Not Disturb',
      offline: 'Offline'
    },
    language: 'Language',
    languageDefault: 'Default',
    changesApply: 'Changes apply after restart.',
    defaultMember: 'Default Member',
    selectMember: 'Select Member',
    selectTerminal: 'Select Terminal',
    terminalAuto: 'System Default',
    terminalAutoHint: 'Uses the OS default shell.',
    terminalCustom: 'Custom Terminal',
    terminalName: 'Terminal Name',
    terminalNamePlaceholder: 'e.g. PowerShell',
    terminalPath: 'Terminal Executable',
    terminalPathPlaceholder: 'Select a terminal executable',
    terminalBrowse: 'Browse',
    terminalEmpty: 'No terminals detected on this device.',
    terminalNotAvailable: 'Available in the desktop app only.',
    refreshList: 'Refresh List',
    cancel: 'Cancel',
    saveChanges: 'Save Changes',
    userSettings: 'User Settings',
    myAccount: 'My Account',
    appSettings: 'App Settings',
    appearance: 'Appearance',
    appearanceSubtitle: 'Switch themes and match the workspace look to your environment.',
    members: 'Members',
    notifications: 'Notifications',
    keybinds: 'Keybinds',
    createTeam: 'Create Team',
    leaveTeam: 'Leave Team',
    logOut: 'Log Out',
    memberName: 'Member Name',
    memberNamePlaceholder: 'Enter member label',
    commandInput: 'Command Line Input',
    commandPlaceholder: 'CLI startup command, e.g. gemini',
    confirm: 'Confirm',
    notificationsSubtitle: 'Choose when and how you want to be notified.',
    notificationOptions: {
      desktop: 'Desktop Notifications',
      desktopDesc: 'Show system notifications for new messages.',
      sound: 'Sound Alerts',
      soundDesc: 'Play a sound when new messages arrive.',
      mentionsOnly: 'Mentions Only',
      mentionsOnlyDesc: 'Only notify when you are mentioned.',
      previews: 'Message Previews',
      previewsDesc: 'Display message content in alerts.',
      quietHours: 'Quiet Hours',
      quietHoursDesc: 'Silence alerts during scheduled hours.'
    },
    quietHoursFrom: 'From',
    quietHoursTo: 'To',
    keybindsSubtitle: 'Configure shortcuts and keybinding profiles.',
    keybindsProfile: 'Keybinding Profile',
    keybindsEnable: 'Enable Keyboard Shortcuts',
    keybindsHints: 'Show Shortcut Hints',
    keybindsReset: 'Reset to Defaults',
    keybindsListTitle: 'Shortcut Reference',
    data: 'Data',
    dataSubtitle: 'Repair or reset local chat storage for this workspace.',
    dataRepairTitle: 'Repair message database',
    dataRepairHint: 'Scan and remove unreadable or corrupted messages.',
    dataRepairAction: 'Repair messages',
    dataRepairConfirm: 'Scan this workspace chat database and remove unreadable messages?',
    dataRepairResult: 'Repair completed. Removed {count} messages.',
    dataClearTitle: 'Clear chat history',
    dataClearHint: 'Delete all messages and attachments for this workspace.',
    dataClearAction: 'Clear all messages',
    chatStreamTitle: 'Chat streaming output',
    chatStreamHint: 'Stream terminal output into chat while the command is running.',
    dataClearConfirm: 'This will permanently remove all chat messages for this workspace. Continue?',
    dataClearResult: 'Cleared {messages} messages and {attachments} attachments.',
    dataActionFailed: 'Operation failed. Please try again.',
    terminalCallChainsTitle: 'Call chains',
    terminalSnapshotAuditTitle: 'Terminal snapshot audit',
    terminalSnapshotAuditHint: 'Open and close terminal windows to compare frontend and backend snapshots.',
    terminalSnapshotAuditAction: 'Run snapshot audit',
    terminalSnapshotAuditRunning: 'Running snapshot audit...',
    terminalSnapshotAuditNotAvailable: 'Available in the desktop app only.',
    terminalSnapshotAuditStatus: {
      idle: 'Snapshot audit idle.',
      running: 'Snapshot audit running...',
      passed: 'Snapshot audit passed.',
      failed: 'Snapshot audit failed.'
    },
    terminalSnapshotAuditLegend: 'FB: front vs backend · FR: front vs reopen · BR: backend vs reopen',
    terminalSnapshotAuditRound: 'Round {round}',
    terminalSnapshotAuditCreated: 'created for audit',
    terminalSnapshotAuditNoMembers: 'No terminal members available.',
    terminalSnapshotAuditCheck: {
      ok: 'OK',
      ng: 'NG'
    },
    keybindProfiles: {
      default: 'Default',
      vscode: 'VS Code',
      slack: 'Slack'
    },
    keybindActions: {
      focusSearch: 'Focus search',
      newMessage: 'New message',
      toggleSidebar: 'Toggle sidebar',
      toggleMute: 'Toggle mute',
      jumpToLatest: 'Jump to latest',
      openSettings: 'Open settings'
    },
    memberOptions: {
      gemini: 'Gemini CLI',
      codex: 'Codex',
      claude: 'Claude Code',
      opencode: 'opencode',
      qwen: 'Qwen Code',
      terminal: 'Terminal',
      custom: 'Custom CLI'
    },
    memberKind: {
      default: 'Default Terminal',
      custom: 'Custom Terminal'
    },
    memberActions: {
      menuLabel: 'Terminal actions',
      test: 'Test Terminal',
      edit: 'Edit',
      remove: 'Delete'
    },
    terminalFriends: {
      title: 'Terminal Friends',
      desc: 'Remove terminal members and reset name counters.',
      action: 'Delete Terminal Friends',
      confirmCurrent: 'Delete terminal friends in this project and reset counters?',
      resultCurrent: 'Removed {count} terminal friends from this project.',
      failed: 'Failed to delete terminal friends.',
      noWorkspace: 'Please open a workspace first.'
    },
    terminalTestErrors: {
      shellBinaryNotFound: 'Terminal executable not found: {path}',
      shellLaunchFailed: 'Failed to launch terminal: {error}'
    },
    terminalTestFailed: 'Test Terminal failed: {error}',
    themeOptions: {
      dark: {
        label: 'Dark',
        desc: 'Default theme designed for low-light focus.'
      },
      light: {
        label: 'Light',
        desc: 'Bright layout optimized for daylight work.'
      },
      system: {
        label: 'System',
        desc: 'Follows your operating system appearance.'
      }
    }
  },
  language: {
    enUS: 'English (United States)',
    zhCN: 'Chinese (Simplified)'
  }
};
