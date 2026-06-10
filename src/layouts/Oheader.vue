<script lang="ts" setup>
import { onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import BackgroundVideo from '../components/BackgroundVideo.vue'
import {
  PhMinus,
  PhX,
  PhPlay,
  PhList,
  PhClock,
  PhPlug,
  PhWrench,
  PhFolderOpen,
  PhTerminalWindow,
  PhGear,
  PhFlame,
  PhKey,
} from '@phosphor-icons/vue'
import config from '../lib/config'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { installState, resetInstallState } from '../lib/useInstall'
import { oneClickState, finishOneClickSetup } from '../lib/useOneClick'
import { Dialog } from '../lib/useDialog'
import { consoleStatus } from '../lib/consoleState'

const { t, locale } = useI18n()

// 浏览器降级：非 Tauri 环境下提供空实现
const isTauri = !!(window as any).__TAURI_INTERNALS__
const appWindow = isTauri ? getCurrentWindow() : null
let unlistenClose: (() => void) | null = null
let isForceClosing = false

const requestClose = async () => {
  if (!isTauri || !appWindow) return

  if (isForceClosing) {
    await appWindow.close()
    return
  }

  const isRunningTask =
    (installState.show && ['downloading', 'extracting', 'installing', 'deleting'].includes(installState.status)) ||
    oneClickState.isActive

  if (isRunningTask) {
    Dialog.warning({
      title: t('common.warning'),
      msg:
        locale.value === 'zh-CN'
          ? '自动化流程或安装任务正在进行中。强制关闭将中断所有进度并退回初始状态，确定要关闭吗？'
          : 'Automation or installation tasks are running. Force closing will interrupt all progress and reset to the initial state. Are you sure you want to close?',
      showCancel: true,
      confirmText: locale.value === 'zh-CN' ? '确认关闭' : 'Force Close',
      cancelText: t('common.cancel'),
      onConfirm: async () => {
        isForceClosing = true
        try {
          await invoke('cancel_install')
        } catch (e) {
          console.error(e)
        }
        resetInstallState()
        if (oneClickState.isActive) {
          finishOneClickSetup()
        }
        try {
          await invoke('stop_sillytavern')
        } catch (e) {
          console.error('Failed to stop sillytavern on close:', e)
        }
        await appWindow.close()
      },
    })
  } else {
    isForceClosing = true
    try {
      await invoke('stop_sillytavern')
    } catch (e) {
      console.error('Failed to stop sillytavern on close:', e)
    }
    await appWindow.close()
  }
}

const minimize = async () => {
  if (!isTauri || !appWindow) return
  await appWindow.minimize()
}

onMounted(async () => {
  if (!isTauri || !appWindow) return
  unlistenClose = await appWindow.onCloseRequested(async event => {
    if (!isForceClosing) {
      event.preventDefault()
      requestClose()
    }
  })
})

onUnmounted(() => {
  if (unlistenClose) {
    unlistenClose()
  }
})
</script>

<template>
  <div class="flex flex-col h-screen w-screen overflow-hidden" style="background: #120c08;">
    <!-- 1. Header & Navigation - Medieval Tavern Style -->
    <header
      data-tauri-drag-region
      class="app-titlebar h-14 shrink-0 flex items-center justify-between px-5 z-60"
    >
      <div class="flex items-center gap-3 w-48">
        <div class="w-8 h-8 rounded-lg flex items-center justify-center overflow-hidden shadow-md" style="box-shadow: 0 0 12px rgba(230, 180, 34, 0.2);">
          <img :src="config.appIcon" alt="Tavern Deepseek" class="w-full h-full object-cover" />
        </div>
        <div class="flex flex-col leading-none">
          <span class="text-sm tracking-wider text-parchment-200" style="font-family: Georgia, 'Times New Roman', serif; text-shadow: 0 0 8px rgba(230, 180, 34, 0.15);">
            {{ locale === 'zh-CN' ? 'Tavern Deepseek' : 'Tavern Deepseek' }}
          </span>
          <span class="text-[10px] text-parchment-600 tracking-widest" style="color: #9e7a5c;">
            {{ locale === 'zh-CN' ? '酒馆启动器 v' : 'Launcher v' }}{{ config.appVersion }}
          </span>
        </div>
      </div>

      <div class="flex items-center gap-1">
        <div class="flex items-center w-40 justify-end h-full gap-1">
          <button
            class="h-8 w-8 rounded-lg flex items-center justify-center transition-all duration-200"
            style="color: #a08060;"
            @mouseenter="(e: any) => { e.target.style.background = 'rgba(74, 53, 36, 0.5)'; e.target.style.color = '#eed4b0'; }"
            @mouseleave="(e: any) => { e.target.style.background = 'transparent'; e.target.style.color = '#a08060'; }"
            @click="minimize()"
          >
            <PhMinus class="w-4 h-4" />
          </button>
          <button
            class="h-8 w-8 rounded-lg flex items-center justify-center transition-all duration-200"
            style="color: #a08060;"
            @mouseenter="(e: any) => { e.target.style.background = 'rgba(192, 57, 43, 0.4)'; e.target.style.color = '#e0c8a0'; }"
            @mouseleave="(e: any) => { e.target.style.background = 'transparent'; e.target.style.color = '#a08060'; }"
            @click="requestClose()"
          >
            <PhX class="w-4 h-4" />
          </button>
        </div>
      </div>
    </header>

    <!-- 2. Body (Sidebar & Content) -->
    <div class="flex flex-1 overflow-hidden relative">
      <!-- Sidebar - Medieval Tavern Navigation -->
      <aside
        class="w-24 shrink-0 flex flex-col justify-between py-5 z-50"
        style="background: #1a1410; border-right: 1px solid rgba(180, 140, 100, 0.12);"
      >
        <!-- Sidebar: 1-8 Top, 9-10 Bottom -->
        <div class="flex flex-col gap-2 px-3">
          <!-- 1. 酒馆大模型 -->
          <router-link to="/deepseek" class="nav-link flex flex-col items-center justify-center w-full aspect-square rounded-xl transition-all duration-300 group" active-class="nav-link-active">
            <PhFlame :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
            <span class="text-[11px] font-medium text-center leading-tight">{{ t('nav.deepseek') }}</span>
          </router-link>
          <!-- 2. API连接 -->
          <router-link to="/api-config" class="nav-link flex flex-col items-center justify-center w-full aspect-square rounded-xl transition-all duration-300 group" active-class="nav-link-active">
            <PhKey :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
            <span class="text-[11px] font-medium text-center leading-tight">API 连接</span>
          </router-link>
          <!-- 3. 一键启动 -->
          <router-link to="/" class="nav-link flex flex-col items-center justify-center w-full aspect-square rounded-xl transition-all duration-300 group" active-class="nav-link-active">
            <PhPlay :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
            <span class="text-[11px] font-medium text-center leading-tight">{{ t('nav.quickStart') }}</span>
          </router-link>
          <!-- 4. 角色卡管理 -->
          <router-link to="/resources" class="nav-link flex flex-col items-center justify-center w-full aspect-square rounded-xl transition-all duration-300 group" active-class="nav-link-active">
            <PhFolderOpen :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
            <span class="text-[11px] font-medium text-center leading-tight">角色卡管理</span>
          </router-link>
          <!-- 5. 拓展管理 -->
          <router-link to="/extensions" class="nav-link flex flex-col items-center justify-center w-full aspect-square rounded-xl transition-all duration-300 group" active-class="nav-link-active">
            <PhPlug :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
            <span class="text-[11px] font-medium text-center leading-tight">{{ t('nav.extensionManagement') }}</span>
          </router-link>
          <!-- 6. 酒馆选项 -->
          <router-link to="/tavern" class="nav-link flex flex-col items-center justify-center w-full aspect-square rounded-xl transition-all duration-300 group" active-class="nav-link-active">
            <PhList :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
            <span class="text-[11px] font-medium text-center leading-tight">{{ t('nav.tavernOptions') }}</span>
          </router-link>
          <!-- 7. 版本管理 -->
          <router-link to="/versions" class="nav-link flex flex-col items-center justify-center w-full aspect-square rounded-xl transition-all duration-300 group" active-class="nav-link-active">
            <PhClock :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
            <span class="text-[11px] font-medium text-center leading-tight">{{ t('nav.versionManagement') }}</span>
          </router-link>
          <!-- 8. 教程 -->
          <router-link to="/tools" class="nav-link flex flex-col items-center justify-center w-full aspect-square rounded-xl transition-all duration-300 group" active-class="nav-link-active">
            <PhWrench :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
            <span class="text-[11px] font-medium text-center leading-tight">教程</span>
          </router-link>
        </div>

        <!-- Bottom Menu -->
        <div class="flex flex-col gap-2 px-3">
          <!-- 10. 控制台 -->
          <router-link to="/console"
            :active-class="
              consoleStatus === 2
                ? '!bg-emerald-900/20 !text-emerald-400'
                : '!bg-red-900/20 !text-red-400'
            "
            :class="[
              'nav-link flex flex-col items-center justify-center w-full aspect-square rounded-xl transition-all duration-300 group',
              consoleStatus === 2
                ? '!text-emerald-500 hover:!bg-emerald-900/25 hover:!text-emerald-400'
                : '!text-red-500 hover:!bg-red-900/25 hover:!text-red-400',
            ]"
          >
            <PhTerminalWindow :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
            <span class="text-[11px] font-medium text-center leading-tight">{{ t('nav.console') }}</span>
          </router-link>
          <!-- 10. 设置 -->
          <router-link
            to="/settings"
            class="nav-link flex flex-col items-center justify-center w-full aspect-square rounded-xl transition-all duration-300 group"
            active-class="nav-link-active"
          >
            <PhGear :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
            <span class="text-[11px] font-medium text-center leading-tight">{{ t('nav.settings') }}</span>
          </router-link>
        </div>
      </aside>

      <!-- 背景视频 -->
      <BackgroundVideo />

      <!-- Main Content -->
      <main class="flex-1 relative overflow-y-auto z-10" style="background: transparent;">
        <div class="max-w-6xl mx-auto px-6 py-10 pb-24 h-full relative z-10">
          <slot></slot>
        </div>
      </main>

      <slot name="Modal"></slot>
    </div>
  </div>
</template>

<style scoped>
/* Titlebar - Medieval Tavern Glass */
.app-titlebar {
  -webkit-app-region: drag !important;
  background: linear-gradient(180deg, rgba(34, 26, 20, 0.95), rgba(26, 20, 16, 0.95));
  backdrop-filter: blur(20px);
  border-bottom: 1px solid rgba(180, 140, 100, 0.15);
  position: relative;
  font-family: var(--font-main) !important;
}

.app-titlebar::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(230, 180, 34, 0.1), rgba(212, 165, 116, 0.15), rgba(230, 180, 34, 0.1), transparent);
  pointer-events: none;
}

.app-titlebar button {
  -webkit-app-region: no-drag;
}

.app-titlebar a {
  -webkit-app-region: no-drag;
}

/* Navigation Links */
.nav-link {
  position: relative;
  color: #b8926a;
  font-family: var(--font-main) !important;
}

.nav-link:hover {
  background: rgba(74, 53, 36, 0.5);
  color: #eed4b0;
}

.nav-link-active {
  background: rgba(74, 53, 36, 0.6) !important;
  color: #e6b422 !important;
  box-shadow: 0 0 15px rgba(230, 180, 34, 0.12);
  border: 1px solid rgba(212, 165, 116, 0.25);
}

.nav-link-active::before {
  content: '';
  position: absolute;
  left: -8px;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 60%;
  background: linear-gradient(180deg, transparent, #e6b422, transparent);
  border-radius: 2px;
  box-shadow: 0 0 8px rgba(230, 180, 34, 0.2);
}
</style>
