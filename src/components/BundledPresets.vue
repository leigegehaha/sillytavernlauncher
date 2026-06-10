<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { PhGear, PhStar } from '@phosphor-icons/vue'
import { toast } from 'vue-sonner'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const isTauri = !!(window as any).__TAURI_INTERNALS__
const emit = defineEmits<{
  (e: 'presetImported'): void
}>()

interface PresetEntry {
  fileName: string
  category: string
  size: number
  isRecommended: boolean
}

interface RustPresetEntry {
  fileName: string
  size: number
  modifiedMs: number | null
  category: string | null
}

// Categories
const categories = [
  { key: '推荐', label: '⭐ 推荐', desc: '搭配酒馆专用模型最佳', color: '#e6b422', order: 1 },
  { key: '调酒师', label: '🍸 调酒师', desc: '经典调酒师系列预设', color: '#d4a574', order: 2 },
  { key: '精选预设', label: '🏆 精选预设', desc: '社区优质预设合集', color: '#a08060', order: 3 },
]

const activeCategory = ref('推荐')
const presets = ref<PresetEntry[]>([])
const loading = ref(false)
const importingPreset = ref<Set<string>>(new Set())

const loadPresets = async () => {
  loading.value = true
  try {
    if (isTauri) {
      const rawList = await invoke<RustPresetEntry[]>('list_bundled_presets')
      presets.value = rawList
        .filter(p => p.category !== null)
        .map(p => ({
          fileName: p.fileName,
          category: p.category!,
          size: p.size,
          isRecommended: p.fileName.includes('酒馆专用'),
        }))
    }
  } catch (e: any) {
    console.error('Failed to load presets:', e)
    presets.value = []
  } finally {
    loading.value = false
  }
}

const filteredPresets = computed(() =>
  presets.value.filter(p => p.category === activeCategory.value)
)

const sortedCategories = computed(() =>
  [...categories].sort((a, b) => a.order - b.order)
)

const formattedSize = (bytes: number) => {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

const getDisplayName = (fileName: string): string => {
  return fileName.replace(/\.(json)$/i, '')
}

const importPreset = async (preset: PresetEntry) => {
  importingPreset.value = new Set([...importingPreset.value, preset.fileName])
  try {
    await invoke('import_bundled_preset', {
      category: preset.category,
      fileName: preset.fileName,
    })
    toast.success(`已导入：${getDisplayName(preset.fileName)}`)
    emit('presetImported')
  } catch (e: any) {
    toast.error(e?.message || '导入失败')
  } finally {
    const next = new Set(importingPreset.value)
    next.delete(preset.fileName)
    importingPreset.value = next
  }
}

onMounted(async () => {
  await loadPresets()
})
</script>

<template>
  <div class="flex flex-col h-full space-y-5">
    <!-- Category tabs -->
    <div class="flex flex-wrap gap-2">
      <button
        v-for="cat in sortedCategories"
        :key="cat.key"
        type="button"
        class="px-4 py-2.5 rounded-xl text-sm font-medium transition-all duration-200 flex items-center gap-2"
        :class="[
          activeCategory === cat.key
            ? 'text-[#e6b422] shadow-sm'
            : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200',
        ]"
        :style="activeCategory === cat.key
          ? { background: '#3a2a1a', border: '1px solid rgba(230, 180, 34, 0.3)' }
          : { border: '1px solid transparent' }"
        @click="activeCategory = cat.key"
      >
        <span class="text-base">{{ cat.label }}</span>
      </button>
    </div>

    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <p class="text-sm font-medium text-slate-700 dark:text-slate-200">
          {{ categories.find(c => c.key === activeCategory)?.label }}
        </p>
        <p class="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
          {{ filteredPresets.length }} 个预设 · {{ categories.find(c => c.key === activeCategory)?.desc }}
        </p>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="text-center py-12 text-sm text-slate-400">加载中...</div>

    <!-- Preset list -->
    <div
      v-else-if="filteredPresets.length === 0"
      class="text-center py-12 bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700"
    >
      <div class="text-sm text-slate-500">该分类暂无预设</div>
    </div>
    <div v-else class="space-y-2">
      <div
        v-for="preset in filteredPresets"
        :key="preset.fileName"
        class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 flex items-center gap-4 group hover:shadow-soft transition-all duration-200"
        :class="preset.isRecommended ? 'ring-1' : ''"
        :style="preset.isRecommended ? { borderColor: 'rgba(230, 180, 34, 0.3)' } : {}"
      >
        <!-- Icon -->
        <div
          class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0"
          :style="preset.isRecommended
            ? { background: 'rgba(230, 180, 34, 0.12)' }
            : { background: '#f0ede8' }"
        >
          <PhStar v-if="preset.isRecommended" class="w-5 h-5" style="color: #e6b422" />
          <PhGear v-else class="w-5 h-5 text-slate-500 dark:text-slate-400" />
        </div>

        <!-- Info -->
        <div class="flex-1 min-w-0">
          <div class="text-sm font-semibold text-slate-800 dark:text-slate-100 truncate">
            {{ getDisplayName(preset.fileName) }}
            <span
              v-if="preset.isRecommended"
              class="ml-2 px-1.5 py-0.5 text-[10px] font-bold rounded-md inline-block"
              style="background: linear-gradient(135deg, rgba(230, 180, 34, 0.2), rgba(212, 165, 116, 0.2)); color: #e6b422;"
            >
              酒馆专用推荐
            </span>
          </div>
          <div class="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
            {{ formattedSize(preset.size) }}
          </div>
        </div>

        <!-- Action -->
        <button
          class="px-4 py-2 rounded-lg text-xs font-medium transition-all duration-200 flex items-center gap-1.5 shrink-0"
          :style="{
            background: importingPreset.has(preset.fileName)
              ? 'rgba(39, 174, 96, 0.15)'
              : preset.isRecommended
                ? 'linear-gradient(135deg, rgba(230, 180, 34, 0.15), rgba(212, 165, 116, 0.15))'
                : 'rgba(74, 53, 36, 0.2)',
            color: importingPreset.has(preset.fileName)
              ? '#27ae60'
              : preset.isRecommended ? '#e6b422' : '#d4a574',
            border: importingPreset.has(preset.fileName)
              ? '1px solid rgba(39, 174, 96, 0.3)'
              : preset.isRecommended
                ? '1px solid rgba(230, 180, 34, 0.3)'
                : '1px solid rgba(212, 165, 116, 0.2)',
          }"
          :disabled="importingPreset.has(preset.fileName)"
          @click="importPreset(preset)"
        >
          {{ importingPreset.has(preset.fileName) ? '导入中...' : '导入预设' }}
        </button>
      </div>
    </div>
  </div>
</template>
