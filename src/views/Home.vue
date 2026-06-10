<template>
  <div class="h-full flex flex-col gap-6 text-parchment-200">
    <!-- 顶部 Hero Banner -->
    <div class="w-full rounded-2xl overflow-hidden relative group shrink-0 py-10 sm:py-14" style="background: linear-gradient(135deg, rgba(34, 26, 20, 0.95), rgba(26, 20, 16, 0.9), rgba(45, 31, 20, 0.95)); border: 1px solid rgba(180, 140, 100, 0.2); box-shadow: 0 0 30px rgba(230, 180, 34, 0.08);">
      <div class="absolute top-0 right-0 w-48 h-48 rounded-full opacity-10" style="background: radial-gradient(circle, #e6b422 0%, transparent 70%);"></div>
      <div class="absolute bottom-0 left-0 w-36 h-36 rounded-full opacity-08" style="background: radial-gradient(circle, #d4a574 0%, transparent 70%);"></div>
      <div class="relative z-10 flex flex-col items-center justify-center gap-4">
        <div class="w-20 h-20 sm:w-24 sm:h-24 rounded-2xl flex items-center justify-center overflow-hidden" style="box-shadow: 0 0 30px rgba(230, 180, 34, 0.25);">
          <img src="../assets/logo.png" alt="Tavern Deepseek" class="w-full h-full object-cover" />
        </div>
        <h1 class="text-2xl sm:text-3xl font-bold text-center tracking-wider" style="font-family: Georgia, 'Times New Roman', serif; color: #e6b422; text-shadow: 0 0 15px rgba(230, 180, 34, 0.3), 0 2px 4px rgba(0, 0, 0, 0.5);">
          Silly Tavern 开启！
        </h1>
        <p class="text-sm sm:text-base font-medium tracking-wider text-center opacity-80" style="color: #d4a574; text-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);">
          酒馆专属大模型驱动
        </p>
        <!-- 一键启动按钮 - 放在标题下显眼位置 -->
        <button
          class="shrink-0 px-10 py-4 rounded-2xl shadow-lg border-none text-white flex flex-col items-center justify-center gap-1.5 group relative overflow-hidden transition-all duration-300 hover:scale-105 active:scale-95"
          :style="status === 1 || status === 2 ? { background: 'linear-gradient(135deg, #7a2e1f, #c0392b)', boxShadow: '0 0 30px rgba(192, 57, 43, 0.3)' } : { background: 'linear-gradient(135deg, #6b4a35, #c49a1a)', boxShadow: '0 0 30px rgba(230, 180, 34, 0.25)' }"
          @click="handleToggleProcess"
        >
          <div class="absolute inset-0 bg-white/10 translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-in-out rounded-2xl"></div>
          <div class="flex items-center gap-2.5 z-10">
            <StopCircleIcon v-if="status === 1 || status === 2" class="w-6 h-6 fill-current" />
            <PlayIcon v-else class="w-6 h-6 fill-current" />
            <span class="text-xl font-bold tracking-wider" style="font-family: Georgia, 'Times New Roman', serif;">
              {{ status === 1 || status === 2 ? t('home.stopProcess') : t('home.startProcess') }}
            </span>
          </div>
          <span class="text-xs font-medium opacity-90 z-10">
            {{ status === 1 || status === 2 ? t('home.stopDesc') : t('home.startDesc') }}
          </span>
        </button>
        <div class="w-24 h-px" style="background: linear-gradient(90deg, transparent, rgba(230, 180, 34, 0.4), transparent);"></div>
        <!-- 内置版本信息 -->
        <div class="flex items-center gap-2 px-4 py-2 rounded-xl" style="background: rgba(180, 140, 100, 0.08); border: 1px solid rgba(180, 140, 100, 0.15);">
          <span class="text-xs" style="color: #9e7a5c;">{{ t('home.bundledVersionDesc') }}</span>
          <span class="text-xs font-bold px-2 py-0.5 rounded-lg" style="background: rgba(230, 180, 34, 0.12); color: #e6b422;">
            v{{ tavernVersion || '...' }}
          </span>
        </div>
      </div>
    </div>

    <!-- 中部：快捷目录 + 控制面板 -->
    <div class="flex-1 flex flex-col md:flex-row gap-6">
      <!-- 左侧：快捷目录 -->
      <div class="flex-[3] p-6 rounded-2xl flex flex-col tavern-card">
        <h2 class="text-lg font-bold mb-5 flex items-center text-parchment-200 shrink-0" style="font-family: Georgia, 'Times New Roman', serif;">
          <FolderOpenIcon class="w-5 h-5 mr-2" style="color: #e6b422;" />
          {{ t('home.quickDirectories') }}
        </h2>
        <div class="grid grid-cols-2 sm:grid-cols-3 gap-4">
          <button
            v-for="btn in dirs"
            :key="btn.id"
            class="flex flex-col items-center justify-center gap-3 p-4 rounded-xl border transition-all duration-300 group"
            style="background: rgba(42, 30, 20, 0.5); border-color: rgba(180, 140, 100, 0.15);"
            @mouseenter="(e: any) => { e.target.style.background = 'rgba(74, 53, 36, 0.5)'; e.target.style.borderColor = 'rgba(212, 165, 116, 0.4)'; }"
            @mouseleave="(e: any) => { e.target.style.background = 'rgba(42, 30, 20, 0.5)'; e.target.style.borderColor = 'rgba(180, 140, 100, 0.15)'; }"
            @click="btn.action"
          >
            <component :is="btn.icon" class="w-8 h-8 transition-colors duration-300" style="color: #a08060;" />
            <span class="text-sm font-medium transition-colors" style="color: #c49a70;">{{ btn.label }}</span>
          </button>
        </div>
      </div>

      <!-- 右侧：版本信息 + 更新 + 启动 -->
      <div class="flex-[2] flex flex-col gap-6">
        <!-- 版本信息 -->
        <div class="flex-1 p-6 rounded-2xl flex flex-col justify-center tavern-card">
          <h2 class="text-lg font-bold mb-5 flex items-center text-parchment-200" style="font-family: Georgia, 'Times New Roman', serif;">
            <InfoIcon class="w-5 h-5 mr-2" style="color: #e6b422;" />
            {{ t('home.systemInfo') }}
          </h2>
          <div class="flex flex-col gap-4 text-sm">
            <!-- 启动器版本 -->
            <div class="flex items-center justify-between p-3 rounded-xl border" style="background: rgba(42, 30, 20, 0.4); border-color: rgba(180, 140, 100, 0.12);">
              <span class="font-medium flex items-center gap-2" style="color: #c49a70;">
                <BoxIcon class="w-4 h-4" /> {{ t('home.launcherVersion') }}
              </span>
              <span class="font-bold text-parchment-200">v{{ appVersion || '...' }}</span>
            </div>
            <!-- 内置酒馆版本 -->
            <div class="flex items-center justify-between p-3 rounded-xl border" style="background: rgba(42, 30, 20, 0.4); border-color: rgba(180, 140, 100, 0.12);">
              <span class="font-medium flex items-center gap-2" style="color: #c49a70;">
                <BeerIcon class="w-4 h-4" /> {{ t('home.bundledVersion') }}
              </span>
              <span class="font-bold text-parchment-200">v{{ tavernVersion || '...' }}</span>
            </div>
            <!-- 最新在线版本 -->
            <div class="flex items-center justify-between p-3 rounded-xl border" style="background: rgba(42, 30, 20, 0.4); border-color: rgba(180, 140, 100, 0.12);">
              <span class="font-medium flex items-center gap-2" style="color: #c49a70;">
                <CloudIcon class="w-4 h-4" /> {{ t('home.latestOnline') }}
              </span>
              <div class="flex items-center gap-2">
                <span v-if="checkingUpdate" class="text-xs" style="color: #d4a574;">{{ t('home.checkingUpdate') }}</span>
                <span v-else-if="latestOnlineVersion" class="font-bold" style="color: #27ae60;">v{{ latestOnlineVersion }}</span>
                <span v-else class="text-xs" style="color: #9e7a5c;">-</span>
              </div>
            </div>
            <!-- 更新按钮 -->
            <button
              :disabled="checkingUpdate"
              class="w-full py-3 rounded-xl text-sm font-bold transition-all duration-300 active:scale-95 flex items-center justify-center gap-2 disabled:opacity-50"
              :style="hasUpdate ? { background: 'linear-gradient(135deg, #8b6914, #d4a017)', color: '#fdf4dc', boxShadow: '0 0 20px rgba(230, 180, 34, 0.2)' } : { background: 'rgba(42, 30, 20, 0.5)', color: '#a08060', border: '1px solid rgba(180, 140, 100, 0.2)' }"
              @mouseenter="(e: any) => { if (hasUpdate) { e.target.style.boxShadow = '0 0 30px rgba(230, 180, 34, 0.35)'; e.target.style.transform = 'translateY(-1px)'; } }"
              @mouseleave="(e: any) => { if (hasUpdate) { e.target.style.boxShadow = '0 0 20px rgba(230, 180, 34, 0.2)'; e.target.style.transform = 'translateY(0)'; } }"
              @click="checkUpdate"
            >
              <RefreshCcwIcon v-if="checkingUpdate" class="w-4 h-4 animate-spin" />
              <ArrowUpCircleIcon v-else-if="hasUpdate" class="w-4 h-4" />
              <CheckCircleIcon v-else class="w-4 h-4" />
              <span>{{ checkingUpdate ? t('home.checkingUpdate') : hasUpdate ? `${t('home.hasUpdate')} — ${t('home.updateNow')}` : t('home.isLatest') }}</span>
            </button>
          </div>
        </div>

        <!-- 一键启动已移至顶部 Hero 区域，此处仅保留版本信息 -->
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import {
  Play as PlayIcon,
  StopCircle as StopCircleIcon,
  Folder as FolderIcon,
  FolderOpen as FolderOpenIcon,
  FileText as FileTextIcon,
  Beer as BeerIcon,
  Box as BoxIcon,
  Database as DatabaseIcon,
  Puzzle as PuzzleIcon,
  Info as InfoIcon,
  GitBranch as GitIcon,
  RefreshCcw as RefreshCcwIcon,
  ArrowUpCircle as ArrowUpCircleIcon,
  CheckCircle as CheckCircleIcon,
  Cloud as CloudIcon,
} from 'lucide-vue-next'

import {
  consoleStatus as status,
  startProcess,
  stopProcess,
} from '../lib/consoleState'
import { openUrl } from '@tauri-apps/plugin-opener'
import { Dialog } from '../lib/useDialog'
import { toast } from 'vue-sonner'

const { t } = useI18n()
const router = useRouter()
const appVersion = ref('')
const tavernVersion = ref('')
const nodePath = ref('')
const gitPath = ref('')
const latestOnlineVersion = ref('')
const checkingUpdate = ref(false)
const hasUpdate = ref(false)

const openDir = async (dirType: string) => {
  try {
    let customPath = null
    if (dirType === 'node' && nodePath.value) customPath = nodePath.value
    else if (dirType === 'git' && gitPath.value) customPath = gitPath.value
    await invoke('open_directory', { dirType, customPath })
  } catch (error) {
    console.error(`Failed to open ${dirType} directory:`, error)
  }
}

const dirs = [
  { id: 'root', label: t('home.rootDir'), icon: FolderIcon, action: () => openDir('root') },
  { id: 'data', label: t('home.dataDir'), icon: DatabaseIcon, action: () => openDir('data') },
  { id: 'logs', label: t('home.logsDir'), icon: FileTextIcon, action: () => openDir('logs') },
  { id: 'tavern', label: t('home.tavernDir'), icon: BeerIcon, action: () => openDir('tavern') },
  { id: 'extension', label: t('home.extensionDir'), icon: PuzzleIcon, action: () => {} },
  { id: 'node', label: t('home.nodeDir'), icon: BoxIcon, action: () => openDir('node') },
  { id: 'git', label: t('home.gitDir'), icon: GitIcon, action: () => openDir('git') },
]

const checkUpdate = async () => {
  if (checkingUpdate.value) return
  checkingUpdate.value = true
  hasUpdate.value = false
  try {
    // 从 GitHub API 获取最新 Release
    const releases: any[] = await invoke('fetch_sillytavern_releases')
    if (releases && releases.length > 0) {
      const latest = releases[0]
      latestOnlineVersion.value = latest.tag_name?.replace('v', '') || latest.name
      if (latestOnlineVersion.value && tavernVersion.value) {
        hasUpdate.value = latestOnlineVersion.value !== tavernVersion.value
      }
    }
  } catch (_e) {
    // 浏览器模式无法调用 API
  }
  checkingUpdate.value = false
}

const fetchVersions = async () => {
  try {
    const appVer = await invoke<string>('get_app_version')
    appVersion.value = appVer
    localStorage.setItem('app_settings_app_version_cache', appVer)
  } catch (_e) { appVersion.value = '2.0.0' }

  try {
    const tavernVerItem: any = await invoke('get_tavern_version')
    tavernVersion.value = tavernVerItem?.version || '1.12.11'
  } catch (_e) { tavernVersion.value = '1.12.11' }

  try {
    const nodeInfo: any = await invoke('check_nodejs')
    nodePath.value = nodeInfo?.path || ''
  } catch (_e) {}

  try {
    const gitInfo: any = await invoke('check_git')
    gitPath.value = gitInfo?.path || ''
  } catch (_e) {}

  // 后台自动检查更新
  checkUpdate()
}

const handleToggleProcess = async () => {
  if (status.value === 1 || status.value === 2) {
    router.push('/console')
    await stopProcess()
    return
  }

  // 启动前：将 API 配置注入到酒馆的 secrets.json
  try {
    const secrets = await invoke<any>('read_secrets')
    const hasApiKey = Object.values(secrets || {}).some(
      (v: any) => v?.apiKey && v?.enabled
    )

    if (hasApiKey) {
      // 重新写入以确保 SillyTavern 格式的 key 存在
      await invoke('write_secrets', { secrets: secrets })
    } else {
      Dialog.warning({
        title: 'API 连接未配置',
        msg: '未检测到有效的 API Key，建议先配置 API 连接以确保酒馆正常运行。',
        showCancel: true,
        confirmText: '配置 API',
        cancelText: '仍然启动',
        onConfirm: () => {
          router.push('/api-config')
        },
        onCancel: async () => {
          router.push('/console')
          await startProcess()
        },
      })
      return
    }
  } catch {
    // 无法读取 secrets，直接启动
  }

  router.push('/console')
  await startProcess()
}

onMounted(() => {
  fetchVersions()
})
</script>
