<template>
  <div class="flex h-full w-full overflow-hidden">
    <aside class="w-16 md:w-[280px] bg-glass-sidebar glass-panel border-r border-white/5 shrink-0 py-4 md:py-8 px-2 md:px-3 flex flex-col">
      <div class="px-4 mb-6 hidden md:block">
        <h2 class="text-white font-bold text-xl tracking-tight">{{ t('settings.title') }}</h2>
      </div>

      <div class="flex-1 overflow-y-auto custom-scrollbar px-1 md:px-2 space-y-4 md:space-y-6">
        <div>
          <h3 class="px-3 mb-2 text-white/40 text-[11px] font-bold uppercase tracking-widest hidden md:block">{{ t('settings.userSettings') }}</h3>
          <div class="space-y-0.5">
            <button type="button" :title="t('settings.myAccount')" @click="scrollToSection('account')" :class="sectionButtonClass('account')">
              <span class="material-symbols-outlined text-[20px]" :class="activeSection === 'account' ? 'text-primary' : ''">person</span>
              <span class="text-[14px] font-medium hidden md:inline">{{ t('settings.myAccount') }}</span>
            </button>
          </div>
        </div>

        <div>
          <h3 class="px-3 mb-2 text-white/40 text-[11px] font-bold uppercase tracking-widest hidden md:block">{{ t('settings.appSettings') }}</h3>
          <div class="space-y-0.5">
            <button type="button" :title="t('settings.appearance')" @click="scrollToSection('appearance')" :class="sectionButtonClass('appearance')">
              <span class="material-symbols-outlined text-[20px]" :class="activeSection === 'appearance' ? 'text-primary' : ''">palette</span>
              <span class="text-[14px] font-medium hidden md:inline">{{ t('settings.appearance') }}</span>
            </button>
            <button type="button" :title="t('settings.language')" @click="scrollToSection('language')" :class="sectionButtonClass('language')">
              <span class="material-symbols-outlined text-[20px]" :class="activeSection === 'language' ? 'text-primary' : ''">translate</span>
              <span class="text-[14px] font-medium hidden md:inline">{{ t('settings.language') }}</span>
            </button>
            <button type="button" :title="t('settings.members')" @click="scrollToSection('members')" :class="sectionButtonClass('members')">
              <span class="material-symbols-outlined text-[20px]" :class="activeSection === 'members' ? 'text-primary' : ''">groups</span>
              <span class="text-[14px] font-medium hidden md:inline">{{ t('settings.members') }}</span>
            </button>
            <button type="button" :title="t('settings.notifications')" @click="scrollToSection('notifications')" :class="sectionButtonClass('notifications')">
              <span class="material-symbols-outlined text-[20px]" :class="activeSection === 'notifications' ? 'text-primary' : ''">notifications</span>
              <span class="text-[14px] font-medium hidden md:inline">{{ t('settings.notifications') }}</span>
            </button>
            <button type="button" :title="t('settings.keybinds')" @click="scrollToSection('keybinds')" :class="sectionButtonClass('keybinds')">
              <span class="material-symbols-outlined text-[20px]" :class="activeSection === 'keybinds' ? 'text-primary' : ''">keyboard_command_key</span>
              <span class="text-[14px] font-medium hidden md:inline">{{ t('settings.keybinds') }}</span>
            </button>
          </div>
        </div>

        <div class="pt-4 mt-4 border-t border-white/5">
          <button
            type="button"
            @click="emit('logout')"
            :title="t('settings.leaveTeam')"
            class="w-full text-left flex items-center gap-3 px-3 py-2.5 rounded-lg text-red-400/70 hover:text-red-400 hover:bg-red-500/10 transition-all group justify-center md:justify-start"
          >
            <span class="material-symbols-outlined text-[20px]">logout</span>
            <span class="text-[14px] font-medium hidden md:inline">{{ t('settings.leaveTeam') }}</span>
          </button>
        </div>
      </div>
    </aside>

    <main ref="contentRef" class="flex-1 flex flex-col bg-transparent relative min-w-0 overflow-y-auto custom-scrollbar p-12">
      <div class="max-w-3xl w-full mx-auto pb-20">
        <header class="mb-10">
          <h1 class="text-3xl font-bold text-white mb-2">{{ t('settings.preferences') }}</h1>
          <p class="text-white/40 text-sm">{{ t('settings.preferencesSubtitle') }}</p>
        </header>

        <section ref="accountRef" id="account" class="mb-12 scroll-mt-8">
          <h2 class="text-white/90 text-lg font-semibold mb-2 flex items-center gap-2">
            <span class="material-symbols-outlined text-primary">person</span>
            {{ t('settings.myAccount') }}
          </h2>
          <p class="text-white/40 text-sm">{{ t('settings.accountSubtitle') }}</p>

          <div class="bg-white/[0.03] border border-white/5 rounded-2xl p-6 mt-6">
            <div class="flex flex-col md:flex-row gap-6">
              <div class="w-16 h-16 rounded-2xl bg-white/5 border border-white/10 flex items-center justify-center text-white shadow-lg">
                <span class="text-lg font-semibold">{{ accountInitials }}</span>
              </div>
              <div class="flex-1 grid grid-cols-1 md:grid-cols-2 gap-4">
                <div class="space-y-2">
                  <label class="text-[11px] font-bold text-white/40 uppercase tracking-wider block">{{ t('settings.displayName') }}</label>
                  <input
                    v-model="draftSettings.account.displayName"
                    :class="inputClass"
                    :placeholder="t('settings.displayNamePlaceholder')"
                    type="text"
                    @blur="trimAccountField('displayName')"
                  />
                </div>
                <div class="space-y-2">
                  <label class="text-[11px] font-bold text-white/40 uppercase tracking-wider block">{{ t('settings.emailAddress') }}</label>
                  <input
                    v-model="draftSettings.account.email"
                    :class="inputClass"
                    :placeholder="t('settings.emailPlaceholder')"
                    type="email"
                    @blur="trimAccountField('email')"
                  />
                </div>
                <div class="space-y-2">
                  <label class="text-[11px] font-bold text-white/40 uppercase tracking-wider block">{{ t('settings.jobTitle') }}</label>
                  <input
                    v-model="draftSettings.account.title"
                    :class="inputClass"
                    :placeholder="t('settings.jobTitlePlaceholder')"
                    type="text"
                    @blur="trimAccountField('title')"
                  />
                </div>
                <div class="space-y-2">
                  <label class="text-[11px] font-bold text-white/40 uppercase tracking-wider block">{{ t('settings.timeZone') }}</label>
                  <select v-model="draftSettings.account.timezone" :class="selectClass">
                    <option v-for="zone in timeZones" :key="zone" :value="zone">
                      {{ zone }}
                    </option>
                  </select>
                </div>
              </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-5">
              <div class="space-y-2">
                <label class="text-[11px] font-bold text-white/40 uppercase tracking-wider block">{{ t('settings.status') }}</label>
                <select v-model="draftSettings.account.status" :class="selectClass">
                  <option v-for="status in statusOptions" :key="status.id" :value="status.id">
                    {{ t(status.labelKey) }}
                  </option>
                </select>
              </div>
              <div class="space-y-2">
                <label class="text-[11px] font-bold text-white/40 uppercase tracking-wider block">{{ t('settings.statusMessage') }}</label>
                <input
                  v-model="draftSettings.account.statusMessage"
                  :class="inputClass"
                  :placeholder="t('settings.statusMessagePlaceholder')"
                  type="text"
                  @blur="trimAccountField('statusMessage')"
                />
              </div>
            </div>
          </div>

          <div class="mt-6 flex justify-end">
            <button
              type="button"
              class="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-red-400/80 hover:text-red-400 hover:bg-red-500/10 transition-all"
            >
              <span class="material-symbols-outlined text-[18px]">logout</span>
              <span class="text-[14px] font-semibold">{{ t('settings.logOut') }}</span>
            </button>
          </div>
        </section>

        <div class="w-full h-[1px] bg-gradient-to-r from-transparent via-white/10 to-transparent mb-12"></div>

        <section ref="appearanceRef" id="appearance" class="mb-12 scroll-mt-8">
          <h2 class="text-white/90 text-lg font-semibold mb-2 flex items-center gap-2">
            <span class="material-symbols-outlined text-primary">palette</span>
            {{ t('settings.appearance') }}
          </h2>
          <p class="text-white/40 text-sm">{{ t('settings.appearanceSubtitle') }}</p>

          <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mt-6">
            <button
              v-for="option in themeOptions"
              :key="option.id"
              type="button"
              @click="setTheme(option.id)"
              :class="[
                'text-left rounded-2xl border p-4 transition-all group relative',
                option.id === theme
                  ? 'border-primary/40 bg-primary/[0.08] shadow-[0_0_20px_rgb(var(--color-primary)_/_0.18)]'
                  : 'border-white/5 bg-white/[0.02] hover:bg-white/[0.05] hover:border-white/10'
              ]"
            >
              <div class="flex items-center justify-between">
                <span class="text-[13px] font-semibold text-white">{{ t(option.labelKey) }}</span>
                <span
                  v-if="option.id === theme"
                  class="w-6 h-6 rounded-full bg-primary flex items-center justify-center shadow-lg shadow-primary/20"
                >
                  <span class="material-symbols-outlined text-on-primary text-[14px] font-bold">check</span>
                </span>
              </div>
              <div class="flex items-center gap-2 mt-3">
                <div class="h-8 w-8 rounded-lg border border-white/10" :style="{ background: themePreview[option.id].base }"></div>
                <div class="h-8 flex-1 rounded-lg border border-white/10" :style="{ background: themePreview[option.id].panel }"></div>
                <div class="h-8 w-10 rounded-lg border border-white/10" :style="{ background: themePreview[option.id].accent }"></div>
              </div>
              <p class="text-[12px] text-white/40 mt-3 leading-relaxed">{{ t(option.descriptionKey) }}</p>
            </button>
          </div>
        </section>

        <div class="w-full h-[1px] bg-gradient-to-r from-transparent via-white/10 to-transparent mb-12"></div>
        <section ref="languageRef" id="language" class="mb-12 scroll-mt-8">
          <h2 class="text-white/90 text-lg font-semibold mb-5 flex items-center gap-2">
            <span class="material-symbols-outlined text-primary">translate</span>
            {{ t('settings.language') }}
          </h2>
          <div class="bg-white/[0.03] border border-white/5 rounded-2xl overflow-hidden">
            <template v-for="(option, index) in localeOptions" :key="option.id">
              <button
                class="w-full flex items-center justify-between p-4 transition-colors text-left group"
                :class="option.id === locale ? 'bg-primary/[0.08] border-l-[3px] border-primary cursor-default' : 'hover:bg-white/[0.05]'"
                @click="setLocale(option.id)"
                type="button"
              >
                <div class="flex items-center gap-4">
                  <span class="text-2xl">{{ option.flag }}</span>
                  <div class="flex flex-col">
                    <span class="text-[15px] font-medium transition-colors" :class="option.id === locale ? 'text-white' : 'text-white/70 group-hover:text-white'">
                      {{ t(option.labelKey) }}
                    </span>
                    <span v-if="option.id === locale" class="text-white/40 text-xs">{{ t('settings.languageDefault') }}</span>
                  </div>
                </div>
                <div v-if="option.id === locale" class="w-8 h-8 rounded-full bg-primary flex items-center justify-center shadow-lg shadow-primary/20">
                  <span class="material-symbols-outlined text-on-primary text-sm font-bold">check</span>
                </div>
              </button>
              <div v-if="index < localeOptions.length - 1" class="w-full h-[1px] bg-white/5"></div>
            </template>
          </div>
          <div class="mt-4 flex items-center justify-between px-2">
            <span class="text-[13px] text-white/30">{{ t('settings.changesApply') }}</span>
            <div class="flex items-center gap-3">
              <span class="text-[14px] font-medium text-white/70">{{ t('settings.spellCheck') }}</span>
              <div class="relative inline-block w-10 align-middle select-none">
                <input
                  id="spellcheck"
                  v-model="draftSettings.language.spellCheck"
                  :class="toggleInputClass"
                  type="checkbox"
                />
                <label class="block overflow-hidden h-6 rounded-full bg-panel-strong/80 peer-checked:bg-primary cursor-pointer transition-colors" for="spellcheck"></label>
              </div>
            </div>
          </div>
        </section>

        <div class="w-full h-[1px] bg-gradient-to-r from-transparent via-white/10 to-transparent mb-12"></div>

        <section ref="membersRef" id="members" class="mb-12 scroll-mt-8">
          <h2 class="text-white/90 text-lg font-semibold mb-5 flex items-center gap-2">
            <span class="material-symbols-outlined text-primary">groups</span>
            {{ t('settings.defaultMember') }}
          </h2>
          <div class="bg-white/[0.03] border border-white/5 rounded-2xl p-6">
            <div class="flex justify-between items-center mb-4">
              <p class="text-[13px] font-medium text-white/60 uppercase tracking-wider">{{ t('settings.selectMember') }}</p>
              <button type="button" class="text-primary hover:text-primary-hover text-[13px] font-medium transition-colors" @click="resetMemberDraft">
                {{ t('settings.refreshList') }}
              </button>
            </div>

            <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6" @click="openMemberMenuId = null">
              <div v-for="member in memberOptions" :key="member.id" class="relative group cursor-pointer" @click="selectMemberCard(member.id)">
                <div
                  :class="[
                    'cursor-pointer block h-full rounded-xl border p-4 flex flex-col items-center gap-2 transition-all',
                    selectedMember === member.id
                      ? 'bg-primary/[0.08] border-primary/50 shadow-[0_0_20px_rgb(var(--color-primary)_/_0.12)]'
                      : 'border-white/10 bg-white/[0.02] hover:bg-white/5 hover:border-white/20'
                  ]"
                >
                  <div
                    :class="[
                      'w-12 h-12 rounded-xl border border-white/10 flex items-center justify-center text-white shadow-lg',
                      member.isCustom ? 'bg-white/5 group-hover:bg-white/10' : `bg-gradient-to-tr ${member.gradient}`
                    ]"
                  >
                    <span class="material-symbols-outlined text-[28px]">{{ member.icon }}</span>
                  </div>
                  <div class="text-center">
                    <div class="text-white font-semibold text-sm">{{ member.label }}</div>
                    <div v-if="member.command" class="text-[11px] text-white/40 mt-1 truncate max-w-[120px]">{{ member.command }}</div>
                    <div class="text-[10px] uppercase tracking-widest text-white/35 mt-1">{{ member.kindLabel }}</div>
                  </div>
                  <div v-if="selectedMember === member.id" class="absolute top-2 right-9 animate-in fade-in zoom-in duration-200">
                    <div class="w-5 h-5 rounded-full bg-primary flex items-center justify-center shadow-md">
                      <span class="material-symbols-outlined text-on-primary text-[14px] font-bold">check</span>
                    </div>
                  </div>
                </div>
                <div class="absolute top-2 right-2">
                  <button
                    type="button"
                    class="w-6 h-6 rounded-full bg-white/10 text-white/70 hover:text-white hover:bg-white/20 flex items-center justify-center transition-colors"
                    :aria-label="t('settings.memberActions.menuLabel')"
                    @click.stop="toggleMemberMenu(member.id)"
                  >
                    <span class="material-symbols-outlined text-[16px]">more_vert</span>
                  </button>
                  <div
                    v-if="openMemberMenuId === member.id"
                    class="absolute right-0 mt-2 w-36 rounded-xl border border-white/10 bg-panel/95 backdrop-blur-xl shadow-xl overflow-hidden z-10"
                    @click.stop
                  >
                    <button
                      type="button"
                      class="w-full text-left px-3 py-2 text-[12px] text-white/80 hover:text-white hover:bg-white/10 transition-colors"
                      @click="handleMemberTest(member.id)"
                    >
                      {{ t('settings.memberActions.test') }}
                    </button>
                    <button
                      v-if="member.isDeletable"
                      type="button"
                      class="w-full text-left px-3 py-2 text-[12px] text-white/80 hover:text-white hover:bg-white/10 transition-colors"
                      @click="startEditMember(member.id)"
                    >
                      {{ t('settings.memberActions.edit') }}
                    </button>
                    <button
                      v-if="member.isDeletable"
                      type="button"
                      class="w-full text-left px-3 py-2 text-[12px] text-red-300/80 hover:text-red-300 hover:bg-red-500/10 transition-colors"
                      @click="removeCustomMember(member.id)"
                    >
                      {{ t('settings.memberActions.remove') }}
                    </button>
                  </div>
                </div>
              </div>
              <div class="relative group cursor-pointer" @click="openCustomMemberForm">
                <div class="cursor-pointer block h-full rounded-xl border border-dashed border-white/20 p-4 flex flex-col items-center gap-2 transition-all bg-white/[0.02] hover:bg-white/5 hover:border-white/30">
                  <div class="w-12 h-12 rounded-xl border border-white/10 flex items-center justify-center text-white/60 shadow-lg bg-white/5 group-hover:bg-white/10">
                    <span class="material-symbols-outlined text-[28px]">add</span>
                  </div>
                  <div class="text-center">
                    <div class="text-white/70 font-semibold text-sm">{{ t('settings.memberOptions.custom') }}</div>
                  </div>
                </div>
              </div>
            </div>

            <div v-if="isAddingCustom" class="animate-in fade-in slide-in-from-top-2 duration-300">
              <div class="bg-white/[0.02] border border-white/10 rounded-xl p-5 relative">
                <div class="grid grid-cols-1 gap-4">
                  <div class="space-y-2">
                    <label class="text-[11px] font-bold text-white/40 uppercase tracking-wider block">{{ t('settings.memberName') }}</label>
                    <input
                      v-model="customName"
                      :class="inputClass"
                      :placeholder="t('settings.memberNamePlaceholder')"
                      type="text"
                    />
                  </div>
                  <div class="space-y-2">
                    <label class="text-[11px] font-bold text-white/40 uppercase tracking-wider block">{{ t('settings.commandInput') }}</label>
                    <div class="relative">
                      <span class="absolute left-3 top-1/2 -translate-y-1/2 text-white/30 font-mono text-sm">$</span>
                      <input
                        v-model="customCommand"
                        class="block w-full pl-7 pr-4 py-2.5 bg-surface/80 border border-white/10 rounded-lg text-white placeholder-white/20 focus:outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/50 transition-all font-mono text-sm"
                        :placeholder="t('settings.commandPlaceholder')"
                        type="text"
                      />
                    </div>
                  </div>
                </div>
                <div class="flex justify-end gap-2 mt-4">
                  <button
                    type="button"
                    class="w-8 h-8 rounded-lg bg-red-500/10 hover:bg-red-500/20 text-red-500 flex items-center justify-center transition-colors"
                    :title="t('settings.cancel')"
                    @click="resetCustomMemberForm"
                  >
                    <span class="material-symbols-outlined text-sm font-bold">close</span>
                  </button>
                  <button
                    type="button"
                    class="w-8 h-8 rounded-lg bg-emerald-500/10 hover:bg-emerald-500/20 text-emerald-500 flex items-center justify-center transition-colors"
                    :title="t('settings.confirm')"
                    @click="applyCustomMember"
                  >
                    <span class="material-symbols-outlined text-sm font-bold">check</span>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </section>

        <div class="w-full h-[1px] bg-gradient-to-r from-transparent via-white/10 to-transparent mb-12"></div>
        <section ref="notificationsRef" id="notifications" class="mb-12 scroll-mt-8">
          <h2 class="text-white/90 text-lg font-semibold mb-2 flex items-center gap-2">
            <span class="material-symbols-outlined text-primary">notifications</span>
            {{ t('settings.notifications') }}
          </h2>
          <p class="text-white/40 text-sm">{{ t('settings.notificationsSubtitle') }}</p>

          <div class="bg-white/[0.03] border border-white/5 rounded-2xl mt-6 overflow-hidden">
            <div v-for="(option, index) in notificationOptions" :key="option.key">
              <div class="flex items-center justify-between gap-6 px-6 py-4">
                <div>
                  <div class="text-sm font-semibold text-white">{{ t(option.labelKey) }}</div>
                  <div class="text-xs text-white/40 mt-1">{{ t(option.descriptionKey) }}</div>
                </div>
                <div class="relative inline-block w-10 align-middle select-none">
                  <input
                    :id="`notification-${option.key}`"
                    v-model="draftSettings.notifications[option.key]"
                    :class="toggleInputClass"
                    type="checkbox"
                  />
                  <label
                    class="block overflow-hidden h-6 rounded-full bg-panel-strong/80 peer-checked:bg-primary cursor-pointer transition-colors"
                    :for="`notification-${option.key}`"
                  ></label>
                </div>
              </div>
              <div v-if="index < notificationOptions.length - 1" class="w-full h-[1px] bg-white/5"></div>
            </div>

            <div v-if="draftSettings.notifications.quietHoursEnabled" class="px-6 pb-5">
              <div class="flex flex-col md:flex-row gap-4 mt-1">
                <div class="flex-1 space-y-2">
                  <label class="text-[11px] font-bold text-white/40 uppercase tracking-wider block">{{ t('settings.quietHoursFrom') }}</label>
                  <input v-model="draftSettings.notifications.quietHoursStart" :class="inputClass" type="time" />
                </div>
                <div class="flex-1 space-y-2">
                  <label class="text-[11px] font-bold text-white/40 uppercase tracking-wider block">{{ t('settings.quietHoursTo') }}</label>
                  <input v-model="draftSettings.notifications.quietHoursEnd" :class="inputClass" type="time" />
                </div>
              </div>
            </div>
          </div>
        </section>

        <div class="w-full h-[1px] bg-gradient-to-r from-transparent via-white/10 to-transparent mb-12"></div>

        <section ref="keybindsRef" id="keybinds" class="scroll-mt-8">
          <h2 class="text-white/90 text-lg font-semibold mb-2 flex items-center gap-2">
            <span class="material-symbols-outlined text-primary">keyboard_command_key</span>
            {{ t('settings.keybinds') }}
          </h2>
          <p class="text-white/40 text-sm">{{ t('settings.keybindsSubtitle') }}</p>

          <div class="bg-white/[0.03] border border-white/5 rounded-2xl mt-6 p-6 space-y-6">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="space-y-2">
                <label class="text-[11px] font-bold text-white/40 uppercase tracking-wider block">{{ t('settings.keybindsProfile') }}</label>
                <select v-model="draftSettings.keybinds.profile" :class="selectClass">
                  <option v-for="profile in keybindProfiles" :key="profile.id" :value="profile.id">
                    {{ t(profile.labelKey) }}
                  </option>
                </select>
              </div>
              <div class="flex items-end justify-between gap-4">
                <div class="flex items-center gap-3">
                  <span class="text-[14px] font-medium text-white/70">{{ t('settings.keybindsEnable') }}</span>
                  <div class="relative inline-block w-10 align-middle select-none">
                    <input id="keybinds-enabled" v-model="draftSettings.keybinds.enabled" :class="toggleInputClass" type="checkbox" />
                    <label class="block overflow-hidden h-6 rounded-full bg-panel-strong/80 peer-checked:bg-primary cursor-pointer transition-colors" for="keybinds-enabled"></label>
                  </div>
                </div>
                <div class="flex items-center gap-3">
                  <span class="text-[14px] font-medium text-white/70">{{ t('settings.keybindsHints') }}</span>
                  <div class="relative inline-block w-10 align-middle select-none">
                    <input id="keybinds-hints" v-model="draftSettings.keybinds.showHints" :class="toggleInputClass" type="checkbox" />
                    <label class="block overflow-hidden h-6 rounded-full bg-panel-strong/80 peer-checked:bg-primary cursor-pointer transition-colors" for="keybinds-hints"></label>
                  </div>
                </div>
              </div>
            </div>

            <div>
              <div class="flex items-center justify-between mb-3">
                <span class="text-[12px] font-bold text-white/40 uppercase tracking-wider">{{ t('settings.keybindsListTitle') }}</span>
                <button type="button" class="text-primary hover:text-primary-hover text-[13px] font-medium transition-colors" @click="resetKeybinds">
                  {{ t('settings.keybindsReset') }}
                </button>
              </div>
              <div class="space-y-2">
                <div
                  v-for="binding in activeKeybindProfile.bindings"
                  :key="binding.actionKey"
                  class="flex items-center justify-between px-4 py-2 rounded-lg bg-white/[0.02] border border-white/5"
                >
                  <span class="text-sm text-white/70">{{ t(binding.actionKey) }}</span>
                  <span class="text-xs font-mono text-white/70 bg-white/5 px-2 py-1 rounded-md border border-white/10">{{ binding.keys }}</span>
                </div>
              </div>
            </div>
          </div>
        </section>

        <div class="mt-8 flex justify-end gap-3 pt-6 border-t border-white/5">
          <button
            type="button"
            @click="handleCancel"
            :disabled="!isDirty"
            :class="[
              'px-5 py-2.5 rounded-xl border border-white/10 text-white/70 hover:text-white hover:bg-white/5 hover:border-white/20 transition-all text-sm font-medium',
              !isDirty ? 'opacity-40 cursor-not-allowed hover:bg-transparent hover:text-white/70' : ''
            ]"
          >
            {{ t('settings.cancel') }}
          </button>
          <button
            type="button"
            @click="handleSave"
            :disabled="!isDirty"
            :class="[
              'px-6 py-2.5 rounded-xl bg-primary hover:bg-primary-hover text-on-primary font-bold shadow-glow transition-all active:scale-95 text-sm',
              !isDirty ? 'opacity-50 cursor-not-allowed hover:bg-primary' : ''
            ]"
          >
            {{ t('settings.saveChanges') }}
          </button>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useLocale } from '@/shared/composables/useLocale';
import { useTheme, type AppTheme } from '@/shared/composables/useTheme';
import {
  cloneSettings,
  useSettings,
  type AccountStatus,
  type KeybindProfile,
  type SettingsState
} from '@/shared/composables/useSettings';

type MemberOption = {
  id: string;
  nameKey: string;
  icon: string;
  gradient: string;
};

type MemberDisplayOption = {
  id: string;
  label: string;
  command: string;
  kindLabel: string;
  icon: string;
  gradient?: string;
  isCustom: boolean;
  isDeletable: boolean;
};

type NotificationToggleKey = 'desktop' | 'sound' | 'mentionsOnly' | 'previews' | 'weeklyDigest' | 'quietHoursEnabled';

const emit = defineEmits<{ (e: 'logout'): void }>();

type SectionId = 'account' | 'appearance' | 'language' | 'members' | 'notifications' | 'keybinds';

const { t } = useI18n();
const { locale, setLocale, localeOptions } = useLocale();
const { theme, setTheme, themeOptions } = useTheme();
const { settings, saveSettings } = useSettings();

const savedTheme = ref(theme.value);
const savedLocale = ref(locale.value);
const draftSettings = ref<SettingsState>(cloneSettings(settings.value));
const customName = ref('');
const customCommand = ref('');
const isAddingCustom = ref(false);
const openMemberMenuId = ref<string | null>(null);
const editingMemberId = ref<string | null>(null);

const inputClass =
  'block w-full px-4 py-2.5 bg-surface/80 border border-white/10 rounded-lg text-white placeholder-white/20 focus:outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/50 transition-all text-sm';
const selectClass = `${inputClass} appearance-none`;
const toggleInputClass =
  'peer absolute block w-5 h-5 rounded-full bg-white border-4 border-white/20 appearance-none cursor-pointer transition-all duration-300 top-[2px] checked:right-0 checked:border-primary';

const contentRef = ref<HTMLElement | null>(null);
const accountRef = ref<HTMLElement | null>(null);
const appearanceRef = ref<HTMLElement | null>(null);
const languageRef = ref<HTMLElement | null>(null);
const membersRef = ref<HTMLElement | null>(null);
const notificationsRef = ref<HTMLElement | null>(null);
const keybindsRef = ref<HTMLElement | null>(null);
const activeSection = ref<SectionId>('account');

const serializeSettings = (value: SettingsState) => JSON.stringify(value);
const isDirty = computed(
  () =>
    serializeSettings(draftSettings.value) !== serializeSettings(settings.value) ||
    theme.value !== savedTheme.value ||
    locale.value !== savedLocale.value
);

const applySettingsToDraft = (next: SettingsState) => {
  draftSettings.value = cloneSettings(next);
  customName.value = '';
  customCommand.value = '';
  isAddingCustom.value = false;
  openMemberMenuId.value = null;
  editingMemberId.value = null;
};

const handleSave = () => {
  const next = saveSettings(draftSettings.value);
  applySettingsToDraft(next);
  savedTheme.value = theme.value;
  savedLocale.value = locale.value;
};

const handleCancel = () => {
  applySettingsToDraft(settings.value);
  if (theme.value !== savedTheme.value) {
    setTheme(savedTheme.value);
  }
  if (locale.value !== savedLocale.value) {
    setLocale(savedLocale.value);
  }
};

watch(
  settings,
  (next) => {
    if (!isDirty.value) {
      applySettingsToDraft(next);
    }
  },
  { deep: true }
);

const selectedMember = computed({
  get: () => draftSettings.value.members.defaultMemberId,
  set: (value: string) => {
    draftSettings.value.members.defaultMemberId = value;
  }
});

const resetMemberDraft = () => {
  const next = cloneSettings(settings.value);
  draftSettings.value.members = next.members;
  customName.value = '';
  customCommand.value = '';
  isAddingCustom.value = false;
  openMemberMenuId.value = null;
  editingMemberId.value = null;
};

const buildCustomMemberId = () => `custom-cli-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;

const openCustomMemberForm = () => {
  editingMemberId.value = null;
  isAddingCustom.value = true;
  customName.value = '';
  customCommand.value = '';
};

const applyCustomMember = () => {
  const name = customName.value.trim();
  const command = customCommand.value.trim();
  if (!name && !command) {
    return;
  }
  if (editingMemberId.value) {
    draftSettings.value.members.customMembers = draftSettings.value.members.customMembers.map((member) =>
      member.id === editingMemberId.value ? { ...member, name, command } : member
    );
  } else {
    const existingIds = new Set(draftSettings.value.members.customMembers.map((member) => member.id));
    let id = buildCustomMemberId();
    while (existingIds.has(id)) {
      id = buildCustomMemberId();
    }
    draftSettings.value.members.customMembers = [...draftSettings.value.members.customMembers, { id, name, command }];
    selectedMember.value = id;
  }
  resetCustomMemberForm();
};

const resetCustomMemberForm = () => {
  customName.value = '';
  customCommand.value = '';
  isAddingCustom.value = false;
  editingMemberId.value = null;
};

const removeCustomMember = (id: string) => {
  draftSettings.value.members.customMembers = draftSettings.value.members.customMembers.filter((member) => member.id !== id);
  if (draftSettings.value.members.defaultMemberId === id) {
    draftSettings.value.members.defaultMemberId = baseMembers[0]?.id ?? 'gemini-cli';
  }
  if (editingMemberId.value === id) {
    resetCustomMemberForm();
  }
  if (openMemberMenuId.value === id) {
    openMemberMenuId.value = null;
  }
};

const selectMemberCard = (id: string) => {
  selectedMember.value = id;
  openMemberMenuId.value = null;
};

const toggleMemberMenu = (id: string) => {
  openMemberMenuId.value = openMemberMenuId.value === id ? null : id;
};

const handleMemberTest = (_id: string) => {
  openMemberMenuId.value = null;
};

const startEditMember = (id: string) => {
  const target = draftSettings.value.members.customMembers.find((member) => member.id === id);
  if (!target) {
    openMemberMenuId.value = null;
    return;
  }
  editingMemberId.value = id;
  isAddingCustom.value = true;
  customName.value = target.name;
  customCommand.value = target.command;
  openMemberMenuId.value = null;
};

const accountInitials = computed(() => {
  const name = draftSettings.value.account.displayName.trim();
  if (!name) return 'NA';
  const parts = name.split(' ').filter(Boolean);
  const initials = parts
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase() ?? '')
    .join('');
  return initials || 'NA';
});

const trimAccountField = (field: 'displayName' | 'email' | 'title' | 'statusMessage') => {
  const value = draftSettings.value.account[field].trim();
  draftSettings.value.account[field] = field === 'email' ? value.toLowerCase() : value;
};

const timeZones = ['UTC', 'America/Los_Angeles', 'America/New_York', 'Europe/London', 'Europe/Berlin', 'Asia/Shanghai', 'Asia/Tokyo', 'Australia/Sydney'];

const statusOptions: Array<{ id: AccountStatus; labelKey: string }> = [
  { id: 'online', labelKey: 'settings.statusOptions.online' },
  { id: 'away', labelKey: 'settings.statusOptions.away' },
  { id: 'dnd', labelKey: 'settings.statusOptions.dnd' }
];

const notificationOptions: Array<{ key: NotificationToggleKey; labelKey: string; descriptionKey: string }> = [
  { key: 'desktop', labelKey: 'settings.notificationOptions.desktop', descriptionKey: 'settings.notificationOptions.desktopDesc' },
  { key: 'sound', labelKey: 'settings.notificationOptions.sound', descriptionKey: 'settings.notificationOptions.soundDesc' },
  { key: 'mentionsOnly', labelKey: 'settings.notificationOptions.mentionsOnly', descriptionKey: 'settings.notificationOptions.mentionsOnlyDesc' },
  { key: 'previews', labelKey: 'settings.notificationOptions.previews', descriptionKey: 'settings.notificationOptions.previewsDesc' },
  { key: 'weeklyDigest', labelKey: 'settings.notificationOptions.weeklyDigest', descriptionKey: 'settings.notificationOptions.weeklyDigestDesc' },
  { key: 'quietHoursEnabled', labelKey: 'settings.notificationOptions.quietHours', descriptionKey: 'settings.notificationOptions.quietHoursDesc' }
];

const keybindProfiles: Array<{
  id: KeybindProfile;
  labelKey: string;
  bindings: Array<{ actionKey: string; keys: string }>;
}> = [
  {
    id: 'default',
    labelKey: 'settings.keybindProfiles.default',
    bindings: [
      { actionKey: 'settings.keybindActions.focusSearch', keys: 'Ctrl + K' },
      { actionKey: 'settings.keybindActions.newMessage', keys: 'Ctrl + Enter' },
      { actionKey: 'settings.keybindActions.toggleSidebar', keys: 'Ctrl + B' },
      { actionKey: 'settings.keybindActions.toggleMute', keys: 'Ctrl + Shift + M' },
      { actionKey: 'settings.keybindActions.jumpToLatest', keys: 'Alt + J' },
      { actionKey: 'settings.keybindActions.openSettings', keys: 'Ctrl + ,' }
    ]
  },
  {
    id: 'vscode',
    labelKey: 'settings.keybindProfiles.vscode',
    bindings: [
      { actionKey: 'settings.keybindActions.focusSearch', keys: 'Ctrl + P' },
      { actionKey: 'settings.keybindActions.newMessage', keys: 'Ctrl + Enter' },
      { actionKey: 'settings.keybindActions.toggleSidebar', keys: 'Ctrl + B' },
      { actionKey: 'settings.keybindActions.toggleMute', keys: 'Ctrl + Shift + M' },
      { actionKey: 'settings.keybindActions.jumpToLatest', keys: 'Alt + End' },
      { actionKey: 'settings.keybindActions.openSettings', keys: 'Ctrl + ,' }
    ]
  },
  {
    id: 'slack',
    labelKey: 'settings.keybindProfiles.slack',
    bindings: [
      { actionKey: 'settings.keybindActions.focusSearch', keys: 'Ctrl + K' },
      { actionKey: 'settings.keybindActions.newMessage', keys: 'Ctrl + N' },
      { actionKey: 'settings.keybindActions.toggleSidebar', keys: 'Ctrl + Shift + S' },
      { actionKey: 'settings.keybindActions.toggleMute', keys: 'Ctrl + Shift + M' },
      { actionKey: 'settings.keybindActions.jumpToLatest', keys: 'Alt + J' },
      { actionKey: 'settings.keybindActions.openSettings', keys: 'Ctrl + ,' }
    ]
  }
];

const activeKeybindProfile = computed(
  () => keybindProfiles.find((profile) => profile.id === draftSettings.value.keybinds.profile) ?? keybindProfiles[0]
);

const resetKeybinds = () => {
  draftSettings.value.keybinds = {
    enabled: true,
    showHints: true,
    profile: 'default'
  };
};

const baseMembers: MemberOption[] = [
  { id: 'gemini-cli', nameKey: 'settings.memberOptions.gemini', icon: 'token', gradient: 'from-blue-600 to-cyan-400' },
  { id: 'codex', nameKey: 'settings.memberOptions.codex', icon: 'code', gradient: 'from-emerald-600 to-green-400' },
  { id: 'claude-code', nameKey: 'settings.memberOptions.claude', icon: 'terminal', gradient: 'from-orange-600 to-amber-400' }
];

const memberOptions = computed<MemberDisplayOption[]>(() => {
  const baseOptions = baseMembers.map((member) => ({
    id: member.id,
    label: t(member.nameKey),
    command: '',
    kindLabel: t('settings.memberKind.default'),
    icon: member.icon,
    gradient: member.gradient,
    isCustom: false,
    isDeletable: false
  }));

  const customOptions = draftSettings.value.members.customMembers.map((member) => ({
    id: member.id,
    label: member.name || t('settings.memberOptions.custom'),
    command: member.command,
    kindLabel: t('settings.memberKind.custom'),
    icon: 'terminal',
    isCustom: true,
    isDeletable: true
  }));

  return [...baseOptions, ...customOptions];
});

const sectionButtonClass = (section: SectionId) => [
  'w-full text-left flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all group justify-center md:justify-start md:gap-3 gap-0',
  activeSection.value === section
    ? 'bg-primary/10 text-white border border-primary/20 shadow-[0_1px_2px_rgba(0,0,0,0.05)]'
    : 'text-white/60 hover:text-white hover:bg-white/5'
];

const themePreview: Record<AppTheme, { base: string; panel: string; accent: string }> = {
  dark: {
    base: '#0f0f12',
    panel: '#1c1d24',
    accent: '#38bdf8'
  },
  light: {
    base: '#f8fafd',
    panel: '#f0f4f9',
    accent: '#0b57d0'
  },
  system: {
    base: 'linear-gradient(135deg, #0f0f12 0%, #0f0f12 50%, #f8fafd 50%, #f8fafd 100%)',
    panel: 'linear-gradient(135deg, #1c1d24 0%, #1c1d24 50%, #f0f4f9 50%, #f0f4f9 100%)',
    accent: 'linear-gradient(135deg, #38bdf8 0%, #0b57d0 100%)'
  }
};

const scrollToSection = (section: SectionId) => {
  const sectionMap: Record<SectionId, HTMLElement | null> = {
    account: accountRef.value,
    appearance: appearanceRef.value,
    language: languageRef.value,
    members: membersRef.value,
    notifications: notificationsRef.value,
    keybinds: keybindsRef.value
  };
  activeSection.value = section;
  sectionMap[section]?.scrollIntoView({ behavior: 'smooth', block: 'start' });
};

let observer: IntersectionObserver | null = null;

onMounted(() => {
  if (!contentRef.value) return;
  const sections: Array<{ id: SectionId; ref: typeof accountRef }> = [
    { id: 'account', ref: accountRef },
    { id: 'appearance', ref: appearanceRef },
    { id: 'language', ref: languageRef },
    { id: 'members', ref: membersRef },
    { id: 'notifications', ref: notificationsRef },
    { id: 'keybinds', ref: keybindsRef }
  ];

  observer = new IntersectionObserver(
    (entries) => {
      const visible = entries
        .filter((entry) => entry.isIntersecting)
        .sort((a, b) => b.intersectionRatio - a.intersectionRatio);

      const nextSection = visible[0]?.target.getAttribute('data-section-id') as SectionId | null;
      if (nextSection) {
        activeSection.value = nextSection;
      }
    },
    {
      root: contentRef.value,
      threshold: [0.25, 0.6, 0.9],
      rootMargin: '0px 0px -55% 0px'
    }
  );

  sections.forEach(({ id, ref }) => {
    if (ref.value) {
      ref.value.dataset.sectionId = id;
      observer?.observe(ref.value);
    }
  });
});

onBeforeUnmount(() => {
  observer?.disconnect();
});
</script>
