<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'vue-sonner'
import {
  PhKey,
  PhCopy,
  PhTrash,
  PhArrowCounterClockwise,
  PhGlobeHemisphereWest,
  PhLightning,
  PhRocket,
  PhBrain,
  PhBookOpen,
  PhFeather,
  PhShieldCheck,
  PhWrench,
  PhClock,
  PhEye,
  PhEyeSlash,
} from '@phosphor-icons/vue'

const isTauri = !!(window as any).__TAURI_INTERNALS__

// ── Paste key ──
const manualKeyInput = ref('')
const savingKey = ref(false)
const openingWebview = ref(false)

// ── Saved keys (localStorage) ──
interface SavedKey {
  label: string
  key: string
  savedAt: number
}
const savedKeys = ref<SavedKey[]>([])
const revealedIdx = ref<Set<number>>(new Set())

const STORAGE_KEY = 'tavern_saved_keys'

const loadSavedKeys = () => {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) savedKeys.value = JSON.parse(raw)
  } catch { savedKeys.value = [] }
}

const saveKeyToStorage = (label: string, key: string) => {
  savedKeys.value = savedKeys.value.filter(k => k.key !== key)
  savedKeys.value.unshift({ label, key, savedAt: Date.now() })
  if (savedKeys.value.length > 10) savedKeys.value = savedKeys.value.slice(0, 10)
  localStorage.setItem(STORAGE_KEY, JSON.stringify(savedKeys.value))
}

const removeSavedKey = (idx: number) => {
  savedKeys.value.splice(idx, 1)
  localStorage.setItem(STORAGE_KEY, JSON.stringify(savedKeys.value))
}

const toggleReveal = (idx: number) => {
  const next = new Set(revealedIdx.value)
  if (next.has(idx)) next.delete(idx)
  else next.add(idx)
  revealedIdx.value = next
}

const copyKey = (key: string) => {
  navigator.clipboard.writeText(key).then(() => toast.success('已复制完整 API Key'))
}

const openKeyWebview = async () => {
  if (!isTauri) return
  openingWebview.value = true
  try {
    await invoke('open_tavern_key_webview', {})
  } catch (e: any) {
    toast.error('打开网页失败: ' + (e?.toString() || '未知错误'))
  } finally {
    openingWebview.value = false
  }
}

const autoFillApiConfig = async (key: string) => {
  if (key.includes('*')) {
    toast.error('无法使用掩码密钥，请从网页复制完整 Key')
    return
  }
  savingKey.value = true
  try {
    if (!isTauri) return
    const existing = await invoke<any>('read_secrets')
    const updated = {
      ...existing,
      deepseek_tavern: {
        apiKey: key,
        model: (existing as any)?.deepseek_tavern?.model || 'deepseek-tavern-v2-pro',
        endpoint: 'https://deepseektavern.com/v1',
        enabled: true,
      },
    }
    await invoke('write_secrets', { secrets: updated })
    saveKeyToStorage('手动粘贴', key)
    toast.success('API Key 已自动填入酒馆配置！')
  } catch (e: any) {
    toast.error('自动填入失败: ' + (e?.toString() || ''))
  } finally {
    savingKey.value = false
  }
}

const pasteAndSave = async () => {
  const key = manualKeyInput.value.trim()
  if (!key) { toast.error('请粘贴 API Key'); return }
  await autoFillApiConfig(key)
  manualKeyInput.value = ''
}

onMounted(() => loadSavedKeys())
</script>

<template>
  <div class="rounded-2xl p-5 space-y-4 transition-all duration-200"
    style="background: rgba(42, 30, 20, 0.6); border: 1px solid rgba(230, 180, 34, 0.2);">

    <!-- ══════════════════════════════════════════════════════ -->
    <!-- Part 1: Hero Overview                                -->
    <!-- ══════════════════════════════════════════════════════ -->
    <div class="rounded-xl p-5 space-y-3"
      style="background: linear-gradient(135deg, rgba(34, 26, 20, 0.95), rgba(45, 31, 20, 0.88)); border: 1px solid rgba(230, 180, 34, 0.18);">

      <!-- Title -->
      <div class="flex items-center gap-3">
        <div class="w-12 h-12 rounded-xl flex items-center justify-center shrink-0"
          style="background: linear-gradient(135deg, #6b4a35, #8b6914); box-shadow: 0 0 20px rgba(230, 180, 34, 0.15);">
          <PhBrain :size="24" style="color: #fdf4dc;" />
        </div>
        <div>
          <h3 class="text-lg font-bold tracking-wide" style="font-family: Georgia, serif; color: #e6b422; text-shadow: 0 0 8px rgba(230, 180, 34, 0.15);">
            DeepSeek Tavern Pro
          </h3>
          <p class="text-sm mt-0.5" style="color: #9e7a5c;">
            专为 SillyTavern 酒馆角色扮演而生 · 专家微调 · 文学级写作
          </p>
        </div>
      </div>

      <!-- Key Stats Bar -->
      <div class="grid grid-cols-4 gap-2">
        <div class="text-center px-2 py-2.5 rounded-lg" style="background: rgba(230, 180, 34, 0.06); border: 1px solid rgba(230, 180, 34, 0.1);">
          <div class="text-sm font-bold" style="color: #e6b422;">1.6T</div>
          <div class="text-[10px]" style="color: #7a5c44;">总参数量</div>
        </div>
        <div class="text-center px-2 py-2.5 rounded-lg" style="background: rgba(52, 152, 219, 0.06); border: 1px solid rgba(52, 152, 219, 0.1);">
          <div class="text-sm font-bold" style="color: #3498db;">100B</div>
          <div class="text-[10px]" style="color: #7a5c44;">微调参数</div>
        </div>
        <div class="text-center px-2 py-2.5 rounded-lg" style="background: rgba(39, 174, 96, 0.06); border: 1px solid rgba(39, 174, 96, 0.1);">
          <div class="text-sm font-bold" style="color: #27ae60;">1M</div>
          <div class="text-[10px]" style="color: #7a5c44;">上下文窗口</div>
        </div>
        <div class="text-center px-2 py-2.5 rounded-lg" style="background: rgba(212, 165, 116, 0.06); border: 1px solid rgba(212, 165, 116, 0.1);">
          <div class="text-sm font-bold" style="color: #d4a574;">99.9%</div>
          <div class="text-[10px]" style="color: #7a5c44;">API 可用性</div>
        </div>
      </div>

      <!-- Description -->
      <p class="text-xs leading-relaxed" style="color: #8b7355;">
        基于 DeepSeek V4 Pro MoE 架构（<span style="color: #c49a70;">1.6T 参数 · 49B 推理激活</span>），对文字专家模块进行 100B 参数精准微调。
        训练数据全部来自真实酒馆场景——上千张角色卡、数百种预设与世界书配置。天然理解角色扮演的叙事结构，
        无需复杂提示词工程即可输出文学级写作质量。
      </p>

      <!-- Feature Tags -->
      <div class="flex flex-wrap gap-1.5 text-[10px]">
        <span class="px-2 py-0.5 rounded" style="background: rgba(230, 180, 34, 0.08); color: #d4a574;">文学级写作</span>
        <span class="px-2 py-0.5 rounded" style="background: rgba(230, 180, 34, 0.08); color: #d4a574;">酒馆原生优化</span>
        <span class="px-2 py-0.5 rounded" style="background: rgba(52, 152, 219, 0.08); color: #85c1e9;">1M 超长上下文</span>
        <span class="px-2 py-0.5 rounded" style="background: rgba(52, 152, 219, 0.08); color: #85c1e9;">叙事自由</span>
        <span class="px-2 py-0.5 rounded" style="background: rgba(39, 174, 96, 0.08); color: #82e0aa;">OpenAI 兼容</span>
        <span class="px-2 py-0.5 rounded" style="background: rgba(39, 174, 96, 0.08); color: #82e0aa;">抗中断机制</span>
      </div>
    </div>

    <!-- ══════════════════════════════════════════════════════ -->
    <!-- Part 2: Three Creative Modes                         -->
    <!-- ══════════════════════════════════════════════════════ -->
    <div class="rounded-xl p-4 space-y-3"
      style="background: rgba(20, 14, 10, 0.4); border: 1px solid rgba(180, 140, 100, 0.1);">
      <div class="flex items-center gap-2">
        <PhFeather :size="15" style="color: #e6b422;" />
        <span class="text-sm font-bold" style="color: #c49a70;">三档创作模式</span>
        <span class="text-[11px]" style="color: #6b5344;">通过模型名后缀自由切换</span>
      </div>

      <div class="grid grid-cols-3 gap-2.5">
        <!-- Pro -->
        <div class="p-3.5 rounded-lg text-center space-y-1.5"
          style="background: linear-gradient(180deg, rgba(230, 180, 34, 0.08), rgba(230, 180, 34, 0.02)); border: 1px solid rgba(230, 180, 34, 0.18);">
          <div class="w-8 h-8 rounded-full mx-auto flex items-center justify-center"
            style="background: rgba(230, 180, 34, 0.15);">
            <span class="text-sm">⚖️</span>
          </div>
          <div class="text-[11px] font-mono font-bold" style="color: #e6b422;">v2-pro</div>
          <div class="text-[10px]" style="color: #c49a70;">均衡模式</div>
          <div class="text-[10px] leading-relaxed" style="color: #7a5c44;">
            兼顾创意与稳定<br />默认推荐
          </div>
        </div>
        <!-- Low -->
        <div class="p-3.5 rounded-lg text-center space-y-1.5"
          style="background: linear-gradient(180deg, rgba(52, 152, 219, 0.06), rgba(52, 152, 219, 0.01)); border: 1px solid rgba(52, 152, 219, 0.15);">
          <div class="w-8 h-8 rounded-full mx-auto flex items-center justify-center"
            style="background: rgba(52, 152, 219, 0.12);">
            <span class="text-sm">🎨</span>
          </div>
          <div class="text-[11px] font-mono font-bold" style="color: #3498db;">v2-pro-low</div>
          <div class="text-[10px]" style="color: #85c1e9;">自由创作</div>
          <div class="text-[10px] leading-relaxed" style="color: #7a5c44;">
            想象力更奔放<br />更少约束
          </div>
        </div>
        <!-- XHigh -->
        <div class="p-3.5 rounded-lg text-center space-y-1.5"
          style="background: linear-gradient(180deg, rgba(39, 174, 96, 0.06), rgba(39, 174, 96, 0.01)); border: 1px solid rgba(39, 174, 96, 0.15);">
          <div class="w-8 h-8 rounded-full mx-auto flex items-center justify-center"
            style="background: rgba(39, 174, 96, 0.12);">
            <span class="text-sm" style="color: #27ae60;">📐</span>
          </div>
          <div class="text-[11px] font-mono font-bold" style="color: #27ae60;">v2-pro-xhigh</div>
          <div class="text-[10px]" style="color: #82e0aa;">严谨模式</div>
          <div class="text-[10px] leading-relaxed" style="color: #7a5c44;">
            严格遵循提示词<br />精准可控
          </div>
        </div>
      </div>
    </div>

    <!-- ══════════════════════════════════════════════════════ -->
    <!-- Part 3: Core Capabilities                            -->
    <!-- ══════════════════════════════════════════════════════ -->
    <div class="rounded-xl p-4 space-y-3"
      style="background: rgba(20, 14, 10, 0.4); border: 1px solid rgba(180, 140, 100, 0.1);">
      <div class="flex items-center gap-2">
        <PhLightning :size="15" style="color: #e6b422;" />
        <span class="text-sm font-bold" style="color: #c49a70;">核心能力</span>
      </div>

      <div class="space-y-2.5">
        <div class="flex gap-3 items-start">
          <div class="w-6 h-6 rounded flex items-center justify-center shrink-0 mt-0.5"
            style="background: rgba(230, 180, 34, 0.1);">
            <PhBookOpen :size="12" style="color: #e6b422;" />
          </div>
          <div>
            <div class="text-xs font-bold" style="color: #c49a70;">文学级写作质量</div>
            <div class="text-[11px] leading-relaxed" style="color: #7a5c44;">
              内置完整文学写作协议——漏斗式开篇、感官细节、丁达尔光影、动作中断。非AI式平淡叙述，具有真实质感的文字。
            </div>
          </div>
        </div>
        <div class="flex gap-3 items-start">
          <div class="w-6 h-6 rounded flex items-center justify-center shrink-0 mt-0.5"
            style="background: rgba(52, 152, 219, 0.1);">
            <PhShieldCheck :size="12" style="color: #3498db;" />
          </div>
          <div>
            <div class="text-xs font-bold" style="color: #85c1e9;">叙事自由 · 不拒绝</div>
            <div class="text-[11px] leading-relaxed" style="color: #7a5c44;">
              不拒绝符合角色设定的叙事请求，完整执行角色卡定义的 NSFW 内容。抗中断机制确保沉浸式体验。
            </div>
          </div>
        </div>
        <div class="flex gap-3 items-start">
          <div class="w-6 h-6 rounded flex items-center justify-center shrink-0 mt-0.5"
            style="background: rgba(39, 174, 96, 0.1);">
            <PhClock :size="12" style="color: #27ae60;" />
          </div>
          <div>
            <div class="text-xs font-bold" style="color: #82e0aa;">100 万 Token 超长上下文</div>
            <div class="text-[11px] leading-relaxed" style="color: #7a5c44;">
              适合长篇小说创作、复杂世界书、多角色切换。不会丢失前文的细节和情感脉络。
            </div>
          </div>
        </div>
        <div class="flex gap-3 items-start">
          <div class="w-6 h-6 rounded flex items-center justify-center shrink-0 mt-0.5"
            style="background: rgba(212, 165, 116, 0.1);">
            <PhWrench :size="12" style="color: #d4a574;" />
          </div>
          <div>
            <div class="text-xs font-bold" style="color: #d4a574;">酒馆原生训练</div>
            <div class="text-[11px] leading-relaxed" style="color: #7a5c44;">
              OPA-DenseV4 训练方法，Claude Opus 4.6 知识蒸馏。训练数据含上千张角色卡 + 数百种预设，天然理解角色扮演叙事。
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- ══════════════════════════════════════════════════════ -->
    <!-- Part 4: Competitor Comparison Table                  -->
    <!-- ══════════════════════════════════════════════════════ -->
    <div class="rounded-xl p-4 space-y-2.5"
      style="background: rgba(20, 14, 10, 0.4); border: 1px solid rgba(180, 140, 100, 0.1);">
      <div class="text-sm font-bold" style="color: #c49a70;">方案对比</div>
      <div class="overflow-x-auto">
        <table class="w-full text-[10px] border-collapse" style="color: #7a5c44;">
          <thead>
            <tr style="border-bottom: 1px solid rgba(180, 140, 100, 0.15);">
              <th class="text-left py-1.5 pr-2" style="color: #9e7a5c;">方案</th>
              <th class="text-center py-1.5 px-1">写作</th>
              <th class="text-center py-1.5 px-1">沉浸感</th>
              <th class="text-center py-1.5 px-1">NSFW</th>
              <th class="text-center py-1.5 px-1">上下文</th>
              <th class="text-right py-1.5 pl-1" style="color: #9e7a5c;">成本</th>
            </tr>
          </thead>
          <tbody>
            <tr style="border-bottom: 1px solid rgba(180, 140, 100, 0.06);">
              <td class="py-1.5 pr-2 font-bold" style="color: #e6b422;">Tavern Pro</td>
              <td class="text-center py-1.5" style="color: #27ae60;">接近Opus</td>
              <td class="text-center py-1.5" style="color: #27ae60;">优秀</td>
              <td class="text-center py-1.5" style="color: #27ae60;">自由</td>
              <td class="text-center py-1.5" style="color: #3498db;">1M</td>
              <td class="text-right py-1.5" style="color: #27ae60;">中等</td>
            </tr>
            <tr style="border-bottom: 1px solid rgba(180, 140, 100, 0.06);">
              <td class="py-1.5 pr-2" style="color: #9e7a5c;">Claude Opus 4.6</td>
              <td class="text-center py-1.5" style="color: #e6b422;">顶级</td>
              <td class="text-center py-1.5" style="color: #e6b422;">极佳</td>
              <td class="text-center py-1.5" style="color: #c0392b;">受限</td>
              <td class="text-center py-1.5">200K</td>
              <td class="text-right py-1.5" style="color: #c0392b;">极高</td>
            </tr>
            <tr style="border-bottom: 1px solid rgba(180, 140, 100, 0.06);">
              <td class="py-1.5 pr-2" style="color: #9e7a5c;">DeepSeek V4 原版</td>
              <td class="text-center py-1.5">良好</td>
              <td class="text-center py-1.5">一般</td>
              <td class="text-center py-1.5" style="color: #c0392b;">严格审查</td>
              <td class="text-center py-1.5" style="color: #3498db;">1M</td>
              <td class="text-right py-1.5" style="color: #27ae60;">中等</td>
            </tr>
            <tr>
              <td class="py-1.5 pr-2" style="color: #9e7a5c;">本地小模型</td>
              <td class="text-center py-1.5" style="color: #c0392b;">较弱</td>
              <td class="text-center py-1.5" style="color: #c0392b;">3轮崩</td>
              <td class="text-center py-1.5" style="color: #27ae60;">自由</td>
              <td class="text-center py-1.5">4-32K</td>
              <td class="text-right py-1.5" style="color: #27ae60;">免费</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- ══════════════════════════════════════════════════════ -->
    <!-- Part 5: API Access Info                              -->
    <!-- ══════════════════════════════════════════════════════ -->
    <div class="rounded-xl p-4 space-y-2"
      style="background: rgba(20, 14, 10, 0.4); border: 1px solid rgba(180, 140, 100, 0.1);">
      <div class="flex items-center gap-2">
        <PhRocket :size="15" style="color: #3498db;" />
        <span class="text-sm font-bold" style="color: #c49a70;">接入信息</span>
      </div>
      <div class="grid grid-cols-2 gap-1.5 text-[11px]">
        <div class="flex gap-1.5">
          <span style="color: #6b5344;">端点</span>
          <span class="font-mono" style="color: #d4a574;">https://deepseektavern.com/v1</span>
        </div>
        <div class="flex gap-1.5">
          <span style="color: #6b5344;">协议</span>
          <span style="color: #27ae60;">OpenAI 兼容接口</span>
        </div>
        <div class="flex gap-1.5">
          <span style="color: #6b5344;">客户端</span>
          <span style="color: #d4a574;">SillyTavern · Cherry Studio · NextChat · OpenAI SDK</span>
        </div>
        <div class="flex gap-1.5">
          <span style="color: #6b5344;">定价</span>
          <span style="color: #27ae60;">约为 Opus 的 1/10</span>
        </div>
      </div>
    </div>

    <!-- ══════════════════════════════════════════════════════ -->
    <!-- Part 6: Step 1 - Get API Key                         -->
    <!-- ══════════════════════════════════════════════════════ -->
    <div class="rounded-xl p-4 space-y-3"
      style="background: linear-gradient(135deg, rgba(34, 26, 20, 0.88), rgba(45, 31, 20, 0.78)); border: 1px solid rgba(230, 180, 34, 0.2);">
      <div class="flex items-center gap-2">
        <span class="w-5 h-5 rounded-full flex items-center justify-center text-[11px] font-bold"
          style="background: #8b6914; color: #fdf4dc;">1</span>
        <span class="text-sm font-bold" style="color: #c49a70;">获取 API Key</span>
      </div>

      <p class="text-xs" style="color: #7a5c44;">
        点击下方按钮在<strong style="color: #c49a70;">浏览器</strong>中打开 DeepSeek Tavern 控制台。注册/登录 →
        <strong style="color: #c49a70;">创建 API Key</strong>（选无限额度）→
        创建后立即<strong style="color: #e6b422;">复制完整 Key</strong>（只显示一次！）
      </p>

      <button
        class="w-full py-3 rounded-xl text-sm font-bold flex items-center justify-center gap-2 transition-all active:scale-95"
        style="background: linear-gradient(135deg, #6b4a35, #8b6914); color: #fdf4dc; border: 1px solid rgba(230, 180, 34, 0.25); box-shadow: 0 4px 20px rgba(230, 180, 34, 0.15);"
        @click="openKeyWebview"
        :disabled="openingWebview"
      >
        <PhArrowCounterClockwise v-if="openingWebview" :size="16" class="animate-spin" />
        <PhGlobeHemisphereWest v-else :size="16" />
        {{ openingWebview ? '正在打开...' : '🌐 打开酒馆大模型控制台' }}
      </button>
      <p class="text-xs text-center" style="color: #6b5344;">
        将在浏览器中打开 deepseektavern.com/console
      </p>
    </div>

    <!-- ══════════════════════════════════════════════════════ -->
    <!-- Part 7: Step 2 - Paste & Apply                       -->
    <!-- ══════════════════════════════════════════════════════ -->
    <div class="rounded-xl p-4 space-y-3"
      style="background: linear-gradient(135deg, rgba(20, 14, 10, 0.7), rgba(20, 14, 10, 0.5)); border: 1px solid rgba(52, 152, 219, 0.15);">
      <div class="flex items-center gap-2">
        <span class="w-5 h-5 rounded-full flex items-center justify-center text-[11px] font-bold"
          style="background: #1a5276; color: #d4e6f1;">2</span>
        <span class="text-sm font-bold" style="color: #c49a70;">粘贴 Key 并填入酒馆</span>
      </div>

      <div class="flex gap-2">
        <input
          v-model="manualKeyInput"
          type="text"
          placeholder="粘贴从网站复制的完整 API Key..."
          class="flex-1 px-3 py-2.5 rounded-lg text-sm font-mono outline-none"
          style="background: rgba(20, 14, 10, 0.6); color: #d4a574; border: 1px solid rgba(180, 140, 100, 0.2);"
        />
        <button
          class="shrink-0 px-4 py-2.5 rounded-lg text-sm font-bold flex items-center gap-1.5 transition-all active:scale-95"
          :disabled="savingKey || !manualKeyInput.trim()"
          style="background: linear-gradient(135deg, #1a5276, #2471a3); color: #d4e6f1; border: 1px solid rgba(52, 152, 219, 0.3);"
          @click="pasteAndSave"
        >
          <PhArrowCounterClockwise v-if="savingKey" :size="12" class="animate-spin" />
          <PhKey v-else :size="12" />
          {{ savingKey ? '保存中' : '填入酒馆' }}
        </button>
      </div>
    </div>

    <!-- ══════════════════════════════════════════════════════ -->
    <!-- Part 8: Saved Keys                                   -->
    <!-- ══════════════════════════════════════════════════════ -->
    <div v-if="savedKeys.length > 0" class="rounded-xl p-4 space-y-2"
      style="background: rgba(20, 14, 10, 0.4); border: 1px solid rgba(180, 140, 100, 0.1);">
      <div class="flex items-center justify-between">
        <span class="text-sm font-bold" style="color: #c49a70;">已保存的 Key</span>
        <span class="text-[11px]" style="color: #6b5344;">{{ savedKeys.length }} 个</span>
      </div>
      <div class="space-y-1 max-h-40 overflow-y-auto">
        <div
          v-for="(item, idx) in savedKeys"
          :key="item.savedAt"
          class="flex items-center gap-2 px-3 py-2 rounded-lg text-xs"
          style="background: rgba(20, 14, 10, 0.3); border: 1px solid rgba(180, 140, 100, 0.08);"
        >
          <div class="flex-1 min-w-0">
            <div class="text-[11px] font-bold truncate" style="color: #c49a70;">{{ item.label }}</div>
            <div class="flex items-center gap-1">
              <span class="text-[10px] font-mono truncate" style="color: #6b5344;">
                {{ revealedIdx.has(idx) ? item.key : (item.key.slice(0, 16) + '...') }}
              </span>
              <button class="p-0.5 rounded transition-colors"
                :style="{ color: revealedIdx.has(idx) ? '#e6b422' : '#6b5344' }"
                @click="toggleReveal(idx)" title="显示/隐藏完整 Key">
                <PhEye :size="10" v-if="!revealedIdx.has(idx)" />
                <PhEyeSlash :size="10" v-else />
              </button>
            </div>
          </div>
          <button class="shrink-0 p-1 rounded transition-colors" style="color: #9e7a5c;" @click="copyKey(item.key)" title="复制">
            <PhCopy :size="12" />
          </button>
          <button class="shrink-0 p-1 rounded transition-colors" style="color: #3498db;" @click="autoFillApiConfig(item.key)" title="填入酒馆">
            <PhKey :size="12" />
          </button>
          <button class="shrink-0 p-1 rounded transition-colors" style="color: #c0392b;" @click="removeSavedKey(idx)" title="删除">
            <PhTrash :size="12" />
          </button>
        </div>
      </div>
    </div>

  </div>
</template>
