<template>
  <div class="flex h-full w-full overflow-hidden">
    <aside class="w-16 md:w-[280px] bg-glass-sidebar glass-panel shrink-0 py-4 md:py-8 px-2 md:px-3 flex flex-col">
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
            <button type="button" :title="t('settings.data')" @click="scrollToSection('data')" :class="sectionButtonClass('data')">
              <span class="material-symbols-outlined text-[20px]" :class="activeSection === 'data' ? 'text-primary' : ''">database</span>
              <span class="text-[14px] font-medium hidden md:inline">{{ t('settings.data') }}</span>
            </button>
          </div>
        </div>

        <div class="pt-4 mt-4">
          <div class="space-y-2">
            <button
              type="button"
              @click="handleCreateTeam"
              :title="t('settings.createTeam')"
              class="w-full text-left flex items-center gap-3 px-3 py-2.5 rounded-lg text-white/70 hover:text-white hover:bg-white/5 transition-all group justify-center md:justify-start"
            >
              <span class="material-symbols-outlined text-[20px]">group_add</span>
              <span class="text-[14px] font-medium hidden md:inline">{{ t('settings.createTeam') }}</span>
            </button>
            <button
              type="button"
              @click="handleLeaveWorkspace"
              :title="t('settings.leaveTeam')"
              class="w-full text-left flex items-center gap-3 px-3 py-2.5 rounded-lg text-red-400/70 hover:text-red-400 hover:bg-red-500/10 transition-all group justify-center md:justify-start"
            >
              <span class="material-symbols-outlined text-[20px]">logout</span>
              <span class="text-[14px] font-medium hidden md:inline">{{ t('settings.leaveTeam') }}</span>
            </button>
          </div>
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
              <div class="relative">
                <button
                  ref="avatarButtonRef"
                  type="button"
                  class="group relative w-16 h-16 rounded-2xl border border-white/10 shadow-lg overflow-hidden"
                  @click="toggleAvatarMenu($event)"
                >
                  <AvatarBadge
                    :avatar="accountAvatar"
                    :label="t('common.userAvatarAlt')"
                    class="w-full h-full rounded-2xl"
                  />
                  <div class="absolute inset-0 bg-black/45 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                    <span class="material-symbols-outlined text-white text-[18px]">edit</span>
                  </div>
                </button>
                <div
                  v-if="avatarMenuOpen"
                  ref="avatarMenuRef"
                  class="fixed w-80 rounded-2xl border border-white/10 bg-panel/95 backdrop-blur-xl shadow-2xl overflow-hidden z-20"
                  :style="avatarMenuStyle"
                >
                  <div class="px-4 pt-3">
                    <div class="text-[11px] font-bold uppercase tracking-widest text-white/40">{{ t('settings.avatarTitle') }}</div>
                    <div class="text-[12px] text-white/50 mt-1">{{ t('settings.avatarSubtitle') }}</div>
                  </div>
                  <div class="grid grid-cols-5 gap-3 px-4 py-4">
                    <button
                      v-for="preset in avatarPresets"
                      :key="preset.id"
                      type="button"
                      class="relative group"
                      @click="selectAvatarPreset(preset.id)"
                    >
                      <AvatarBadge
                        :avatar="`css:${preset.id}`"
                        :label="t(preset.labelKey)"
                        class="w-11 h-11 rounded-xl border border-white/10 group-hover:border-primary/50 transition-colors"
                      />
                      <span class="sr-only">{{ t(preset.labelKey) }}</span>
                      <span
                        v-if="selectedAvatarId === preset.id"
                        class="absolute -top-1 -right-1 w-5 h-5 rounded-full bg-primary text-on-primary flex items-center justify-center shadow-lg"
                      >
                        <span class="material-symbols-outlined text-[12px]">check</span>
                      </span>
                    </button>
                  </div>
                  <div v-if="customAvatars.length" class="px-4 pb-4">
                    <div class="text-[10px] font-semibold uppercase tracking-widest text-white/40">
                      {{ t('settings.avatarUploads') }}
                    </div>
                    <div class="grid grid-cols-5 gap-3 mt-3">
                      <div v-for="asset in customAvatars" :key="asset.id" class="relative group">
                        <button type="button" class="relative" @click="selectCustomAvatar(asset.id)">
                          <AvatarBadge
                            :avatar="toLocalAvatar(asset.id)"
                            :label="t('settings.avatarUploads')"
                            class="w-11 h-11 rounded-xl border border-white/10 group-hover:border-primary/50 transition-colors"
                          />
                          <span
                            v-if="selectedLocalAvatarId === asset.id"
                            class="absolute -top-1 -right-1 w-5 h-5 rounded-full bg-primary text-on-primary flex items-center justify-center shadow-lg"
                          >
                            <span class="material-symbols-outlined text-[12px]">check</span>
                          </span>
                        </button>
                        <button
                          type="button"
                          class="absolute bottom-1 right-1 z-10 w-5 h-5 rounded-full bg-black/70 text-white/70 hover:text-white hover:bg-red-500/80 border border-white/10 opacity-0 group-hover:opacity-100 transition-opacity"
                          :aria-label="t('common.remove')"
                          @click.stop="removeCustomAvatar(asset.id)"
                        >
                          <span class="material-symbols-outlined text-[12px]">close</span>
                        </button>
                      </div>
                    </div>
                  </div>
                  <div class="border-t border-white/10 px-4 py-3 flex items-center gap-2">
                    <button
                      type="button"
                      class="flex-1 inline-flex items-center justify-center gap-2 px-3 py-2 rounded-lg border border-white/10 bg-white/5 text-[12px] font-semibold text-white/70 hover:text-white hover:border-primary/40 hover:bg-white/10 transition-colors"
                      @click="triggerAvatarUpload"
                    >
                      <span class="material-symbols-outlined text-[18px]">upload</span>
                      {{ t('settings.avatarUpload') }}
                    </button>
                    <button
                      v-if="isCustomAvatar"
                      type="button"
                      class="inline-flex items-center justify-center gap-2 px-3 py-2 rounded-lg border border-white/10 bg-white/5 text-[12px] font-semibold text-white/60 hover:text-white hover:border-primary/40 hover:bg-white/10 transition-colors"
                      @click="resetAvatar"
                    >
                      <span class="material-symbols-outlined text-[18px]">restart_alt</span>
                      {{ t('settings.avatarReset') }}
                    </button>
                    <input ref="avatarInputRef" type="file" class="hidden" accept="image/*" @change="handleAvatarUpload" />
                  </div>
                  <div v-if="avatarError" class="px-4 pb-2 text-[11px] text-red-300">{{ avatarError }}</div>
                  <div class="px-4 pb-3 text-[11px] text-white/40">{{ t('settings.avatarHint') }}</div>
                </div>
              </div>
              <div class="flex-1 grid grid-cols-1 md:grid-cols-2 gap-4">
                <div class="space-y-2">
                  <label class="text-[11px] font-bold text-white/40 uppercase tracking-wider block">{{ t('settings.displayName') }}</label>
                  <button
                    v-if="!isEditingDisplayName"
                    type="button"
                    class="group w-full flex items-center justify-between px-4 py-2.5 rounded-lg border border-white/10 bg-white/[0.03] text-[14px] font-medium text-white/60 transition-all hover:bg-white/[0.06] hover:border-primary/40 hover:text-white"
                    @click="startEditDisplayName"
                  >
                    <span :class="displayNameValue ? 'text-white' : 'text-white/40'">{{ displayNameLabel }}</span>
                    <span class="material-symbols-outlined text-[16px] text-white/20 group-hover:text-primary transition-colors">edit</span>
                  </button>
                  <input
                    v-else
                    ref="displayNameInputRef"
                    v-model="draftSettings.account.displayName"
                    :class="inputClass"
                    :placeholder="t('settings.displayNamePlaceholder')"
                    type="text"
                    @keydown.esc.prevent="cancelEditDisplayName"
                    @keydown.enter.prevent="commitDisplayName"
                    @blur="commitDisplayName"
                  />
                </div>
                <div class="space-y-2">
                  <label class="text-[11px] font-bold text-white/40 uppercase tracking-wider block">{{ t('settings.emailAddress') }}</label>
                  <input
                    v-model="draftSettings.account.email"
                    :class="[inputClass, 'cursor-not-allowed bg-white/[0.03] text-white/60']"
                    :placeholder="t('settings.emailPlaceholder', { at: '@' })"
                    type="email"
                    readonly
                    aria-readonly="true"
                  />
                  <button
                    type="button"
                    class="mt-2 inline-flex items-center gap-2 px-3 py-1.5 rounded-lg border border-white/10 bg-white/5 text-[12px] font-semibold text-white/60 hover:text-white hover:border-primary/40 hover:bg-white/10 transition-colors"
                    @click="handleChangeEmail"
                  >
                    <span class="material-symbols-outlined text-[16px]">mail</span>
                    {{ t('settings.changeEmail') }}
                  </button>
                </div>
                <div class="space-y-2 md:col-span-2">
                  <label class="text-[11px] font-bold text-white/40 uppercase tracking-wider block">{{ t('settings.timeZone') }}</label>
                  <select v-model="draftSettings.account.timezone" class="settings-select" :aria-label="t('settings.timeZone')">
                    <option v-for="zone in timeZones" :key="zone.id" :value="zone.id">
                      {{ t(zone.labelKey) }}
                    </option>
                  </select>
                </div>
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
                <div
                  :class="['h-8 w-8 rounded-lg', option.id === 'system' ? '' : 'border border-white/10']"
                  :style="{ background: themePreview[option.id].base }"
                ></div>
                <div
                  :class="['h-8 flex-1 rounded-lg', option.id === 'system' ? '' : 'border border-white/10']"
                  :style="{ background: themePreview[option.id].panel }"
                ></div>
                <div
                  :class="['h-8 w-10 rounded-lg', option.id === 'system' ? '' : 'border border-white/10']"
                  :style="{ background: themePreview[option.id].accent }"
                ></div>
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
          <div class="mt-4 px-2">
            <span class="text-[13px] text-white/30">{{ t('settings.changesApply') }}</span>
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
              <div v-for="member in memberOptions" :key="member.id" class="relative group cursor-pointer" @click="selectMemberCard(member)">
                <div
                  :class="[
                    'cursor-pointer block h-full rounded-xl border p-4 flex flex-col items-center gap-2 transition-all',
                    selectedMemberId === member.id
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
                    <div class="text-[10px] uppercase tracking-widest text-white/60 mt-1">{{ member.kindLabel }}</div>
                  </div>
                  <div v-if="selectedMemberId === member.id" class="absolute top-2 right-9 animate-in fade-in zoom-in duration-200">
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
                      @click="handleMemberTest"
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

        <div class="w-full h-[1px] bg-gradient-to-r from-transparent via-white/10 to-transparent my-12"></div>

        <section ref="dataRef" id="data" class="scroll-mt-8">
          <h2 class="text-white/90 text-lg font-semibold mb-2 flex items-center gap-2">
            <span class="material-symbols-outlined text-primary">database</span>
            {{ t('settings.data') }}
          </h2>
          <p class="text-white/40 text-sm">{{ t('settings.dataSubtitle') }}</p>

          <div class="bg-white/[0.03] border border-white/5 rounded-2xl mt-6 p-6 space-y-4">
            <div class="flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
              <div>
                <div class="text-[14px] font-semibold text-white/80">{{ t('settings.dataRepairTitle') }}</div>
                <div class="text-[12px] text-white/40 mt-1">{{ t('settings.dataRepairHint') }}</div>
              </div>
              <button
                type="button"
                class="inline-flex items-center justify-center gap-2 px-4 py-2 rounded-lg border border-white/10 bg-white/5 text-[12px] font-semibold text-white/70 hover:text-white hover:border-primary/40 hover:bg-white/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                :disabled="repairingChatDb || !currentWorkspace"
                @click="handleRepairChatDb"
              >
                <span class="material-symbols-outlined text-[16px]">build_circle</span>
                {{ t('settings.dataRepairAction') }}
              </button>
            </div>

            <div class="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 pt-2 border-t border-white/5">
              <div>
                <div class="text-[14px] font-semibold text-white/80">{{ t('settings.dataClearTitle') }}</div>
                <div class="text-[12px] text-white/40 mt-1">{{ t('settings.dataClearHint') }}</div>
              </div>
              <button
                type="button"
                class="inline-flex items-center justify-center gap-2 px-4 py-2 rounded-lg border border-red-500/30 bg-red-500/10 text-[12px] font-semibold text-red-200 hover:text-white hover:border-red-400/60 hover:bg-red-500/20 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                :disabled="clearingChatDb || !currentWorkspace"
                @click="handleClearChatDb"
              >
                <span class="material-symbols-outlined text-[16px]">delete_sweep</span>
                {{ t('settings.dataClearAction') }}
              </button>
            </div>

            <div v-if="dataActionMessage" class="text-[12px] text-white/50">
              {{ dataActionMessage }}
            </div>
          </div>
        </section>

      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useI18n } from 'vue-i18n';
import { isTauri } from '@tauri-apps/api/core';
import { ask } from '@tauri-apps/plugin-dialog';
import { useThemeStore, themeOptions, type AppTheme } from '@/features/global/themeStore';
import { openTerminalWindow } from '@/features/terminal/openTerminalWindow';
import { openWorkspaceSelectionWindow } from '@/shared/tauri/windows';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { useChatStore } from '@/features/chat/chatStore';
import { clearAllChatMessages, repairChatMessages } from '@/features/chat/chatBridge';
import AvatarBadge from '@/shared/components/AvatarBadge.vue';
import { AVATAR_PRESETS, DEFAULT_AVATAR_ID } from '@/shared/constants/avatars';
import {
  clearAvatarUrlCache,
  ensureAvatar,
  getCssAvatarId,
  getLocalAvatarId,
  isCssAvatar,
  isLocalAvatar,
  toCssAvatar,
  toLocalAvatar
} from '@/shared/utils/avatar';
import { BASE_TERMINALS, CUSTOM_TERMINAL_ICON } from '@/shared/constants/terminalCatalog';
import { TIME_ZONE_OPTIONS } from '@/shared/constants/timeZones';
import {
  deleteAvatarAsset,
  listAvatarAssets,
  storeAvatarAsset,
  type AvatarAsset
} from '@/shared/tauri/avatars';
import {
  DEFAULT_MEMBER_INDEX,
  clampMemberSelectionIndex,
  resolveMemberIdFromSelectionIndex,
  resolveMemberSelectionIndexFromId,
  type MemberSelectionIndex
} from '@/shared/utils/memberSelection';
import {
  cloneSettings,
  localeOptions,
  useSettingsStore,
  type KeybindProfile,
  type SettingsState
} from '@/features/global/settingsStore';
import { useProjectStore } from '@/features/workspace/projectStore';
import { CURRENT_USER_ID } from '@/features/chat/data';
import { generateUlid } from '@/features/chat/chatBridge';

type MemberDisplayOption = {
  id: string;
  label: string;
  command: string;
  kindLabel: string;
  icon: string;
  gradient?: string;
  isCustom: boolean;
  isDeletable: boolean;
  group: 0 | 1;
  index: number;
};

type NotificationToggleKey = 'desktop' | 'sound' | 'mentionsOnly' | 'previews' | 'weeklyDigest' | 'quietHoursEnabled';

const emit = defineEmits<{ (e: 'logout'): void }>();
const workspaceStore = useWorkspaceStore();
const { closeWorkspace } = workspaceStore;
const { currentWorkspace } = storeToRefs(workspaceStore);
const projectStore = useProjectStore();

type SectionId = 'account' | 'appearance' | 'language' | 'members' | 'notifications' | 'keybinds' | 'data';

const { t } = useI18n();
const settingsStore = useSettingsStore();
const { settings, locale } = storeToRefs(settingsStore);
const { saveSettings, setAccountDisplayName, setLocale } = settingsStore;
const themeStore = useThemeStore();
const { theme } = storeToRefs(themeStore);
const { setTheme } = themeStore;
const chatStore = useChatStore();
const { loadSession, reset: resetChat } = chatStore;
const { updateMember } = projectStore;

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
const dataRef = ref<HTMLElement | null>(null);
const sectionRefs: Array<{ id: SectionId; ref: typeof accountRef }> = [
  { id: 'account', ref: accountRef },
  { id: 'appearance', ref: appearanceRef },
  { id: 'language', ref: languageRef },
  { id: 'members', ref: membersRef },
  { id: 'notifications', ref: notificationsRef },
  { id: 'keybinds', ref: keybindsRef },
  { id: 'data', ref: dataRef }
];
const activeSection = ref<SectionId>('account');
const isAutoScrolling = ref(false);
const autoScrollTimeoutId = ref<number | null>(null);
const targetScrollTop = ref<number | null>(null);
const displayNameInputRef = ref<HTMLInputElement | null>(null);
const isEditingDisplayName = ref(false);
const displayNameValue = computed(() => draftSettings.value.account.displayName.trim());
const displayNameSnapshot = ref('');
const skipDisplayNameCommit = ref(false);
const DEFAULT_OWNER_NAME = 'Owner';
const displayNameLabel = computed(() => displayNameValue.value || DEFAULT_OWNER_NAME);
const avatarPresets = AVATAR_PRESETS;
const avatarMenuOpen = ref(false);
const avatarMenuRef = ref<HTMLElement | null>(null);
const avatarButtonRef = ref<HTMLElement | null>(null);
const avatarInputRef = ref<HTMLInputElement | null>(null);
const avatarError = ref<string | null>(null);
const avatarMenuPosition = ref({ left: 0, top: 0 });
const avatarMenuStyle = computed(() => ({
  left: `${avatarMenuPosition.value.left}px`,
  top: `${avatarMenuPosition.value.top}px`
}));
const lastCssAvatarId = ref(DEFAULT_AVATAR_ID);
const accountAvatar = computed(() => ensureAvatar(draftSettings.value.account.avatar));
const selectedAvatarId = computed(() => getCssAvatarId(accountAvatar.value));
const selectedLocalAvatarId = computed(() => getLocalAvatarId(accountAvatar.value));
const isCustomAvatar = computed(() => !isCssAvatar(accountAvatar.value));
const MAX_AVATAR_BYTES = 2 * 1024 * 1024;
const AVATAR_MENU_PADDING = 12;
const supportsAvatarStorage = isTauri();
const customAvatars = ref<AvatarAsset[]>([]);
const customAvatarsLoading = ref(false);
const repairingChatDb = ref(false);
const clearingChatDb = ref(false);
const dataActionMessage = ref<string | null>(null);

const serializeSettings = (value: SettingsState) => {
  const { account, ...rest } = value;
  const accountRest = { ...account };
  delete accountRest.status;
  return JSON.stringify({ ...rest, account: accountRest });
};
const syncDraftSettings = (next: SettingsState) => {
  draftSettings.value = cloneSettings(next);
};

const buildPersistableSettings = (value: SettingsState) => {
  const nextDraft = cloneSettings(value);
  nextDraft.account.status = settings.value.account.status;
  if (isEditingDisplayName.value) {
    nextDraft.account.displayName = settings.value.account.displayName;
  }
  return nextDraft;
};

const startEditDisplayName = async () => {
  displayNameSnapshot.value = draftSettings.value.account.displayName;
  skipDisplayNameCommit.value = false;
  isEditingDisplayName.value = true;
  await nextTick();
  displayNameInputRef.value?.focus();
  displayNameInputRef.value?.select();
};

const commitDisplayName = () => {
  if (skipDisplayNameCommit.value) {
    skipDisplayNameCommit.value = false;
    return;
  }
  const nextName = draftSettings.value.account.displayName.trim();
  if (!nextName) {
    draftSettings.value.account.displayName = displayNameSnapshot.value;
    isEditingDisplayName.value = false;
    return;
  }
  const next = setAccountDisplayName(nextName);
  draftSettings.value.account.displayName = next.account.displayName;
  displayNameSnapshot.value = next.account.displayName;
  isEditingDisplayName.value = false;
};

const cancelEditDisplayName = () => {
  skipDisplayNameCommit.value = true;
  draftSettings.value.account.displayName = displayNameSnapshot.value;
  isEditingDisplayName.value = false;
};

const clampAvatarMenu = () => {
  if (!avatarMenuRef.value) return;
  const rect = avatarMenuRef.value.getBoundingClientRect();
  const maxLeft = window.innerWidth - rect.width - AVATAR_MENU_PADDING;
  const maxTop = window.innerHeight - rect.height - AVATAR_MENU_PADDING;
  avatarMenuPosition.value = {
    left: Math.max(AVATAR_MENU_PADDING, Math.min(avatarMenuPosition.value.left, maxLeft)),
    top: Math.max(AVATAR_MENU_PADDING, Math.min(avatarMenuPosition.value.top, maxTop))
  };
};

const positionAvatarMenu = (event?: MouseEvent) => {
  const button = avatarButtonRef.value;
  const clickLeft = event?.clientX;
  const clickTop = event?.clientY;
  if (typeof clickLeft === 'number' && typeof clickTop === 'number') {
    avatarMenuPosition.value = { left: clickLeft, top: clickTop };
  } else if (button) {
    const rect = button.getBoundingClientRect();
    avatarMenuPosition.value = { left: rect.left, top: rect.bottom + 8 };
  }
  nextTick(() => {
    clampAvatarMenu();
  });
};

const toggleAvatarMenu = (event?: MouseEvent) => {
  if (avatarMenuOpen.value) {
    avatarMenuOpen.value = false;
    return;
  }
  avatarError.value = null;
  avatarMenuOpen.value = true;
  positionAvatarMenu(event);
  void loadCustomAvatars();
};

const selectAvatarPreset = (id: string) => {
  draftSettings.value.account.avatar = toCssAvatar(id);
  avatarError.value = null;
  avatarMenuOpen.value = false;
};

const triggerAvatarUpload = () => {
  avatarError.value = null;
  avatarInputRef.value?.click();
};

const AVATAR_EXTENSION_BY_MIME: Record<string, string> = {
  'image/png': 'png',
  'image/jpeg': 'jpg',
  'image/jpg': 'jpg',
  'image/webp': 'webp',
  'image/gif': 'gif'
};

const resolveImageExtension = (file: File) => {
  if (file.type && AVATAR_EXTENSION_BY_MIME[file.type]) {
    return AVATAR_EXTENSION_BY_MIME[file.type];
  }
  const match = file.name.toLowerCase().match(/\.([a-z0-9]+)$/);
  if (match?.[1]) {
    return match[1];
  }
  return 'png';
};

const resolveMimeExtension = (mime: string) => AVATAR_EXTENSION_BY_MIME[mime.toLowerCase()] ?? 'png';

const fileToBytes = async (file: File) => {
  const buffer = await file.arrayBuffer();
  return Array.from(new Uint8Array(buffer));
};

const dataUrlToBytes = (dataUrl: string) => {
  const [header, payload] = dataUrl.split(',');
  if (!header || !payload) {
    throw new Error('Invalid data URL');
  }
  const match = header.match(/data:([^;]+);base64/i);
  const mime = match?.[1] ?? '';
  const binary = window.atob(payload);
  const bytes = new Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return { bytes, extension: resolveMimeExtension(mime) };
};

const upsertCustomAvatar = (asset: AvatarAsset) => {
  const existing = customAvatars.value.find((item) => item.id === asset.id);
  if (existing) {
    customAvatars.value = customAvatars.value.map((item) => (item.id === asset.id ? asset : item));
    return;
  }
  customAvatars.value = [asset, ...customAvatars.value];
};

const loadCustomAvatars = async () => {
  if (!supportsAvatarStorage || customAvatarsLoading.value) {
    return;
  }
  customAvatarsLoading.value = true;
  try {
    const assets = await listAvatarAssets();
    customAvatars.value = Array.isArray(assets) ? assets : [];
    await nextTick();
    clampAvatarMenu();
  } catch (error) {
    console.error('Failed to load avatar assets.', error);
  } finally {
    customAvatarsLoading.value = false;
  }
};

const selectCustomAvatar = (id: string) => {
  draftSettings.value.account.avatar = toLocalAvatar(id);
  avatarError.value = null;
  avatarMenuOpen.value = false;
};

const removeCustomAvatar = async (id: string) => {
  avatarError.value = null;
  try {
    await deleteAvatarAsset(id);
    customAvatars.value = customAvatars.value.filter((item) => item.id !== id);
    clearAvatarUrlCache(id);
    if (selectedLocalAvatarId.value === id) {
      resetAvatar();
    }
    await nextTick();
    clampAvatarMenu();
  } catch (error) {
    console.error('Failed to delete avatar asset.', error);
    avatarError.value = t('settings.avatarErrors.storageFailed');
  }
};

const handleAvatarUpload = async (event: Event) => {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  avatarError.value = null;
  if (!file.type.startsWith('image/')) {
    avatarError.value = t('settings.avatarErrors.invalidType');
    input.value = '';
    return;
  }
  if (file.size > MAX_AVATAR_BYTES) {
    avatarError.value = t('settings.avatarErrors.tooLarge');
    input.value = '';
    return;
  }
  if (!supportsAvatarStorage) {
    avatarError.value = t('settings.avatarErrors.storageFailed');
    input.value = '';
    return;
  }
  try {
    const bytes = await fileToBytes(file);
    const extension = resolveImageExtension(file);
    const asset = await storeAvatarAsset(bytes, extension);
    upsertCustomAvatar(asset);
    draftSettings.value.account.avatar = toLocalAvatar(asset.id);
    avatarMenuOpen.value = false;
  } catch {
    avatarError.value = t('settings.avatarErrors.storageFailed');
  } finally {
    input.value = '';
  }
};

const resetAvatar = () => {
  draftSettings.value.account.avatar = toCssAvatar(lastCssAvatarId.value);
  avatarError.value = null;
  avatarMenuOpen.value = false;
};

const migrateDataUrlAvatar = async () => {
  if (!supportsAvatarStorage) return;
  const current = draftSettings.value.account.avatar;
  if (!current || typeof current !== 'string') return;
  if (isCssAvatar(current) || isLocalAvatar(current)) return;
  if (!current.startsWith('data:image/')) return;
  try {
    const { bytes, extension } = dataUrlToBytes(current);
    if (bytes.length > MAX_AVATAR_BYTES) {
      avatarError.value = t('settings.avatarErrors.tooLarge');
      return;
    }
    const asset = await storeAvatarAsset(bytes, extension);
    upsertCustomAvatar(asset);
    draftSettings.value.account.avatar = toLocalAvatar(asset.id);
  } catch (error) {
    console.error('Failed to migrate avatar data URL.', error);
  }
};

const handleChangeEmail = () => {
  // TODO: implement change email flow.
  void 0;
};

watch(
  settings,
  (next) => {
    if (serializeSettings(next) === serializeSettings(draftSettings.value)) {
      return;
    }
    syncDraftSettings(next);
  },
  { deep: true }
);

watch(
  draftSettings,
  (next) => {
    const candidate = buildPersistableSettings(next);
    if (serializeSettings(candidate) === serializeSettings(settings.value)) {
      return;
    }
    const saved = saveSettings(candidate);
    syncDraftSettings(saved);
  },
  { deep: true }
);

watch(accountAvatar, (next) => {
  const id = getCssAvatarId(next);
  if (id) {
    lastCssAvatarId.value = id;
  }
});

watch(
  () => draftSettings.value.account.avatar,
  (next, prev) => {
    if (!next || next === prev) return;
    void updateMember(CURRENT_USER_ID, { avatar: next });
  }
);

const selectedMemberIndex = computed({
  get: () => draftSettings.value.members.defaultMemberIndex,
  set: (value: MemberSelectionIndex) => {
    draftSettings.value.members.defaultMemberIndex = clampMemberSelectionIndex(
      value,
      draftSettings.value.members.customMembers
    );
  }
});

const selectedMemberId = computed(() => {
  const resolved = resolveMemberIdFromSelectionIndex(
    selectedMemberIndex.value,
    draftSettings.value.members.customMembers
  );
  return resolved ?? BASE_TERMINALS[0]?.id ?? '';
});

const syncDefaultMemberIndex = (preferredId?: string) => {
  const candidateId =
    preferredId ??
    resolveMemberIdFromSelectionIndex(
      selectedMemberIndex.value,
      draftSettings.value.members.customMembers
    );
  if (candidateId) {
    const nextIndex = resolveMemberSelectionIndexFromId(candidateId, draftSettings.value.members.customMembers);
    if (nextIndex) {
      selectedMemberIndex.value = nextIndex;
      return;
    }
  }
  selectedMemberIndex.value = DEFAULT_MEMBER_INDEX;
};

const resetMemberDraft = () => {
  const next = cloneSettings(settings.value);
  draftSettings.value.members = next.members;
  customName.value = '';
  customCommand.value = '';
  isAddingCustom.value = false;
  openMemberMenuId.value = null;
  editingMemberId.value = null;
  syncDefaultMemberIndex();
};

const persistMemberSettings = (nextMembers: SettingsState['members']) => {
  const next = cloneSettings(settings.value);
  next.members = nextMembers;
  const saved = saveSettings(next);
  draftSettings.value.members = cloneSettings(saved).members;
  return saved;
};

const buildCustomMemberId = async () => {
  try {
    return await generateUlid();
  } catch (error) {
    console.error('Failed to generate custom member id.', error);
    return null;
  }
};

const openCustomMemberForm = () => {
  editingMemberId.value = null;
  isAddingCustom.value = true;
  customName.value = '';
  customCommand.value = '';
};

const handleLeaveWorkspace = () => {
  closeWorkspace();
  emit('logout');
};

const handleCreateTeam = async () => {
  if (!isTauri()) {
    return;
  }
  try {
    await openWorkspaceSelectionWindow();
  } catch (error) {
    console.error('Failed to open create team window.', error);
  }
};

const applyCustomMember = async () => {
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
    let id = await buildCustomMemberId();
    if (!id) {
      return;
    }
    while (existingIds.has(id)) {
      id = await buildCustomMemberId();
      if (!id) {
        return;
      }
    }
    draftSettings.value.members.customMembers = [...draftSettings.value.members.customMembers, { id, name, command }];
    syncDefaultMemberIndex(id);
  }
  persistMemberSettings(draftSettings.value.members);
  resetCustomMemberForm();
};

const resetCustomMemberForm = () => {
  customName.value = '';
  customCommand.value = '';
  isAddingCustom.value = false;
  editingMemberId.value = null;
};

const removeCustomMember = (id: string) => {
  const currentDefaultId = resolveMemberIdFromSelectionIndex(
    selectedMemberIndex.value,
    draftSettings.value.members.customMembers
  );
  draftSettings.value.members.customMembers = draftSettings.value.members.customMembers.filter((member) => member.id !== id);
  if (!currentDefaultId || currentDefaultId === id) {
    selectedMemberIndex.value = DEFAULT_MEMBER_INDEX;
  } else {
    syncDefaultMemberIndex(currentDefaultId);
  }
  persistMemberSettings(draftSettings.value.members);
  if (editingMemberId.value === id) {
    resetCustomMemberForm();
  }
  if (openMemberMenuId.value === id) {
    openMemberMenuId.value = null;
  }
};

const selectMemberCard = (member: MemberDisplayOption) => {
  if (selectedMemberId.value === member.id) {
    openMemberMenuId.value = null;
    return;
  }
  selectedMemberIndex.value = [member.group, member.index];
  persistMemberSettings(draftSettings.value.members);
  openMemberMenuId.value = null;
};

const toggleMemberMenu = (id: string) => {
  openMemberMenuId.value = openMemberMenuId.value === id ? null : id;
};

const handleMemberTest = async () => {
  openMemberMenuId.value = null;
  try {
    const workspace = currentWorkspace.value;
    await openTerminalWindow({
      workspaceId: workspace?.id,
      workspaceName: workspace?.name,
      workspacePath: workspace?.path
    });
  } catch (error) {
    console.error('Failed to open terminal window.', error);
  }
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

const timeZones = TIME_ZONE_OPTIONS;


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

const confirmDataAction = async (message: string, title: string) => {
  if (isTauri()) {
    return ask(message, {
      title,
      kind: 'warning',
      okLabel: t('settings.confirm'),
      cancelLabel: t('settings.cancel')
    });
  }
  return window.confirm(message);
};

const handleRepairChatDb = async () => {
  const workspaceId = currentWorkspace.value?.id;
  if (!workspaceId || repairingChatDb.value) return;
  const confirmed = await confirmDataAction(
    t('settings.dataRepairConfirm'),
    t('settings.dataRepairTitle')
  );
  if (!confirmed) return;
  repairingChatDb.value = true;
  dataActionMessage.value = null;
  try {
    const result = await repairChatMessages(workspaceId);
    await loadSession();
    dataActionMessage.value = t('settings.dataRepairResult', { count: result.removedMessages });
  } catch (error) {
    console.error('Failed to repair chat database.', error);
    dataActionMessage.value = t('settings.dataActionFailed');
  } finally {
    repairingChatDb.value = false;
  }
};

const handleClearChatDb = async () => {
  const workspaceId = currentWorkspace.value?.id;
  if (!workspaceId || clearingChatDb.value) return;
  const confirmed = await confirmDataAction(
    t('settings.dataClearConfirm'),
    t('settings.dataClearTitle')
  );
  if (!confirmed) return;
  clearingChatDb.value = true;
  dataActionMessage.value = null;
  try {
    const result = await clearAllChatMessages(workspaceId);
    resetChat();
    await loadSession();
    dataActionMessage.value = t('settings.dataClearResult', {
      messages: result.removedMessages,
      attachments: result.removedAttachments
    });
  } catch (error) {
    console.error('Failed to clear chat database.', error);
    dataActionMessage.value = t('settings.dataActionFailed');
  } finally {
    clearingChatDb.value = false;
  }
};

const memberOptions = computed<MemberDisplayOption[]>(() => {
  const baseOptions = BASE_TERMINALS.map((member, index) => ({
    id: member.id,
    label: t(member.nameKey),
    command: member.command,
    kindLabel: t('settings.memberKind.default'),
    icon: member.icon,
    gradient: member.gradient,
    isCustom: false,
    isDeletable: false,
    group: 0 as const,
    index
  }));

  const customOptions = draftSettings.value.members.customMembers.map((member, index) => ({
    id: member.id,
    label: member.name || t('settings.memberOptions.custom'),
    command: member.command,
    kindLabel: t('settings.memberKind.custom'),
    icon: CUSTOM_TERMINAL_ICON,
    isCustom: true,
    isDeletable: true,
    group: 1 as const,
    index
  }));

  return [...baseOptions, ...customOptions];
});

const sectionButtonClass = (section: SectionId) => [
  'relative w-full text-left flex items-center gap-3 px-3 py-2.5 rounded-lg border border-transparent transition-all group justify-center md:justify-start md:gap-3 gap-0',
  activeSection.value === section
    ? "bg-primary/15 text-white border-primary/30 shadow-[0_1px_3px_rgba(0,0,0,0.18)] ring-1 ring-primary/30 before:content-[''] before:absolute before:left-1 before:top-2 before:bottom-2 before:w-1 before:rounded-full before:bg-primary"
    : 'text-white/60 hover:text-white hover:bg-white/5 hover:border-white/10'
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
    keybinds: keybindsRef.value,
    data: dataRef.value
  };
  const target = sectionMap[section];
  if (!contentRef.value || !target) return;
  activeSection.value = section;
  const containerTop = contentRef.value.getBoundingClientRect().top;
  const targetTop = target.getBoundingClientRect().top;
  const top = targetTop - containerTop + contentRef.value.scrollTop;
  isAutoScrolling.value = true;
  targetScrollTop.value = top;
  if (autoScrollTimeoutId.value !== null) {
    window.clearTimeout(autoScrollTimeoutId.value);
  }
  contentRef.value.scrollTo({ top, behavior: 'smooth' });
  autoScrollTimeoutId.value = window.setTimeout(() => {
    isAutoScrolling.value = false;
    targetScrollTop.value = null;
    autoScrollTimeoutId.value = null;
  }, 800);
};

const updateActiveSection = () => {
  if (!contentRef.value) return;
  const container = contentRef.value;
  const containerTop = container.getBoundingClientRect().top;
  const focusOffset = Math.min(container.clientHeight * 0.35, 200);
  const focusLine = container.scrollTop + focusOffset;
  const bottomDistance = container.scrollHeight - (container.scrollTop + container.clientHeight);
  let nextSection = sectionRefs[0]?.id ?? activeSection.value;
  let maxTop = -Infinity;

  if (bottomDistance <= 80) {
    const lastSection = sectionRefs[sectionRefs.length - 1]?.id;
    if (lastSection) {
      activeSection.value = lastSection;
      return;
    }
  }

  sectionRefs.forEach(({ id, ref }) => {
    if (!ref.value) return;
    const top = ref.value.getBoundingClientRect().top - containerTop + container.scrollTop;
    if (top <= focusLine && top >= maxTop) {
      maxTop = top;
      nextSection = id;
    }
  });

  if (nextSection !== activeSection.value) {
    activeSection.value = nextSection;
  }
};

const handleContentScroll = () => {
  if (!contentRef.value) return;
  if (!isAutoScrolling.value) {
    updateActiveSection();
    return;
  }
  if (targetScrollTop.value !== null) {
    const distance = Math.abs(contentRef.value.scrollTop - targetScrollTop.value);
    if (distance <= 2) {
      isAutoScrolling.value = false;
      targetScrollTop.value = null;
      if (autoScrollTimeoutId.value !== null) {
        window.clearTimeout(autoScrollTimeoutId.value);
        autoScrollTimeoutId.value = null;
      }
      return;
    }
  }
  if (autoScrollTimeoutId.value !== null) {
    window.clearTimeout(autoScrollTimeoutId.value);
  }
  autoScrollTimeoutId.value = window.setTimeout(() => {
    isAutoScrolling.value = false;
    targetScrollTop.value = null;
    autoScrollTimeoutId.value = null;
  }, 120);
};

const handleAvatarMenuOutside = (event: MouseEvent) => {
  if (!avatarMenuOpen.value) return;
  const target = event.target as Node;
  if (avatarMenuRef.value?.contains(target) || avatarButtonRef.value?.contains(target)) {
    return;
  }
  avatarMenuOpen.value = false;
};

onMounted(() => {
  if (contentRef.value) {
    contentRef.value.addEventListener('scroll', handleContentScroll, { passive: true });
    updateActiveSection();
  }
  document.addEventListener('mousedown', handleAvatarMenuOutside);
  void migrateDataUrlAvatar();
});

onBeforeUnmount(() => {
  if (autoScrollTimeoutId.value !== null) {
    window.clearTimeout(autoScrollTimeoutId.value);
  }
  contentRef.value?.removeEventListener('scroll', handleContentScroll);
  document.removeEventListener('mousedown', handleAvatarMenuOutside);
});
</script>
