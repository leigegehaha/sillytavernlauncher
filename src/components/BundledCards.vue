<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { resourceDir } from '@tauri-apps/api/path'
import { PhCards, PhArrowCounterClockwise } from '@phosphor-icons/vue'
import { toast } from 'vue-sonner'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const isTauri = !!(window as any).__TAURI_INTERNALS__
const emit = defineEmits<{
  (e: 'cardImported'): void
}>()

interface CardEntry {
  fileName: string
  category: string
  size: number
}

interface RustCardEntry {
  fileName: string
  size: number
  modifiedMs: number | null
  category: string | null
}

// Categories with metadata
const categories = [
  { key: '正常向', label: '正常向', icon: '🌟', desc: '通用角色卡', order: 1 },
  { key: '同人', label: '同人角色', icon: '🎭', desc: '动漫/游戏/电影角色', order: 2 },
  { key: '修仙', label: '修仙世界', icon: '⚔️', desc: '仙侠修真角色', order: 3 },
  { key: '古风', label: '古风雅韵', icon: '🏯', desc: '古典中国风格', order: 4 },
  { key: '校园', label: '校园生活', icon: '📚', desc: '校园青春角色', order: 5 },
  { key: '欧美', label: '欧美风格', icon: '🏰', desc: '西方奇幻/现代', order: 6 },
  { key: '纯爱', label: '纯爱浪漫', icon: '💕', desc: '恋爱向角色', order: 7 },
  { key: '英文卡', label: 'English Cards', icon: '🌍', desc: 'English character cards', order: 8 },
  { key: '单人', label: '单人角色', icon: '👤', desc: '单人角色合集', order: 9 },
  { key: '双人', label: '双人组合', icon: '👥', desc: '双人/多角角色', order: 10 },
  { key: '整活', label: '趣味整活', icon: '🎪', desc: '搞笑/创意卡', order: 11 },
]

const activeCategory = ref(categories[0].key)
const cards = ref<CardEntry[]>([])
const thumbUrls = ref<Record<string, string>>({})
const loading = ref(false)
const importingCard = ref<Set<string>>(new Set())

// Load card list from bundled resources
const loadCards = async () => {
  loading.value = true
  try {
    if (isTauri) {
      const rawList = await invoke<RustCardEntry[]>('list_bundled_cards')
      cards.value = rawList
        .filter(c => c.category !== null)
        .map(c => ({
          fileName: c.fileName,
          category: c.category!,
          size: c.size,
        }))
    }
  } catch (e: any) {
    console.error('Failed to load bundled cards:', e)
    cards.value = []
  } finally {
    loading.value = false
  }
}

// Filtered cards for active category
const filteredCards = computed(() =>
  cards.value.filter(c => c.category === activeCategory.value)
)

// Category order for display
const sortedCategories = computed(() =>
  [...categories].sort((a, b) => a.order - b.order)
)

// Load thumbnails for all cards in category
const loadCategoryThumbs = async (category: string) => {
  const catCards = cards.value.filter(c => c.category === category)
  for (const card of catCards) {
    if (thumbUrls.value[card.fileName]) continue
    try {
      if (isTauri) {
        const bytes = await invoke<number[]>('read_bundled_card_thumb', {
          category: card.category,
          fileName: card.fileName,
        })
        const u8 = new Uint8Array(bytes)
        const url = URL.createObjectURL(new Blob([u8], { type: 'image/png' }))
        thumbUrls.value = { ...thumbUrls.value, [card.fileName]: url }
      }
    } catch {
      // Silently skip thumbnails that fail to load
    }
  }
}

// Import a single card
const importCard = async (card: CardEntry) => {
  importingCard.value = new Set([...importingCard.value, card.fileName])
  try {
    await invoke('import_bundled_card', {
      category: card.category,
      fileName: card.fileName,
    })
    toast.success(`已导入：${getDisplayName(card.fileName)}`)
    emit('cardImported')
  } catch (e: any) {
    toast.error(e?.message || '导入失败')
  } finally {
    const next = new Set(importingCard.value)
    next.delete(card.fileName)
    importingCard.value = next
  }
}

// Import all cards in current category
const importAll = async () => {
  for (const card of filteredCards.value) {
    await importCard(card)
  }
}

// Extract display name from file name (remove extension, clean up)
function getDisplayName(fileName: string): string {
  return fileName.replace(/\.(png|webp|json)$/i, '')
}

watch(activeCategory, async (cat) => {
  await loadCategoryThumbs(cat)
})

onMounted(async () => {
  await loadCards()
  if (cards.value.length > 0) {
    await loadCategoryThumbs(activeCategory.value)
  }
})
</script>

<template>
  <div class="flex flex-col h-full space-y-5">
    <!-- Category selector -->
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
        <span class="text-base">{{ cat.icon }}</span>
        <span>{{ cat.label }}</span>
      </button>
    </div>

    <!-- Category description and import all button -->
    <div class="flex items-center justify-between">
      <div>
        <p class="text-sm font-medium text-slate-700 dark:text-slate-200">
          {{ categories.find(c => c.key === activeCategory)?.label }}
        </p>
        <p class="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
          {{ filteredCards.length }} 张角色卡 · {{ categories.find(c => c.key === activeCategory)?.desc }}
        </p>
      </div>
      <button
        v-if="filteredCards.length > 0"
        class="px-3 py-1.5 rounded-lg text-xs font-medium transition-colors flex items-center gap-1.5"
        style="background: rgba(74, 53, 36, 0.3); color: #d4a574; border: 1px solid rgba(212, 165, 116, 0.2);"
        @click="importAll"
      >
        <PhArrowCounterClockwise :size="14" />
        一键导入全部
      </button>
    </div>

    <!-- Cards grid -->
    <div v-if="loading" class="text-center py-12 text-sm text-slate-400 dark:text-slate-500">
      加载中...
    </div>
    <div
      v-else-if="filteredCards.length === 0"
      class="text-center py-12 bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700"
    >
      <div class="text-sm text-slate-500">该分类暂无角色卡</div>
    </div>
    <div v-else class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
      <div
        v-for="card in filteredCards"
        :key="card.fileName"
        class="bg-white dark:bg-slate-800 rounded-2xl border border-slate-200 dark:border-slate-700 shadow-sm hover:shadow-soft transition-shadow overflow-hidden flex flex-col group"
      >
        <!-- Card thumbnail -->
        <div class="bg-slate-100 dark:bg-slate-700 aspect-2/3 shrink-0 relative">
          <img
            v-if="thumbUrls[card.fileName]"
            :src="thumbUrls[card.fileName]"
            class="w-full h-full object-cover"
            :alt="getDisplayName(card.fileName)"
            loading="lazy"
          />
          <div
            v-else
            class="w-full h-full flex items-center justify-center text-slate-400 dark:text-slate-600"
          >
            <PhCards :size="32" />
          </div>
        </div>
        <!-- Card info -->
        <div class="p-3 flex-1 flex flex-col justify-between">
          <div class="text-sm font-medium text-slate-800 dark:text-slate-100 line-clamp-2 leading-tight">
            {{ getDisplayName(card.fileName) }}
          </div>
          <button
            class="mt-2 w-full py-1.5 rounded-lg text-xs font-medium transition-all duration-200 flex items-center justify-center gap-1"
            :style="{
              background: importingCard.has(card.fileName)
                ? 'rgba(39, 174, 96, 0.15)'
                : 'rgba(74, 53, 36, 0.2)',
              color: importingCard.has(card.fileName) ? '#27ae60' : '#d4a574',
              border: importingCard.has(card.fileName)
                ? '1px solid rgba(39, 174, 96, 0.3)'
                : '1px solid rgba(212, 165, 116, 0.2)',
            }"
            :disabled="importingCard.has(card.fileName)"
            @click="importCard(card)"
          >
            {{ importingCard.has(card.fileName) ? '导入中...' : '导入' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
