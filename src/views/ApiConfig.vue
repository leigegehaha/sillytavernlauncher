<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { PhKey, PhCheckCircle, PhWarning, PhFloppyDisk, PhTrash, PhArrowCounterClockwise, PhEye, PhEyeSlash } from '@phosphor-icons/vue'
import { toast } from 'vue-sonner'
import { useI18n } from 'vue-i18n'
import { openUrl } from '@tauri-apps/plugin-opener'

const { t } = useI18n()
const isTauri = !!(window as any).__TAURI_INTERNALS__

// API provider configs
interface ApiProvider {
  key: string
  label: string
  endpoint: string
  modelOptions?: { value: string; label: string; desc: string }[]
  apiKey: string
  model: string
  enabled: boolean
}

const providers = ref<ApiProvider[]>([
  {
    key: 'deepseek_tavern',
    label: '酒馆专属模型',
    endpoint: 'https://deepseektavern.com/v1',
    modelOptions: [
      { value: 'deepseek-tavern-v2-pro', label: '均衡模式', desc: '兼顾创意与稳定，推荐日常使用' },
      { value: 'deepseek-tavern-v2-pro-low', label: '自由创作', desc: '想象力更奔放，适合创意写作' },
      { value: 'deepseek-tavern-v2-pro-xhigh', label: '严谨模式', desc: '严格遵循提示词，适合复杂任务' },
    ],
    apiKey: '',
    model: 'deepseek-tavern-v2-pro',
    enabled: true,
  },
  {
    key: 'openai',
    label: 'OpenAI 兼容',
    endpoint: 'https://api.openai.com/v1',
    apiKey: '',
    model: '',
    enabled: false,
  },
])

const testing = ref<Record<string, boolean>>({})
const fetchingModels = ref<Record<string, boolean>>({})
const modelListCache = ref<Record<string, string[]>>({})
const saving = ref(false)
const loading = ref(true)
const showKeys = ref<Record<string, boolean>>({})

// Load from secrets.json on mount
const loadSecrets = async () => {
  loading.value = true
  try {
    if (!isTauri) { loading.value = false; return }
    const secrets = await invoke<any>('read_secrets')

    for (const p of providers.value) {
      const cfg = secrets?.[p.key]
      if (cfg) {
        p.apiKey = cfg.apiKey || ''
        p.model = cfg.model || p.modelOptions?.[0]?.value || ''
        p.enabled = cfg.enabled ?? p.enabled
        p.endpoint = cfg.endpoint || p.endpoint
      }
    }
  } catch (e: any) {
    console.error('Failed to load secrets:', e)
  } finally {
    loading.value = false
  }
}

// Save to secrets.json
const saveSecrets = async () => {
  saving.value = true
  try {
    if (!isTauri) return
    const secrets: Record<string, any> = {}
    for (const p of providers.value) {
      secrets[p.key] = {
        apiKey: p.apiKey,
        model: p.model,
        endpoint: p.endpoint,
        enabled: p.enabled,
      }
    }
    await invoke('write_secrets', { secrets: secrets as any })
    toast.success('API 配置已保存')
  } catch (e: any) {
    toast.error(e?.message || '保存失败')
  } finally {
    saving.value = false
  }
}

// Test API connection
const testConnection = async (provider: ApiProvider) => {
  testing.value = { ...testing.value, [provider.key]: true }
  try {
    if (!isTauri || !provider.apiKey) {
      toast.error('请先填写 API Key')
      return
    }
    const result = await invoke<{ ok: boolean; msg: string }>('test_api_connection', {
      endpoint: provider.endpoint + '/models',
      apiKey: provider.apiKey,
    })
    if (result.ok) {
      toast.success('连接成功！API 可用')
    } else {
      toast.error(result.msg || '连接失败')
    }
  } catch (e: any) {
    toast.error(e?.message || '测试失败')
  } finally {
    testing.value = { ...testing.value, [provider.key]: false }
  }
}

// Fetch model list from API endpoint
const fetchModels = async (provider: ApiProvider) => {
  fetchingModels.value = { ...fetchingModels.value, [provider.key]: true }
  try {
    if (!isTauri || !provider.apiKey) {
      toast.error('请先填写 API Key')
      return
    }
    if (!provider.endpoint) {
      toast.error('请先填写 API 端点')
      return
    }
    const result = await invoke<{ models: string[] }>('fetch_model_list', {
      endpoint: provider.endpoint,
      apiKey: provider.apiKey,
    })
    modelListCache.value = { ...modelListCache.value, [provider.key]: result.models }
    toast.success(`获取到 ${result.models.length} 个模型`)
  } catch (e: any) {
    toast.error(e?.message || '获取模型列表失败')
  } finally {
    fetchingModels.value = { ...fetchingModels.value, [provider.key]: false }
  }
}

// Mask API key for display
const maskedKey = (key: string) => {
  if (!key) return ''
  if (key.length <= 8) return '****'
  return key.slice(0, 4) + '****' + key.slice(-4)
}

onMounted(() => loadSecrets())
</script>

<template>
  <div class="flex flex-col h-full space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-lg font-bold flex items-center gap-2 text-parchment-200" style="font-family: Georgia, 'Times New Roman', serif;">
          <PhKey class="w-5 h-5" style="color: #e6b422;" />
          API 连接配置
        </h2>
        <p class="text-sm mt-1 opacity-70" style="color: #9e7a5c;">
          配置大模型 API 密钥，启动酒馆即可使用
        </p>
      </div>
      <button
        class="px-5 py-2.5 rounded-xl text-sm font-bold transition-all duration-200 flex items-center gap-2"
        style="background: linear-gradient(135deg, #6b4a35, #8b6914); color: #fdf4dc; box-shadow: 0 0 15px rgba(230, 180, 34, 0.15); border: 1px solid rgba(230, 180, 34, 0.25);"
        :disabled="saving"
        @click="saveSecrets"
      >
        <PhFloppyDisk :size="16" />
        {{ saving ? '保存中...' : '保存配置' }}
      </button>
    </div>

    <div v-if="loading" class="text-center py-12 text-sm opacity-50" style="color: #9e7a5c;">加载中...</div>

    <!-- API Providers -->
    <div v-if="!loading" class="space-y-4">
      <div
        v-for="p in providers"
        :key="p.key"
        class="rounded-2xl p-5 space-y-4 transition-all duration-200"
        :style="{
          background: p.enabled ? 'rgba(42, 30, 20, 0.6)' : 'rgba(42, 30, 20, 0.3)',
          border: p.enabled ? '1px solid rgba(230, 180, 34, 0.2)' : '1px solid rgba(180, 140, 100, 0.1)',
        }"
      >
        <!-- Provider header -->
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <button
              class="w-5 h-5 rounded-md flex items-center justify-center transition-colors"
              :style="{
                background: p.enabled ? 'rgba(39, 174, 96, 0.2)' : 'rgba(150, 150, 150, 0.15)',
                border: p.enabled ? '1px solid rgba(39, 174, 96, 0.4)' : '1px solid rgba(150, 150, 150, 0.3)',
              }"
              @click="p.enabled = !p.enabled"
            >
              <PhCheckCircle v-if="p.enabled" :size="14" style="color: #27ae60;" />
            </button>
            <span
              class="text-base font-bold"
              :style="{ color: p.enabled ? '#e6b422' : '#9e7a5c' }"
              style="font-family: Georgia, 'Times New Roman', serif;"
            >
              {{ p.label }}
            </span>
            <span
              v-if="p.key === 'deepseek_tavern'"
              class="px-2 py-0.5 text-[10px] font-bold rounded-md"
              style="background: linear-gradient(135deg, rgba(230, 180, 34, 0.2), rgba(212, 165, 116, 0.2)); color: #e6b422;"
            >
              推荐
            </span>
          </div>
          <span
            v-if="p.apiKey"
            class="text-xs px-2 py-1 rounded-lg"
            style="background: rgba(39, 174, 96, 0.1); color: #27ae60;"
          >
            API 已设置
          </span>
        </div>

        <div v-if="p.enabled" class="space-y-3 pl-8 animate-in fade-in slide-in-from-top-1 duration-200">
          <!-- Endpoint -->
          <div>
            <label class="text-xs font-medium block mb-1.5" style="color: #a08060;">API 端点</label>
            <input
              v-model="p.endpoint"
              type="text"
              class="w-full px-3 py-2 rounded-lg text-sm font-mono outline-none transition-colors"
              style="background: rgba(20, 14, 10, 0.6); border: 1px solid rgba(180, 140, 100, 0.2); color: #d4a574;"
              :style="{ 'border-color': p.endpoint ? 'rgba(180, 140, 100, 0.3)' : 'rgba(192, 57, 43, 0.3)', color: '#d4a574' }"
            />
          </div>

          <!-- API Key -->
          <div>
            <div class="flex items-center justify-between mb-1.5">
              <label class="text-xs font-medium" style="color: #a08060;">API Key</label>
              <span v-if="p.apiKey" class="text-xs font-mono" style="color: #9e7a5c;">{{ maskedKey(p.apiKey) }}</span>
            </div>
            <div class="flex gap-2">
              <input
                v-model="p.apiKey"
                :type="showKeys[p.key] ? 'text' : 'password'"
                class="flex-1 px-3 py-2 rounded-lg text-sm font-mono outline-none transition-colors"
                :placeholder="p.key === 'deepseek_tavern' ? 'sk-... 从 deepseektavern.com 获取' : 'sk-...'"
                style="background: rgba(20, 14, 10, 0.6); border: 1px solid rgba(180, 140, 100, 0.2); color: #d4a574;"
                :style="{ 'border-color': p.apiKey ? 'rgba(39, 174, 96, 0.3)' : 'rgba(180, 140, 100, 0.2)' }"
              />
              <button
                v-if="p.apiKey"
                class="shrink-0 px-2 rounded-lg text-xs transition-colors"
                :style="{ background: showKeys[p.key] ? 'rgba(230,180,34,0.15)' : 'rgba(20,14,10,0.4)', color: showKeys[p.key] ? '#e6b422' : '#806040', border: '1px solid ' + (showKeys[p.key] ? 'rgba(230,180,34,0.25)' : 'rgba(180,140,100,0.15)') }"
                @click="showKeys[p.key] = !showKeys[p.key]"
                :title="showKeys[p.key] ? '隐藏 API Key' : '显示 API Key'">
                <PhEye :size="14" v-if="!showKeys[p.key]" />
                <PhEyeSlash :size="14" v-else />
              </button>
            </div>
          </div>

          <!-- Model selector: radio buttons for providers with modelOptions, text input for OpenAI-compatible -->
          <div v-if="p.modelOptions && p.modelOptions.length > 0">
            <label class="text-xs font-medium block mb-1.5" style="color: #a08060;">模型选择</label>
            <div class="space-y-1.5">
              <button
                v-for="opt in p.modelOptions"
                :key="opt.value"
                class="w-full text-left px-3 py-2 rounded-lg text-sm transition-all duration-200"
                :class="p.model === opt.value ? 'font-bold' : ''"
                :style="{
                  background: p.model === opt.value ? 'rgba(230, 180, 34, 0.12)' : 'rgba(20, 14, 10, 0.4)',
                  border: p.model === opt.value ? '1px solid rgba(230, 180, 34, 0.35)' : '1px solid rgba(180, 140, 100, 0.1)',
                  color: p.model === opt.value ? '#e6b422' : '#c49a70',
                }"
                @click="p.model = opt.value"
              >
                <div class="flex items-center justify-between">
                  <span>{{ opt.label }}</span>
                  <span class="text-[10px] font-mono opacity-60">{{ opt.value }}</span>
                </div>
                <div class="text-[11px] mt-0.5 opacity-60">{{ opt.desc }}</div>
              </button>
            </div>
          </div>

          <!-- Model ID text input for OpenAI-compatible providers -->
          <div v-else>
            <label class="text-xs font-medium block mb-1.5" style="color: #a08060;">模型 ID</label>
            <div class="flex items-center gap-2">
              <input
                v-model="p.model"
                type="text"
                class="flex-1 px-3 py-2 rounded-lg text-sm font-mono outline-none transition-colors"
                placeholder="例如：gpt-4o, claude-sonnet-4-20250514"
                style="background: rgba(20, 14, 10, 0.6); border: 1px solid rgba(180, 140, 100, 0.2); color: #d4a574;"
                :style="{ 'border-color': p.model ? 'rgba(39, 174, 96, 0.3)' : 'rgba(180, 140, 100, 0.2)' }"
              />
              <button
                class="shrink-0 px-3 py-2 rounded-lg text-xs font-medium transition-all duration-200 flex items-center gap-1.5"
                :disabled="fetchingModels[p.key] || !p.apiKey || !p.endpoint"
                :style="{
                  background: fetchingModels[p.key] ? 'rgba(230, 180, 34, 0.08)' : 'rgba(39, 174, 96, 0.08)',
                  color: fetchingModels[p.key] ? '#e6b422' : (p.apiKey && p.endpoint) ? '#27ae60' : '#9e7a5c',
                  border: '1px solid ' + (fetchingModels[p.key] ? 'rgba(230, 180, 34, 0.2)' : (p.apiKey && p.endpoint) ? 'rgba(39, 174, 96, 0.2)' : 'rgba(150, 150, 150, 0.15)'),
                  cursor: fetchingModels[p.key] || !p.apiKey || !p.endpoint ? 'not-allowed' : 'pointer',
                  opacity: fetchingModels[p.key] || !p.apiKey || !p.endpoint ? 0.6 : 1,
                }"
                @click="fetchModels(p)"
              >
                <PhArrowCounterClockwise v-if="fetchingModels[p.key]" :size="12" class="animate-spin" />
                <span v-else>获取模型列表</span>
              </button>
            </div>

            <!-- Fetched model list -->
            <div
              v-if="modelListCache[p.key] && modelListCache[p.key].length > 0"
              class="mt-2 p-2 rounded-lg space-y-0.5 max-h-48 overflow-y-auto"
              style="background: rgba(20, 14, 10, 0.4); border: 1px solid rgba(180, 140, 100, 0.1);"
            >
              <button
                v-for="m in modelListCache[p.key]"
                :key="m"
                class="w-full text-left px-2.5 py-1.5 rounded-md text-xs font-mono transition-all duration-150 hover:bg-opacity-100"
                :style="{
                  background: p.model === m ? 'rgba(230, 180, 34, 0.12)' : 'transparent',
                  color: p.model === m ? '#e6b422' : '#c49a70',
                }"
                @click="p.model = m"
              >
                {{ m }}
              </button>
            </div>
          </div>

          <!-- Test button -->
          <button
            class="mt-2 text-xs font-medium flex items-center gap-1.5 transition-colors px-3 py-1.5 rounded-lg"
            :disabled="testing[p.key] || !p.apiKey"
            :style="{
              background: testing[p.key] ? 'rgba(230, 180, 34, 0.08)' : 'rgba(39, 174, 96, 0.08)',
              color: testing[p.key] ? '#e6b422' : p.apiKey ? '#27ae60' : '#9e7a5c',
              border: '1px solid ' + (testing[p.key] ? 'rgba(230, 180, 34, 0.2)' : p.apiKey ? 'rgba(39, 174, 96, 0.2)' : 'rgba(150, 150, 150, 0.15)'),
            }"
            @click="testConnection(p)"
          >
            <PhArrowCounterClockwise v-if="testing[p.key]" :size="12" class="animate-spin" />
            <PhCheckCircle v-else :size="12" />
            {{ testing[p.key] ? '测试中...' : '测试连接' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Info box -->
    <div class="mt-4 p-4 rounded-xl" style="background: rgba(42, 30, 20, 0.4); border: 1px solid rgba(180, 140, 100, 0.12);">
      <div class="flex items-start gap-2">
        <PhWarning :size="16" style="color: #e6b422; margin-top: 1px; flex-shrink: 0;" />
        <div class="text-xs" style="color: #9e7a5c; line-height: 1.6;">
          API Key 存储在本地 secrets.json 中，不会上传到任何服务器。<br />
          酒馆专用模型 Key 请前往 <a href="#" style="color: #e6b422; text-decoration: underline;" @click.prevent="openUrl('https://deepseektavern.com')">deepseektavern.com</a> 注册获取。
        </div>
      </div>
    </div>
  </div>
</template>
