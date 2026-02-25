export default {
  app: {
    name: 'Nexus Dashboard'
  },
  nav: {
    chat: 'Chat & Modals',
    workspaces: 'Workspaces',
    store: 'Skill Store',
    plugins: 'Plugins',
    settings: 'Settings'
  },
  common: {
    userAvatarAlt: 'User',
    remove: 'Remove'
  },
  chat: {
    channelName: 'design-critique',
    channelDisplay: '#design-critique',
    channelDescription: '',
    directMessageDescription: 'Direct message with {name}',
    header: {
      todo: 'Todo',
      inventory: 'Inventory'
    },
    input: {
      placeholder: 'Message {channel}',
      directPlaceholder: 'Message @{name}',
      send: 'Send',
      stop: 'Stop',
      hint: 'Enter to send • Shift+Enter for newline',
      quickPrompts: {
        summarize: 'Summarize the latest discussion',
        draftReply: 'Draft a polite reply',
        extractTasks: 'Extract action items'
      }
    },
    sidebar: {
      workspaceName: 'Frontend Squad',
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
      mention: '@ Mention',
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
      unlimitedAccess: 'Unlimited Access',
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
        library: 'My Skills Library'
      }
    },
    current: {
      activeFolders: 'Active Folders',
      syncAll: 'Sync All',
      updated: 'Updated 2h ago',
      active: 'Active'
    },
    library: {
      searchPlaceholder: 'Search your library...',
      importTitle: 'Import Skill',
      importSubtitle: 'From URL or Local File',
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
      installed: 'Installed'
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
    installed: 'Installed',
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
    createTitle: 'Create New Team',
    createSubtitle: 'Initialize a fresh collaborative environment',
    recentTitle: 'Recent Workspaces',
    more: 'More',
    searchPlaceholder: 'Search teams...',
    menu: {
      settings: 'Workspace Settings',
      invite: 'Invite Members',
      copyId: 'Copy ID',
      leave: 'Leave Workspace'
    },
    activeLabel: 'ACTIVE',
    times: {
      twoMinutes: '2m ago',
      fourHours: '4h ago',
      oneDay: '1d ago'
    },
    list: {
      frontend: {
        title: 'Frontend Squad',
        desc: 'Main monorepo for the consumer-facing React application.'
      },
      mobile: {
        title: 'Mobile API',
        desc: 'GraphQL gateway and microservices for iOS/Android apps.'
      },
      infrastructure: {
        title: 'Infrastructure',
        desc: 'Terraform scripts and Kubernetes configuration files.'
      }
    },
    recent: {
      designSystems: 'Design Systems',
      marketingTeam: 'Marketing Team',
      backendCore: 'Backend Core'
    }
  },
  settings: {
    title: 'Settings',
    preferences: 'Preferences',
    preferencesSubtitle: 'Customize your account, notifications, and workspace preferences.',
    accountSubtitle: 'Manage your profile, presence, and contact details.',
    displayName: 'Display Name',
    displayNamePlaceholder: 'Enter your display name',
    emailAddress: 'Email Address',
    emailPlaceholder: 'name@example.com',
    jobTitle: 'Job Title',
    jobTitlePlaceholder: 'e.g. Product Designer',
    timeZone: 'Time Zone',
    status: 'Status',
    statusMessage: 'Status Message',
    statusMessagePlaceholder: 'Share what you are working on',
    statusOptions: {
      online: 'Online',
      away: 'Away',
      dnd: 'Do Not Disturb'
    },
    language: 'Language',
    languageDefault: 'Default',
    changesApply: 'Changes apply after restart.',
    spellCheck: 'Spell Check',
    defaultMember: 'Default Member',
    selectMember: 'Select Member',
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
    leaveTeam: 'Leave Team',
    logOut: 'Log Out',
    memberName: 'Member Name',
    memberNamePlaceholder: 'Enter member label',
    commandInput: 'Command Line Input',
    commandPlaceholder: 'e.g. /usr/local/bin/my-cli',
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
      weeklyDigest: 'Weekly Digest',
      weeklyDigestDesc: 'Receive a weekly summary of activity.',
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
